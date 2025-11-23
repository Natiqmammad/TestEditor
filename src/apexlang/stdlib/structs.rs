use std::collections::HashMap;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{expect_tuple_arg, expect_usize_arg};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($name:literal, $func:ident) => {
            functions.insert(
                $name.to_string(),
                NativeCallable::new(concat!("structs::", $name), $func),
            );
        };
    }

    add!("copy", copy_value);
    add!("clone_tuple", clone_tuple);
    add!("copy_replace", copy_replace);
    add!("deep_clone", deep_clone);
    add!("copy_append", copy_append);
    add!("tuple_concat", tuple_concat);

    registry.register_module("structs", functions);
}

fn copy_value(args: &[Value]) -> Result<Value, ApexError> {
    args.get(0)
        .cloned()
        .ok_or_else(|| ApexError::new("structs.copy expects a value"))
}

fn clone_tuple(args: &[Value]) -> Result<Value, ApexError> {
    let tuple = expect_tuple_arg(args, 0, "structs.clone_tuple")?;
    Ok(Value::Tuple(tuple))
}

fn copy_replace(args: &[Value]) -> Result<Value, ApexError> {
    let mut tuple = expect_tuple_arg(args, 0, "structs.copy_replace")?;
    let index = expect_usize_arg(args, 1, "structs.copy_replace")?;
    let replacement = args
        .get(2)
        .cloned()
        .ok_or_else(|| ApexError::new("structs.copy_replace expects a replacement value"))?;
    if index >= tuple.len() {
        return Err(ApexError::new("structs.copy_replace index out of bounds"));
    }
    tuple[index] = replacement;
    Ok(Value::Tuple(tuple))
}

fn tuple_concat(args: &[Value]) -> Result<Value, ApexError> {
    let mut left = expect_tuple_arg(args, 0, "structs.tuple_concat")?;
    let right = expect_tuple_arg(args, 1, "structs.tuple_concat")?;
    left.extend(right);
    Ok(Value::Tuple(left))
}

fn deep_clone(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .ok_or_else(|| ApexError::new("structs.deep_clone expects a value"))?;
    Ok(recursive_clone(value))
}

fn copy_append(args: &[Value]) -> Result<Value, ApexError> {
    let mut tuple = expect_tuple_arg(args, 0, "structs.copy_append")?;
    if args.len() < 2 {
        return Err(ApexError::new(
            "structs.copy_append expects a tuple and at least one value to append",
        ));
    }
    for value in args.iter().skip(1) {
        tuple.push(recursive_clone(value));
    }
    Ok(Value::Tuple(tuple))
}

fn recursive_clone(value: &Value) -> Value {
    match value {
        Value::Tuple(values) => Value::Tuple(values.iter().map(recursive_clone).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_and_clone_are_pure() {
        let tuple = Value::Tuple(vec![Value::Int(1.into()), Value::Bool(true)]);
        let cloned = clone_tuple(&[tuple.clone()]).expect("clone");
        assert_eq!(tuple, cloned);
        let copy = copy_value(&[tuple.clone()]).expect("copy");
        assert_eq!(tuple, copy);
    }

    #[test]
    fn copy_replace_and_concat_build_new_tuples() {
        let tuple = Value::Tuple(vec![Value::Int(1.into()), Value::Int(2.into())]);
        let replaced = copy_replace(&[tuple.clone(), Value::Int(1.into()), Value::Int(9.into())])
            .expect("replace");
        assert_eq!(
            replaced,
            Value::Tuple(vec![Value::Int(1.into()), Value::Int(9.into())])
        );
        let concat = tuple_concat(&[tuple.clone(), replaced]).expect("concat");
        if let Value::Tuple(values) = concat {
            assert_eq!(values.len(), 4);
        } else {
            panic!("expected tuple");
        }
    }

    #[test]
    fn deep_clone_and_append_are_total() {
        let nested = Value::Tuple(vec![
            Value::Int(1.into()),
            Value::Tuple(vec![Value::Bool(true), Value::String("x".into())]),
        ]);
        let cloned = deep_clone(&[nested.clone()]).expect("deep clone");
        assert_eq!(nested, cloned);

        let appended =
            copy_append(&[nested, Value::Int(9.into()), Value::Bool(false)]).expect("append");
        if let Value::Tuple(values) = appended {
            assert_eq!(values.len(), 4);
            assert_eq!(values[3], Value::Bool(false));
        } else {
            panic!("expected tuple");
        }
    }
}
