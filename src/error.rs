//! Error handling types and utilities for the WASM Arrow library.
//!
//! This module provides error types that follow the API specification's
//! constraints - avoiding try/catch blocks and providing Result types instead.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Error codes for different types of Arrow errors
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidFormat = "INVALID_FORMAT",
    SchemaMismatch = "SCHEMA_MISMATCH", 
    OutOfBounds = "OUT_OF_BOUNDS",
    TypeMismatch = "TYPE_MISMATCH",
    MemoryError = "MEMORY_ERROR",
    IOError = "IO_ERROR",
    NotImplemented = "NOT_IMPLEMENTED",
}

/// Arrow error structure for WASM
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowError {
    code: ErrorCode,
    message: String,
    details: Option<String>,
}

#[wasm_bindgen]
impl ArrowError {
    #[wasm_bindgen(constructor)]
    pub fn new(code: ErrorCode, message: &str) -> ArrowError {
        ArrowError {
            code,
            message: message.to_string(),
            details: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn details(&self) -> Option<String> {
        self.details.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_details(&mut self, details: Option<String>) {
        self.details = details;
    }
}

impl std::fmt::Display for ArrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)?;
        if let Some(ref details) = self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for ArrowError {}

impl From<arrow::error::ArrowError> for ArrowError {
    fn from(err: arrow::error::ArrowError) -> Self {
        use arrow::error::ArrowError as AErr;
        
        let code = match &err {
            AErr::InvalidArgumentError(_) => ErrorCode::InvalidFormat,
            AErr::SchemaError(_) => ErrorCode::SchemaMismatch,
            AErr::MemoryError(_) => ErrorCode::MemoryError,
            AErr::IoError(_, _) => ErrorCode::IOError,
            AErr::NotYetImplemented(_) => ErrorCode::NotImplemented,
            _ => ErrorCode::InvalidFormat,
        };

        ArrowError {
            code,
            message: err.to_string(),
            details: None,
        }
    }
}

/// Result type for operations that can fail
/// 
/// This replaces try/catch patterns as per API specification
#[wasm_bindgen]
pub struct WasmResult {
    ok: bool,
    value_js: Option<JsValue>,
    error: Option<ArrowError>,
}

#[wasm_bindgen]
impl WasmResult {
    /// Create a successful result
    pub fn success(value: JsValue) -> WasmResult {
        WasmResult {
            ok: true,
            value_js: Some(value),
            error: None,
        }
    }

    /// Create an error result
    pub fn from_error(error: ArrowError) -> WasmResult {
        WasmResult {
            ok: false,
            value_js: None,
            error: Some(error),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn ok(&self) -> bool {
        self.ok
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        self.value_js.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter, js_name = "errorValue")]
    pub fn error_value(&self) -> JsValue {
        if let Some(ref err) = self.error {
            serde_wasm_bindgen::to_value(err).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }
}

/// Error callback type for handling errors
pub type ErrorCallback = js_sys::Function;

/// Set global error handler callback
static mut ERROR_HANDLER: Option<ErrorCallback> = None;

/// Register an error handler callback
#[wasm_bindgen(js_name = "onError")]
pub fn on_error(handler: ErrorCallback) {
    unsafe {
        ERROR_HANDLER = Some(handler);
    }
}

/// Call the error handler if one is registered
pub fn call_error_handler(error: &ArrowError) {
    unsafe {
        if let Some(ref handler) = ERROR_HANDLER {
            let js_error = serde_wasm_bindgen::to_value(error).unwrap_or(JsValue::NULL);
            let _ = handler.call1(&JsValue::NULL, &js_error);
        }
    }
}

/// Utility macro for creating errors
#[macro_export]
macro_rules! arrow_error {
    ($code:expr, $msg:expr) => {
        crate::error::ArrowError::new($code, $msg)
    };
    ($code:expr, $msg:expr, $details:expr) => {{
        let mut err = crate::error::ArrowError::new($code, $msg);
        err.set_details(Some($details.to_string()));
        err
    }};
}