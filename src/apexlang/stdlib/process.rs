use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use num_bigint::BigInt;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::expect_string_arg;
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();
    functions.insert(
        "run".to_string(),
        NativeCallable::new("process::run", run_command),
    );
    functions.insert(
        "which".to_string(),
        NativeCallable::new("process::which", which_command),
    );
    registry.register_module("process", functions);
}

fn run_command(args: &[Value]) -> Result<Value, ApexError> {
    let program = expect_string_arg(args, 0, "process.run")?;
    let mut command = Command::new(&program);
    for (index, value) in args.iter().enumerate().skip(1) {
        let arg = match value {
            Value::String(text) => text.clone(),
            _ => {
                return Err(ApexError::new(format!(
                    "process.run expects string arguments (arg #{})",
                    index + 1
                )))
            }
        };
        command.arg(arg);
    }
    let output = command
        .output()
        .map_err(|err| ApexError::new(format!("Failed to run '{}': {}", program, err)))?;
    let code = output.status.code().unwrap_or_default();
    Ok(Value::Tuple(vec![
        Value::Int(BigInt::from(code)),
        Value::String(String::from_utf8_lossy(&output.stdout).to_string()),
        Value::String(String::from_utf8_lossy(&output.stderr).to_string()),
    ]))
}

fn which_command(args: &[Value]) -> Result<Value, ApexError> {
    let program = expect_string_arg(args, 0, "process.which")?;
    if let Some(path) = resolve_program(&program) {
        Ok(Value::Tuple(vec![Value::Bool(true), Value::String(path)]))
    } else {
        Ok(Value::Tuple(vec![
            Value::Bool(false),
            Value::String(String::new()),
        ]))
    }
}

fn resolve_program(program: &str) -> Option<String> {
    let candidate = Path::new(program);
    if candidate.is_absolute() || program.contains('/') {
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
        return None;
    }
    if let Some(path_var) = env::var_os("PATH") {
        for entry in env::split_paths(&path_var) {
            let mut buf = PathBuf::from(entry);
            buf.push(program);
            if buf.exists() {
                return Some(buf.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_shell_command() {
        let result = run_command(&[
            Value::String("sh".into()),
            Value::String("-c".into()),
            Value::String("echo apex".into()),
        ])
        .expect("process");
        if let Value::Tuple(values) = result {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::Int(0.into()));
            if let Value::String(stdout) = &values[1] {
                assert!(stdout.contains("apex"));
            } else {
                panic!("expected stdout string");
            }
        } else {
            panic!("expected tuple");
        }
    }

    #[test]
    fn which_finds_shell() {
        let result = which_command(&[Value::String("sh".into())]).expect("which");
        if let Value::Tuple(values) = result {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Bool(true));
        } else {
            panic!("expected tuple");
        }
    }
}
