use std::collections::HashMap;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;
use num_bigint::BigInt;

use super::support::expect_string_arg;
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();
    functions.insert(
        "inline".to_string(),
        NativeCallable::new("asm::inline", inline_block),
    );
    registry.register_module("asm", functions);
}

fn inline_block(args: &[Value]) -> Result<Value, ApexError> {
    let source = expect_string_arg(args, 0, "asm.inline")?;
    let mut registers = vec![BigInt::from(0); 4];
    for line in source.split(';') {
        let instruction = line.trim();
        if instruction.is_empty() {
            continue;
        }
        execute_instruction(instruction, &mut registers)?;
    }
    Ok(Value::Tuple(
        registers.into_iter().map(Value::Int).collect::<Vec<_>>(),
    ))
}

fn execute_instruction(line: &str, registers: &mut [BigInt]) -> Result<(), ApexError> {
    let mut parts = line.split_whitespace();
    let opcode = parts
        .next()
        .ok_or_else(|| ApexError::new("Inline assembly instruction is empty"))?
        .to_lowercase();
    let operands_text = line[opcode.len()..].trim();
    let operands: Vec<&str> = if operands_text.is_empty() {
        Vec::new()
    } else {
        operands_text
            .split(',')
            .map(|operand| operand.trim())
            .filter(|operand| !operand.is_empty())
            .collect()
    };

    match opcode.as_str() {
        "nop" => {
            if !operands.is_empty() {
                return Err(ApexError::new("nop does not accept operands"));
            }
        }
        "mov" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "mov")?;
            registers[dst] = value;
        }
        "add" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "add")?;
            registers[dst] += value;
        }
        "sub" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "sub")?;
            registers[dst] -= value;
        }
        "mul" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "mul")?;
            registers[dst] *= value;
        }
        "and" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "and")?;
            registers[dst] &= value;
        }
        "or" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "or")?;
            registers[dst] |= value;
        }
        "xor" => {
            let (dst, value) = expect_binary_operands(&operands, registers, "xor")?;
            registers[dst] ^= value;
        }
        other => {
            return Err(ApexError::new(format!("Unsupported opcode '{}'", other)));
        }
    }

    Ok(())
}

fn expect_binary_operands(
    operands: &[&str],
    registers: &[BigInt],
    name: &str,
) -> Result<(usize, BigInt), ApexError> {
    if operands.len() != 2 {
        return Err(ApexError::new(format!(
            "{} expects exactly two operands",
            name
        )));
    }
    let dst = parse_register(operands[0])?;
    if dst >= registers.len() {
        return Err(ApexError::new("Register index out of range"));
    }
    let value = parse_operand(operands[1], registers)?;
    Ok((dst, value))
}

fn parse_register(token: &str) -> Result<usize, ApexError> {
    if !token.starts_with('r') {
        return Err(ApexError::new(format!(
            "Expected register operand but found '{}'",
            token
        )));
    }
    token[1..]
        .parse::<usize>()
        .map_err(|_| ApexError::new(format!("Unable to parse register '{}'", token)))
}

fn parse_operand(token: &str, registers: &[BigInt]) -> Result<BigInt, ApexError> {
    if token.starts_with('r') {
        let index = parse_register(token)?;
        return registers
            .get(index)
            .cloned()
            .ok_or_else(|| ApexError::new(format!("Register '{}' is out of range", token)));
    }
    if let Some(hex) = token.strip_prefix("0x") {
        return BigInt::parse_bytes(hex.as_bytes(), 16).ok_or_else(|| {
            ApexError::new(format!("Unable to parse hexadecimal literal '{}'", token))
        });
    }
    BigInt::parse_bytes(token.as_bytes(), 10)
        .ok_or_else(|| ApexError::new(format!("Unable to parse literal '{}'", token)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_inline_program() {
        let program = Value::String("mov r0, 5; add r0, r0; xor r1, 0x3".to_string());
        let result = inline_block(&[program]).expect("executes");
        if let Value::Tuple(values) = result {
            assert_eq!(values.len(), 4);
            assert_eq!(values[0], Value::Int(10.into()));
            assert_eq!(values[1], Value::Int(3.into()));
        } else {
            panic!("expected tuple");
        }
    }
}
