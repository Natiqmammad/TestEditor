use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use uuid::Uuid;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{expect_string_arg, expect_tuple_arg};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("filesystem::", $name), $func),
            );
        };
    }

    add!(&mut functions, "read_text", read_text);
    add!(&mut functions, "write_text", write_text);
    add!(&mut functions, "append_text", append_text);
    add!(&mut functions, "file_exists", file_exists);
    add!(&mut functions, "read_bytes", read_bytes);
    add!(&mut functions, "write_bytes", write_bytes);
    add!(&mut functions, "delete", delete_path);
    add!(&mut functions, "list_dir", list_dir);
    registry.register_module("filesystem", functions);
}

fn read_text(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.read_text")?;
    let contents = fs::read_to_string(&path)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", path, err)))?;
    Ok(Value::String(contents))
}

fn write_text(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.write_text")?;
    let contents = expect_string_arg(args, 1, "filesystem.write_text")?;
    fs::write(&path, contents)
        .map_err(|err| ApexError::new(format!("Failed to write '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn append_text(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.append_text")?;
    let contents = expect_string_arg(args, 1, "filesystem.append_text")?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| ApexError::new(format!("Failed to open '{}': {}", path, err)))?;
    file.write_all(contents.as_bytes())
        .map_err(|err| ApexError::new(format!("Failed to append to '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn file_exists(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.file_exists")?;
    Ok(Value::Bool(Path::new(&path).exists()))
}

fn read_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.read_bytes")?;
    let bytes = fs::read(&path)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", path, err)))?;
    Ok(Value::Tuple(
        bytes
            .into_iter()
            .map(|byte| Value::Int(BigInt::from(byte)))
            .collect(),
    ))
}

fn write_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.write_bytes")?;
    let tuple = expect_tuple_arg(args, 1, "filesystem.write_bytes")?;
    let mut bytes = Vec::with_capacity(tuple.len());
    for value in tuple {
        match value {
            Value::Int(v) => {
                let byte = v.to_u32().ok_or_else(|| {
                    ApexError::new("filesystem.write_bytes expects byte values in [0, 255]")
                })?;
                if byte > 255 {
                    return Err(ApexError::new(
                        "filesystem.write_bytes expects byte values in [0, 255]",
                    ));
                }
                bytes.push(byte as u8);
            }
            _ => {
                return Err(ApexError::new(
                    "filesystem.write_bytes expects tuple of integers",
                ))
            }
        }
    }
    fs::write(&path, bytes)
        .map_err(|err| ApexError::new(format!("Failed to write '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn delete_path(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.delete")?;
    if Path::new(&path).is_dir() {
        fs::remove_dir_all(&path).map_err(|err| {
            ApexError::new(format!("Failed to delete directory '{}': {}", path, err))
        })?;
    } else if Path::new(&path).exists() {
        fs::remove_file(&path)
            .map_err(|err| ApexError::new(format!("Failed to delete file '{}': {}", path, err)))?;
    } else {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(true))
}

fn list_dir(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "filesystem.list_dir")?;
    let entries = fs::read_dir(&path)
        .map_err(|err| ApexError::new(format!("Failed to list '{}': {}", path, err)))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|err| ApexError::new(format!("Failed to read entry: {}", err)))?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(Value::String(name.to_string()));
        }
    }
    Ok(Value::Tuple(names))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn writes_and_reads_text() {
        let path = tempfile_path("text");
        write_text(&[Value::String(path.clone()), Value::String("hello".into())]).unwrap();
        append_text(&[Value::String(path.clone()), Value::String(" world".into())]).unwrap();
        let contents = read_text(&[Value::String(path.clone())]).unwrap();
        assert_eq!(contents, Value::String("hello world".into()));
        assert_eq!(
            file_exists(&[Value::String(path.clone())]).unwrap(),
            Value::Bool(true)
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn writes_and_reads_bytes() {
        let path = tempfile_path("bytes");
        let tuple = Value::Tuple(vec![Value::Int(1.into()), Value::Int(2.into())]);
        write_bytes(&[Value::String(path.clone()), tuple]).unwrap();
        let bytes = read_bytes(&[Value::String(path.clone())]).unwrap();
        if let Value::Tuple(values) = bytes {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Int(1.into()));
            assert_eq!(values[1], Value::Int(2.into()));
        } else {
            panic!("expected tuple");
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn lists_and_deletes_paths() {
        let dir = tempfile_path("dir");
        fs::create_dir_all(&dir).unwrap();
        let file_path = format!("{}/entry.txt", dir);
        write_text(&[
            Value::String(file_path.clone()),
            Value::String("data".into()),
        ])
        .unwrap();
        let listing = list_dir(&[Value::String(dir.clone())]).unwrap();
        if let Value::Tuple(values) = listing {
            assert_eq!(values.len(), 1);
        } else {
            panic!("expected tuple");
        }
        assert_eq!(
            delete_path(&[Value::String(file_path.clone())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            delete_path(&[Value::String(file_path.clone())]).unwrap(),
            Value::Bool(false)
        );
        fs::remove_dir_all(dir).unwrap();
    }

    fn tempfile_path(label: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("apex_fs_{}_{}.txt", label, Uuid::new_v4()));
        path.to_string_lossy().to_string()
    }
}
