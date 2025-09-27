//! Table manipulation and operations for the WASM Arrow library.
//!
//! This module provides table creation, loading, and manipulation operations.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use arrow_array::RecordBatch;
use arrow_ipc::reader::{FileReader, StreamReader};
use arrow_ipc::writer::{IpcWriteOptions, FileWriter};
use arrow_select::concat::concat_batches;
use crate::{Schema, error::ArrowError, core::HandleId, types::{CompressionType, MetadataVersion}};
use std::io::Cursor;
use std::sync::Arc;

/// Write options for Arrow IPC format
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOptions {
    compression: Option<CompressionType>,
    alignment: Option<usize>,
    metadata_version: Option<MetadataVersion>,
    metadata: std::collections::HashMap<String, String>,
}

#[wasm_bindgen]
impl WriteOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WriteOptions {
        WriteOptions {
            compression: None,
            alignment: None,
            metadata_version: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    #[wasm_bindgen(js_name = "withCompression")]
    pub fn with_compression(&self, compression: CompressionType) -> WriteOptions {
        let mut options = self.clone();
        options.compression = Some(compression);
        options
    }

    #[wasm_bindgen(js_name = "withAlignment")]
    pub fn with_alignment(&self, alignment: usize) -> WriteOptions {
        let mut options = self.clone();
        options.alignment = Some(alignment);
        options
    }

    #[wasm_bindgen(js_name = "withMetadataVersion")]
    pub fn with_metadata_version(&self, version: MetadataVersion) -> WriteOptions {
        let mut options = self.clone();
        options.metadata_version = Some(version);
        options
    }

    #[wasm_bindgen(js_name = "withMetadata")]
    pub fn with_metadata(&self, metadata: JsValue) -> Result<WriteOptions, JsValue> {
        let metadata_map: std::collections::HashMap<String, String> = 
            serde_wasm_bindgen::from_value(metadata)
                .map_err(|e| JsValue::from_str(&format!("Invalid metadata: {}", e)))?;
        
        let mut options = self.clone();
        options.metadata = metadata_map;
        Ok(options)
    }
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Row interface for accessing table data
#[wasm_bindgen]
pub struct Row {
    table_handle: HandleId,
    row_index: usize,
}

#[wasm_bindgen]
impl Row {
    /// Get value by column name
    #[wasm_bindgen]
    pub fn get(&self, column: &str) -> JsValue {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if let Ok(column_index) = batch.schema().index_of(column) {
                    self.get_at(column_index)
                } else {
                    JsValue::NULL
                }
            } else {
                JsValue::NULL
            }
        })
    }

    /// Get value by column index
    #[wasm_bindgen(js_name = "getAt")]
    pub fn get_at(&self, index: usize) -> JsValue {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if index < batch.num_columns() && self.row_index < batch.num_rows() {
                    let column = batch.column(index);
                    let schema = batch.schema();
                    let field = schema.field(index);
                    
                    if column.is_null(self.row_index) {
                        return JsValue::NULL;
                    }
                    
                    match field.data_type() {
                        ArrowDataType::Int32 => {
                            if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                JsValue::from(int_array.value(self.row_index))
                            } else {
                                JsValue::from_str("Cast error: Int32")
                            }
                        },
                        ArrowDataType::Float64 => {
                            if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                JsValue::from(float_array.value(self.row_index))
                            } else {
                                JsValue::from_str("Cast error: Float64")
                            }
                        },
                        ArrowDataType::Utf8 => {
                            if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                JsValue::from_str(string_array.value(self.row_index))
                            } else {
                                JsValue::from_str("Cast error: String")
                            }
                        },
                        ArrowDataType::Boolean => {
                            if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                JsValue::from(bool_array.value(self.row_index))
                            } else {
                                JsValue::from_str("Cast error: Boolean")
                            }
                        },
                        _ => {
                            JsValue::from_str(&format!("Unsupported type: {:?}", field.data_type()))
                        }
                    }
                } else {
                    JsValue::UNDEFINED
                }
            } else {
                JsValue::UNDEFINED
            }
        })
    }

    /// Convert row to object
    #[wasm_bindgen(js_name = "toObject")]
    pub fn to_object(&self) -> JsValue {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.row_index < batch.num_rows() {
                    let schema = batch.schema();
                    let row_obj = js_sys::Object::new();
                    
                    // Extract values for each column
                    for (col_idx, field) in schema.fields().iter().enumerate() {
                        let column = batch.column(col_idx);
                        let field_name = field.name();
                        
                        let js_value = if column.is_null(self.row_index) {
                            JsValue::NULL
                        } else {
                            match field.data_type() {
                                ArrowDataType::Int32 => {
                                    if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                        JsValue::from(int_array.value(self.row_index))
                                    } else {
                                        JsValue::from_str("Cast error: Int32")
                                    }
                                },
                                ArrowDataType::Float64 => {
                                    if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                        JsValue::from(float_array.value(self.row_index))
                                    } else {
                                        JsValue::from_str("Cast error: Float64")
                                    }
                                },
                                ArrowDataType::Utf8 => {
                                    if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                        JsValue::from_str(string_array.value(self.row_index))
                                    } else {
                                        JsValue::from_str("Cast error: String")
                                    }
                                },
                                ArrowDataType::Boolean => {
                                    if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                        JsValue::from(bool_array.value(self.row_index))
                                    } else {
                                        JsValue::from_str("Cast error: Boolean")
                                    }
                                },
                                _ => {
                                    JsValue::from_str(&format!("Unsupported type: {:?}", field.data_type()))
                                }
                            }
                        };
                        
                        // Set the property on the row object
                        let _ = js_sys::Reflect::set(&row_obj, &JsValue::from_str(field_name), &js_value);
                    }
                    
                    row_obj.into()
                } else {
                    JsValue::NULL
                }
            } else {
                JsValue::NULL
            }
        })
    }
}

/// Table interface for Arrow record batches
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Table {
    pub(crate) handle: HandleId,
}

#[wasm_bindgen]
impl Table {
    /// Get number of rows
    #[wasm_bindgen(getter, js_name = "numRows")]
    pub fn num_rows(&self) -> usize {
        crate::core::with_table_registry(|registry| {
            registry.get(self.handle)
                .map(|batch| batch.num_rows())
                .unwrap_or(0)
        })
    }

    /// Get number of columns
    #[wasm_bindgen(getter, js_name = "numColumns")]
    pub fn num_columns(&self) -> usize {
        crate::core::with_table_registry(|registry| {
            registry.get(self.handle)
                .map(|batch| batch.num_columns())
                .unwrap_or(0)
        })
    }

    /// Get table schema
    #[wasm_bindgen(getter)]
    pub fn schema(&self) -> Option<Schema> {
        crate::core::with_table_registry(|registry| {
            registry.get(self.handle).map(|batch| {
                crate::schema::create_schema_from_arrow((*batch.schema()).clone())
            })
        })
    }

    /// Get column by name with comprehensive error handling
    #[wasm_bindgen(js_name = "getColumn")]
    pub fn get_column(&self, name: &str) -> Result<crate::column::Column, JsValue> {
        // Validate input
        if name.is_empty() {
            return Err(JsValue::from_str("Column name cannot be empty"));
        }

        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.handle) {
                match batch.schema().index_of(name) {
                    Ok(index) => {
                        // Column found, create Column object
                        Ok(crate::column::Column::from_table_column(self.handle, index))
                    },
                    Err(_) => {
                        // Column not found, provide helpful error message
                        let schema = batch.schema();
                        let available_columns: Vec<String> = schema.fields()
                            .iter()
                            .map(|field| field.name().clone())
                            .collect();
                        
                        Err(JsValue::from_str(&format!(
                            "Column '{}' not found. Available columns: [{}]", 
                            name, 
                            available_columns.join(", ")
                        )))
                    }
                }
            } else {
                Err(JsValue::from_str("Table has been disposed or is invalid"))
            }
        })
    }

    /// Get column by index
    #[wasm_bindgen(js_name = "getColumnAt")]
    pub fn get_column_at(&self, index: usize) -> Option<crate::column::Column> {
        crate::core::with_table_registry(|registry| {
            registry.get(self.handle).and_then(|batch| {
                if index < batch.num_columns() {
                    Some(crate::column::Column::from_table_column(self.handle, index))
                } else {
                    None
                }
            })
        })
    }

    /// Create zero-copy slice of the table with comprehensive validation
    #[wasm_bindgen]
    pub fn slice(&self, offset: usize, length: usize) -> Result<Table, JsValue> {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.handle) {
                let num_rows = batch.num_rows();
                
                // Validate input parameters according to API specification
                if length == 0 {
                    return Err(JsValue::from_str("Slice length must be greater than 0"));
                }
                
                if offset >= num_rows {
                    return Err(JsValue::from_str(&format!(
                        "Slice offset {} is out of bounds. Table has {} rows", 
                        offset, num_rows
                    )));
                }
                
                if offset + length > num_rows {
                    return Err(JsValue::from_str(&format!(
                        "Slice range [{}..{}] exceeds table bounds. Table has {} rows",
                        offset, offset + length, num_rows
                    )));
                }
                
                // Perform zero-copy slice using Arrow's built-in slice method
                let sliced_batch = batch.slice(offset, length);
                
                // Verify the slice was created correctly
                if sliced_batch.num_rows() != length {
                    return Err(JsValue::from_str(&format!(
                        "Internal error: slice operation produced {} rows instead of expected {}",
                        sliced_batch.num_rows(), length
                    )));
                }
                
                // Register the new sliced batch and create a new Table handle
                let new_handle = crate::core::with_table_registry(|reg| {
                    reg.insert(sliced_batch)
                });
                
                Ok(Table { handle: new_handle })
            } else {
                Err(JsValue::from_str("Table has been disposed or is invalid"))
            }
        })
    }

    /// Select specific columns
    #[wasm_bindgen]
    pub fn select(&self, columns: JsValue) -> Result<Table, JsValue> {
        let column_names: Vec<String> = serde_wasm_bindgen::from_value(columns)
            .map_err(|e| JsValue::from_str(&format!("Invalid column names: {}", e)))?;

        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.handle) {
                let schema = batch.schema();
                
                // Find column indices for the requested column names
                let mut column_indices = Vec::new();
                for column_name in &column_names {
                    match schema.index_of(column_name) {
                        Ok(index) => column_indices.push(index),
                        Err(_) => return Err(JsValue::from_str(&format!("Column '{}' not found", column_name)))
                    }
                }
                
                // Create new schema with selected fields
                let selected_fields: Vec<_> = column_indices.iter()
                    .map(|&index| schema.field(index).clone())
                    .collect();
                let new_schema = std::sync::Arc::new(arrow_schema::Schema::new(selected_fields));
                
                // Create new columns by selecting from the existing batch
                let selected_columns: Vec<_> = column_indices.iter()
                    .map(|&index| batch.column(index).clone())
                    .collect();
                
                // Create new RecordBatch with selected columns
                match arrow_array::RecordBatch::try_new(new_schema, selected_columns) {
                    Ok(new_batch) => {
                        let handle = crate::core::with_table_registry(|reg| {
                            reg.insert(new_batch)
                        });
                        Ok(Table { handle })
                    },
                    Err(e) => Err(JsValue::from_str(&format!("Failed to create selected table: {}", e)))
                }
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }

    /// Filter table based on predicate
    #[wasm_bindgen]
    pub fn filter(&self, predicate: &js_sys::Function) -> Result<Table, JsValue> {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.handle) {
                let num_rows = batch.num_rows();
                let schema = batch.schema();
                let mut keep_rows = Vec::new();
                
                // Test each row with the predicate
                for row_index in 0..num_rows {
                    // Create row object for this row
                    let row_obj = js_sys::Object::new();
                    
                    // Extract values for each column in this row
                    for (col_idx, field) in schema.fields().iter().enumerate() {
                        let column = batch.column(col_idx);
                        let field_name = field.name();
                        
                        let js_value = if column.is_null(row_index) {
                            JsValue::NULL
                        } else {
                            match field.data_type() {
                                ArrowDataType::Int32 => {
                                    if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                        JsValue::from(int_array.value(row_index))
                                    } else {
                                        JsValue::from_str("Cast error: Int32")
                                    }
                                },
                                ArrowDataType::Float64 => {
                                    if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                        JsValue::from(float_array.value(row_index))
                                    } else {
                                        JsValue::from_str("Cast error: Float64")
                                    }
                                },
                                ArrowDataType::Utf8 => {
                                    if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                        JsValue::from_str(string_array.value(row_index))
                                    } else {
                                        JsValue::from_str("Cast error: String")
                                    }
                                },
                                ArrowDataType::Boolean => {
                                    if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                        JsValue::from(bool_array.value(row_index))
                                    } else {
                                        JsValue::from_str("Cast error: Boolean")
                                    }
                                },
                                _ => {
                                    JsValue::from_str(&format!("Unsupported type: {:?}", field.data_type()))
                                }
                            }
                        };
                        
                        // Set the property on the row object
                        let _ = js_sys::Reflect::set(&row_obj, &JsValue::from_str(field_name), &js_value);
                    }
                    
                    // Call the JavaScript predicate function with the row object
                    let this = JsValue::NULL;
                    let result = predicate.call1(&this, &row_obj);
                    
                    match result {
                        Ok(js_result) => {
                            if js_result.is_truthy() {
                                keep_rows.push(row_index);
                            }
                        },
                        Err(_) => {
                            // If predicate throws an error, skip this row
                            continue;
                        }
                    }
                }
                
                if keep_rows.is_empty() {
                    // Create empty table with same schema
                    let schema = batch.schema();
                    
                    // Check if all field types are supported
                    for field in schema.fields() {
                        match field.data_type() {
                            arrow_schema::DataType::Int32 |
                            arrow_schema::DataType::Float64 |
                            arrow_schema::DataType::Utf8 |
                            arrow_schema::DataType::Boolean => {},
                            _ => {
                                return Err(JsValue::from_str(&format!("Unsupported data type for filtering: {:?}", field.data_type())));
                            }
                        }
                    }
                    
                    let empty_columns: Vec<arrow_array::ArrayRef> = schema.fields().iter()
                        .map(|field| {
                            match field.data_type() {
                                arrow_schema::DataType::Int32 => {
                                    std::sync::Arc::new(arrow_array::Int32Array::from(Vec::<Option<i32>>::new())) as arrow_array::ArrayRef
                                },
                                arrow_schema::DataType::Float64 => {
                                    std::sync::Arc::new(arrow_array::Float64Array::from(Vec::<Option<f64>>::new())) as arrow_array::ArrayRef
                                },
                                arrow_schema::DataType::Utf8 => {
                                    std::sync::Arc::new(arrow_array::StringArray::from(Vec::<Option<String>>::new())) as arrow_array::ArrayRef
                                },
                                arrow_schema::DataType::Boolean => {
                                    std::sync::Arc::new(arrow_array::BooleanArray::from(Vec::<Option<bool>>::new())) as arrow_array::ArrayRef
                                },
                                _ => {
                                    // This should never happen due to the check above
                                    unreachable!()
                                }
                            }
                        })
                        .collect();
                    
                    match arrow_array::RecordBatch::try_new(schema, empty_columns) {
                        Ok(empty_batch) => {
                            let handle = crate::core::with_table_registry(|reg| {
                                reg.insert(empty_batch)
                            });
                            Ok(Table { handle })
                        },
                        Err(e) => Err(JsValue::from_str(&format!("Failed to create empty filtered table: {}", e)))
                    }
                } else {
                    // Create new arrays with only the selected rows
                    let schema = batch.schema();
                    let filtered_columns: Result<Vec<_>, _> = batch.columns().iter().enumerate()
                        .map(|(col_idx, column)| {
                            let field = schema.field(col_idx);
                            match field.data_type() {
                                arrow_schema::DataType::Int32 => {
                                    if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                        let filtered_values: Vec<Option<i32>> = keep_rows.iter()
                                            .map(|&row_idx| {
                                                if int_array.is_null(row_idx) {
                                                    None
                                                } else {
                                                    Some(int_array.value(row_idx))
                                                }
                                            })
                                            .collect();
                                        Ok(std::sync::Arc::new(arrow_array::Int32Array::from(filtered_values)) as arrow_array::ArrayRef)
                                    } else {
                                        Err(format!("Failed to cast column {} to Int32Array", col_idx))
                                    }
                                },
                                arrow_schema::DataType::Float64 => {
                                    if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                        let filtered_values: Vec<Option<f64>> = keep_rows.iter()
                                            .map(|&row_idx| {
                                                if float_array.is_null(row_idx) {
                                                    None
                                                } else {
                                                    Some(float_array.value(row_idx))
                                                }
                                            })
                                            .collect();
                                        Ok(std::sync::Arc::new(arrow_array::Float64Array::from(filtered_values)) as arrow_array::ArrayRef)
                                    } else {
                                        Err(format!("Failed to cast column {} to Float64Array", col_idx))
                                    }
                                },
                                arrow_schema::DataType::Utf8 => {
                                    if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                        let filtered_values: Vec<Option<String>> = keep_rows.iter()
                                            .map(|&row_idx| {
                                                if string_array.is_null(row_idx) {
                                                    None
                                                } else {
                                                    Some(string_array.value(row_idx).to_string())
                                                }
                                            })
                                            .collect();
                                        Ok(std::sync::Arc::new(arrow_array::StringArray::from(filtered_values)) as arrow_array::ArrayRef)
                                    } else {
                                        Err(format!("Failed to cast column {} to StringArray", col_idx))
                                    }
                                },
                                arrow_schema::DataType::Boolean => {
                                    if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                        let filtered_values: Vec<Option<bool>> = keep_rows.iter()
                                            .map(|&row_idx| {
                                                if bool_array.is_null(row_idx) {
                                                    None
                                                } else {
                                                    Some(bool_array.value(row_idx))
                                                }
                                            })
                                            .collect();
                                        Ok(std::sync::Arc::new(arrow_array::BooleanArray::from(filtered_values)) as arrow_array::ArrayRef)
                                    } else {
                                        Err(format!("Failed to cast column {} to BooleanArray", col_idx))
                                    }
                                },
                                _ => {
                                    Err(format!("Unsupported data type for filtering: {:?}", field.data_type()))
                                }
                            }
                        })
                        .collect();
                    
                    match filtered_columns {
                        Ok(columns) => {
                            match arrow_array::RecordBatch::try_new(schema, columns) {
                                Ok(filtered_batch) => {
                                    let handle = crate::core::with_table_registry(|reg| {
                                        reg.insert(filtered_batch)
                                    });
                                    Ok(Table { handle })
                                },
                                Err(e) => Err(JsValue::from_str(&format!("Failed to create filtered table: {}", e)))
                            }
                        },
                        Err(e) => Err(JsValue::from_str(&format!("Failed to filter columns: {}", e)))
                    }
                }
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }

    /// Convert table to array of objects
    #[wasm_bindgen(js_name = "toArray")]
    pub fn to_array(&self) -> JsValue {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.handle) {
                let schema = batch.schema();
                let num_rows = batch.num_rows();
                let result_array = js_sys::Array::new();
                
                // Convert each row to a JavaScript object
                for row_idx in 0..num_rows {
                    let row_obj = js_sys::Object::new();
                    
                    // Extract values for each column
                    for (col_idx, field) in schema.fields().iter().enumerate() {
                        let column = batch.column(col_idx);
                        let field_name = field.name();
                        
                        let js_value = match field.data_type() {
                            ArrowDataType::Int32 => {
                                if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                    if int_array.is_null(row_idx) {
                                        JsValue::NULL
                                    } else {
                                        JsValue::from(int_array.value(row_idx))
                                    }
                                } else {
                                    JsValue::from_str("Cast error: Int32")
                                }
                            },
                            ArrowDataType::Float64 => {
                                if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                    if float_array.is_null(row_idx) {
                                        JsValue::NULL
                                    } else {
                                        JsValue::from(float_array.value(row_idx))
                                    }
                                } else {
                                    JsValue::from_str("Cast error: Float64")
                                }
                            },
                            ArrowDataType::Utf8 => {
                                if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                    if string_array.is_null(row_idx) {
                                        JsValue::NULL
                                    } else {
                                        JsValue::from_str(string_array.value(row_idx))
                                    }
                                } else {
                                    JsValue::from_str("Cast error: String")
                                }
                            },
                            ArrowDataType::Boolean => {
                                if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                    if bool_array.is_null(row_idx) {
                                        JsValue::NULL
                                    } else {
                                        JsValue::from(bool_array.value(row_idx))
                                    }
                                } else {
                                    JsValue::from_str("Cast error: Boolean")
                                }
                            },
                            _ => {
                                // For unsupported types, convert to string representation
                                if column.is_null(row_idx) {
                                    JsValue::NULL
                                } else {
                                    JsValue::from_str(&format!("Unsupported type: {:?}", field.data_type()))
                                }
                            }
                        };
                        
                        // Set the property on the row object
                        let _ = js_sys::Reflect::set(&row_obj, &JsValue::from_str(field_name), &js_value);
                    }
                    
                    result_array.push(&row_obj);
                }
                
                result_array.into()
            } else {
                JsValue::NULL
            }
        })
    }

    /// Serialize table to IPC format
    #[wasm_bindgen(js_name = "toIPC")]
    pub fn to_ipc(&self, options: Option<WriteOptions>) -> Result<js_sys::Uint8Array, JsValue> {
        let _write_options = options.unwrap_or_default();
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.handle) {
                let mut buffer = Vec::new();
                {
                    let mut writer = FileWriter::try_new(&mut buffer, &batch.schema())
                        .map_err(|e| JsValue::from_str(&format!("Failed to create writer: {}", e)))?;
                    
                    writer.write(&batch)
                        .map_err(|e| JsValue::from_str(&format!("Failed to write batch: {}", e)))?;
                    
                    writer.finish()
                        .map_err(|e| JsValue::from_str(&format!("Failed to finish writing: {}", e)))?;
                }
                
                Ok(js_sys::Uint8Array::from(buffer.as_slice()))
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }

    /// Dispose of the table handle
    #[wasm_bindgen]
    pub fn dispose(&self) {
        crate::core::with_table_registry(|registry| {
            registry.remove(self.handle);
        });
    }
}

/// Create a Table instance from IPC-formatted Arrow data
#[wasm_bindgen(js_name = "tableFromIPC")]
pub fn table_from_ipc(buffer: js_sys::Uint8Array) -> Result<Table, JsValue> {
    let buffer_vec = buffer.to_vec();
    let cursor = Cursor::new(buffer_vec);
    let reader = FileReader::try_new(cursor, None)
        .map_err(|e| JsValue::from_str(&format!("Failed to create reader: {}", e)))?;

    // Collect all batches from the IPC data
    let mut batches = Vec::new();
    let mut schema = None;
    
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| JsValue::from_str(&format!("Failed to read batch: {}", e)))?;
        
        // Store the schema from the first batch
        if schema.is_none() {
            schema = Some(batch.schema());
        }
        
        batches.push(batch);
    }

    // Check if we have any batches
    if batches.is_empty() {
        return Err(JsValue::from_str("No batches found in IPC data"));
    }

    // If we have only one batch, use it directly
    let final_batch = if batches.len() == 1 {
        batches.into_iter().next().unwrap()
    } else {
        // Concatenate multiple batches
        let schema = schema.unwrap();
        concat_batches(&schema, &batches)
            .map_err(|e| JsValue::from_str(&format!("Failed to concatenate batches: {}", e)))?
    };
    
    let handle = crate::core::with_table_registry(|registry| {
        registry.insert(final_batch)
    });
    
    Ok(Table { handle })
}

/// Create a Table from JSON data with schema inference
#[wasm_bindgen(js_name = "tableFromJSON")]
pub fn table_from_json(data: JsValue, _schema: Option<Schema>) -> Result<Table, JsValue> {
    use arrow_array::{Int32Array, Float64Array, StringArray, RecordBatch};
    use arrow_schema::{Schema as ArrowSchema, Field, DataType as ArrowDataType};
    use std::sync::Arc;

    // Parse JSON array from JavaScript
    let json_array: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON data: {}", e)))?;

    if json_array.is_empty() {
        return Err(JsValue::from_str("Cannot create table from empty data"));
    }

    // Simple schema inference from the first row
    let first_row = &json_array[0];
    let mut fields = Vec::new();
    let mut column_names = Vec::new();

    if let serde_json::Value::Object(obj) = first_row {
        for (key, value) in obj {
            column_names.push(key.clone());
            let arrow_type = match value {
                serde_json::Value::Number(n) => {
                    if n.is_i64() {
                        ArrowDataType::Int32 // Simplified - use Int32 for integers
                    } else {
                        ArrowDataType::Float64
                    }
                },
                serde_json::Value::String(_) => ArrowDataType::Utf8,
                serde_json::Value::Bool(_) => ArrowDataType::Boolean,
                _ => ArrowDataType::Utf8, // Default to string for complex types
            };
            fields.push(Field::new(key, arrow_type, true)); // Allow nulls
        }
    } else {
        return Err(JsValue::from_str("JSON data must be an array of objects"));
    }

    let schema = Arc::new(ArrowSchema::new(fields));
    
    // Build columns
    let mut columns: Vec<Arc<dyn arrow_array::Array>> = Vec::new();
    
    for (col_idx, col_name) in column_names.iter().enumerate() {
        let field = &schema.fields()[col_idx];
        
        match field.data_type() {
            ArrowDataType::Int32 => {
                let values: Vec<Option<i32>> = json_array.iter()
                    .map(|row| {
                        if let serde_json::Value::Object(obj) = row {
                            obj.get(col_name)
                                .and_then(|v| v.as_i64())
                                .map(|i| i as i32)
                        } else {
                            None
                        }
                    })
                    .collect();
                columns.push(Arc::new(Int32Array::from(values)));
            },
            ArrowDataType::Float64 => {
                let values: Vec<Option<f64>> = json_array.iter()
                    .map(|row| {
                        if let serde_json::Value::Object(obj) = row {
                            obj.get(col_name).and_then(|v| v.as_f64())
                        } else {
                            None
                        }
                    })
                    .collect();
                columns.push(Arc::new(Float64Array::from(values)));
            },
            ArrowDataType::Utf8 => {
                let values: Vec<Option<String>> = json_array.iter()
                    .map(|row| {
                        if let serde_json::Value::Object(obj) = row {
                            obj.get(col_name).and_then(|v| {
                                match v {
                                    serde_json::Value::String(s) => Some(s.clone()),
                                    serde_json::Value::Number(n) => Some(n.to_string()),
                                    serde_json::Value::Bool(b) => Some(b.to_string()),
                                    serde_json::Value::Null => None,
                                    _ => Some(format!("{}", v)),
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                columns.push(Arc::new(StringArray::from(values)));
            },
            _ => {
                return Err(JsValue::from_str(&format!("Unsupported data type for column {}", col_name)));
            }
        }
    }

    // Create RecordBatch
    let batch = RecordBatch::try_new(schema, columns)
        .map_err(|e| JsValue::from_str(&format!("Failed to create record batch: {}", e)))?;

    // Store in registry and return handle
    let handle = crate::core::with_table_registry(|registry| {
        registry.insert(batch)
    });

    Ok(Table { handle })
}

/// Create table from record batch (internal function)
pub fn create_table_from_batch(batch: RecordBatch) -> Table {
    let handle = crate::core::with_table_registry(|registry| {
        registry.insert(batch)
    });
    
    Table { handle }
}