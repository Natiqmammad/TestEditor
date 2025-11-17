use std::collections::HashMap;

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($name:literal, $func:ident) => {
            functions.insert(
                $name.to_string(),
                NativeCallable::new(concat!("fractions::", $name), $func),
            );
        };
    }

    add!("fraction_reduce", fraction_reduce);
    add!("fraction_add", fraction_add);
    add!("fraction_subtract", fraction_subtract);
    add!("fraction_multiply", fraction_multiply);
    add!("fraction_divide", fraction_divide);
    add!("fraction_reciprocal", fraction_reciprocal);
    add!("fraction_mediant", fraction_mediant);
    add!("fraction_farey_neighbors", fraction_farey_neighbors);
    add!("fraction_egyptian_terms", fraction_egyptian_terms);
    add!("fraction_is_proper", fraction_is_proper);
    add!("fraction_is_unit", fraction_is_unit);
    add!("fraction_is_terminating", fraction_is_terminating);
    add!("fraction_period_length", fraction_period_length);
    add!("fraction_to_decimal", fraction_to_decimal);
    add!("fraction_numerator", fraction_numerator);
    add!("fraction_denominator", fraction_denominator);
    add!("decimal_to_fraction", decimal_to_fraction);

    registry.register_module("fractions", functions);
}

fn expect_length(args: &[Value], expected: usize) -> Result<(), ApexError> {
    if args.len() != expected {
        return Err(ApexError::new(format!(
            "Expected {} arguments but received {}",
            expected,
            args.len()
        )));
    }
    Ok(())
}

fn expect_int(value: &Value, name: &str) -> Result<BigInt, ApexError> {
    match value {
        Value::Int(v) => Ok(v.clone()),
        _ => Err(ApexError::new(format!("{} expects an integer", name))),
    }
}

fn expect_positive_limit(value: &Value, name: &str) -> Result<BigInt, ApexError> {
    let int_value = expect_int(value, name)?;
    if int_value <= BigInt::zero() {
        return Err(ApexError::new(format!("{} must be positive", name)));
    }
    Ok(int_value)
}

fn normalize_fraction(
    numerator: BigInt,
    denominator: BigInt,
) -> Result<(BigInt, BigInt), ApexError> {
    if denominator.is_zero() {
        return Err(ApexError::new("Fraction denominator cannot be zero"));
    }
    let mut num = numerator;
    let mut den = denominator;
    if den.is_negative() {
        num = -num;
        den = -den;
    }
    let gcd = num.gcd(&den);
    if !gcd.is_zero() {
        num /= &gcd;
        den /= gcd;
    }
    Ok((num, den))
}

fn tuple_from_fraction(num: BigInt, den: BigInt) -> Value {
    Value::Tuple(vec![Value::Int(num), Value::Int(den)])
}

fn fraction_from_args(args: &[Value], offset: usize) -> Result<(BigInt, BigInt), ApexError> {
    let numerator = expect_int(&args[offset], "numerator")?;
    let denominator = expect_int(&args[offset + 1], "denominator")?;
    normalize_fraction(numerator, denominator)
}

fn fraction_from_tuple(value: &Value) -> Result<(BigInt, BigInt), ApexError> {
    match value {
        Value::Tuple(values) if values.len() == 2 => {
            let numerator = match &values[0] {
                Value::Int(v) => v.clone(),
                _ => {
                    return Err(ApexError::new(
                        "Fraction tuples must store integer numerators",
                    ))
                }
            };
            let denominator = match &values[1] {
                Value::Int(v) => v.clone(),
                _ => {
                    return Err(ApexError::new(
                        "Fraction tuples must store integer denominators",
                    ))
                }
            };
            normalize_fraction(numerator, denominator)
        }
        Value::Tuple(_) => Err(ApexError::new(
            "Fraction tuples must contain exactly two entries",
        )),
        _ => Err(ApexError::new("Expected a fraction tuple")),
    }
}

fn parse_single_fraction(args: &[Value]) -> Result<(BigInt, BigInt), ApexError> {
    if args.len() == 1 {
        return fraction_from_tuple(&args[0]);
    }
    expect_length(args, 2)?;
    fraction_from_args(args, 0)
}

fn parse_two_fractions(args: &[Value]) -> Result<((BigInt, BigInt), (BigInt, BigInt)), ApexError> {
    if args.len() == 2 {
        let first = fraction_from_tuple(&args[0])?;
        let second = fraction_from_tuple(&args[1])?;
        return Ok((first, second));
    }
    if args.len() == 4 {
        let first = fraction_from_args(args, 0)?;
        let second = fraction_from_args(args, 2)?;
        return Ok((first, second));
    }
    Err(ApexError::new(
        "Expected either two fraction tuples or four integer arguments",
    ))
}

fn fraction_reduce(args: &[Value]) -> Result<Value, ApexError> {
    expect_length(args, 2)?;
    let (num, den) = fraction_from_args(args, 0)?;
    Ok(tuple_from_fraction(num, den))
}

fn fraction_add(args: &[Value]) -> Result<Value, ApexError> {
    let ((a_num, a_den), (b_num, b_den)) = parse_two_fractions(args)?;
    let numerator = a_num * &b_den + b_num * &a_den;
    let denominator = a_den * b_den;
    let (num, den) = normalize_fraction(numerator, denominator)?;
    Ok(tuple_from_fraction(num, den))
}

fn fraction_subtract(args: &[Value]) -> Result<Value, ApexError> {
    let ((a_num, a_den), (b_num, b_den)) = parse_two_fractions(args)?;
    let numerator = a_num * &b_den - b_num * &a_den;
    let denominator = a_den * b_den;
    let (num, den) = normalize_fraction(numerator, denominator)?;
    Ok(tuple_from_fraction(num, den))
}

fn fraction_multiply(args: &[Value]) -> Result<Value, ApexError> {
    let ((a_num, a_den), (b_num, b_den)) = parse_two_fractions(args)?;
    let (num, den) = normalize_fraction(a_num * b_num, a_den * b_den)?;
    Ok(tuple_from_fraction(num, den))
}

fn fraction_divide(args: &[Value]) -> Result<Value, ApexError> {
    let ((a_num, a_den), (b_num, b_den)) = parse_two_fractions(args)?;
    if b_num.is_zero() {
        return Err(ApexError::new("Cannot divide by a zero numerator"));
    }
    let (num, den) = normalize_fraction(a_num * b_den.clone(), a_den * b_num)?;
    Ok(tuple_from_fraction(num, den))
}

fn fraction_reciprocal(args: &[Value]) -> Result<Value, ApexError> {
    let (num, den) = parse_single_fraction(args)?;
    if num.is_zero() {
        return Err(ApexError::new("Zero has no reciprocal"));
    }
    Ok(tuple_from_fraction(den, num))
}

fn fraction_mediant(args: &[Value]) -> Result<Value, ApexError> {
    let ((a_num, a_den), (b_num, b_den)) = parse_two_fractions(args)?;
    let (num, den) = normalize_fraction(a_num + b_num, a_den + b_den)?;
    Ok(tuple_from_fraction(num, den))
}

fn fraction_farey_neighbors(args: &[Value]) -> Result<Value, ApexError> {
    let ((a_num, a_den), (b_num, b_den)) = parse_two_fractions(args)?;
    let determinant = &a_num * &b_den - &b_num * &a_den;
    Ok(Value::Bool(determinant.abs() == BigInt::one()))
}

fn fraction_egyptian_terms(args: &[Value]) -> Result<Value, ApexError> {
    let (mut num, mut den) = parse_single_fraction(args)?;
    if num.is_negative() || den.is_negative() {
        return Err(ApexError::new(
            "Egyptian fraction decomposition expects positive fractions",
        ));
    }
    if num >= den {
        return Err(ApexError::new(
            "Egyptian fraction decomposition only applies to proper fractions",
        ));
    }
    let mut terms = Vec::new();
    let mut iterations = 0;
    while !num.is_zero() {
        iterations += 1;
        if iterations > 64 {
            return Err(ApexError::new(
                "Egyptian fraction decomposition exceeded 64 terms",
            ));
        }
        let unit_den = (&den + &num - BigInt::one()).div_floor(&num);
        terms.push(Value::Int(unit_den.clone()));
        let new_num = num * unit_den.clone() - den.clone();
        let new_den = den * unit_den;
        let (reduced_num, reduced_den) = normalize_fraction(new_num, new_den)?;
        num = reduced_num;
        den = reduced_den;
    }
    Ok(Value::Tuple(terms))
}

fn fraction_is_proper(args: &[Value]) -> Result<Value, ApexError> {
    let (num, den) = parse_single_fraction(args)?;
    Ok(Value::Bool(num.abs() < den))
}

fn fraction_is_unit(args: &[Value]) -> Result<Value, ApexError> {
    let (num, den) = parse_single_fraction(args)?;
    Ok(Value::Bool(
        num.abs() == BigInt::one() && den >= BigInt::one(),
    ))
}

fn fraction_is_terminating(args: &[Value]) -> Result<Value, ApexError> {
    let (_, mut den) = parse_single_fraction(args)?;
    den = den.abs();
    while (&den % 2) == BigInt::zero() {
        den /= 2;
    }
    while (&den % 5) == BigInt::zero() {
        den /= 5;
    }
    Ok(Value::Bool(den == BigInt::one()))
}

fn fraction_period_length(args: &[Value]) -> Result<Value, ApexError> {
    let (_, mut den) = parse_single_fraction(args)?;
    den = den.abs();
    while (&den % 2) == BigInt::zero() {
        den /= 2;
    }
    while (&den % 5) == BigInt::zero() {
        den /= 5;
    }
    if den == BigInt::one() {
        return Ok(Value::Int(BigInt::zero()));
    }
    let ten = BigInt::from(10);
    let mut remainder = ten.mod_floor(&den);
    let mut period = BigInt::one();
    let mut guard = 0u64;
    while remainder != BigInt::one() {
        remainder = (remainder * &ten).mod_floor(&den);
        period += BigInt::one();
        guard += 1;
        if guard > 100_000 {
            return Err(ApexError::new(
                "Repeating-period search exceeded 100000 iterations",
            ));
        }
    }
    Ok(Value::Int(period))
}

fn fraction_to_decimal(args: &[Value]) -> Result<Value, ApexError> {
    let (num, den) = parse_single_fraction(args)?;
    let rational = BigRational::new(num, den);
    let value = rational
        .to_f64()
        .ok_or_else(|| ApexError::new("Fraction is too large to convert into a finite decimal"))?;
    Ok(Value::Number(value))
}

fn fraction_numerator(args: &[Value]) -> Result<Value, ApexError> {
    expect_length(args, 1)?;
    let (num, _) = fraction_from_tuple(&args[0])?;
    Ok(Value::Int(num))
}

fn fraction_denominator(args: &[Value]) -> Result<Value, ApexError> {
    expect_length(args, 1)?;
    let (_, den) = fraction_from_tuple(&args[0])?;
    Ok(Value::Int(den))
}

fn decimal_to_fraction(args: &[Value]) -> Result<Value, ApexError> {
    expect_length(args, 2)?;
    let limit = expect_positive_limit(&args[1], "max denominator")?;
    let limit_u64 = limit
        .to_u64()
        .ok_or_else(|| ApexError::new("max denominator is too large"))?;
    match &args[0] {
        Value::Int(value) => Ok(tuple_from_fraction(value.clone(), BigInt::one())),
        Value::Number(value) => {
            if !value.is_finite() {
                return Err(ApexError::new(
                    "Only finite decimal values can be converted to fractions",
                ));
            }
            let (num, den) = approximate_fraction(*value, limit_u64)?;
            Ok(tuple_from_fraction(num, den))
        }
        _ => Err(ApexError::new(
            "decimal_to_fraction expects either an Int or Number value",
        )),
    }
}

fn approximate_fraction(value: f64, max_den: u64) -> Result<(BigInt, BigInt), ApexError> {
    if max_den == 0 {
        return Err(ApexError::new("max denominator must be positive"));
    }
    if value == 0.0 {
        return Ok((BigInt::zero(), BigInt::one()));
    }
    let sign = if value.is_sign_negative() { -1 } else { 1 };
    let mut x = value.abs();
    let mut h0 = BigInt::zero();
    let mut h1 = BigInt::one();
    let mut k0 = BigInt::one();
    let mut k1 = BigInt::zero();
    let limit = BigInt::from(max_den);
    loop {
        let a = x.floor();
        if a > (i128::MAX as f64) {
            return Err(ApexError::new(
                "Decimal magnitude is too large for fraction approximation",
            ));
        }
        let a_big = BigInt::from(a as i128);
        let h2 = &a_big * &h1 + &h0;
        let k2 = &a_big * &k1 + &k0;
        if k2 > limit {
            break;
        }
        h0 = h1;
        h1 = h2;
        k0 = k1;
        k1 = k2;
        let frac = x - a;
        if frac.abs() < 1e-12 {
            break;
        }
        x = 1.0 / frac;
    }
    if k1.is_zero() {
        return Err(ApexError::new(
            "Failed to approximate decimal with the requested denominator bound",
        ));
    }
    let numerator = if sign < 0 { -h1 } else { h1 };
    normalize_fraction(numerator, k1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i64) -> Value {
        Value::Int(BigInt::from(value))
    }

    fn tuple(values: &[i64]) -> Value {
        Value::Tuple(values.iter().map(|v| int(*v)).collect())
    }

    #[test]
    fn reduces_and_adds_fractions() {
        let reduced = fraction_reduce(&[int(6), int(8)]).unwrap();
        assert_eq!(reduced, tuple(&[3, 4]));

        let sum = fraction_add(&[int(1), int(2), int(1), int(3)]).unwrap();
        assert_eq!(sum, tuple(&[5, 6]));
    }

    #[test]
    fn mediant_and_farey_behave() {
        let mediant = fraction_mediant(&[int(1), int(3), int(1), int(2)]).unwrap();
        assert_eq!(mediant, tuple(&[2, 5]));

        let neighbors = fraction_farey_neighbors(&[int(1), int(3), int(2), int(5)]).unwrap();
        assert_eq!(neighbors, Value::Bool(true));
    }

    #[test]
    fn tuple_inputs_flow_into_binary_ops() {
        let left = tuple(&[1, 3]);
        let right = tuple(&[1, 2]);
        let sum = fraction_add(&[left.clone(), right.clone()]).unwrap();
        assert_eq!(sum, tuple(&[5, 6]));

        let mediant = fraction_mediant(&[left, right]).unwrap();
        assert_eq!(mediant, tuple(&[2, 5]));
    }

    #[test]
    fn detects_terminating_and_period_length() {
        let terminating = fraction_is_terminating(&[int(3), int(8)]).unwrap();
        assert_eq!(terminating, Value::Bool(true));

        let period = fraction_period_length(&[int(1), int(7)]).unwrap();
        assert_eq!(period, Value::Int(BigInt::from(6)));
    }

    #[test]
    fn decimal_conversion_round_trips() {
        let decimal = fraction_to_decimal(&[int(3), int(8)]).unwrap();
        assert!(matches!(decimal, Value::Number(value) if (value - 0.375).abs() < 1e-12));

        let rational = decimal_to_fraction(&[Value::Number(0.125), int(128)]).unwrap();
        assert_eq!(rational, tuple(&[1, 8]));
    }

    #[test]
    fn tuple_flows_feed_followup_calls() {
        let sum = fraction_add(&[int(1), int(4), int(1), int(6)]).unwrap();
        let decimal = fraction_to_decimal(&[sum.clone()]).unwrap();
        assert!(matches!(decimal, Value::Number(value) if (value - 0.416666666666).abs() < 1e-6));

        let numerator = fraction_numerator(&[sum.clone()]).unwrap();
        assert_eq!(numerator, int(5));

        let denominator = fraction_denominator(&[sum]).unwrap();
        assert_eq!(denominator, int(12));
    }

    #[test]
    fn egyptian_terms_cover_examples() {
        let terms = fraction_egyptian_terms(&[int(4), int(13)]).unwrap();
        match terms {
            Value::Tuple(values) => {
                let denominators: Vec<i64> = values
                    .into_iter()
                    .map(|value| match value {
                        Value::Int(v) => v.to_i64().unwrap(),
                        _ => 0,
                    })
                    .collect();
                assert_eq!(denominators, vec![4, 18, 468]);
            }
            _ => panic!("Unexpected result"),
        }
    }
}
