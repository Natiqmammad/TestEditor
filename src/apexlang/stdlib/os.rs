use std::collections::HashMap;
use std::env;
use std::process;

use num_bigint::BigInt;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::expect_string_arg;
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("os::", $name), $func),
            );
        };
    }

    add!(&mut functions, "cwd", cwd);
    add!(&mut functions, "temp_dir", temp_dir);
    add!(&mut functions, "env_var", env_var);
    add!(&mut functions, "pointer_width", pointer_width);
    add!(&mut functions, "pid", pid);
    add!(&mut functions, "args", args);
    registry.register_module("os", functions);
}

fn cwd(_args: &[Value]) -> Result<Value, ApexError> {
    let dir = env::current_dir()
        .map_err(|err| ApexError::new(format!("Failed to fetch current directory: {}", err)))?;
    Ok(Value::String(dir.to_string_lossy().to_string()))
}

fn temp_dir(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::String(env::temp_dir().to_string_lossy().to_string()))
}

fn env_var(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "os.env_var")?;
    if let Ok(value) = env::var(&name) {
        Ok(Value::Tuple(vec![Value::Bool(true), Value::String(value)]))
    } else {
        Ok(Value::Tuple(vec![
            Value::Bool(false),
            Value::String(String::new()),
        ]))
    }
}

fn pointer_width(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Int(BigInt::from(std::mem::size_of::<usize>() * 8)))
}

fn pid(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Int(BigInt::from(process::id())))
}

fn args(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Tuple(
        env::args().map(Value::String).collect::<Vec<Value>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_os_details() {
        assert!(matches!(cwd(&[]), Ok(Value::String(_))));
        assert!(matches!(temp_dir(&[]), Ok(Value::String(_))));
        let tuple = env_var(&[Value::String("PATH".into())]).expect("env var");
        if let Value::Tuple(values) = tuple {
            assert_eq!(values.len(), 2);
        } else {
            panic!("expected tuple");
        }
        let width = pointer_width(&[]).expect("width");
        assert!(matches!(width, Value::Int(_)));
        let pid_value = pid(&[]).expect("pid");
        assert!(matches!(pid_value, Value::Int(_)));
        let args_value = args(&[]).expect("args");
        if let Value::Tuple(values) = args_value {
            assert!(!values.is_empty());
        } else {
            panic!("expected tuple");
        }
    }
}
