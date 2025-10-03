//! OS library for AFNS
//! 
//! This module provides operating system interfaces including:
//! - Process: Process management
//! - Environment: Environment variables
//! - File System: File system operations
//! - Directory: Directory operations
//! - Path: Path manipulation

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::io::{self, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/// Process management
#[derive(Debug)]
pub struct AFNSProcess {
    command: Command,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    env_vars: std::collections::HashMap<String, String>,
}

impl AFNSProcess {
    /// Create a new process
    pub fn new(program: String) -> Self {
        let mut command = Command::new(&program);
        Self {
            command,
            args: vec![program],
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
        }
    }

    /// Add an argument
    pub fn arg(&mut self, arg: String) -> &mut Self {
        self.command.arg(&arg);
        self.args.push(arg);
        self
    }

    /// Set the working directory
    pub fn working_dir(&mut self, dir: String) -> &mut Self {
        self.working_dir = Some(PathBuf::from(dir));
        self.command.current_dir(&self.working_dir.as_ref().unwrap());
        self
    }

    /// Execute the process
    pub fn execute(&mut self) -> Result<AFNSProcessResult, String> {
        let output = self.command.output()
            .map_err(|e| format!("Failed to execute process: {}", e))?;

        Ok(AFNSProcessResult {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

/// Process execution result
#[derive(Debug)]
pub struct AFNSProcessResult {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl AFNSProcessResult {
    /// Get the exit status
    pub fn status(&self) -> &ExitStatus {
        &self.status
    }

    /// Check if the process succeeded
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Get the stdout
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Get the stderr
    pub fn stderr(&self) -> &str {
        &self.stderr
    }
}

/// Environment variables
#[derive(Debug)]
pub struct AFNSEnvironment;

impl AFNSEnvironment {
    /// Get an environment variable
    pub fn get(key: String) -> Option<String> {
        env::var(&key).ok()
    }

    /// Set an environment variable
    pub fn set(key: String, value: String) -> Result<(), String> {
        env::set_var(&key, &value);
        Ok(())
    }

    /// Get the current working directory
    pub fn current_dir() -> Result<String, String> {
        env::current_dir()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get current directory: {}", e))
    }
}

/// File system operations
#[derive(Debug)]
pub struct AFNSFileSystem;

impl AFNSFileSystem {
    /// Check if a path exists
    pub fn exists(path: String) -> bool {
        Path::new(&path).exists()
    }

    /// Check if a path is a file
    pub fn is_file(path: String) -> bool {
        Path::new(&path).is_file()
    }

    /// Check if a path is a directory
    pub fn is_dir(path: String) -> bool {
        Path::new(&path).is_dir()
    }

    /// Create a directory
    pub fn create_dir(path: String) -> Result<(), String> {
        fs::create_dir(&path)
            .map_err(|e| format!("Failed to create directory: {}", e))
    }

    /// Remove a file
    pub fn remove_file(path: String) -> Result<(), String> {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove file: {}", e))
    }
}

// Type aliases for common use cases
pub type Process = AFNSProcess;
pub type Environment = AFNSEnvironment;
pub type FileSystem = AFNSFileSystem;