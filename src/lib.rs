mod errors;
mod mem;

use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow_ipc::writer::IpcWriteOptions;
use std::io::Cursor;
use std::sync::Arc;
use arrow::record_batch::RecordBatch;

pub use errors::{ArrowWasmError, Result};
pub use mem::{TableHandle, TableData};

// Re-export core functions from mem module
pub use mem::{
    table_row_count,
    export_column_by_name,
    free_table,
    get_memory_info,
    table_column_count,
    get_column_names,
};

// Console logging setup for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Initialize panic hook for better error reporting
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// Optional initialization with configuration
#[wasm_bindgen]
pub fn init_with_options(enable_console_logs: bool) {
    if enable_console_logs {
        console_error_panic_hook::set_once();
    }
}

// Core API function: Read table from bytes (Arrow IPC format)
#[wasm_bindgen]
pub fn read_table_from_bytes(data: &[u8]) -> std::result::Result<TableHandle, JsValue> {
    let cursor = Cursor::new(data);
    let reader = StreamReader::try_new(cursor, None)
        .map_err(|e| ArrowWasmError::Ipc(e.to_string()))?;
    
    let mut batches = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| ArrowWasmError::Ipc(e.to_string()))?;
        batches.push(batch);
    }
    
    if batches.is_empty() {
        return Err(ArrowWasmError::InvalidInput("No record batches found in data".to_string()).into());
    }
    
    let table_data = TableData::new(batches)?;
    let handle = mem::store_table(table_data)?;
    Ok(handle)
}

// Core API function: Write table to Arrow IPC format
#[wasm_bindgen]
pub fn write_table_to_ipc(handle: TableHandle, enable_lz4: bool) -> std::result::Result<Uint8Array, JsValue> {
    let table = mem::get_table(handle)?;
    
    let mut buffer = Vec::new();
    let options = IpcWriteOptions::default()
        .try_with_compression(if enable_lz4 { 
            Some(arrow_ipc::CompressionType::LZ4_FRAME) 
        } else { 
            None 
        })
        .map_err(|e| ArrowWasmError::Ipc(e.to_string()))?;
    
    {
        let mut writer = StreamWriter::try_new_with_options(&mut buffer, &table.schema, options)
            .map_err(|e| ArrowWasmError::Ipc(e.to_string()))?;
        
        for batch in &table.batches {
            writer.write(batch)
                .map_err(|e| ArrowWasmError::Ipc(e.to_string()))?;
        }
        
        writer.finish()
            .map_err(|e| ArrowWasmError::Ipc(e.to_string()))?;
    }
    
    // Create Uint8Array from the buffer (this creates a copy)
    let uint8_array = Uint8Array::new_with_length(buffer.len() as u32);
    uint8_array.copy_from(&buffer);
    
    Ok(uint8_array)
}

// Create a simple table from column data (for testing)
#[wasm_bindgen]
pub fn create_test_table() -> std::result::Result<TableHandle, JsValue> {
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    
    // Create simple test data
    let int_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
    let string_array = StringArray::from(vec!["a", "b", "c", "d", "e"]);
    
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
    ]));
    
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(int_array), Arc::new(string_array)],
    ).map_err(|e| ArrowWasmError::Arrow(e))?;
    
    let table_data = TableData::new(vec![batch])?;
    let handle = mem::store_table(table_data)?;
    Ok(handle)
}

// Get schema information as JSON string
#[wasm_bindgen]
pub fn get_table_schema_json(handle: TableHandle) -> std::result::Result<String, JsValue> {
    let table = mem::get_table(handle)?;
    let fields_info: Vec<_> = table.schema.fields().iter().map(|field| {
        format!("{{\"name\": \"{}\", \"type\": \"{:?}\", \"nullable\": {}}}", 
               field.name(), field.data_type(), field.is_nullable())
    }).collect();
    let schema_json = format!("[{}]", fields_info.join(","));
    Ok(schema_json)
}

// Enhanced column export with type information
#[wasm_bindgen]
pub fn export_column_with_type(handle: TableHandle, column_name: &str) -> std::result::Result<JsValue, JsValue> {
    let table = mem::get_table(handle)?;
    let arrays = table.get_column_by_name(column_name)?;
    
    if arrays.is_empty() {
        return Err(ArrowWasmError::InvalidInput("No data in column".to_string()).into());
    }
    
    // Get column type information
    let field_index = table.schema
        .index_of(column_name)
        .map_err(|_| ArrowWasmError::InvalidInput(format!("Column '{}' not found", column_name)))?;
    let field = table.schema.field(field_index);
    
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"column_name".into(), &column_name.into())?;
    js_sys::Reflect::set(&result, &"data_type".into(), &format!("{:?}", field.data_type()).into())?;
    js_sys::Reflect::set(&result, &"nullable".into(), &field.is_nullable().into())?;
    
    // For the first array, export raw buffer data
    let array = &arrays[0];
    let data = array.to_data();
    
    if let Some(buffer) = data.buffers().first() {
        let bytes = unsafe {
            js_sys::Uint8Array::view(buffer.as_slice())
        };
        js_sys::Reflect::set(&result, &"data".into(), &bytes)?;
    }
    
    Ok(result.into())
}

// Get detailed table information
#[wasm_bindgen]
pub fn get_table_info(handle: TableHandle) -> std::result::Result<JsValue, JsValue> {
    let table = mem::get_table(handle)?;
    
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"row_count".into(), &(table.row_count() as u32).into())?;
    js_sys::Reflect::set(&result, &"column_count".into(), &(table.column_count() as u32).into())?;
    js_sys::Reflect::set(&result, &"batch_count".into(), &(table.batches.len() as u32).into())?;
    
    // Column names array
    let column_names = js_sys::Array::new();
    for field in table.schema.fields() {
        column_names.push(&field.name().into());
    }
    js_sys::Reflect::set(&result, &"column_names".into(), &column_names)?;
    
    Ok(result.into())
}