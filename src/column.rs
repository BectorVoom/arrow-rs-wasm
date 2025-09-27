//! Column interface and array builders for the WASM Arrow library.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{DataType, core::HandleId};
use arrow_array::{Array, ArrayRef, Int32Array, Float64Array, StringArray, BooleanArray, 
                  Int64Array, Float32Array, NullArray};
use arrow_array::builder::{Int32Builder, Int64Builder, Float32Builder, Float64Builder, 
                          StringBuilder, BooleanBuilder, NullBuilder};

/// Column statistics
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    null_count: usize,
    min_value: Option<String>,
    max_value: Option<String>, 
    distinct_count: Option<usize>,
}

#[wasm_bindgen]
impl ColumnStatistics {
    #[wasm_bindgen(getter, js_name = "nullCount")]
    pub fn null_count(&self) -> usize {
        self.null_count
    }

    #[wasm_bindgen(getter)]
    pub fn min(&self) -> Option<String> {
        self.min_value.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn max(&self) -> Option<String> {
        self.max_value.clone()
    }

    #[wasm_bindgen(getter, js_name = "distinctCount")]
    pub fn distinct_count(&self) -> Option<usize> {
        self.distinct_count
    }
}

/// Column interface for Arrow arrays
#[wasm_bindgen]
pub struct Column {
    pub(crate) table_handle: HandleId,
    pub(crate) column_index: usize,
}

impl Column {
    pub fn from_table_column(table_handle: HandleId, column_index: usize) -> Column {
        Column {
            table_handle,
            column_index,
        }
    }
}

#[wasm_bindgen]
impl Column {
    /// Get column name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    batch.schema().field(self.column_index).name().clone()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        })
    }

    /// Get column data type
    #[wasm_bindgen(getter, js_name = "dataType")]
    pub fn data_type(&self) -> DataType {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    batch.schema().field(self.column_index).data_type().into()
                } else {
                    DataType::new_null()
                }
            } else {
                DataType::new_null()
            }
        })
    }

    /// Get column length
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    batch.column(self.column_index).len()
                } else {
                    0
                }
            } else {
                0
            }
        })
    }

    /// Get null count
    #[wasm_bindgen(getter, js_name = "nullCount")]
    pub fn null_count(&self) -> usize {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    batch.column(self.column_index).null_count()
                } else {
                    0
                }
            } else {
                0
            }
        })
    }

    /// Get value at index with enhanced type safety and error handling
    #[wasm_bindgen]
    pub fn get(&self, index: usize) -> JsValue {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index >= batch.num_columns() {
                    // Column index is invalid
                    return JsValue::UNDEFINED;
                }
                
                let column = batch.column(self.column_index);
                let schema = batch.schema();
                let field = schema.field(self.column_index);
                
                if index >= column.len() {
                    // Index out of bounds - return undefined per JavaScript conventions
                    return JsValue::UNDEFINED;
                }
                
                if column.is_null(index) {
                    return JsValue::NULL;
                }
                
                // Type-safe value extraction with proper error handling
                match field.data_type() {
                    ArrowDataType::Int32 => {
                        if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                            JsValue::from(int_array.value(index))
                        } else {
                            // This should never happen if the schema is correct
                            JsValue::from_str("Internal error: Failed to cast Int32 column")
                        }
                    },
                    ArrowDataType::Float64 => {
                        if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                            let value = float_array.value(index);
                            // Handle special float values appropriately
                            if value.is_nan() {
                                JsValue::from_f64(std::f64::NAN)
                            } else if value.is_infinite() {
                                JsValue::from_f64(value)
                            } else {
                                JsValue::from(value)
                            }
                        } else {
                            JsValue::from_str("Internal error: Failed to cast Float64 column")
                        }
                    },
                    ArrowDataType::Utf8 => {
                        if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                            JsValue::from_str(string_array.value(index))
                        } else {
                            JsValue::from_str("Internal error: Failed to cast String column")
                        }
                    },
                    ArrowDataType::Boolean => {
                        if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                            JsValue::from(bool_array.value(index))
                        } else {
                            JsValue::from_str("Internal error: Failed to cast Boolean column")
                        }
                    },
                    ArrowDataType::Int8 => {
                        if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int8Array>() {
                            JsValue::from(int_array.value(index) as i32)
                        } else {
                            JsValue::from_str("Internal error: Failed to cast Int8 column")
                        }
                    },
                    ArrowDataType::Int16 => {
                        if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int16Array>() {
                            JsValue::from(int_array.value(index) as i32)
                        } else {
                            JsValue::from_str("Internal error: Failed to cast Int16 column")
                        }
                    },
                    ArrowDataType::UInt8 => {
                        if let Some(uint_array) = column.as_any().downcast_ref::<arrow_array::UInt8Array>() {
                            JsValue::from(uint_array.value(index) as u32)
                        } else {
                            JsValue::from_str("Internal error: Failed to cast UInt8 column")
                        }
                    },
                    ArrowDataType::UInt16 => {
                        if let Some(uint_array) = column.as_any().downcast_ref::<arrow_array::UInt16Array>() {
                            JsValue::from(uint_array.value(index) as u32)
                        } else {
                            JsValue::from_str("Internal error: Failed to cast UInt16 column")
                        }
                    },
                    ArrowDataType::UInt32 => {
                        if let Some(uint_array) = column.as_any().downcast_ref::<arrow_array::UInt32Array>() {
                            JsValue::from(uint_array.value(index))
                        } else {
                            JsValue::from_str("Internal error: Failed to cast UInt32 column")
                        }
                    },
                    ArrowDataType::Float32 => {
                        if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float32Array>() {
                            let value = float_array.value(index) as f64;
                            if value.is_nan() {
                                JsValue::from_f64(std::f64::NAN)
                            } else if value.is_infinite() {
                                JsValue::from_f64(value)
                            } else {
                                JsValue::from(value)
                            }
                        } else {
                            JsValue::from_str("Internal error: Failed to cast Float32 column")
                        }
                    },
                    _ => {
                        // For unsupported types, provide clear error message
                        JsValue::from_str(&format!("Unsupported data type: {:?}", field.data_type()))
                    }
                }
            } else {
                // Table has been disposed or is invalid
                JsValue::UNDEFINED
            }
        })
    }

    /// Get value at index (alias for get)
    #[wasm_bindgen(js_name = "getValue")]
    pub fn get_value(&self, index: usize) -> JsValue {
        self.get(index)
    }

    /// Check if value at index is null
    #[wasm_bindgen(js_name = "isNull")]
    pub fn is_null(&self, index: usize) -> bool {
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    let array = batch.column(self.column_index);
                    index < array.len() && array.is_null(index)
                } else {
                    false
                }
            } else {
                false
            }
        })
    }

    /// Check if value at index is valid (not null)
    #[wasm_bindgen(js_name = "isValid")]
    pub fn is_valid(&self, index: usize) -> bool {
        !self.is_null(index)
    }

    /// Create zero-copy slice of column
    #[wasm_bindgen]
    pub fn slice(&self, offset: usize, length: usize) -> Column {
        use arrow_array::Array;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    let column = batch.column(self.column_index);
                    let schema = batch.schema();
                    let field = schema.field(self.column_index);
                    
                    // Validate slice bounds
                    let array_len = column.len();
                    if offset >= array_len {
                        // Return empty column of same type
                        let empty_column = match field.data_type() {
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
                                // For unsupported types, return the same column
                                return Column {
                                    table_handle: self.table_handle,
                                    column_index: self.column_index,
                                };
                            }
                        };
                        
                        // Create new table with just this empty column
                        let new_schema = std::sync::Arc::new(arrow_schema::Schema::new(vec![field.clone()]));
                        match arrow_array::RecordBatch::try_new(new_schema, vec![empty_column]) {
                            Ok(new_batch) => {
                                let handle = crate::core::with_table_registry(|reg| {
                                    reg.insert(new_batch)
                                });
                                Column {
                                    table_handle: handle,
                                    column_index: 0,
                                }
                            },
                            Err(_) => {
                                // If failed to create new batch, return original column
                                Column {
                                    table_handle: self.table_handle,
                                    column_index: self.column_index,
                                }
                            }
                        }
                    } else {
                        // Calculate actual slice length
                        let actual_length = std::cmp::min(length, array_len - offset);
                        
                        // Create sliced array
                        let sliced_array = column.slice(offset, actual_length);
                        
                        // Create new table with just this sliced column
                        let new_schema = std::sync::Arc::new(arrow_schema::Schema::new(vec![field.clone()]));
                        match arrow_array::RecordBatch::try_new(new_schema, vec![sliced_array]) {
                            Ok(new_batch) => {
                                let handle = crate::core::with_table_registry(|reg| {
                                    reg.insert(new_batch)
                                });
                                Column {
                                    table_handle: handle,
                                    column_index: 0,
                                }
                            },
                            Err(_) => {
                                // If failed to create new batch, return original column
                                Column {
                                    table_handle: self.table_handle,
                                    column_index: self.column_index,
                                }
                            }
                        }
                    }
                } else {
                    // Column index out of bounds, return original column
                    Column {
                        table_handle: self.table_handle,
                        column_index: self.column_index,
                    }
                }
            } else {
                // Table not found, return original column
                Column {
                    table_handle: self.table_handle,
                    column_index: self.column_index,
                }
            }
        })
    }

    /// Convert entire column to JavaScript array with full type support
    #[wasm_bindgen(js_name = "toArray")]
    pub fn to_array(&self) -> JsValue {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index >= batch.num_columns() {
                    // Column index is invalid, return empty array
                    return js_sys::Array::new().into();
                }
                
                let column = batch.column(self.column_index);
                let schema = batch.schema();
                let field = schema.field(self.column_index);
                let result_array = js_sys::Array::new();
                
                // Extract each value from the column with full type safety
                for i in 0..column.len() {
                    let js_value = if column.is_null(i) {
                        JsValue::NULL
                    } else {
                        match field.data_type() {
                            ArrowDataType::Int32 => {
                                if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                    JsValue::from(int_array.value(i))
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast Int32 column")
                                }
                            },
                            ArrowDataType::Float64 => {
                                if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                    let value = float_array.value(i);
                                    if value.is_nan() {
                                        JsValue::from_f64(std::f64::NAN)
                                    } else if value.is_infinite() {
                                        JsValue::from_f64(value)
                                    } else {
                                        JsValue::from(value)
                                    }
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast Float64 column")
                                }
                            },
                            ArrowDataType::Utf8 => {
                                if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                    JsValue::from_str(string_array.value(i))
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast String column")
                                }
                            },
                            ArrowDataType::Boolean => {
                                if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                    JsValue::from(bool_array.value(i))
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast Boolean column")
                                }
                            },
                            ArrowDataType::Int8 => {
                                if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int8Array>() {
                                    JsValue::from(int_array.value(i) as i32)
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast Int8 column")
                                }
                            },
                            ArrowDataType::Int16 => {
                                if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int16Array>() {
                                    JsValue::from(int_array.value(i) as i32)
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast Int16 column")
                                }
                            },
                            ArrowDataType::UInt8 => {
                                if let Some(uint_array) = column.as_any().downcast_ref::<arrow_array::UInt8Array>() {
                                    JsValue::from(uint_array.value(i) as u32)
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast UInt8 column")
                                }
                            },
                            ArrowDataType::UInt16 => {
                                if let Some(uint_array) = column.as_any().downcast_ref::<arrow_array::UInt16Array>() {
                                    JsValue::from(uint_array.value(i) as u32)
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast UInt16 column")
                                }
                            },
                            ArrowDataType::UInt32 => {
                                if let Some(uint_array) = column.as_any().downcast_ref::<arrow_array::UInt32Array>() {
                                    JsValue::from(uint_array.value(i))
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast UInt32 column")
                                }
                            },
                            ArrowDataType::Float32 => {
                                if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float32Array>() {
                                    let value = float_array.value(i) as f64;
                                    if value.is_nan() {
                                        JsValue::from_f64(std::f64::NAN)
                                    } else if value.is_infinite() {
                                        JsValue::from_f64(value)
                                    } else {
                                        JsValue::from(value)
                                    }
                                } else {
                                    JsValue::from_str("Internal error: Failed to cast Float32 column")
                                }
                            },
                            _ => {
                                JsValue::from_str(&format!("Unsupported data type: {:?}", field.data_type()))
                            }
                        }
                    };
                    
                    result_array.push(&js_value);
                }
                
                result_array.into()
            } else {
                // Table has been disposed or is invalid
                js_sys::Array::new().into()
            }
        })
    }

    /// Get column statistics
    #[wasm_bindgen]
    pub fn statistics(&self) -> ColumnStatistics {
        use arrow_array::Array;
        use arrow_schema::DataType as ArrowDataType;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(self.table_handle) {
                if self.column_index < batch.num_columns() {
                    let column = batch.column(self.column_index);
                    let schema = batch.schema();
                    let field = schema.field(self.column_index);
                    let null_count = column.null_count();
                    
                    let (min_value, max_value) = match field.data_type() {
                        ArrowDataType::Int32 => {
                            if let Some(int_array) = column.as_any().downcast_ref::<arrow_array::Int32Array>() {
                                let mut min_val: Option<i32> = None;
                                let mut max_val: Option<i32> = None;
                                
                                for i in 0..int_array.len() {
                                    if !int_array.is_null(i) {
                                        let val = int_array.value(i);
                                        min_val = Some(min_val.map_or(val, |m| m.min(val)));
                                        max_val = Some(max_val.map_or(val, |m| m.max(val)));
                                    }
                                }
                                
                                (min_val.map(|v| v.to_string()), max_val.map(|v| v.to_string()))
                            } else {
                                (None, None)
                            }
                        },
                        ArrowDataType::Float64 => {
                            if let Some(float_array) = column.as_any().downcast_ref::<arrow_array::Float64Array>() {
                                let mut min_val: Option<f64> = None;
                                let mut max_val: Option<f64> = None;
                                
                                for i in 0..float_array.len() {
                                    if !float_array.is_null(i) {
                                        let val = float_array.value(i);
                                        if !val.is_nan() {
                                            min_val = Some(min_val.map_or(val, |m| m.min(val)));
                                            max_val = Some(max_val.map_or(val, |m| m.max(val)));
                                        }
                                    }
                                }
                                
                                (min_val.map(|v| v.to_string()), max_val.map(|v| v.to_string()))
                            } else {
                                (None, None)
                            }
                        },
                        ArrowDataType::Utf8 => {
                            if let Some(string_array) = column.as_any().downcast_ref::<arrow_array::StringArray>() {
                                let mut min_val: Option<&str> = None;
                                let mut max_val: Option<&str> = None;
                                
                                for i in 0..string_array.len() {
                                    if !string_array.is_null(i) {
                                        let val = string_array.value(i);
                                        min_val = Some(min_val.map_or(val, |m| if val < m { val } else { m }));
                                        max_val = Some(max_val.map_or(val, |m| if val > m { val } else { m }));
                                    }
                                }
                                
                                (min_val.map(|v| v.to_string()), max_val.map(|v| v.to_string()))
                            } else {
                                (None, None)
                            }
                        },
                        ArrowDataType::Boolean => {
                            if let Some(bool_array) = column.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                                let mut has_false = false;
                                let mut has_true = false;
                                
                                for i in 0..bool_array.len() {
                                    if !bool_array.is_null(i) {
                                        let val = bool_array.value(i);
                                        if val {
                                            has_true = true;
                                        } else {
                                            has_false = true;
                                        }
                                    }
                                }
                                
                                let min = if has_false { Some("false".to_string()) } else if has_true { Some("true".to_string()) } else { None };
                                let max = if has_true { Some("true".to_string()) } else if has_false { Some("false".to_string()) } else { None };
                                
                                (min, max)
                            } else {
                                (None, None)
                            }
                        },
                        _ => (None, None)
                    };
                    
                    ColumnStatistics {
                        null_count,
                        min_value,
                        max_value,
                        distinct_count: None, // TODO: Calculate distinct count would be expensive
                    }
                } else {
                    ColumnStatistics {
                        null_count: 0,
                        min_value: None,
                        max_value: None,
                        distinct_count: None,
                    }
                }
            } else {
                ColumnStatistics {
                    null_count: 0,
                    min_value: None,
                    max_value: None,
                    distinct_count: None,
                }
            }
        })
    }
}

/// Array builder interface
/// Internal enum to hold different Arrow builders
enum ArrowBuilderType {
    Null(NullBuilder),
    Boolean(BooleanBuilder),
    Int32(Int32Builder),
    Int64(Int64Builder),
    Float32(Float32Builder),
    Float64(Float64Builder),
    Utf8(StringBuilder),
}

#[wasm_bindgen]
pub struct ArrayBuilder {
    data_type: DataType,
    builder: ArrowBuilderType,
}

#[wasm_bindgen]
impl ArrayBuilder {
    /// Create a new ArrayBuilder for the specified data type
    #[wasm_bindgen(constructor)]
    pub fn new(data_type: DataType, capacity: Option<usize>) -> Result<ArrayBuilder, JsValue> {
        let cap = capacity.unwrap_or(256);
        
        let builder = match data_type.type_id() {
            0 => ArrowBuilderType::Null(NullBuilder::new()),
            1 => ArrowBuilderType::Boolean(BooleanBuilder::with_capacity(cap)),
            2 => ArrowBuilderType::Int32(Int32Builder::with_capacity(cap)),
            3 => ArrowBuilderType::Int64(Int64Builder::with_capacity(cap)),
            4 => ArrowBuilderType::Float32(Float32Builder::with_capacity(cap)),
            5 => ArrowBuilderType::Float64(Float64Builder::with_capacity(cap)),
            6 => ArrowBuilderType::Utf8(StringBuilder::with_capacity(cap, cap * 10)), // Estimate string capacity
            _ => return Err(JsValue::from_str(&format!("Unsupported data type: {}", data_type.type_id())))
        };
        
        Ok(ArrayBuilder {
            data_type,
            builder,
        })
    }

    /// Append a value
    #[wasm_bindgen]
    pub fn append(&mut self, value: JsValue) -> Result<(), JsValue> {
        match &mut self.builder {
            ArrowBuilderType::Null(builder) => {
                builder.append_null();
                Ok(())
            },
            ArrowBuilderType::Boolean(builder) => {
                if value.is_null() || value.is_undefined() {
                    builder.append_null();
                } else {
                    let bool_val = value.as_bool().unwrap_or(false);
                    builder.append_value(bool_val);
                }
                Ok(())
            },
            ArrowBuilderType::Int32(builder) => {
                if value.is_null() || value.is_undefined() {
                    builder.append_null();
                } else {
                    let int_val = value.as_f64().ok_or_else(|| JsValue::from_str("Value cannot be converted to number"))? as i32;
                    builder.append_value(int_val);
                }
                Ok(())
            },
            ArrowBuilderType::Int64(builder) => {
                if value.is_null() || value.is_undefined() {
                    builder.append_null();
                } else {
                    let int_val = value.as_f64().ok_or_else(|| JsValue::from_str("Value cannot be converted to number"))? as i64;
                    builder.append_value(int_val);
                }
                Ok(())
            },
            ArrowBuilderType::Float32(builder) => {
                if value.is_null() || value.is_undefined() {
                    builder.append_null();
                } else {
                    let float_val = value.as_f64().ok_or_else(|| JsValue::from_str("Value cannot be converted to number"))? as f32;
                    builder.append_value(float_val);
                }
                Ok(())
            },
            ArrowBuilderType::Float64(builder) => {
                if value.is_null() || value.is_undefined() {
                    builder.append_null();
                } else {
                    let float_val = value.as_f64().ok_or_else(|| JsValue::from_str("Value cannot be converted to number"))?;
                    builder.append_value(float_val);
                }
                Ok(())
            },
            ArrowBuilderType::Utf8(builder) => {
                if value.is_null() || value.is_undefined() {
                    builder.append_null();
                } else {
                    let str_val = value.as_string().ok_or_else(|| JsValue::from_str("Value cannot be converted to string"))?;
                    builder.append_value(&str_val);
                }
                Ok(())
            },
        }
    }

    /// Append a null value
    #[wasm_bindgen(js_name = "appendNull")]
    pub fn append_null(&mut self) {
        match &mut self.builder {
            ArrowBuilderType::Null(builder) => builder.append_null(),
            ArrowBuilderType::Boolean(builder) => builder.append_null(),
            ArrowBuilderType::Int32(builder) => builder.append_null(),
            ArrowBuilderType::Int64(builder) => builder.append_null(),
            ArrowBuilderType::Float32(builder) => builder.append_null(),
            ArrowBuilderType::Float64(builder) => builder.append_null(),
            ArrowBuilderType::Utf8(builder) => builder.append_null(),
        }
    }

    /// Append multiple values
    #[wasm_bindgen(js_name = "appendValues")]
    pub fn append_values(&mut self, values: JsValue) -> Result<(), JsValue> {
        // Convert JsValue to Array
        let array = js_sys::Array::from(&values);
        
        // Iterate over each value and append it
        for i in 0..array.length() {
            let value = array.get(i);
            self.append(value)?;
        }
        
        Ok(())
    }

    /// Finish building and create column
    #[wasm_bindgen]
    pub fn finish(&mut self) -> Result<Column, JsValue> {
        use arrow_schema::{Schema as ArrowSchema, Field as ArrowField, DataType as ArrowDataType};
        use arrow_array::RecordBatch;
        use std::sync::Arc;
        
        // Build the array based on the builder type
        let array: ArrayRef = match &mut self.builder {
            ArrowBuilderType::Null(builder) => {
                // Create new builder since NullBuilder doesn't have clone
                let array = NullBuilder::new().finish();
                Arc::new(array)
            },
            ArrowBuilderType::Boolean(builder) => {
                let array = std::mem::replace(builder, BooleanBuilder::new()).finish();
                Arc::new(array)
            },
            ArrowBuilderType::Int32(builder) => {
                let array = std::mem::replace(builder, Int32Builder::new()).finish();
                Arc::new(array)
            },
            ArrowBuilderType::Int64(builder) => {
                let array = std::mem::replace(builder, Int64Builder::new()).finish();
                Arc::new(array)
            },
            ArrowBuilderType::Float32(builder) => {
                let array = std::mem::replace(builder, Float32Builder::new()).finish();
                Arc::new(array)
            },
            ArrowBuilderType::Float64(builder) => {
                let array = std::mem::replace(builder, Float64Builder::new()).finish();
                Arc::new(array)
            },
            ArrowBuilderType::Utf8(builder) => {
                let array = std::mem::replace(builder, StringBuilder::new()).finish();
                Arc::new(array)
            },
        };
        
        // Create Arrow data type for the schema
        let arrow_data_type = match self.data_type.type_id() {
            0 => ArrowDataType::Null,
            1 => ArrowDataType::Boolean,
            2 => ArrowDataType::Int32,
            3 => ArrowDataType::Int64,
            4 => ArrowDataType::Float32,
            5 => ArrowDataType::Float64,
            6 => ArrowDataType::Utf8,
            _ => return Err(JsValue::from_str("Unsupported data type for schema creation"))
        };
        
        // Create a schema with a single field
        let field = ArrowField::new("built_column", arrow_data_type, true);
        let schema = ArrowSchema::new(vec![field]);
        
        // Create a record batch with the single array
        let batch = RecordBatch::try_new(Arc::new(schema), vec![array])
            .map_err(|e| JsValue::from_str(&format!("Failed to create record batch: {}", e)))?;
        
        // Register the batch and get a handle
        let handle = crate::core::with_table_registry(|registry| {
            registry.insert(batch)
        });
        
        // Return a Column pointing to the first column of the batch
        Ok(Column::from_table_column(handle, 0))
    }

    /// Clear the builder
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        // Replace the current builder with a new empty one
        self.builder = match self.data_type.type_id() {
            0 => ArrowBuilderType::Null(NullBuilder::new()),
            1 => ArrowBuilderType::Boolean(BooleanBuilder::new()),
            2 => ArrowBuilderType::Int32(Int32Builder::new()),
            3 => ArrowBuilderType::Int64(Int64Builder::new()),
            4 => ArrowBuilderType::Float32(Float32Builder::new()),
            5 => ArrowBuilderType::Float64(Float64Builder::new()),
            6 => ArrowBuilderType::Utf8(StringBuilder::new()),
            _ => return, // Should not happen for valid data types
        };
    }
}

/// Create array builder for given data type
#[wasm_bindgen(js_name = "createBuilder")]
pub fn create_builder(data_type: DataType, capacity: Option<usize>) -> Result<ArrayBuilder, JsValue> {
    ArrayBuilder::new(data_type, capacity)
}