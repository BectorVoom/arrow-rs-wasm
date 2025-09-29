//! Arrow WebAssembly Library
//! 
//! A WebAssembly library for reading, writing, and manipulating Arrow data
//! with zero-copy semantics and LZ4 compression support.

use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use serde::{Serialize, Deserialize};
use web_sys::console;

// Re-export core modules
pub mod errors;
pub mod mem;
pub mod fs;
pub mod ipc;
pub mod plugin;

// Re-export key types for convenience
pub use errors::{CoreError, CoreResult};
pub use mem::{TableHandle, SchemaSummary, MemoryStats};
pub use ipc::CompressionConfig;

/// Initialize the WASM module with panic hooks and logging
#[wasm_bindgen(start)]
pub fn init() {
    // Set up better panic messages in debug mode
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    // Initialize any global state if needed
    log_to_console("Arrow WASM library initialized");
}

/// Initialize WASM module with custom options
#[wasm_bindgen]
pub fn init_with_options(enable_console_logs: bool) -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    if enable_console_logs {
        log_to_console("Arrow WASM library initialized with console logging enabled");
    }
    
    Ok(())
}

/// Log a message to the browser console
fn log_to_console(message: &str) {
    console::log_1(&format!("[Arrow WASM] {}", message).into());
}

/// Create a Table from an in-memory IPC stream (zero-copy where possible)
#[wasm_bindgen]
pub fn create_table_from_ipc(ipc_data: &[u8]) -> Result<TableHandle, JsValue> {
    fs::read_table_from_bytes(ipc_data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Read Arrow/Feather/Parquet file bytes and return a managed TableHandle
#[wasm_bindgen]
pub fn read_table_from_bytes(data: &[u8]) -> Result<TableHandle, JsValue> {
    fs::read_table_from_bytes(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Read a table from a JavaScript ArrayBuffer
#[wasm_bindgen]
pub fn read_table_from_array_buffer(buffer: &js_sys::ArrayBuffer) -> Result<TableHandle, JsValue> {
    let uint8_array = js_sys::Uint8Array::new(buffer);
    let data = uint8_array.to_vec();
    
    fs::read_table_from_bytes(&data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Write a Table to an in-memory Arrow IPC file with LZ4 option
#[wasm_bindgen]
pub fn write_table_to_ipc(handle: TableHandle, enable_lz4: bool) -> Result<Vec<u8>, JsValue> {
    ipc::write_table_with_compression(handle, enable_lz4)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Write table to IPC bytes and return as JavaScript ArrayBuffer
#[wasm_bindgen]
pub fn write_table_to_array_buffer(handle: TableHandle, enable_lz4: bool) -> Result<js_sys::ArrayBuffer, JsValue> {
    let bytes = ipc::write_table_with_compression(handle, enable_lz4)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let js_array = js_sys::Uint8Array::from(&bytes[..]);
    Ok(js_array.buffer())
}

/// Write table to IPC stream format
#[wasm_bindgen]
pub fn write_table_to_ipc_stream(handle: TableHandle, enable_lz4: bool) -> Result<Vec<u8>, JsValue> {
    let options = ipc::create_ipc_options(enable_lz4);
    ipc::write_table_to_ipc_stream_with_options(handle, &options)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Write table to Feather format
#[wasm_bindgen]
pub fn write_table_to_feather(handle: TableHandle) -> Result<Vec<u8>, JsValue> {
    fs::write_table_to_feather(handle)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Free the Table represented by handle
#[wasm_bindgen]
pub fn free_table(handle: TableHandle) -> Result<(), JsValue> {
    mem::free_table(handle)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Return a JSON `SchemaSummary` string for the table
#[wasm_bindgen]
pub fn table_schema_summary(handle: TableHandle) -> Result<String, JsValue> {
    mem::get_table_schema_summary(handle)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get table metadata as JSON string
#[wasm_bindgen]
pub fn get_table_metadata(handle: TableHandle) -> Result<JsValue, JsValue> {
    let metadata = mem::get_table_metadata(handle)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    to_value(&metadata)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Check if a handle is valid
#[wasm_bindgen]
pub fn is_valid_handle(handle: TableHandle) -> bool {
    mem::is_valid_handle(handle)
}

/// Get the number of currently registered tables
#[wasm_bindgen]
pub fn get_table_count() -> usize {
    mem::get_table_count()
}

/// Get memory usage statistics as JSON
#[wasm_bindgen]
pub fn get_memory_stats() -> Result<JsValue, JsValue> {
    let stats = mem::get_memory_stats()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    to_value(&stats)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Clear all tables (for testing and cleanup)
#[wasm_bindgen]
pub fn clear_all_tables() {
    mem::clear_all_tables();
}

/// Check if LZ4 compression is supported
#[wasm_bindgen]
pub fn is_lz4_supported() -> bool {
    ipc::is_lz4_supported()
}

/// Get supported compression types
#[wasm_bindgen]
pub fn get_supported_compression_types() -> Result<JsValue, JsValue> {
    let types = ipc::get_supported_compression_types();
    to_value(&types)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Validate that data appears to be valid Arrow data
#[wasm_bindgen]
pub fn validate_arrow_data(data: &[u8]) -> Result<bool, JsValue> {
    fs::validate_arrow_data(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get format information about a table
#[wasm_bindgen]
pub fn get_table_format_info(handle: TableHandle) -> Result<String, JsValue> {
    fs::get_table_format_info(handle)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Analyze compression information from IPC data
#[wasm_bindgen]
pub fn analyze_ipc_compression(data: &[u8]) -> Result<String, JsValue> {
    ipc::analyze_ipc_compression(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Create compression configuration
#[wasm_bindgen]
pub fn create_compression_config(
    enabled: bool,
    compression_type: &str,
    preserve_dict_id: bool,
) -> Result<JsValue, JsValue> {
    let config = CompressionConfig {
        enabled,
        compression_type: compression_type.to_string(),
        preserve_dict_id,
    };
    
    to_value(&config)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Register a plugin by its exported registration name
#[wasm_bindgen]
pub fn register_plugin(plugin_id: &str) -> Result<(), JsValue> {
    plugin::register_plugin(plugin_id)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get list of available plugin types that can be dynamically loaded
#[wasm_bindgen]
pub fn get_available_plugin_types() -> Result<JsValue, JsValue> {
    let types = plugin::get_available_plugin_types()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    to_value(&types)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Discover all plugins that could be registered
#[wasm_bindgen]
pub fn discover_available_plugins() -> Result<JsValue, JsValue> {
    let plugins = plugin::discover_available_plugins()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    to_value(&plugins)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Validate plugin ID format without registering
#[wasm_bindgen]
pub fn validate_plugin_id_format(plugin_id: &str) -> Result<String, JsValue> {
    plugin::validate_plugin_id_format(plugin_id)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Validate a plugin by confirming its expected registration signature
#[wasm_bindgen]
pub fn validate_plugin(plugin_id: &str) -> Result<(), JsValue> {
    plugin::validate_plugin(plugin_id)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get information about registered plugins
#[wasm_bindgen]
pub fn get_plugin_info() -> Result<JsValue, JsValue> {
    let info = plugin::get_plugin_info()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    to_value(&info)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Register the geometry plugin
#[wasm_bindgen]
pub fn register_geometry_plugin() -> Result<(), JsValue> {
    plugin::register_geometry_plugin()
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Demonstrate geometry plugin functionality
#[wasm_bindgen]
pub fn demonstrate_geometry_plugin() -> Result<(), JsValue> {
    plugin::demonstrate_geometry_plugin()
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Create a sample geometry field for testing
#[wasm_bindgen]
pub fn create_sample_geometry_field(name: &str, geometry_type: &str) -> Result<JsValue, JsValue> {
    let field = plugin::create_sample_geometry_field(name, geometry_type);
    
    // Convert the field to a JSON representation
    let field_info = serde_json::json!({
        "name": field.name(),
        "data_type": format!("{:?}", field.data_type()),
        "nullable": field.is_nullable(),
        "metadata": field.metadata(),
    });
    
    to_value(&field_info)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Create sample WKB point data for testing
#[wasm_bindgen]
pub fn create_sample_point_wkb(x: f64, y: f64) -> Vec<u8> {
    plugin::create_sample_point_wkb(x, y)
}

/// Error handling utilities for JavaScript
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmError {
    message: String,
    error_type: String,
}

#[wasm_bindgen]
impl WasmError {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn error_type(&self) -> String {
        self.error_type.clone()
    }
}

impl From<CoreError> for WasmError {
    fn from(error: CoreError) -> Self {
        let error_type = match error {
            CoreError::Io(_) => "IoError",
            CoreError::Ipc(_) => "IpcError",
            CoreError::Parquet(_) => "ParquetError",
            CoreError::Plugin(_) => "PluginError",
            CoreError::InvalidHandle(_) => "InvalidHandleError",
            CoreError::Memory(_) => "MemoryError",
            CoreError::Schema(_) => "SchemaError",
            CoreError::Validation(_) => "ValidationError",
            CoreError::Other(_) => "OtherError",
        };
        
        Self {
            message: error.to_string(),
            error_type: error_type.to_string(),
        }
    }
}

/// Convert a CoreError to a structured JavaScript error
#[wasm_bindgen]
pub fn core_error_to_js(error_message: &str) -> JsValue {
    // This is a helper function for internal use
    to_value(&WasmError {
        message: error_message.to_string(),
        error_type: "CoreError".to_string(),
    }).unwrap_or_else(|_| JsValue::from_str(error_message))
}

/// Version information
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get build information
#[wasm_bindgen]
pub fn get_build_info() -> Result<JsValue, JsValue> {
    let info = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "lz4_supported": ipc::is_lz4_supported(),
        "compression_types": ipc::get_supported_compression_types(),
    });
    
    to_value(&info)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_initialization() {
        // Test that initialization doesn't panic
        init_with_options(true).unwrap();
        
        // Test version info
        let version = get_version();
        assert!(!version.is_empty());
    }
    
    #[wasm_bindgen_test]
    fn test_table_operations() {
        // Clear any existing tables
        clear_all_tables();
        
        // Verify initial state
        assert_eq!(get_table_count(), 0);
        
        // Test invalid handle
        assert!(!is_valid_handle(999));
        
        // Test memory stats
        let stats = get_memory_stats().unwrap();
        // Should be a valid JS value
        assert!(!stats.is_undefined());
    }
    
    #[wasm_bindgen_test]
    fn test_compression_support() {
        // Test compression support detection
        let lz4_supported = is_lz4_supported();
        println!("LZ4 supported: {}", lz4_supported);
        
        // Test getting supported types
        let types = get_supported_compression_types().unwrap();
        assert!(!types.is_undefined());
    }
    
    #[wasm_bindgen_test]
    fn test_build_info() {
        let build_info = get_build_info().unwrap();
        assert!(!build_info.is_undefined());
    }
}