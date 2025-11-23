use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::str::{self, FromStr};

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde_json::{self, Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{expect_string_arg, expect_tuple_arg};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($name:literal, $func:ident) => {
            functions.insert(
                $name.to_string(),
                NativeCallable::new(concat!("serde::", $name), $func),
            );
        };
    }

    add!("to_json", to_json);
    add!("pretty_json", pretty_json);
    add!("from_json", from_json);
    add!("to_yaml", to_yaml);
    add!("from_yaml", from_yaml);
    add!("to_toml", to_toml);
    add!("from_toml", from_toml);
    add!("to_xml", to_xml);
    add!("from_xml", from_xml);
    add!("to_bytes", to_bytes);
    add!("from_bytes", from_bytes);
    add!("to_bin", to_bin);
    add!("from_bin", from_bin);
    add!("to_base64", to_base64);
    add!("from_base64", from_base64);
    add!("to_csv", to_csv);
    add!("from_csv", from_csv);

    registry.register_module("serde", functions);
}

fn to_json(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_json expects a value"))?;
    let json = value_to_json_value(value);
    serde_json::to_string(&json)
        .map(Value::String)
        .map_err(|err| ApexError::new(format!("serde.to_json failed: {}", err)))
}

fn pretty_json(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.pretty_json expects a value"))?;
    let json = value_to_json_value(value);
    serde_json::to_string_pretty(&json)
        .map(Value::String)
        .map_err(|err| ApexError::new(format!("serde.pretty_json failed: {}", err)))
}

fn from_json(args: &[Value]) -> Result<Value, ApexError> {
    let text = expect_string_arg(args, 0, "serde.from_json")?;
    let json: JsonValue = serde_json::from_str(&text)
        .map_err(|err| ApexError::new(format!("serde.from_json failed: {}", err)))?;
    json_value_to_value(&json)
}

fn to_yaml(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_yaml expects a value"))?;
    let json = value_to_json_value(value);
    let mut text = String::from("---\n");
    let pretty = serde_json::to_string_pretty(&json)
        .map_err(|err| ApexError::new(format!("serde.to_yaml failed: {}", err)))?;
    text.push_str(&pretty);
    text.push('\n');
    Ok(Value::String(text))
}

fn from_yaml(args: &[Value]) -> Result<Value, ApexError> {
    let text = expect_string_arg(args, 0, "serde.from_yaml")?;
    let trimmed = text.trim();
    let without_header = if let Some(stripped) = trimmed.strip_prefix("---") {
        stripped.trim_start()
    } else {
        trimmed
    };
    let cleaned = if let Some(stripped) = without_header.strip_suffix("...") {
        stripped.trim_end()
    } else {
        without_header
    };
    let json: JsonValue = serde_json::from_str(cleaned)
        .map_err(|err| ApexError::new(format!("serde.from_yaml failed: {}", err)))?;
    json_value_to_value(&json)
}

fn to_toml(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_toml expects a value"))?;
    let json = value_to_json_value(value);
    Ok(Value::String(json_to_toml(&json)))
}

fn from_toml(args: &[Value]) -> Result<Value, ApexError> {
    let text = expect_string_arg(args, 0, "serde.from_toml")?;
    let mut parser = TomlParser::new(&text);
    let json = parser.parse_document()?;
    json_value_to_value(&json)
}

fn to_xml(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_xml expects a value"))?;
    Ok(Value::String(value_to_xml(value)))
}

fn from_xml(args: &[Value]) -> Result<Value, ApexError> {
    let text = expect_string_arg(args, 0, "serde.from_xml")?;
    xml_to_value(&text)
}

fn to_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let json_value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_bytes expects a value"))?;
    let json = value_to_json_value(json_value);
    let bytes = serde_json::to_vec(&json)
        .map_err(|err| ApexError::new(format!("serde.to_bytes failed: {}", err)))?;
    Ok(Value::Tuple(
        bytes
            .into_iter()
            .map(|byte| Value::Int(BigInt::from(byte)))
            .collect(),
    ))
}

fn to_bin(args: &[Value]) -> Result<Value, ApexError> {
    let json_value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_bin expects a value"))?;
    let mut bytes = Vec::new();
    encode_value_binary(json_value, &mut bytes)?;
    Ok(Value::Tuple(
        bytes
            .into_iter()
            .map(|byte| Value::Int(BigInt::from(byte)))
            .collect(),
    ))
}

fn from_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let tuple = expect_tuple_arg(args, 0, "serde.from_bytes")?;
    let mut bytes = Vec::with_capacity(tuple.len());
    for value in tuple {
        match value {
            Value::Int(num) => {
                let byte = num
                    .to_u8()
                    .ok_or_else(|| ApexError::new("serde.from_bytes expects 0-255 ints"))?;
                bytes.push(byte);
            }
            _ => {
                return Err(ApexError::new(
                    "serde.from_bytes expects a tuple of byte integers",
                ))
            }
        }
    }
    let json: JsonValue = serde_json::from_slice(&bytes)
        .map_err(|err| ApexError::new(format!("serde.from_bytes failed: {}", err)))?;
    json_value_to_value(&json)
}

fn from_bin(args: &[Value]) -> Result<Value, ApexError> {
    let tuple = expect_tuple_arg(args, 0, "serde.from_bin")?;
    let mut bytes = Vec::with_capacity(tuple.len());
    for (index, value) in tuple.iter().enumerate() {
        match value {
            Value::Int(num) => bytes.push(num.to_u8().ok_or_else(|| {
                ApexError::new(format!(
                    "serde.from_bin expects 0-255 ints at position {}",
                    index
                ))
            })?),
            _ => {
                return Err(ApexError::new(
                    "serde.from_bin expects a tuple of byte integers",
                ))
            }
        }
    }

    let mut cursor = 0;
    let value = decode_value_binary(&bytes, &mut cursor)?;
    if cursor != bytes.len() {
        return Err(ApexError::new(
            "serde.from_bin encountered trailing bytes after decoding",
        ));
    }
    Ok(value)
}

fn to_base64(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_base64 expects a value"))?;
    let json = value_to_json_value(value);
    let bytes = serde_json::to_vec(&json)
        .map_err(|err| ApexError::new(format!("serde.to_base64 failed: {}", err)))?;
    Ok(Value::String(encode_base64_bytes(&bytes)))
}

fn from_base64(args: &[Value]) -> Result<Value, ApexError> {
    let encoded = expect_string_arg(args, 0, "serde.from_base64")?;
    let bytes = decode_base64_string(&encoded)?;
    let json: JsonValue = serde_json::from_slice(&bytes)
        .map_err(|err| ApexError::new(format!("serde.from_base64 decode failed: {}", err)))?;
    json_value_to_value(&json)
}

fn to_csv(args: &[Value]) -> Result<Value, ApexError> {
    let rows = args
        .get(0)
        .ok_or_else(|| ApexError::new("serde.to_csv expects a tuple of rows"))?;
    let mut csv = String::new();

    match rows {
        Value::Tuple(rows) => {
            for (index, row) in rows.iter().enumerate() {
                if index > 0 {
                    csv.push('\n');
                }
                let cells: Vec<Value> = match row {
                    Value::Tuple(cols) => cols.clone(),
                    other => vec![other.clone()],
                };
                for (cell_index, cell) in cells.iter().enumerate() {
                    if cell_index > 0 {
                        csv.push(',');
                    }
                    write_csv_cell(&mut csv, cell);
                }
            }
        }
        other => {
            write_csv_cell(&mut csv, other);
        }
    }

    Ok(Value::String(csv))
}

fn from_csv(args: &[Value]) -> Result<Value, ApexError> {
    let text = expect_string_arg(args, 0, "serde.from_csv")?;
    let mut rows: Vec<Value> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut cols: Vec<Value> = Vec::new();
        for cell in line.split(',') {
            let trimmed = cell.trim();
            if let Ok(int_val) = BigInt::from_str(trimmed) {
                cols.push(Value::Int(int_val));
            } else if let Ok(float_val) = trimmed.parse::<f64>() {
                cols.push(Value::Number(float_val));
            } else if let Ok(boolean) = trimmed.parse::<bool>() {
                cols.push(Value::Bool(boolean));
            } else {
                cols.push(Value::String(trimmed.trim_matches('"').to_string()));
            }
        }
        rows.push(Value::Tuple(cols));
    }
    Ok(Value::Tuple(rows))
}

fn encode_value_binary(value: &Value, out: &mut Vec<u8>) -> Result<(), ApexError> {
    match value {
        Value::Int(num) => {
            out.push(0x01);
            let mut bytes = num.to_signed_bytes_be();
            if bytes.is_empty() {
                bytes.push(0);
            }
            write_length(bytes.len(), out)?;
            out.extend_from_slice(&bytes);
        }
        Value::Number(num) => {
            out.push(0x02);
            out.extend_from_slice(&num.to_bits().to_be_bytes());
        }
        Value::Bool(flag) => {
            out.push(0x03);
            out.push(if *flag { 1 } else { 0 });
        }
        Value::String(text) => {
            out.push(0x04);
            write_length(text.len(), out)?;
            out.extend_from_slice(text.as_bytes());
        }
        Value::Tuple(items) => {
            out.push(0x05);
            write_length(items.len(), out)?;
            for item in items {
                encode_value_binary(item, out)?;
            }
        }
    }
    Ok(())
}

fn decode_value_binary(bytes: &[u8], cursor: &mut usize) -> Result<Value, ApexError> {
    if *cursor >= bytes.len() {
        return Err(ApexError::new(
            "serde.from_bin reached the end of the buffer unexpectedly",
        ));
    }
    let tag = bytes[*cursor];
    *cursor += 1;
    match tag {
        0x01 => {
            let len = read_length(bytes, cursor)?;
            let end = *cursor + len;
            if end > bytes.len() {
                return Err(ApexError::new(
                    "serde.from_bin int payload length exceeds buffer",
                ));
            }
            let slice = &bytes[*cursor..end];
            *cursor = end;
            Ok(Value::Int(BigInt::from_signed_bytes_be(slice)))
        }
        0x02 => {
            if *cursor + 8 > bytes.len() {
                return Err(ApexError::new(
                    "serde.from_bin number payload is incomplete",
                ));
            }
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&bytes[*cursor..*cursor + 8]);
            *cursor += 8;
            Ok(Value::Number(f64::from_bits(u64::from_be_bytes(buf))))
        }
        0x03 => {
            if *cursor >= bytes.len() {
                return Err(ApexError::new("serde.from_bin bool payload is missing"));
            }
            let flag = bytes[*cursor] != 0;
            *cursor += 1;
            Ok(Value::Bool(flag))
        }
        0x04 => {
            let len = read_length(bytes, cursor)?;
            let end = *cursor + len;
            if end > bytes.len() {
                return Err(ApexError::new(
                    "serde.from_bin string payload length exceeds buffer",
                ));
            }
            let slice = &bytes[*cursor..end];
            *cursor = end;
            let text = String::from_utf8(slice.to_vec())
                .map_err(|_| ApexError::new("serde.from_bin string payload is not UTF-8"))?;
            Ok(Value::String(text))
        }
        0x05 => {
            let len = read_length(bytes, cursor)?;
            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                items.push(decode_value_binary(bytes, cursor)?);
            }
            Ok(Value::Tuple(items))
        }
        _ => Err(ApexError::new("serde.from_bin encountered an unknown tag")),
    }
}

fn write_length(len: usize, out: &mut Vec<u8>) -> Result<(), ApexError> {
    let len_u32 = u32::try_from(len).map_err(|_| {
        ApexError::new("serde.to_bin payload is too large to encode with u32 lengths")
    })?;
    out.extend_from_slice(&len_u32.to_be_bytes());
    Ok(())
}

fn read_length(bytes: &[u8], cursor: &mut usize) -> Result<usize, ApexError> {
    if *cursor + 4 > bytes.len() {
        return Err(ApexError::new(
            "serde.from_bin length header extends beyond the buffer",
        ));
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[*cursor..*cursor + 4]);
    *cursor += 4;
    Ok(u32::from_be_bytes(buf) as usize)
}

fn write_csv_cell(buffer: &mut String, value: &Value) {
    let mut rendered = match value {
        Value::Int(int_val) => int_val.to_string(),
        Value::Number(num) => num.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::String(text) => text.clone(),
        Value::Tuple(items) => {
            let inner: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
            inner.join("|")
        }
    };

    let needs_quotes = rendered.contains(',') || rendered.contains('\n') || rendered.contains('"');
    if rendered.contains('"') {
        rendered = rendered.replace('"', "\"\"");
    }

    if needs_quotes {
        buffer.push('"');
        buffer.push_str(&rendered);
        buffer.push('"');
    } else {
        buffer.push_str(&rendered);
    }
}

fn value_to_json_value(value: &Value) -> JsonValue {
    match value {
        Value::Int(big) => {
            if let Some(i) = big.to_i64() {
                JsonValue::Number(JsonNumber::from(i))
            } else if let Some(u) = big.to_u64() {
                JsonValue::Number(JsonNumber::from(u))
            } else {
                JsonValue::String(format!("#int:{}", big))
            }
        }
        Value::Number(num) => JsonNumber::from_f64(*num)
            .map(JsonValue::Number)
            .unwrap_or_else(|| JsonValue::String(num.to_string())),
        Value::Bool(flag) => JsonValue::Bool(*flag),
        Value::String(text) => JsonValue::String(text.clone()),
        Value::Tuple(values) => match tuple_to_object(values) {
            Some(object) => JsonValue::Object(object),
            None => JsonValue::Array(values.iter().map(value_to_json_value).collect()),
        },
    }
}

fn tuple_to_object(values: &[Value]) -> Option<JsonMap<String, JsonValue>> {
    if values.is_empty() {
        return None;
    }
    let mut object = JsonMap::new();
    for value in values {
        match value {
            Value::Tuple(pair) if pair.len() == 2 => match &pair[0] {
                Value::String(key) => {
                    object.insert(key.clone(), value_to_json_value(&pair[1]));
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    Some(object)
}

fn json_value_to_value(value: &JsonValue) -> Result<Value, ApexError> {
    Ok(match value {
        JsonValue::Null => Value::Tuple(Vec::new()),
        JsonValue::Bool(flag) => Value::Bool(*flag),
        JsonValue::Number(num) => {
            if let Some(i) = num.as_i64() {
                Value::Int(BigInt::from(i))
            } else if let Some(u) = num.as_u64() {
                Value::Int(BigInt::from(u))
            } else {
                Value::Number(num.as_f64().unwrap_or(0.0))
            }
        }
        JsonValue::String(text) => {
            if let Some(rest) = text.strip_prefix("#int:") {
                Value::Int(BigInt::from_str(rest).map_err(|err| {
                    ApexError::new(format!("serde: invalid big integer literal: {}", err))
                })?)
            } else {
                Value::String(text.clone())
            }
        }
        JsonValue::Array(values) => Value::Tuple(
            values
                .iter()
                .map(json_value_to_value)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        JsonValue::Object(map) => {
            let mut entries = Vec::with_capacity(map.len());
            for (key, value) in map.iter() {
                entries.push(Value::Tuple(vec![
                    Value::String(key.clone()),
                    json_value_to_value(value)?,
                ]));
            }
            Value::Tuple(entries)
        }
    })
}

fn value_to_xml(value: &Value) -> String {
    fn write_value(value: &Value, output: &mut String) {
        match value {
            Value::Int(v) => {
                output.push_str("<value type=\"int\">");
                output.push_str(&escape_xml(&v.to_string()));
                output.push_str("</value>");
            }
            Value::Number(v) => {
                output.push_str("<value type=\"number\">");
                output.push_str(&escape_xml(&v.to_string()));
                output.push_str("</value>");
            }
            Value::Bool(flag) => {
                output.push_str("<value type=\"bool\">");
                output.push_str(if *flag { "true" } else { "false" });
                output.push_str("</value>");
            }
            Value::String(text) => {
                output.push_str("<value type=\"string\">");
                output.push_str(&escape_xml(text));
                output.push_str("</value>");
            }
            Value::Tuple(items) => {
                output.push_str("<value type=\"tuple\">");
                for item in items {
                    output.push_str("<item>");
                    write_value(item, output);
                    output.push_str("</item>");
                }
                output.push_str("</value>");
            }
        }
    }

    let mut xml = String::new();
    write_value(value, &mut xml);
    xml
}

fn xml_to_value(xml: &str) -> Result<Value, ApexError> {
    let mut parser = XmlParser::new(xml);
    let value = parser.parse_value()?;
    parser.consume_whitespace();
    if !parser.is_eof() {
        return Err(ApexError::new(
            "serde.from_xml encountered trailing characters after root",
        ));
    }
    Ok(value)
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_xml(text: &str) -> Result<String, ApexError> {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            let mut terminated = false;
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == ';' {
                    terminated = true;
                    break;
                }
                entity.push(next);
            }
            if !terminated {
                return Err(ApexError::new(
                    "serde.from_xml encountered an unterminated entity",
                ));
            }
            let decoded = match entity.as_str() {
                "amp" => '&',
                "lt" => '<',
                "gt" => '>',
                "quot" => '\"',
                "apos" => '\'',
                _ => {
                    return Err(ApexError::new(
                        "serde.from_xml encountered an unknown entity",
                    ))
                }
            };
            output.push(decoded);
        } else {
            output.push(ch);
        }
    }
    Ok(output)
}

struct XmlParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            input: source.as_bytes(),
            pos: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.input[self.pos..].starts_with(needle.as_bytes())
    }

    fn consume_str(&mut self, needle: &str) -> Result<(), ApexError> {
        if self.starts_with(needle) {
            self.pos += needle.len();
            Ok(())
        } else {
            Err(ApexError::new("serde.from_xml encountered malformed tags"))
        }
    }

    fn parse_value(&mut self) -> Result<Value, ApexError> {
        self.consume_whitespace();
        self.consume_str("<value")?;
        self.consume_whitespace();
        self.consume_str("type=\"")?;
        let ty = self.read_until(b'"')?;
        self.pos += 1; // closing quote
        self.consume_whitespace();
        self.consume_str(">")?;
        let value = match ty.as_str() {
            "tuple" => self.parse_tuple_items()?,
            "int" | "number" | "bool" | "string" => {
                let text = self.read_text()?;
                self.consume_whitespace();
                let scalar = match ty.as_str() {
                    "int" => Value::Int(BigInt::from_str(&text).map_err(|err| {
                        ApexError::new(format!("serde XML int parse error: {}", err))
                    })?),
                    "number" => Value::Number(text.parse::<f64>().map_err(|err| {
                        ApexError::new(format!("serde XML number parse error: {}", err))
                    })?),
                    "bool" => Value::Bool(text == "true"),
                    "string" => Value::String(text),
                    _ => unreachable!(),
                };
                self.consume_str("</value>")?;
                return Ok(scalar);
            }
            _ => return Err(ApexError::new("serde XML type is not supported")),
        };
        self.consume_whitespace();
        self.consume_str("</value>")?;
        Ok(value)
    }

    fn parse_tuple_items(&mut self) -> Result<Value, ApexError> {
        let mut values = Vec::new();
        loop {
            self.consume_whitespace();
            if self.starts_with("</value>") {
                break;
            }
            self.consume_str("<item>")?;
            let item = self.parse_value()?;
            self.consume_whitespace();
            self.consume_str("</item>")?;
            values.push(item);
        }
        Ok(Value::Tuple(values))
    }

    fn read_until(&mut self, needle: u8) -> Result<String, ApexError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != needle {
            self.pos += 1;
        }
        if self.pos == self.input.len() {
            return Err(ApexError::new(
                "serde.from_xml reached the end of the document unexpectedly",
            ));
        }
        let slice = &self.input[start..self.pos];
        let text = std::str::from_utf8(slice)
            .map_err(|_| ApexError::new("serde.from_xml encountered invalid UTF-8"))?;
        Ok(text.to_string())
    }

    fn read_text(&mut self) -> Result<String, ApexError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != b'<' {
            self.pos += 1;
        }
        let slice = &self.input[start..self.pos];
        let text = std::str::from_utf8(slice)
            .map_err(|_| ApexError::new("serde.from_xml encountered invalid UTF-8"))?;
        unescape_xml(text)
    }
}

fn json_to_toml(value: &JsonValue) -> String {
    match value {
        JsonValue::Object(map) => {
            let mut lines = Vec::with_capacity(map.len());
            for (key, entry) in map.iter() {
                lines.push(format!("{} = {}", key, format_toml_value(entry)));
            }
            lines.join("\n")
        }
        _ => format_toml_value(value),
    }
}

fn format_toml_value(value: &JsonValue) -> String {
    match value {
        JsonValue::String(text) => format!("\"{}\"", escape_toml_string(text)),
        JsonValue::Number(num) => num.to_string(),
        JsonValue::Bool(flag) => {
            if *flag {
                "true".into()
            } else {
                "false".into()
            }
        }
        JsonValue::Array(items) => {
            let mut parts = Vec::with_capacity(items.len());
            for item in items {
                parts.push(format_toml_value(item));
            }
            format!("[{}]", parts.join(", "))
        }
        JsonValue::Object(map) => {
            let mut inner = String::new();
            let mut first = true;
            for (key, entry) in map.iter() {
                if !first {
                    inner.push_str(", ");
                }
                first = false;
                let _ = write!(&mut inner, "{} = {}", key, format_toml_value(entry));
            }
            format!("{{{}}}", inner)
        }
        JsonValue::Null => "[]".into(),
    }
}

fn escape_toml_string(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

struct TomlParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> TomlParser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            input: text.as_bytes(),
            pos: 0,
        }
    }

    fn parse_document(&mut self) -> Result<JsonValue, ApexError> {
        self.skip_blank_lines();
        if self.is_eof() {
            return Ok(JsonValue::Object(JsonMap::new()));
        }
        if self.peek_is_key_value_line() {
            let mut map = JsonMap::new();
            while self.peek_is_key_value_line() {
                let key = self.parse_key()?;
                self.skip_spaces();
                self.expect_char('=')?;
                self.skip_spaces();
                let value = self.parse_value()?;
                map.insert(key, value);
                self.consume_line();
                self.skip_blank_lines();
            }
            Ok(JsonValue::Object(map))
        } else {
            let value = self.parse_value()?;
            self.skip_blank_lines();
            if !self.is_eof() {
                return Err(ApexError::new(
                    "serde.from_toml encountered trailing characters",
                ));
            }
            Ok(value)
        }
    }

    fn parse_key(&mut self) -> Result<String, ApexError> {
        self.skip_spaces();
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(ApexError::new(
                "serde.from_toml expected an identifier before '='",
            ));
        }
        let slice = &self.input[start..self.pos];
        Ok(String::from_utf8_lossy(slice).into_owned())
    }

    fn parse_value(&mut self) -> Result<JsonValue, ApexError> {
        self.skip_spaces();
        let ch = self
            .peek_char()
            .ok_or_else(|| ApexError::new("serde.from_toml expected a value"))?;
        match ch {
            b'"' => Ok(JsonValue::String(self.parse_string()?)),
            b'[' => self.parse_array(),
            b'{' => self.parse_inline_table(),
            b't' | b'f' => self.parse_bool(),
            b'+' | b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(ApexError::new(
                "serde.from_toml encountered an unknown token",
            )),
        }
    }

    fn parse_string(&mut self) -> Result<String, ApexError> {
        self.expect_char('"')?;
        let mut output = String::new();
        while let Some(ch) = self.next_char() {
            match ch {
                b'"' => return Ok(output),
                b'\\' => {
                    let esc = self.next_char().ok_or_else(|| {
                        ApexError::new("serde.from_toml found an unfinished escape")
                    })?;
                    output.push(match esc {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        other => other as char,
                    });
                }
                _ => output.push(ch as char),
            }
        }
        Err(ApexError::new("serde.from_toml reached end of string"))
    }

    fn parse_number(&mut self) -> Result<JsonValue, ApexError> {
        let start = self.pos;
        if matches!(self.peek_char(), Some(b'+' | b'-')) {
            self.pos += 1;
        }

        let mut has_digits = false;
        let mut last_was_separator = false;
        while let Some(ch) = self.peek_char() {
            match ch {
                b'0'..=b'9' => {
                    self.pos += 1;
                    has_digits = true;
                    last_was_separator = false;
                }
                b'_' if has_digits && !last_was_separator => {
                    self.pos += 1;
                    last_was_separator = true;
                }
                _ => break,
            }
        }

        if !has_digits {
            return Err(ApexError::new(
                "serde.from_toml number literal must contain digits",
            ));
        }

        let mut has_fraction = false;
        if self.peek_char() == Some(b'.') {
            if matches!(self.input.get(self.pos + 1), Some(b'0'..=b'9')) {
                has_fraction = true;
                self.pos += 1;
                let mut fraction_digits = 0;
                let mut last_sep = false;
                while let Some(ch) = self.peek_char() {
                    match ch {
                        b'0'..=b'9' => {
                            self.pos += 1;
                            fraction_digits += 1;
                            last_sep = false;
                        }
                        b'_' if fraction_digits > 0 && !last_sep => {
                            self.pos += 1;
                            last_sep = true;
                        }
                        _ => break,
                    }
                }
                if fraction_digits == 0 {
                    return Err(ApexError::new(
                        "serde.from_toml float fractional part requires digits",
                    ));
                }
            }
        }

        let mut has_exponent = false;
        if matches!(self.peek_char(), Some(b'e' | b'E')) {
            has_exponent = true;
            self.pos += 1;
            if matches!(self.peek_char(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            let mut exponent_digits = 0;
            let mut last_sep = false;
            while let Some(ch) = self.peek_char() {
                match ch {
                    b'0'..=b'9' => {
                        self.pos += 1;
                        exponent_digits += 1;
                        last_sep = false;
                    }
                    b'_' if exponent_digits > 0 && !last_sep => {
                        self.pos += 1;
                        last_sep = true;
                    }
                    _ => break,
                }
            }
            if exponent_digits == 0 {
                return Err(ApexError::new(
                    "serde.from_toml exponent component requires digits",
                ));
            }
        }

        let slice = &self.input[start..self.pos];
        let text = std::str::from_utf8(slice)
            .map_err(|_| ApexError::new("serde.from_toml number had invalid UTF-8"))?;
        let cleaned: String = text.chars().filter(|ch| *ch != '_').collect();

        if !has_fraction && !has_exponent {
            if let Ok(int) = cleaned.parse::<i64>() {
                return Ok(JsonValue::Number(JsonNumber::from(int)));
            }
            if let Ok(unsigned) = cleaned.parse::<u64>() {
                return Ok(JsonValue::Number(JsonNumber::from(unsigned)));
            }
            return Err(ApexError::new(
                "serde.from_toml integer literal is out of range",
            ));
        }

        let parsed = cleaned
            .parse::<f64>()
            .map_err(|_| ApexError::new("serde.from_toml could not parse the numeric literal"))?;
        JsonNumber::from_f64(parsed)
            .map(JsonValue::Number)
            .ok_or_else(|| ApexError::new("serde.from_toml number is not finite"))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ApexError> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_spaces();
            if self.peek_char() == Some(b']') {
                self.pos += 1;
                break;
            }
            values.push(self.parse_value()?);
            self.skip_spaces();
            match self.peek_char() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => {
                    return Err(ApexError::new(
                        "serde.from_toml arrays require ',' separators",
                    ))
                }
            }
        }
        Ok(JsonValue::Array(values))
    }

    fn parse_inline_table(&mut self) -> Result<JsonValue, ApexError> {
        self.expect_char('{')?;
        let mut map = JsonMap::new();
        loop {
            self.skip_spaces();
            if self.peek_char() == Some(b'}') {
                self.pos += 1;
                break;
            }
            let key = self.parse_key()?;
            self.skip_spaces();
            self.expect_char('=')?;
            self.skip_spaces();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_spaces();
            match self.peek_char() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => {
                    return Err(ApexError::new(
                        "serde.from_toml inline tables require ',' separators",
                    ))
                }
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ApexError> {
        if self.starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(ApexError::new(
                "serde.from_toml encountered an unknown boolean literal",
            ))
        }
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.input[self.pos..].starts_with(needle.as_bytes())
    }

    fn skip_spaces(&mut self) {
        while matches!(self.peek_char(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    fn skip_blank_lines(&mut self) {
        loop {
            self.skip_spaces();
            match self.peek_char() {
                Some(b'#') => self.consume_line(),
                Some(b'\n') => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    fn consume_line(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            if ch == b'\n' {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<u8> {
        if self.pos >= self.input.len() {
            None
        } else {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ApexError> {
        if self.next_char() == Some(expected as u8) {
            Ok(())
        } else {
            Err(ApexError::new(
                "serde.from_toml encountered malformed syntax",
            ))
        }
    }

    fn peek_is_key_value_line(&self) -> bool {
        let mut idx = self.pos;
        let len = self.input.len();
        while idx < len {
            match self.input[idx] {
                b' ' | b'\t' => idx += 1,
                b'#' | b'\n' => return false,
                ch => {
                    if !(ch.is_ascii_alphanumeric() || ch == b'_') {
                        return false;
                    }
                    break;
                }
            }
        }
        while idx < len && self.input[idx] != b'\n' {
            if self.input[idx] == b'=' {
                return true;
            }
            idx += 1;
        }
        false
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn encode_base64_bytes(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(((bytes.len() + 2) / 3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let combined = (a << 16) | (b << 8) | c;
        output.push(BASE64_TABLE[((combined >> 18) & 0x3f) as usize] as char);
        output.push(BASE64_TABLE[((combined >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(BASE64_TABLE[((combined >> 6) & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(BASE64_TABLE[(combined & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

fn decode_base64_string(text: &str) -> Result<Vec<u8>, ApexError> {
    let cleaned: Vec<u8> = text.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if cleaned.len() % 4 != 0 {
        return Err(ApexError::new(
            "serde.from_base64 expected groups of 4 characters",
        ));
    }
    let mut output = Vec::with_capacity(cleaned.len() / 4 * 3);
    let mut idx = 0;
    while idx < cleaned.len() {
        let chunk = &cleaned[idx..idx + 4];
        let mut values = [0u8; 4];
        let mut padding = 0;
        for (pos, ch) in chunk.iter().enumerate() {
            if *ch == b'=' {
                values[pos] = 0;
                padding += 1;
            } else if let Some(val) = base64_value(*ch) {
                values[pos] = val;
            } else {
                return Err(ApexError::new("serde.from_base64 found invalid characters"));
            }
        }
        let combined = ((values[0] as u32) << 18)
            | ((values[1] as u32) << 12)
            | ((values[2] as u32) << 6)
            | values[3] as u32;
        output.push(((combined >> 16) & 0xff) as u8);
        if padding < 2 {
            output.push(((combined >> 8) & 0xff) as u8);
        }
        if padding == 0 {
            output.push((combined & 0xff) as u8);
        }
        idx += 4;
    }
    Ok(output)
}

fn base64_value(ch: u8) -> Option<u8> {
    match ch {
        b'A'..=b'Z' => Some(ch - b'A'),
        b'a'..=b'z' => Some(ch - b'a' + 26),
        b'0'..=b'9' => Some(ch - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip_preserves_tuples() {
        let value = Value::Tuple(vec![
            Value::Tuple(vec![
                Value::String("name".into()),
                Value::String("apex".into()),
            ]),
            Value::Tuple(vec![Value::String("value".into()), Value::Int(42.into())]),
        ]);
        let json = to_json(&[value.clone()]).expect("serialize");
        let rebuilt = from_json(&[json]).expect("deserialize");
        assert_eq!(value, rebuilt);
    }

    #[test]
    fn csv_round_trips_basic_rows() {
        let rows = Value::Tuple(vec![
            Value::Tuple(vec![Value::Int(1.into()), Value::String("alpha".into())]),
            Value::Tuple(vec![Value::Bool(true), Value::Number(3.5)]),
        ]);

        let csv = to_csv(&[rows.clone()]).expect("to_csv");
        let csv_text = match csv {
            Value::String(text) => text,
            _ => panic!("expected string"),
        };

        assert!(csv_text.contains("alpha"));

        let parsed = from_csv(&[Value::String(csv_text)]).expect("from_csv");
        if let Value::Tuple(lines) = parsed {
            assert_eq!(lines.len(), 2);
            if let Value::Tuple(first) = &lines[0] {
                assert_eq!(first[0], Value::Int(1.into()));
                assert_eq!(first[1], Value::String("alpha".into()));
            } else {
                panic!("expected tuple row");
            }
        } else {
            panic!("expected tuple of rows");
        }
    }

    #[test]
    fn yaml_and_bytes_share_representation() {
        let value = Value::Tuple(vec![Value::Int(1.into()), Value::Number(2.5)]);
        let yaml = to_yaml(&[value.clone()]).expect("yaml");
        let rebuilt = from_yaml(&[yaml.clone()]).expect("from yaml");
        assert_eq!(value, rebuilt);
        let bytes = to_bytes(&[value.clone()]).expect("to bytes");
        let from_bytes_value = from_bytes(&[bytes]).expect("from bytes");
        assert_eq!(value, from_bytes_value);
    }

    #[test]
    fn xml_round_trip_handles_nested_items() {
        let value = Value::Tuple(vec![
            Value::Tuple(vec![Value::String("k".into()), Value::Bool(true)]),
            Value::Tuple(vec![
                Value::String("list".into()),
                Value::Tuple(vec![Value::Int(1.into())]),
            ]),
        ]);
        let xml = to_xml(&[value.clone()]).expect("xml");
        let rebuilt = from_xml(&[xml]).expect("from xml");
        assert_eq!(value, rebuilt);
    }

    #[test]
    fn toml_round_trip_preserves_numbers() {
        let value = Value::Tuple(vec![
            Value::Tuple(vec![Value::String("answer".into()), Value::Int(42.into())]),
            Value::Tuple(vec![Value::String("pi".into()), Value::Number(3.1415)]),
        ]);
        let toml = to_toml(&[value.clone()]).expect("toml");
        let rebuilt = from_toml(&[toml]).expect("from toml");
        assert_eq!(value, rebuilt);
        let yaml = to_yaml(&[value.clone()]).expect("yaml");
        let yaml_back = from_yaml(&[yaml]).expect("from yaml");
        assert_eq!(value, yaml_back);
    }

    #[test]
    fn base64_round_trip_matches_json_bytes() {
        let value = Value::Tuple(vec![Value::Int(5.into()), Value::Number(2.25)]);
        let text = to_base64(&[value.clone()]).expect("to base64");
        let rebuilt = from_base64(&[text]).expect("from base64");
        assert_eq!(value, rebuilt);
    }

    #[test]
    fn bin_round_trip_stays_compact_and_correct() {
        let value = Value::Tuple(vec![
            Value::Tuple(vec![Value::String("key".into()), Value::Bool(true)]),
            Value::Tuple(vec![Value::String("count".into()), Value::Int(7.into())]),
        ]);
        let encoded = to_bin(&[value.clone()]).expect("to bin");
        let rebuilt = from_bin(&[encoded]).expect("from bin");
        assert_eq!(value, rebuilt);
    }

    #[test]
    fn toml_parser_handles_signed_and_separated_integers() {
        let document = r#"
# leading comment
pos = +42
neg = -7
big = 4_294_967_296
inline = { inner = 9_9, nested = { deep = -3 } } # trailing comment
float = 3.5
exp = 5e2
"#;
        let parsed = from_toml(&[Value::String(document.into())]).expect("parse toml");
        let entries = match parsed {
            Value::Tuple(items) => items,
            other => panic!("expected table tuple, got {:?}", other),
        };

        assert_int_entry(&entries, "pos", 42);
        assert_int_entry(&entries, "neg", -7);
        match expect_entry(&entries, "big") {
            Value::Int(num) => assert_eq!(num, &BigInt::from(4_294_967_296u64)),
            other => panic!("expected integer for big, got {:?}", other),
        }

        if let Value::Tuple(inline) = expect_entry(&entries, "inline") {
            assert_int_entry(inline, "inner", 99);
            if let Value::Tuple(nested) = expect_entry(inline, "nested") {
                assert_int_entry(nested, "deep", -3);
            } else {
                panic!("nested inline table should deserialize to tuple entries");
            }
        } else {
            panic!("inline table should deserialize to tuple entries");
        }

        match expect_entry(&entries, "float") {
            Value::Number(num) => assert!((num - 3.5).abs() < 1e-9),
            other => panic!("expected float for float field, got {:?}", other),
        }
        match expect_entry(&entries, "exp") {
            Value::Number(num) => assert!((num - 500.0).abs() < 1e-9),
            other => panic!("expected float for exp field, got {:?}", other),
        }
    }

    fn expect_entry<'a>(entries: &'a [Value], key: &str) -> &'a Value {
        entries
            .iter()
            .find_map(|entry| match entry {
                Value::Tuple(pair) if pair.len() == 2 => match &pair[0] {
                    Value::String(name) if name == key => Some(&pair[1]),
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing key {key}"))
    }

    fn assert_int_entry(entries: &[Value], key: &str, expected: i64) {
        match expect_entry(entries, key) {
            Value::Int(num) => assert_eq!(num, &BigInt::from(expected)),
            other => panic!("expected integer for {key}, got {:?}", other),
        }
    }
}
