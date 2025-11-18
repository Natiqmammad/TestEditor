use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use libc::{gethostname, getpid, getppid};

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
        "env_remove".to_string(),
        NativeCallable::new("proc::env_remove", env_remove),
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
    functions.insert(
        "temp_dir".to_string(),
        NativeCallable::new("proc::temp_dir", temp_dir_path),
    );
    functions.insert(
        "home_dir".to_string(),
        NativeCallable::new("proc::home_dir", home_dir_path),
    );
    functions.insert(
        "pid".to_string(),
        NativeCallable::new("proc::pid", current_pid),
    );
    functions.insert(
        "ppid".to_string(),
        NativeCallable::new("proc::ppid", parent_pid),
    );
    functions.insert(
        "hostname".to_string(),
        NativeCallable::new("proc::hostname", host_name),
    );
    functions.insert(
        "username".to_string(),
        NativeCallable::new("proc::username", current_username),
    );
    functions.insert(
        "uuid_v4".to_string(),
        NativeCallable::new("proc::uuid_v4", uuid_v4),
    );
    functions.insert(
        "exe_path".to_string(),
        NativeCallable::new("proc::exe_path", current_exe_path),
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

fn env_remove(args: &[Value]) -> Result<Value, ApexError> {
    let key = expect_string_arg(args, 0, "proc.env_remove")?;
    env::remove_var(&key);
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

fn temp_dir_path(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::String(env::temp_dir().to_string_lossy().to_string()))
}

fn home_dir_path(_args: &[Value]) -> Result<Value, ApexError> {
    if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        Ok(Value::String(
            PathBuf::from(home).to_string_lossy().to_string(),
        ))
    } else {
        Err(ApexError::new("Home directory is not set"))
    }
}

fn current_pid(_args: &[Value]) -> Result<Value, ApexError> {
    let pid = unsafe { getpid() } as i64;
    Ok(Value::Int(BigInt::from(pid)))
}

fn parent_pid(_args: &[Value]) -> Result<Value, ApexError> {
    let pid = unsafe { getppid() } as i64;
    Ok(Value::Int(BigInt::from(pid)))
}

fn host_name(_args: &[Value]) -> Result<Value, ApexError> {
    let mut buffer = [0u8; 256];
    let result = unsafe { gethostname(buffer.as_mut_ptr() as *mut i8, buffer.len()) };
    if result != 0 {
        return Err(ApexError::new("Failed to read hostname"));
    }
    let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    let host = String::from_utf8_lossy(&buffer[..len]).to_string();
    Ok(Value::String(host))
}

fn current_username(_args: &[Value]) -> Result<Value, ApexError> {
    if let Ok(name) = env::var("USER") {
        if !name.is_empty() {
            return Ok(Value::String(name));
        }
    }
    if let Ok(name) = env::var("USERNAME") {
        if !name.is_empty() {
            return Ok(Value::String(name));
        }
    }
    Err(ApexError::new(
        "Username is not available in the environment",
    ))
}

fn uuid_v4(_args: &[Value]) -> Result<Value, ApexError> {
    Ok(Value::String(Uuid::new_v4().to_string()))
}

fn current_exe_path(_args: &[Value]) -> Result<Value, ApexError> {
    let path = env::current_exe()
        .map_err(|err| ApexError::new(format!("Failed to read exe path: {}", err)))?;
    Ok(Value::String(path.to_string_lossy().to_string()))
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
        env_remove(&[Value::String("APEX_PROC_TEST".into())]).unwrap();
        let removed = env_get(&[Value::String("APEX_PROC_TEST".into())]).unwrap();
        if let Value::Tuple(values) = removed {
            assert_eq!(values[0], Value::Bool(false));
        }
        let cwd = current_dir(&[]).unwrap();
        if let Value::String(path) = cwd {
            assert!(!path.is_empty());
        } else {
            panic!("expected string");
        }
        let temp_dir = temp_dir_path(&[]).unwrap();
        if let Value::String(path) = temp_dir {
            assert!(!path.is_empty());
        } else {
            panic!("expected temp dir string");
        }
        let home = home_dir_path(&[]).unwrap();
        if let Value::String(path) = home {
            assert!(!path.is_empty());
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

    #[test]
    fn pid_and_hostname_exposed() {
        let pid = current_pid(&[]).expect("pid");
        if let Value::Int(value) = pid {
            assert!(value > BigInt::from(0));
        } else {
            panic!("expected int");
        }
        let ppid = parent_pid(&[]).expect("ppid");
        if let Value::Int(value) = ppid {
            assert!(value > BigInt::from(0));
        } else {
            panic!("expected int");
        }
        let hostname = host_name(&[]).expect("hostname");
        if let Value::String(name) = hostname {
            assert!(!name.is_empty());
        } else {
            panic!("expected hostname string");
        }
    }

    #[test]
    fn username_reads_environment() {
        let original = env::var("USER").ok();
        env::set_var("USER", "apex_proc_user_test");
        let user = current_username(&[]).expect("username");
        if let Value::String(name) = user {
            assert_eq!(name, "apex_proc_user_test");
        } else {
            panic!("expected username string");
        }
        match original {
            Some(value) => env::set_var("USER", value),
            None => env::remove_var("USER"),
        }
    }

    #[test]
    fn uuid_and_exe_path_exposed() {
        let uuid_value = uuid_v4(&[]).expect("uuid");
        if let Value::String(text) = uuid_value {
            assert_eq!(text.len(), 36);
            assert!(text.contains('-'));
        } else {
            panic!("expected uuid string");
        }
        let exe = current_exe_path(&[]).expect("exe path");
        if let Value::String(path) = exe {
            assert!(path.contains("TestEditor"));
        } else {
            panic!("expected exe string");
        }
    }
}
