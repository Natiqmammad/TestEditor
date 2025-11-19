//! Error library for AFNS
//!
//! This module provides error handling including:
//! - Custom Error Types
//! - Error Propagation
//! - Result Type

use std::error::Error as StdError;
use std::fmt;

/// Custom error types
#[derive(Debug)]
pub enum AFNSError {
    /// Generic error with message
    Generic(String),
    /// I/O error
    Io(String),
    /// Parse error
    Parse(String),
    /// Network error
    Network(String),
    /// File system error
    FileSystem(String),
    /// Memory error
    Memory(String),
    /// Thread error
    Thread(String),
    /// Channel error
    Channel(String),
    /// Timeout error
    Timeout(String),
    /// Validation error
    Validation(String),
    /// Configuration error
    Configuration(String),
    /// Runtime error
    Runtime(String),
}

impl fmt::Display for AFNSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AFNSError::Generic(msg) => write!(f, "Generic error: {}", msg),
            AFNSError::Io(msg) => write!(f, "I/O error: {}", msg),
            AFNSError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AFNSError::Network(msg) => write!(f, "Network error: {}", msg),
            AFNSError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            AFNSError::Memory(msg) => write!(f, "Memory error: {}", msg),
            AFNSError::Thread(msg) => write!(f, "Thread error: {}", msg),
            AFNSError::Channel(msg) => write!(f, "Channel error: {}", msg),
            AFNSError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            AFNSError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AFNSError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            AFNSError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl StdError for AFNSError {}

impl AFNSError {
    /// Create a generic error
    pub fn generic(msg: String) -> Self {
        Self::Generic(msg)
    }

    /// Create an I/O error
    pub fn io(msg: String) -> Self {
        Self::Io(msg)
    }

    /// Create a parse error
    pub fn parse(msg: String) -> Self {
        Self::Parse(msg)
    }

    /// Create a network error
    pub fn network(msg: String) -> Self {
        Self::Network(msg)
    }

    /// Create a file system error
    pub fn file_system(msg: String) -> Self {
        Self::FileSystem(msg)
    }

    /// Create a memory error
    pub fn memory(msg: String) -> Self {
        Self::Memory(msg)
    }

    /// Create a thread error
    pub fn thread(msg: String) -> Self {
        Self::Thread(msg)
    }

    /// Create a channel error
    pub fn channel(msg: String) -> Self {
        Self::Channel(msg)
    }

    /// Create a timeout error
    pub fn timeout(msg: String) -> Self {
        Self::Timeout(msg)
    }

    /// Create a validation error
    pub fn validation(msg: String) -> Self {
        Self::Validation(msg)
    }

    /// Create a configuration error
    pub fn configuration(msg: String) -> Self {
        Self::Configuration(msg)
    }

    /// Create a runtime error
    pub fn runtime(msg: String) -> Self {
        Self::Runtime(msg)
    }

    /// Get the error message
    pub fn message(&self) -> String {
        self.to_string()
    }

    /// Check if this is a generic error
    pub fn is_generic(&self) -> bool {
        matches!(self, Self::Generic(_))
    }

    /// Check if this is an I/O error
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    /// Check if this is a parse error
    pub fn is_parse(&self) -> bool {
        matches!(self, Self::Parse(_))
    }

    /// Check if this is a network error
    pub fn is_network(&self) -> bool {
        matches!(self, Self::Network(_))
    }

    /// Check if this is a file system error
    pub fn is_file_system(&self) -> bool {
        matches!(self, Self::FileSystem(_))
    }

    /// Check if this is a memory error
    pub fn is_memory(&self) -> bool {
        matches!(self, Self::Memory(_))
    }

    /// Check if this is a thread error
    pub fn is_thread(&self) -> bool {
        matches!(self, Self::Thread(_))
    }

    /// Check if this is a channel error
    pub fn is_channel(&self) -> bool {
        matches!(self, Self::Channel(_))
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this is a configuration error
    pub fn is_configuration(&self) -> bool {
        matches!(self, Self::Configuration(_))
    }

    /// Check if this is a runtime error
    pub fn is_runtime(&self) -> bool {
        matches!(self, Self::Runtime(_))
    }
}

/// Error context for adding additional information
#[derive(Debug)]
pub struct AFNSErrorContext {
    error: AFNSError,
    context: Vec<String>,
}

impl AFNSErrorContext {
    /// Create a new error context
    pub fn new(error: AFNSError) -> Self {
        Self {
            error,
            context: Vec::new(),
        }
    }

    /// Add context information
    pub fn with_context(mut self, context: String) -> Self {
        self.context.push(context);
        self
    }

    /// Get the error
    pub fn error(&self) -> &AFNSError {
        &self.error
    }

    /// Get the context
    pub fn context(&self) -> &Vec<String> {
        &self.context
    }

    /// Get the full error message with context
    pub fn full_message(&self) -> String {
        let mut message = self.error.to_string();
        if !self.context.is_empty() {
            message.push_str("\nContext:");
            for ctx in &self.context {
                message.push_str(&format!("\n  - {}", ctx));
            }
        }
        message
    }
}

impl fmt::Display for AFNSErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_message())
    }
}

impl StdError for AFNSErrorContext {}

/// Error propagation utilities
pub struct AFNSErrorPropagation;

impl AFNSErrorPropagation {
    /// Propagate an error with context
    pub fn propagate_with_context<T, E>(
        result: Result<T, E>,
        context: String,
    ) -> Result<T, AFNSError>
    where
        E: Into<AFNSError>,
    {
        result.map_err(|e| {
            let error: AFNSError = e.into();
            AFNSErrorContext::new(error).with_context(context).into()
        })
    }

    /// Propagate an error with multiple contexts
    pub fn propagate_with_contexts<T, E>(
        result: Result<T, E>,
        contexts: Vec<String>,
    ) -> Result<T, AFNSError>
    where
        E: Into<AFNSError>,
    {
        result.map_err(|e| {
            let error: AFNSError = e.into();
            let mut error_ctx = AFNSErrorContext::new(error);
            for context in contexts {
                error_ctx = error_ctx.with_context(context);
            }
            error_ctx.into()
        })
    }

    /// Convert a standard error to AFNS error
    pub fn from_std_error(error: Box<dyn StdError>) -> AFNSError {
        AFNSError::generic(error.to_string())
    }

    /// Convert a string to AFNS error
    pub fn from_string(error: String) -> AFNSError {
        AFNSError::generic(error)
    }

    /// Convert a string slice to AFNS error
    pub fn from_str(error: &str) -> AFNSError {
        AFNSError::generic(error.to_string())
    }
}

/// Result type alias for AFNS
pub type AFNSResult<T> = std::result::Result<T, AFNSError>;

/// Result type alias for AFNS with context
pub type AFNSResultWithContext<T> = std::result::Result<T, AFNSErrorContext>;

// Type aliases for common use cases
pub type Error = AFNSError;
pub type ErrorContext = AFNSErrorContext;
pub type ErrorPropagation = AFNSErrorPropagation;
pub type Result<T> = AFNSResult<T>;
pub type ResultWithContext<T> = AFNSResultWithContext<T>;
