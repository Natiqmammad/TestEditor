use std::collections::HashMap;
use std::sync::Mutex;

use num_bigint::BigInt;
use once_cell::sync::Lazy;

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
                NativeCallable::new(concat!("signal::", $name), $func),
            );
        };
    }

    add!(&mut functions, "register", register_signal);
    add!(&mut functions, "emit", emit_signal);
    add!(&mut functions, "count", signal_count);
    add!(&mut functions, "tracked", tracked_signals);
    add!(&mut functions, "reset", reset_signal);
    registry.register_module("signal", functions);
}

static SIGNALS: Lazy<Mutex<HashMap<String, u64>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn register_signal(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "signal.register")?;
    let mut signals = SIGNALS
        .lock()
        .map_err(|_| ApexError::new("Signal registry lock poisoned"))?;
    let is_new = signals.insert(name, 0).is_none();
    Ok(Value::Bool(is_new))
}

fn emit_signal(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "signal.emit")?;
    let mut signals = SIGNALS
        .lock()
        .map_err(|_| ApexError::new("Signal registry lock poisoned"))?;
    let entry = signals.entry(name).or_insert(0);
    *entry += 1;
    Ok(Value::Int(BigInt::from(*entry)))
}

fn signal_count(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "signal.count")?;
    let signals = SIGNALS
        .lock()
        .map_err(|_| ApexError::new("Signal registry lock poisoned"))?;
    let value = signals.get(&name).copied().unwrap_or(0);
    Ok(Value::Int(BigInt::from(value)))
}

fn tracked_signals(_args: &[Value]) -> Result<Value, ApexError> {
    let signals = SIGNALS
        .lock()
        .map_err(|_| ApexError::new("Signal registry lock poisoned"))?;
    Ok(Value::Int(BigInt::from(signals.len())))
}

fn reset_signal(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "signal.reset")?;
    let mut signals = SIGNALS
        .lock()
        .map_err(|_| ApexError::new("Signal registry lock poisoned"))?;
    if let Some(entry) = signals.get_mut(&name) {
        *entry = 0;
        Ok(Value::Bool(true))
    } else {
        Ok(Value::Bool(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_signals() {
        register_signal(&[Value::String("USR1".into())]).unwrap();
        emit_signal(&[Value::String("USR1".into())]).unwrap();
        let count = signal_count(&[Value::String("USR1".into())]).unwrap();
        assert_eq!(count, Value::Int(1.into()));
        let tracked = tracked_signals(&[]).unwrap();
        assert!(matches!(tracked, Value::Int(_)));
        assert_eq!(
            reset_signal(&[Value::String("USR1".into())]).unwrap(),
            Value::Bool(true)
        );
        let count = signal_count(&[Value::String("USR1".into())]).unwrap();
        assert_eq!(count, Value::Int(0.into()));
    }
}
