//! File reading and writing operations for the WASM Arrow library.

use wasm_bindgen::prelude::*;
use crate::{Table, table::WriteOptions, error::WasmResult};
use arrow_ipc::reader::FileReader;
use arrow_ipc::writer::FileWriter;
use std::io::Cursor;

/// Read an Arrow file from provided data
#[wasm_bindgen(js_name = "readFile")]
pub async fn read_file(data: &[u8]) -> Result<Table, JsValue> {
    // Validate Arrow file format (magic bytes)
    if data.len() < 6 {
        return Err(JsValue::from_str("Data too short to be valid Arrow file"));
    }

    // Check for Arrow file magic bytes: "ARROW1"
    let magic_bytes = &data[data.len() - 6..];
    if magic_bytes != b"ARROW1" {
        return Err(JsValue::from_str("Invalid Arrow file format - missing magic bytes"));
    }

    let cursor = Cursor::new(data);
    let reader = FileReader::try_new(cursor, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to create file reader: {}", e)))?;

    // Read all record batches from the file
    let mut batches = Vec::new();
    let mut schema = None;
    
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| JsValue::from_str(&format!("Failed to read record batch: {}", e)))?;
        
        // Store schema from first batch
        if schema.is_none() {
            schema = Some(batch.schema());
        }
        
        batches.push(batch);
    }

    if batches.is_empty() {
        return Err(JsValue::from_str("No record batches found in Arrow file"));
    }

    // If single batch, use it directly
    if batches.len() == 1 {
        return Ok(crate::table::create_table_from_batch(batches.into_iter().next().unwrap()));
    }

    // Combine multiple batches into a single batch
    let combined_batch = arrow_select::concat::concat_batches(&schema.unwrap(), &batches)
        .map_err(|e| JsValue::from_str(&format!("Failed to combine record batches: {}", e)))?;

    Ok(crate::table::create_table_from_batch(combined_batch))
}

/// Write a Table to Arrow file format
#[wasm_bindgen(js_name = "writeFile")]
pub async fn write_file(table: &Table, options: Option<WriteOptions>) -> Result<js_sys::Uint8Array, JsValue> {
    // Use the Table's toIPC method which handles the actual writing
    table.to_ipc(options)
}

/// Read file with Result type pattern (no exceptions)
#[wasm_bindgen(js_name = "readFileResult")]
pub async fn read_file_result(data: &[u8]) -> WasmResult {
    match read_file(data).await {
        Ok(table) => {
            let table_js = serde_wasm_bindgen::to_value(&table).unwrap_or(JsValue::NULL);
            WasmResult::success(table_js)
        }
        Err(e) => {
            let error = crate::error::ArrowError::new(
                crate::error::ErrorCode::IOError,
                &format!("Read file failed: {}", e.as_string().unwrap_or_default())
            );
            WasmResult::from_error(error)
        }
    }
}

/// Write file with Result type pattern (no exceptions)
#[wasm_bindgen(js_name = "writeFileResult")]
pub async fn write_file_result(table: &Table, options: Option<WriteOptions>) -> WasmResult {
    match write_file(table, options).await {
        Ok(buffer) => WasmResult::success(buffer.into()),
        Err(e) => {
            let error = crate::error::ArrowError::new(
                crate::error::ErrorCode::IOError,
                &format!("Write file failed: {}", e.as_string().unwrap_or_default())
            );
            WasmResult::from_error(error)
        }
    }
}

/// Utility function to validate Arrow file format
#[wasm_bindgen(js_name = "validateArrowFile")]
pub fn validate_arrow_file(data: &[u8]) -> bool {
    if data.len() < 6 {
        return false;
    }

    // Check magic bytes
    let magic_bytes = &data[data.len() - 6..];
    magic_bytes == b"ARROW1"
}

/// Get Arrow file metadata without reading full content
#[wasm_bindgen(js_name = "getFileMetadata")]
pub fn get_file_metadata(data: &[u8]) -> Result<JsValue, JsValue> {
    if !validate_arrow_file(data) {
        return Err(JsValue::from_str("Invalid Arrow file format"));
    }

    let cursor = Cursor::new(data);
    let reader = FileReader::try_new(cursor, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to create reader: {}", e)))?;

    let schema = reader.schema();
    let num_batches = reader.num_batches();
    
    let metadata = serde_json::json!({
        "schema": {
            "fields": schema.fields().iter().map(|field| {
                serde_json::json!({
                    "name": field.name(),
                    "data_type": format!("{:?}", field.data_type()),
                    "nullable": field.is_nullable()
                })
            }).collect::<Vec<_>>()
        },
        "num_batches": num_batches,
        "metadata": schema.metadata()
    });

    serde_wasm_bindgen::to_value(&metadata)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize metadata: {}", e)))
}