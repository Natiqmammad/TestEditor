use std::collections::HashMap;

#[cfg(test)]
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive};

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("math::", $name), $func),
            );
        };
    }

    add!(functions, "pi", pi);
    add!(functions, "e", e);
    add!(functions, "abs", abs_fn);
    add!(functions, "sqrt", sqrt);
    add!(functions, "cbrt", cbrt);
    add!(functions, "hypot", hypot);
    add!(functions, "pow", pow);
    add!(functions, "exp", exp);
    add!(functions, "ln", ln);
    add!(functions, "log", log);
    add!(functions, "sin", sin_fn);
    add!(functions, "cos", cos_fn);
    add!(functions, "tan", tan_fn);

    registry.register_module("math", functions);
}

fn ensure_len(args: &[Value], expected: usize, name: &str) -> Result<(), ApexError> {
    if args.len() != expected {
        return Err(ApexError::new(format!(
            "{} expected {} argument(s) but received {}",
            name,
            expected,
            args.len()
        )));
    }
    Ok(())
}

fn expect_numeric(args: &[Value], index: usize, name: &str) -> Result<f64, ApexError> {
    match args.get(index) {
        Some(Value::Number(v)) => Ok(*v),
        Some(Value::Int(v)) => v.to_f64().ok_or_else(|| {
            ApexError::new(format!(
                "{} argument {} is too large to convert to f64",
                name,
                index + 1
            ))
        }),
        _ => Err(ApexError::new(format!(
            "{} expects numeric argument at position {}",
            name,
            index + 1
        ))),
    }
}

fn pi(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Number(std::f64::consts::PI))
}

fn e(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Number(std::f64::consts::E))
}

fn abs_fn(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "abs")?;
    match args.get(0) {
        Some(Value::Int(v)) => Ok(Value::Int(v.abs())),
        Some(Value::Number(v)) => Ok(Value::Number(v.abs())),
        _ => Err(ApexError::new(
            "abs expects an integer or floating-point argument",
        )),
    }
}

fn sqrt(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "sqrt")?;
    let value = expect_numeric(args, 0, "sqrt")?;
    if value < 0.0 {
        return Err(ApexError::new("sqrt requires a non-negative input"));
    }
    Ok(Value::Number(value.sqrt()))
}

fn cbrt(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "cbrt")?;
    let value = expect_numeric(args, 0, "cbrt")?;
    Ok(Value::Number(value.cbrt()))
}

fn hypot(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "hypot")?;
    let a = expect_numeric(args, 0, "hypot")?;
    let b = expect_numeric(args, 1, "hypot")?;
    Ok(Value::Number(a.hypot(b)))
}

fn pow(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "pow")?;
    let base = expect_numeric(args, 0, "pow")?;
    let exp = expect_numeric(args, 1, "pow")?;
    Ok(Value::Number(base.powf(exp)))
}

fn exp(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "exp")?;
    let value = expect_numeric(args, 0, "exp")?;
    Ok(Value::Number(value.exp()))
}

fn ln(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "ln")?;
    let value = expect_numeric(args, 0, "ln")?;
    if value <= 0.0 {
        return Err(ApexError::new("ln requires a positive input"));
    }
    Ok(Value::Number(value.ln()))
}

fn log(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "log")?;
    let value = expect_numeric(args, 0, "log")?;
    let base = expect_numeric(args, 1, "log")?;
    if value <= 0.0 || base <= 0.0 || (base - 1.0).abs() < f64::EPSILON {
        return Err(ApexError::new("log requires positive inputs and base != 1"));
    }
    Ok(Value::Number(value.log(base)))
}

fn sin_fn(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "sin")?;
    let value = expect_numeric(args, 0, "sin")?;
    Ok(Value::Number(value.sin()))
}

fn cos_fn(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "cos")?;
    let value = expect_numeric(args, 0, "cos")?;
    Ok(Value::Number(value.cos()))
}

fn tan_fn(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "tan")?;
    let value = expect_numeric(args, 0, "tan")?;
    Ok(Value::Number(value.tan()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(n: i64) -> Value {
        Value::Int(BigInt::from(n))
    }

    fn num(n: f64) -> Value {
        Value::Number(n)
    }

    #[test]
    fn sqrt_handles_bigints() {
        let result = sqrt(&[int(144)]).unwrap();
        assert_eq!(result, num(12.0));
    }

    #[test]
    fn pow_handles_float_inputs() {
        let result = pow(&[num(2.0), num(10.0)]).unwrap();
        assert_eq!(result, num(1024.0));
    }

    #[test]
    fn abs_handles_ints_and_numbers() {
        let int_result = abs_fn(&[int(-42)]).unwrap();
        assert_eq!(int_result, int(42));

        let float_result = abs_fn(&[num(-3.5)]).unwrap();
        assert_eq!(float_result, num(3.5));

        let err = abs_fn(&[Value::Bool(true)]);
        assert!(err.is_err());
    }

    #[test]
    fn trig_functions_respect_radians() {
        let sin_val = sin_fn(&[num(std::f64::consts::PI / 2.0)]).unwrap();
        assert!(
            (if let Value::Number(v) = sin_val {
                v
            } else {
                0.0
            } - 1.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn log_validates_domain() {
        let err = log(&[num(-1.0), num(10.0)]).unwrap_err();
        assert!(err.message().contains("log requires"));
    }
}
