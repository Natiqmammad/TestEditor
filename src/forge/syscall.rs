//! System Call Operations for AFNS
//! Low-level system operations, memory management, and file operations

use std::fs;
use std::path::Path;
use std::process::Command;

/// System call wrapper for AFNS
pub struct AFNSSyscall;

impl AFNSSyscall {
    /// Execute a system command
    pub fn exec_command(command: &str) -> Result<String, String> {
        Command::new("system")
            .arg(command)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .map_err(|e| format!("Command execution failed: {}", e))
    }
    
    /// Read file content
    pub fn read_file(path: &str) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("File read failed: {}", e))
    }
    
    /// Write file content
    pub fn write_file(path: &str, content: &str) -> Result<(), String> {
        fs::write(path, content)
            .map_err(|e| format!("File write failed: {}", e))
    }
    
    /// Check if file exists
    pub fn file_exists(path: &str) -> bool {
        Path::new(path).exists()
    }
    
    /// Create directory
    pub fn create_directory(path: &str) -> Result<(), String> {
        fs::create_dir_all(path.to_string())
            .map_err(|e| format!("Directory creation failed: {}", e))
    }
    
    /// Memory allocation simulation
    pub fn allocate_memory(size: usize) -> Result<*mut u8, String> {
        unsafe {
            libc::malloc(size) as *mut u8
        }
    }
    
    /// Memory deallocation simulation
    pub fn deallocate_memory(ptr: *mut u8) {
        unsafe {
            libc::free(ptr as *mut libc::c_void);
        }
    }
    
    /// Get current working directory
    pub fn get_current_directory() -> Result<String, String> {
        std::env::current_dir()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get current directory: {}", e))
    }
    
    /// Set current working directory
    pub fn set_current_directory(path: &str) -> Result<(), String> {
        std::env::set_current_dir(path)
            .map_err(|e| format!("Failed to set current directory: {}", e))
    }
    
    /// Get environment variable
    pub fn get_environment_variable(name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
    
    /// Set environment variable
    pub fn set_environment_variable(name: &str, value: &str) {
        std::env::set_var(name, value);
    }
    
    /// Terminate current process
    pub fn terminate_process(exit_code: i32) {
        std::process::exit(exit_code);
    }
    
    /// Spawn new process
    pub fn spawn_process(command: &str, args: &[&str]) -> Result<std::process::Child, String> {
        Command::new(command)
            .args(args)
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))
    }
}