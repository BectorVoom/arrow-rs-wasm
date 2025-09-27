//! WASM-Arrow: Zero-Copy Arrow/Feather for WebAssembly
//!
//! This library provides a high-performance WebAssembly interface for Apache Arrow
//! data processing with zero-copy operations between JavaScript and WASM.

use wasm_bindgen::prelude::*;

// Set up panic hook for better error reporting in WASM
#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// Core modules following the API specification
pub mod core;
pub mod types;
pub mod schema;
pub mod table;
pub mod column;
pub mod io;
pub mod compute;
pub mod plugin;
pub mod error;

// Re-export main functionality
pub use crate::core::*;
pub use crate::error::*;
pub use crate::types::*;
pub use crate::schema::*;
pub use crate::table::*;
pub use crate::column::*;

// WASM utility functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro for console logging from WASM
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}

// Global initialization state
use std::sync::Once;
static INIT: Once = Once::new();

/// Initialize the WASM module and set up the memory allocator.
///
/// This function sets up the WASM linear memory, initializes the panic hook
/// for error handling, and prepares the plugin registry.
#[wasm_bindgen(js_name = "initialize")]
pub async fn initialize() -> Result<(), JsValue> {
    INIT.call_once(|| {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        
        // Initialize any global state here
        console_log!("Arrow WASM module initialized");
    });
    
    Ok(())
}

/// Returns the library version information.
#[wasm_bindgen(js_name = "getVersion")]
pub fn get_version() -> VersionInfoWasm {
    let version_info = VersionInfo {
        major: 0,
        minor: 1,
        patch: 0,
        arrow_version: "56.1.0".to_string(), // Hard-coded for now
    };
    
    version_info.into()
}

// Test data module for integration tests
pub mod test_data {
    use arrow_array::{Int32Array, Float64Array, StringArray, RecordBatch};
    use arrow_schema::{Schema, Field, DataType};
    use std::sync::Arc;

    pub fn simple_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("value", DataType::Float64, true),
            Field::new("name", DataType::Utf8, true),
        ]))
    }

    pub fn simple_record_batch() -> arrow::error::Result<RecordBatch> {
        let schema = simple_schema();
        
        let id_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
        let value_array = Float64Array::from(vec![Some(1.0), None, Some(3.0), Some(4.0), None]);
        let name_array = StringArray::from(vec![Some("Alice"), Some("Bob"), None, Some("Charlie"), Some("David")]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array),
                Arc::new(value_array),
                Arc::new(name_array),
            ],
        )
    }

    pub fn large_record_batch(size: usize) -> arrow::error::Result<RecordBatch> {
        let schema = simple_schema();
        
        let id_array = Int32Array::from_iter_values(0..size as i32);
        let value_array = Float64Array::from_iter(
            (0..size).map(|i| if i % 10 == 0 { None } else { Some(i as f64) })
        );
        let name_array = StringArray::from_iter(
            (0..size).map(|i| if i % 7 == 0 { None } else { Some(format!("name_{}", i)) })
        );

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array),
                Arc::new(value_array),
                Arc::new(name_array),
            ],
        )
    }
}