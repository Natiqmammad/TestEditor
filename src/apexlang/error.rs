use std::fmt;

/// Represents an error that can occur while processing ApexLang source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApexError {
    message: String,
}

impl ApexError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[cfg(test)]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ApexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApexError {}
