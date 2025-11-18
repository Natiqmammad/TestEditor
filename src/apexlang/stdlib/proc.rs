use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use num_bigint::BigInt;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::expect_string_arg;
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();
    functions.insert(
        "run".to_string(),
        NativeCallable::new("proc::run", run_command),
    );
    functions.insert(
        "which".to_string(),
        NativeCallable::new("proc::which", which_command),
    );
    functions.insert(
        "env_get".to_string(),
        NativeCallable::new("proc::env_get", env_get),
    );
    functions.insert(
        "env_set".to_string(),
        NativeCallable::new("proc::env_set", env_set),
    );
    functions.insert(
        "cwd".to_string(),
        NativeCallable::new("proc::cwd", current_dir),
    );
    functions.insert(
        "args".to_string(),
        NativeCallable::new("proc::args", program_args),
    );
    functions.insert(
        "set_cwd".to_string(),
        NativeCallable::new("proc::set_cwd", set_current_dir),
    );
    functions.insert(
        "env_list".to_string(),
        NativeCallable::new("proc::env_list", env_list),
    );
    registry.register_module("proc", functions);
}

fn run_command(args: &[Value]) -> Result<Value, ApexError> {
    let program = expect_string_arg(args, 0, "proc.run")?;
    let mut command = Command::new(&program);
    for (index, value) in args.iter().enumerate().skip(1) {
        let arg = match value {
            Value::String(text) => text.clone(),
            _ => {
                return Err(ApexError::new(format!(
                    "proc.run expects string arguments (arg #{})",
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
    let program = expect_string_arg(args, 0, "proc.which")?;
    if let Some(path) = resolve_program(&program) {
        Ok(Value::Tuple(vec![Value::Bool(true), Value::String(path)]))
    } else {
        Ok(Value::Tuple(vec![
            Value::Bool(false),
            Value::String(String::new()),
        ]))
    }
}

fn env_get(args: &[Value]) -> Result<Value, ApexError> {
    let key = expect_string_arg(args, 0, "proc.env_get")?;
    match env::var(&key) {
        Ok(value) => Ok(Value::Tuple(vec![Value::Bool(true), Value::String(value)])),
        Err(_) => Ok(Value::Tuple(vec![
            Value::Bool(false),
            Value::String(String::new()),
        ])),
    }
}

fn env_set(args: &[Value]) -> Result<Value, ApexError> {
    let key = expect_string_arg(args, 0, "proc.env_set")?;
    let value = expect_string_arg(args, 1, "proc.env_set")?;
    env::set_var(&key, &value);
    Ok(Value::Bool(true))
}

fn current_dir(_args: &[Value]) -> Result<Value, ApexError> {
    let dir = env::current_dir()
        .map_err(|err| ApexError::new(format!("Failed to read current dir: {}", err)))?;
    Ok(Value::String(dir.to_string_lossy().to_string()))
}

fn program_args(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Tuple(
        env::args().map(Value::String).collect::<Vec<Value>>(),
    ))
}

fn set_current_dir(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "proc.set_cwd")?;
    env::set_current_dir(&path)
        .map_err(|err| ApexError::new(format!("Failed to change dir: {}", err)))?;
    Ok(Value::Bool(true))
}

fn env_list(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::Tuple(
        env::vars()
            .map(|(key, value)| Value::String(format!("{}={}", key, value)))
            .collect(),
    ))
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
        .expect("proc");
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

    #[test]
    fn env_and_cwd_helpers() {
        env_set(&[
            Value::String("APEX_PROC_TEST".into()),
            Value::String("ok".into()),
        ])
        .unwrap();
        let env_value = env_get(&[Value::String("APEX_PROC_TEST".into())]).unwrap();
        if let Value::Tuple(values) = env_value {
            assert_eq!(values[0], Value::Bool(true));
            assert_eq!(values[1], Value::String("ok".into()));
        } else {
            panic!("expected tuple");
        }
        let cwd = current_dir(&[]).unwrap();
        if let Value::String(path) = cwd {
            assert!(!path.is_empty());
        } else {
            panic!("expected string");
        }
    }

    #[test]
    fn args_listing_and_dir_switch() {
        let args = program_args(&[]).expect("args");
        if let Value::Tuple(values) = args {
            assert!(!values.is_empty());
        } else {
            panic!("expected tuple");
        }
        env_set(&[
            Value::String("APEX_PROC_LIST".into()),
            Value::String("present".into()),
        ])
        .unwrap();
        let envs = env_list(&[]).expect("env list");
        if let Value::Tuple(values) = envs {
            assert!(values
                .iter()
                .any(|entry| entry == &Value::String("APEX_PROC_LIST=present".into())));
        } else {
            panic!("expected tuple");
        }
        let original = env::current_dir().unwrap();
        let mut temp = std::env::temp_dir();
        temp.push(format!("apex_proc_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();
        set_current_dir(&[Value::String(temp.to_string_lossy().to_string())]).unwrap();
        let now = current_dir(&[]).unwrap();
        if let Value::String(path) = now {
            assert!(path.contains("apex_proc_"));
        }
        set_current_dir(&[Value::String(original.to_string_lossy().to_string())]).unwrap();
        std::fs::remove_dir_all(temp).unwrap();
    }
}
