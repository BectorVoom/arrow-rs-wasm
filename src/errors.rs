//! Error types for the Arrow WASM library
//! 
//! This module defines the core error types that cross the WASM boundary.
//! All errors are serializable for safe transport between Rust and JavaScript.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Core error enum used across WASM boundary (string-friendly).
/// 
/// All error variants contain String messages rather than nested error types
/// to ensure safe serialization across the WASM boundary.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CoreError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(String),
    
    /// Arrow IPC operation failed
    #[error("IPC error: {0}")]
    Ipc(String),
    
    /// Parquet operation failed
    #[error("Parquet error: {0}")]
    Parquet(String),
    
    /// Plugin operation failed
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    /// Invalid table handle provided
    #[error("Invalid handle: {0}")]
    InvalidHandle(u32),
    
    /// Memory management error
    #[error("Memory error: {0}")]
    Memory(String),
    
    /// Schema validation error
    #[error("Schema error: {0}")]
    Schema(String),
    
    /// Data validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Generic error for other cases
    #[error("Other: {0}")]
    Other(String),
}

impl CoreError {
    /// Create an IO error from any error type
    pub fn io<E: fmt::Display>(error: E) -> Self {
        CoreError::Io(error.to_string())
    }
    
    /// Create an IPC error from any error type
    pub fn ipc<E: fmt::Display>(error: E) -> Self {
        CoreError::Ipc(error.to_string())
    }
    
    /// Create a Parquet error from any error type
    pub fn parquet<E: fmt::Display>(error: E) -> Self {
        CoreError::Parquet(error.to_string())
    }
    
    /// Create a plugin error from any error type
    pub fn plugin<E: fmt::Display>(error: E) -> Self {
        CoreError::Plugin(error.to_string())
    }
    
    /// Create a memory error from any error type
    pub fn memory<E: fmt::Display>(error: E) -> Self {
        CoreError::Memory(error.to_string())
    }
    
    /// Create a schema error from any error type
    pub fn schema<E: fmt::Display>(error: E) -> Self {
        CoreError::Schema(error.to_string())
    }
    
    /// Create a validation error from any error type
    pub fn validation<E: fmt::Display>(error: E) -> Self {
        CoreError::Validation(error.to_string())
    }
    
    /// Create an other error from any error type
    pub fn other<E: fmt::Display>(error: E) -> Self {
        CoreError::Other(error.to_string())
    }
    
    /// Create an invalid handle error
    pub fn invalid_handle(handle: u32) -> Self {
        CoreError::InvalidHandle(handle)
    }
}

/// Convert from arrow errors
impl From<arrow_schema::ArrowError> for CoreError {
    fn from(error: arrow_schema::ArrowError) -> Self {
        match error {
            arrow_schema::ArrowError::IoError(msg, _) => CoreError::io(msg),
            arrow_schema::ArrowError::ParseError(msg) => CoreError::validation(msg),
            arrow_schema::ArrowError::SchemaError(msg) => CoreError::schema(msg),
            arrow_schema::ArrowError::ComputeError(msg) => CoreError::other(msg),
            arrow_schema::ArrowError::MemoryError(msg) => CoreError::memory(msg),
            arrow_schema::ArrowError::NotYetImplemented(msg) => CoreError::other(format!("Not implemented: {}", msg)),
            _ => CoreError::other(error.to_string()),
        }
    }
}

/// Convert from std::io errors
impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self {
        CoreError::io(error)
    }
}

/// Convert from serde_json errors
impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        CoreError::other(format!("JSON error: {}", error))
    }
}

/// Result type alias for core operations
pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_serialization() {
        let error = CoreError::InvalidHandle(42);
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: CoreError = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            CoreError::InvalidHandle(handle) => assert_eq!(handle, 42),
            _ => panic!("Unexpected error variant"),
        }
    }
    
    #[test]
    fn test_error_display() {
        let error = CoreError::io("file not found");
        assert_eq!(error.to_string(), "IO error: file not found");
        
        let error = CoreError::InvalidHandle(123);
        assert_eq!(error.to_string(), "Invalid handle: 123");
    }
    
    #[test]
    fn test_error_creation_helpers() {
        let io_err = CoreError::io("test io error");
        assert!(matches!(io_err, CoreError::Io(_)));
        
        let ipc_err = CoreError::ipc("test ipc error");
        assert!(matches!(ipc_err, CoreError::Ipc(_)));
        
        let handle_err = CoreError::invalid_handle(999);
        assert!(matches!(handle_err, CoreError::InvalidHandle(999)));
    }
}