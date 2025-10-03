use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use arrow::record_batch::RecordBatch;
use arrow::datatypes::Schema;
use arrow::array::Array;
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;
use crate::errors::{ArrowWasmError, Result};

pub type TableHandle = u32;

#[derive(Debug, Clone)]
pub struct TableData {
    pub batches: Vec<RecordBatch>,
    pub schema: Arc<Schema>,
}

impl TableData {
    pub fn new(batches: Vec<RecordBatch>) -> Result<Self> {
        if batches.is_empty() {
            return Err(ArrowWasmError::InvalidInput("Empty batch list".to_string()));
        }
        
        let schema = batches[0].schema();
        Ok(Self {
            batches,
            schema,
        })
    }
    
    pub fn row_count(&self) -> usize {
        self.batches.iter().map(|batch| batch.num_rows()).sum()
    }
    
    pub fn column_count(&self) -> usize {
        self.schema.fields().len()
    }
    
    pub fn get_column_by_name(&self, name: &str) -> Result<Vec<Arc<dyn Array>>> {
        let field_index = self.schema
            .index_of(name)
            .map_err(|_| ArrowWasmError::InvalidInput(format!("Column '{}' not found", name)))?;
            
        let mut arrays = Vec::new();
        for batch in &self.batches {
            arrays.push(Arc::clone(batch.column(field_index)));
        }
        Ok(arrays)
    }
}

static TABLES: Lazy<Arc<Mutex<HashMap<TableHandle, TableData>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static NEXT_HANDLE: Lazy<Arc<Mutex<TableHandle>>> = 
    Lazy::new(|| Arc::new(Mutex::new(1)));

pub fn store_table(table: TableData) -> Result<TableHandle> {
    let mut tables = TABLES.lock().map_err(|_| 
        ArrowWasmError::Memory("Failed to acquire table store lock".to_string()))?;
    
    let mut next_handle = NEXT_HANDLE.lock().map_err(|_| 
        ArrowWasmError::Memory("Failed to acquire handle lock".to_string()))?;
    
    let handle = *next_handle;
    *next_handle += 1;
    
    tables.insert(handle, table);
    Ok(handle)
}

pub fn get_table(handle: TableHandle) -> Result<TableData> {
    let tables = TABLES.lock().map_err(|_| 
        ArrowWasmError::Memory("Failed to acquire table store lock".to_string()))?;
    
    tables.get(&handle)
        .cloned()
        .ok_or_else(|| ArrowWasmError::InvalidHandle(handle))
}

pub fn remove_table(handle: TableHandle) -> Result<()> {
    let mut tables = TABLES.lock().map_err(|_| 
        ArrowWasmError::Memory("Failed to acquire table store lock".to_string()))?;
    
    tables.remove(&handle)
        .ok_or_else(|| ArrowWasmError::InvalidHandle(handle))?;
    
    Ok(())
}

pub fn table_exists(handle: TableHandle) -> bool {
    if let Ok(tables) = TABLES.lock() {
        tables.contains_key(&handle)
    } else {
        false
    }
}

pub fn get_table_count() -> usize {
    if let Ok(tables) = TABLES.lock() {
        tables.len()
    } else {
        0
    }
}

#[wasm_bindgen]
pub fn table_row_count(handle: TableHandle) -> std::result::Result<usize, JsValue> {
    let table = get_table(handle)?;
    Ok(table.row_count())
}

#[wasm_bindgen]
pub fn table_column_count(handle: TableHandle) -> std::result::Result<usize, JsValue> {
    let table = get_table(handle)?;
    Ok(table.column_count())
}

#[wasm_bindgen]
pub fn get_column_names(handle: TableHandle) -> std::result::Result<Vec<String>, JsValue> {
    let table = get_table(handle)?;
    let names: Vec<String> = table.schema.fields()
        .iter()
        .map(|field| field.name().clone())
        .collect();
    Ok(names)
}

#[wasm_bindgen]
pub fn export_column_by_name(handle: TableHandle, column_name: &str) -> std::result::Result<Uint8Array, JsValue> {
    let table = get_table(handle)?;
    let arrays = table.get_column_by_name(column_name)?;
    
    if arrays.is_empty() {
        return Err(ArrowWasmError::InvalidInput("No data in column".to_string()).into());
    }
    
    // For simplicity, export the first array's raw data
    // In a full implementation, this would handle concatenation and proper serialization
    let array = &arrays[0];
    let data = array.to_data();
    
    // Get the buffer data - this is a simplified zero-copy approach
    let buffer = data.buffers().first()
        .ok_or_else(|| ArrowWasmError::Buffer("No buffer data available".to_string()))?;
    
    // Create Uint8Array view of the buffer data (zero-copy)
    let bytes = unsafe {
        js_sys::Uint8Array::view(buffer.as_slice())
    };
    
    Ok(bytes)
}

#[wasm_bindgen]
pub fn free_table(handle: TableHandle) -> std::result::Result<(), JsValue> {
    remove_table(handle)?;
    Ok(())
}

#[wasm_bindgen]
pub fn get_memory_info() -> JsValue {
    let table_count = get_table_count();
    
    serde_wasm_bindgen::to_value(&serde_json::json!({
        "table_count": table_count,
        "next_handle": if let Ok(handle) = NEXT_HANDLE.lock() { *handle } else { 0 }
    })).unwrap_or(JsValue::NULL)
}

pub fn clear_all_tables() -> Result<()> {
    let mut tables = TABLES.lock().map_err(|_| 
        ArrowWasmError::Memory("Failed to acquire table store lock".to_string()))?;
    tables.clear();
    Ok(())
}