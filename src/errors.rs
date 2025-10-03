use thiserror::Error;
use wasm_bindgen::prelude::*;

#[derive(Error, Debug)]
pub enum ArrowWasmError {
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    
    #[error("IPC error: {0}")]
    Ipc(String),
    
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Invalid table handle: {0}")]
    InvalidHandle(u32),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Buffer error: {0}")]
    Buffer(String),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl From<ArrowWasmError> for JsValue {
    fn from(err: ArrowWasmError) -> JsValue {
        JsValue::from_str(&err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ArrowWasmError>;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);
    
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub(crate) use console_log;

#[wasm_bindgen]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}