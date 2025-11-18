use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

pub fn expect_int_arg(args: &[Value], index: usize, name: &str) -> Result<BigInt, ApexError> {
    match args.get(index) {
        Some(Value::Int(value)) => Ok(value.clone()),
        _ => Err(ApexError::new(format!(
            "{} expects an integer argument at position {}",
            name,
            index + 1
        ))),
    }
}

pub fn expect_usize_arg(args: &[Value], index: usize, name: &str) -> Result<usize, ApexError> {
    let value = expect_int_arg(args, index, name)?;
    value.to_usize().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument at position {} is too large for usize",
            name,
            index + 1
        ))
    })
}

pub fn expect_u32_arg(args: &[Value], index: usize, name: &str) -> Result<u32, ApexError> {
    let value = expect_int_arg(args, index, name)?;
    value.to_u32().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument at position {} is too large for u32",
            name,
            index + 1
        ))
    })
}

pub fn expect_u64_arg(args: &[Value], index: usize, name: &str) -> Result<u64, ApexError> {
    let value = expect_int_arg(args, index, name)?;
    value.to_u64().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument at position {} is too large for u64",
            name,
            index + 1
        ))
    })
}

pub fn expect_u128_arg(args: &[Value], index: usize, name: &str) -> Result<u128, ApexError> {
    let value = expect_int_arg(args, index, name)?;
    value.to_u128().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument at position {} is too large for u128",
            name,
            index + 1
        ))
    })
}

pub fn expect_bool_arg(args: &[Value], index: usize, name: &str) -> Result<bool, ApexError> {
    match args.get(index) {
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(ApexError::new(format!(
            "{} expects a boolean argument at position {}",
            name,
            index + 1
        ))),
    }
}

pub fn expect_string_arg(args: &[Value], index: usize, name: &str) -> Result<String, ApexError> {
    match args.get(index) {
        Some(Value::String(text)) => Ok(text.clone()),
        _ => Err(ApexError::new(format!(
            "{} expects a string argument at position {}",
            name,
            index + 1
        ))),
    }
}

pub fn expect_tuple_arg(args: &[Value], index: usize, name: &str) -> Result<Vec<Value>, ApexError> {
    match args.get(index) {
        Some(Value::Tuple(values)) => Ok(values.clone()),
        _ => Err(ApexError::new(format!(
            "{} expects a tuple argument at position {}",
            name,
            index + 1
        ))),
    }
}

pub fn expect_number_arg(args: &[Value], index: usize, name: &str) -> Result<f64, ApexError> {
    match args.get(index) {
        Some(Value::Number(value)) => Ok(*value),
        Some(Value::Int(value)) => value.to_f64().ok_or_else(|| {
            ApexError::new(format!(
                "{} argument at position {} is too large for f64",
                name,
                index + 1
            ))
        }),
        _ => Err(ApexError::new(format!(
            "{} expects a numeric argument at position {}",
            name,
            index + 1
        ))),
    }
}
