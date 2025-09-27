//! Computational operations on arrays for the WASM Arrow library.

use wasm_bindgen::prelude::*;
use crate::{Column, DataType};

/// Sum aggregation function
#[wasm_bindgen]
pub fn sum(column: &Column) -> Result<f64, JsValue> {
    use arrow_array::Array;
    use arrow_schema::DataType as ArrowDataType;
    
    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                let schema = batch.schema();
                let field = schema.field(column.column_index);
                
                match field.data_type() {
                    ArrowDataType::Int32 => {
                        if let Some(int_array) = array.as_any().downcast_ref::<arrow_array::Int32Array>() {
                            let mut sum = 0i64;
                            for i in 0..int_array.len() {
                                if !int_array.is_null(i) {
                                    sum += int_array.value(i) as i64;
                                }
                            }
                            Ok(sum as f64)
                        } else {
                            Err(JsValue::from_str("Failed to cast to Int32Array"))
                        }
                    },
                    ArrowDataType::Float64 => {
                        if let Some(float_array) = array.as_any().downcast_ref::<arrow_array::Float64Array>() {
                            let mut sum = 0.0;
                            for i in 0..float_array.len() {
                                if !float_array.is_null(i) {
                                    let val = float_array.value(i);
                                    if !val.is_nan() {
                                        sum += val;
                                    }
                                }
                            }
                            Ok(sum)
                        } else {
                            Err(JsValue::from_str("Failed to cast to Float64Array"))
                        }
                    },
                    _ => Err(JsValue::from_str("Sum operation not supported for this data type"))
                }
            } else {
                Err(JsValue::from_str("Column index out of bounds"))
            }
        } else {
            Err(JsValue::from_str("Table not found"))
        }
    })
}

/// Mean aggregation function
#[wasm_bindgen]
pub fn mean(column: &Column) -> Result<f64, JsValue> {
    use arrow_array::Array;
    use arrow_schema::DataType as ArrowDataType;
    
    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                let schema = batch.schema();
                let field = schema.field(column.column_index);
                
                match field.data_type() {
                    ArrowDataType::Int32 => {
                        if let Some(int_array) = array.as_any().downcast_ref::<arrow_array::Int32Array>() {
                            let mut sum = 0i64;
                            let mut count = 0usize;
                            for i in 0..int_array.len() {
                                if !int_array.is_null(i) {
                                    sum += int_array.value(i) as i64;
                                    count += 1;
                                }
                            }
                            if count > 0 {
                                Ok(sum as f64 / count as f64)
                            } else {
                                Err(JsValue::from_str("Cannot compute mean of empty column"))
                            }
                        } else {
                            Err(JsValue::from_str("Failed to cast to Int32Array"))
                        }
                    },
                    ArrowDataType::Float64 => {
                        if let Some(float_array) = array.as_any().downcast_ref::<arrow_array::Float64Array>() {
                            let mut sum = 0.0;
                            let mut count = 0usize;
                            for i in 0..float_array.len() {
                                if !float_array.is_null(i) {
                                    let val = float_array.value(i);
                                    if !val.is_nan() {
                                        sum += val;
                                        count += 1;
                                    }
                                }
                            }
                            if count > 0 {
                                Ok(sum / count as f64)
                            } else {
                                Err(JsValue::from_str("Cannot compute mean of empty column"))
                            }
                        } else {
                            Err(JsValue::from_str("Failed to cast to Float64Array"))
                        }
                    },
                    _ => Err(JsValue::from_str("Mean operation not supported for this data type"))
                }
            } else {
                Err(JsValue::from_str("Column index out of bounds"))
            }
        } else {
            Err(JsValue::from_str("Table not found"))
        }
    })
}

/// Minimum value function
#[wasm_bindgen]
pub fn min(column: &Column) -> JsValue {
    use arrow_array::Array;
    use arrow_schema::DataType as ArrowDataType;
    
    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                let schema = batch.schema();
                let field = schema.field(column.column_index);
                
                match field.data_type() {
                    ArrowDataType::Int32 => {
                        if let Some(int_array) = array.as_any().downcast_ref::<arrow_array::Int32Array>() {
                            let mut min_val: Option<i32> = None;
                            for i in 0..int_array.len() {
                                if !int_array.is_null(i) {
                                    let val = int_array.value(i);
                                    min_val = Some(min_val.map_or(val, |m| m.min(val)));
                                }
                            }
                            min_val.map_or(JsValue::NULL, |v| JsValue::from(v))
                        } else {
                            JsValue::NULL
                        }
                    },
                    ArrowDataType::Float64 => {
                        if let Some(float_array) = array.as_any().downcast_ref::<arrow_array::Float64Array>() {
                            let mut min_val: Option<f64> = None;
                            for i in 0..float_array.len() {
                                if !float_array.is_null(i) {
                                    let val = float_array.value(i);
                                    if !val.is_nan() {
                                        min_val = Some(min_val.map_or(val, |m| m.min(val)));
                                    }
                                }
                            }
                            min_val.map_or(JsValue::NULL, |v| JsValue::from(v))
                        } else {
                            JsValue::NULL
                        }
                    },
                    ArrowDataType::Utf8 => {
                        if let Some(string_array) = array.as_any().downcast_ref::<arrow_array::StringArray>() {
                            let mut min_val: Option<&str> = None;
                            for i in 0..string_array.len() {
                                if !string_array.is_null(i) {
                                    let val = string_array.value(i);
                                    min_val = Some(min_val.map_or(val, |m| if val < m { val } else { m }));
                                }
                            }
                            min_val.map_or(JsValue::NULL, |v| JsValue::from_str(v))
                        } else {
                            JsValue::NULL
                        }
                    },
                    _ => JsValue::NULL
                }
            } else {
                JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    })
}

/// Maximum value function
#[wasm_bindgen]
pub fn max(column: &Column) -> JsValue {
    use arrow_array::Array;
    use arrow_schema::DataType as ArrowDataType;
    
    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                let schema = batch.schema();
                let field = schema.field(column.column_index);
                
                match field.data_type() {
                    ArrowDataType::Int32 => {
                        if let Some(int_array) = array.as_any().downcast_ref::<arrow_array::Int32Array>() {
                            let mut max_val: Option<i32> = None;
                            for i in 0..int_array.len() {
                                if !int_array.is_null(i) {
                                    let val = int_array.value(i);
                                    max_val = Some(max_val.map_or(val, |m| m.max(val)));
                                }
                            }
                            max_val.map_or(JsValue::NULL, |v| JsValue::from(v))
                        } else {
                            JsValue::NULL
                        }
                    },
                    ArrowDataType::Float64 => {
                        if let Some(float_array) = array.as_any().downcast_ref::<arrow_array::Float64Array>() {
                            let mut max_val: Option<f64> = None;
                            for i in 0..float_array.len() {
                                if !float_array.is_null(i) {
                                    let val = float_array.value(i);
                                    if !val.is_nan() {
                                        max_val = Some(max_val.map_or(val, |m| m.max(val)));
                                    }
                                }
                            }
                            max_val.map_or(JsValue::NULL, |v| JsValue::from(v))
                        } else {
                            JsValue::NULL
                        }
                    },
                    ArrowDataType::Utf8 => {
                        if let Some(string_array) = array.as_any().downcast_ref::<arrow_array::StringArray>() {
                            let mut max_val: Option<&str> = None;
                            for i in 0..string_array.len() {
                                if !string_array.is_null(i) {
                                    let val = string_array.value(i);
                                    max_val = Some(max_val.map_or(val, |m| if val > m { val } else { m }));
                                }
                            }
                            max_val.map_or(JsValue::NULL, |v| JsValue::from_str(v))
                        } else {
                            JsValue::NULL
                        }
                    },
                    _ => JsValue::NULL
                }
            } else {
                JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    })
}

/// Count non-null values
#[wasm_bindgen]
pub fn count(column: &Column) -> usize {
    column.length() - column.null_count()
}

/// Count distinct values
#[wasm_bindgen(js_name = "countDistinct")]
pub fn count_distinct(column: &Column) -> usize {
    use std::collections::HashSet;
    use arrow_array::Array;
    use arrow_schema::DataType as ArrowDataType;
    
    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                let schema = batch.schema();
                let field = schema.field(column.column_index);
                
                let mut distinct_values = HashSet::new();
                
                match field.data_type() {
                    ArrowDataType::Int32 => {
                        if let Some(int_array) = array.as_any().downcast_ref::<arrow_array::Int32Array>() {
                            for i in 0..int_array.len() {
                                if !int_array.is_null(i) {
                                    distinct_values.insert(int_array.value(i).to_string());
                                }
                            }
                        }
                    },
                    ArrowDataType::Float64 => {
                        if let Some(float_array) = array.as_any().downcast_ref::<arrow_array::Float64Array>() {
                            for i in 0..float_array.len() {
                                if !float_array.is_null(i) {
                                    distinct_values.insert(float_array.value(i).to_string());
                                }
                            }
                        }
                    },
                    ArrowDataType::Utf8 => {
                        if let Some(string_array) = array.as_any().downcast_ref::<arrow_array::StringArray>() {
                            for i in 0..string_array.len() {
                                if !string_array.is_null(i) {
                                    distinct_values.insert(string_array.value(i).to_string());
                                }
                            }
                        }
                    },
                    ArrowDataType::Boolean => {
                        if let Some(bool_array) = array.as_any().downcast_ref::<arrow_array::BooleanArray>() {
                            for i in 0..bool_array.len() {
                                if !bool_array.is_null(i) {
                                    distinct_values.insert(bool_array.value(i).to_string());
                                }
                            }
                        }
                    },
                    _ => {
                        // For unsupported types, return regular count
                        return count(column);
                    }
                }
                
                distinct_values.len()
            } else {
                0
            }
        } else {
            0
        }
    })
}

/// Cast column to target type
#[wasm_bindgen]
pub fn cast(column: &Column, target_type: DataType) -> Result<Column, JsValue> {
    use arrow_cast::cast::cast;
    use arrow_array::Array;
    
    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                
                // Convert our DataType to Arrow's DataType
                let arrow_target_type = match target_type.type_id() {
                    1 => arrow_schema::DataType::Boolean,
                    2 => arrow_schema::DataType::Int32,
                    3 => arrow_schema::DataType::Int64,
                    4 => arrow_schema::DataType::Float32,
                    5 => arrow_schema::DataType::Float64,
                    6 => arrow_schema::DataType::Utf8,
                    _ => return Err(JsValue::from_str("Unsupported target type for casting"))
                };
                
                // Perform the cast operation
                let casted_array = cast(array.as_ref(), &arrow_target_type)
                    .map_err(|e| JsValue::from_str(&format!("Cast operation failed: {}", e)))?;
                
                // Create new single-column record batch for the casted array
                let schema = arrow_schema::Schema::new(vec![
                    arrow_schema::Field::new("casted_column", arrow_target_type, true)
                ]);
                
                let batch = arrow_array::RecordBatch::try_new(
                    std::sync::Arc::new(schema),
                    vec![casted_array]
                ).map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                
                // Register the new batch and return column handle
                let new_handle = crate::core::with_table_registry(|reg| reg.insert(batch));
                Ok(crate::column::Column::from_table_column(new_handle, 0))
            } else {
                Err(JsValue::from_str("Column index out of bounds"))
            }
        } else {
            Err(JsValue::from_str("Table not found"))
        }
    })
}

/// Take values at specified indices
#[wasm_bindgen]
pub fn take(column: &Column, indices: JsValue) -> Result<Column, JsValue> {
    use arrow_select::take::take;
    use arrow_array::{Array, UInt32Array};
    
    let index_array: Vec<u32> = serde_wasm_bindgen::from_value(indices)
        .map_err(|e| JsValue::from_str(&format!("Invalid indices: {}", e)))?;

    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                
                // Create Arrow array of indices
                let indices_array = UInt32Array::from(index_array);
                
                // Perform the take operation
                let result_array = take(array.as_ref(), &indices_array, None)
                    .map_err(|e| JsValue::from_str(&format!("Take operation failed: {}", e)))?;
                
                // Create new single-column record batch
                let batch_schema = batch.schema();
                let field = batch_schema.field(column.column_index);
                let schema = arrow_schema::Schema::new(vec![field.clone()]);
                
                let batch = arrow_array::RecordBatch::try_new(
                    std::sync::Arc::new(schema),
                    vec![result_array]
                ).map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                
                // Register the new batch and return column handle
                let new_handle = crate::core::with_table_registry(|reg| reg.insert(batch));
                Ok(crate::column::Column::from_table_column(new_handle, 0))
            } else {
                Err(JsValue::from_str("Column index out of bounds"))
            }
        } else {
            Err(JsValue::from_str("Table not found"))
        }
    })
}

/// Filter column with boolean mask
#[wasm_bindgen]
pub fn filter(column: &Column, mask: JsValue) -> Result<Column, JsValue> {
    use arrow_select::filter::filter;
    use arrow_array::{Array, BooleanArray};
    
    let mask_array: Vec<bool> = serde_wasm_bindgen::from_value(mask)
        .map_err(|e| JsValue::from_str(&format!("Invalid mask: {}", e)))?;

    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                
                // Create Arrow boolean array from the mask
                let boolean_mask = BooleanArray::from(mask_array);
                
                // Perform the filter operation
                let filtered_array = filter(array.as_ref(), &boolean_mask)
                    .map_err(|e| JsValue::from_str(&format!("Filter operation failed: {}", e)))?;
                
                // Create new single-column record batch
                let batch_schema = batch.schema();
                let field = batch_schema.field(column.column_index);
                let schema = arrow_schema::Schema::new(vec![field.clone()]);
                
                let batch = arrow_array::RecordBatch::try_new(
                    std::sync::Arc::new(schema),
                    vec![filtered_array]
                ).map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                
                // Register the new batch and return column handle
                let new_handle = crate::core::with_table_registry(|reg| reg.insert(batch));
                Ok(crate::column::Column::from_table_column(new_handle, 0))
            } else {
                Err(JsValue::from_str("Column index out of bounds"))
            }
        } else {
            Err(JsValue::from_str("Table not found"))
        }
    })
}

/// Sort column
#[wasm_bindgen]
pub fn sort(column: &Column, descending: Option<bool>) -> Result<Column, JsValue> {
    use arrow_select::take::take;
    use arrow_array::{Array, UInt32Array};
    use arrow_ord::sort::{sort_to_indices, SortOptions};
    
    let desc = descending.unwrap_or(false);

    crate::core::with_table_registry(|registry| {
        if let Some(batch) = registry.get(column.table_handle) {
            if column.column_index < batch.num_columns() {
                let array = batch.column(column.column_index);
                
                // Create sort options
                let sort_options = SortOptions {
                    descending: desc,
                    nulls_first: false,
                };
                
                // Get sorted indices
                let indices = sort_to_indices(array.as_ref(), Some(sort_options), None)
                    .map_err(|e| JsValue::from_str(&format!("Sort operation failed: {}", e)))?;
                
                // Take values in sorted order
                let sorted_array = take(array.as_ref(), &indices, None)
                    .map_err(|e| JsValue::from_str(&format!("Take operation failed: {}", e)))?;
                
                // Create new single-column record batch
                let batch_schema = batch.schema();
                let field = batch_schema.field(column.column_index);
                let schema = arrow_schema::Schema::new(vec![field.clone()]);
                
                let batch = arrow_array::RecordBatch::try_new(
                    std::sync::Arc::new(schema),
                    vec![sorted_array]
                ).map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                
                // Register the new batch and return column handle
                let new_handle = crate::core::with_table_registry(|reg| reg.insert(batch));
                Ok(crate::column::Column::from_table_column(new_handle, 0))
            } else {
                Err(JsValue::from_str("Column index out of bounds"))
            }
        } else {
            Err(JsValue::from_str("Table not found"))
        }
    })
}

/// Apply unary operation to column
#[wasm_bindgen(js_name = "unaryOp")]
pub fn unary_op(column: &Column, operation: &js_sys::Function) -> Result<Column, JsValue> {
    // TODO: Implement unary operations with JavaScript function
    // For now, return the same column
    Ok(Column::from_table_column(column.table_handle, column.column_index))
}

/// Apply binary operation between two columns
#[wasm_bindgen(js_name = "binaryOp")]
pub fn binary_op(
    left: &Column,
    right: &Column,
    operation: &js_sys::Function,
) -> Result<Column, JsValue> {
    // TODO: Implement binary operations with JavaScript function
    // For now, return the left column
    Ok(Column::from_table_column(left.table_handle, left.column_index))
}

/// Compute module initialization
pub fn init_compute() {
    // TODO: Initialize any compute-related state
}

/// Statistics computation
pub mod stats {
    use super::*;

    /// Compute basic statistics for a column
    #[wasm_bindgen(js_name = "computeStats")]
    pub fn compute_stats(column: &Column) -> JsValue {
        let stats = serde_json::json!({
            "count": count(column),
            "null_count": column.null_count(),
            "min": "N/A", // TODO: Implement min calculation
            "max": "N/A", // TODO: Implement max calculation  
            "mean": mean(column).unwrap_or(0.0),
            "sum": sum(column).unwrap_or(0.0)
        });

        serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
    }

    /// Compute quantiles
    #[wasm_bindgen]
    pub fn quantiles(column: &Column, quantile_values: JsValue) -> Result<JsValue, JsValue> {
        let _quantiles: Vec<f64> = serde_wasm_bindgen::from_value(quantile_values)
            .map_err(|e| JsValue::from_str(&format!("Invalid quantiles: {}", e)))?;

        // TODO: Implement quantile calculation
        Ok(js_sys::Array::new().into())
    }

    /// Compute histogram
    #[wasm_bindgen]
    pub fn histogram(column: &Column, bins: usize) -> Result<JsValue, JsValue> {
        if bins == 0 {
            return Err(JsValue::from_str("Number of bins must be greater than 0"));
        }

        // TODO: Implement histogram calculation
        let histogram = serde_json::json!({
            "bins": bins,
            "counts": vec![0; bins],
            "bin_edges": vec![0.0; bins + 1]
        });

        serde_wasm_bindgen::to_value(&histogram)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize histogram: {}", e)))
    }
}

/// String operations
pub mod string_ops {
    use super::*;

    /// Convert string column to lowercase
    #[wasm_bindgen]
    pub fn lowercase(column: &Column) -> Result<Column, JsValue> {
        use arrow_array::{Array, StringArray};
        use arrow_schema::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
        use arrow_array::RecordBatch;
        use std::sync::Arc;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(column.table_handle) {
                if column.column_index < batch.num_columns() {
                    let array = batch.column(column.column_index);
                    let schema = batch.schema();
                    let field = schema.field(column.column_index);
                    
                    match field.data_type() {
                        ArrowDataType::Utf8 => {
                            if let Some(string_array) = array.as_any().downcast_ref::<StringArray>() {
                                // Apply lowercase to all strings
                                let lowercase_values: Vec<Option<String>> = (0..string_array.len())
                                    .map(|i| {
                                        if string_array.is_null(i) {
                                            None
                                        } else {
                                            Some(string_array.value(i).to_lowercase())
                                        }
                                    })
                                    .collect();
                                
                                // Create new array
                                let new_array = StringArray::from(lowercase_values);
                                
                                // Create new schema with single field
                                let new_field = ArrowField::new("lowercase", ArrowDataType::Utf8, field.is_nullable());
                                let new_schema = ArrowSchema::new(vec![new_field]);
                                
                                // Create new batch
                                let new_batch = RecordBatch::try_new(Arc::new(new_schema), vec![Arc::new(new_array)])
                                    .map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                                
                                // Register and return
                                let handle = crate::core::with_table_registry(|reg| {
                                    reg.insert(new_batch)
                                });
                                
                                Ok(Column::from_table_column(handle, 0))
                            } else {
                                Err(JsValue::from_str("Failed to cast to string array"))
                            }
                        },
                        _ => Err(JsValue::from_str("Lowercase operation only supported for string columns"))
                    }
                } else {
                    Err(JsValue::from_str("Column index out of bounds"))
                }
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }

    /// Convert string column to uppercase
    #[wasm_bindgen]
    pub fn uppercase(column: &Column) -> Result<Column, JsValue> {
        use arrow_array::{Array, StringArray};
        use arrow_schema::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
        use arrow_array::RecordBatch;
        use std::sync::Arc;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(column.table_handle) {
                if column.column_index < batch.num_columns() {
                    let array = batch.column(column.column_index);
                    let schema = batch.schema();
                    let field = schema.field(column.column_index);
                    
                    match field.data_type() {
                        ArrowDataType::Utf8 => {
                            if let Some(string_array) = array.as_any().downcast_ref::<StringArray>() {
                                // Apply uppercase to all strings
                                let uppercase_values: Vec<Option<String>> = (0..string_array.len())
                                    .map(|i| {
                                        if string_array.is_null(i) {
                                            None
                                        } else {
                                            Some(string_array.value(i).to_uppercase())
                                        }
                                    })
                                    .collect();
                                
                                // Create new array
                                let new_array = StringArray::from(uppercase_values);
                                
                                // Create new schema with single field
                                let new_field = ArrowField::new("uppercase", ArrowDataType::Utf8, field.is_nullable());
                                let new_schema = ArrowSchema::new(vec![new_field]);
                                
                                // Create new batch
                                let new_batch = RecordBatch::try_new(Arc::new(new_schema), vec![Arc::new(new_array)])
                                    .map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                                
                                // Register and return
                                let handle = crate::core::with_table_registry(|reg| {
                                    reg.insert(new_batch)
                                });
                                
                                Ok(Column::from_table_column(handle, 0))
                            } else {
                                Err(JsValue::from_str("Failed to cast to string array"))
                            }
                        },
                        _ => Err(JsValue::from_str("Uppercase operation only supported for string columns"))
                    }
                } else {
                    Err(JsValue::from_str("Column index out of bounds"))
                }
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }

    /// Get string length
    #[wasm_bindgen(js_name = "stringLength")]
    pub fn string_length(column: &Column) -> Result<Column, JsValue> {
        use arrow_array::{Array, StringArray, Int32Array};
        use arrow_schema::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
        use arrow_array::RecordBatch;
        use std::sync::Arc;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(column.table_handle) {
                if column.column_index < batch.num_columns() {
                    let array = batch.column(column.column_index);
                    let schema = batch.schema();
                    let field = schema.field(column.column_index);
                    
                    match field.data_type() {
                        ArrowDataType::Utf8 => {
                            if let Some(string_array) = array.as_any().downcast_ref::<StringArray>() {
                                // Calculate lengths
                                let length_values: Vec<Option<i32>> = (0..string_array.len())
                                    .map(|i| {
                                        if string_array.is_null(i) {
                                            None
                                        } else {
                                            Some(string_array.value(i).len() as i32)
                                        }
                                    })
                                    .collect();
                                
                                // Create new array
                                let new_array = Int32Array::from(length_values);
                                
                                // Create new schema with single field
                                let new_field = ArrowField::new("string_length", ArrowDataType::Int32, field.is_nullable());
                                let new_schema = ArrowSchema::new(vec![new_field]);
                                
                                // Create new batch
                                let new_batch = RecordBatch::try_new(Arc::new(new_schema), vec![Arc::new(new_array)])
                                    .map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                                
                                // Register and return
                                let handle = crate::core::with_table_registry(|reg| {
                                    reg.insert(new_batch)
                                });
                                
                                Ok(Column::from_table_column(handle, 0))
                            } else {
                                Err(JsValue::from_str("Failed to cast to string array"))
                            }
                        },
                        _ => Err(JsValue::from_str("String length operation only supported for string columns"))
                    }
                } else {
                    Err(JsValue::from_str("Column index out of bounds"))
                }
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }

    /// Substring operation
    #[wasm_bindgen]
    pub fn substring(column: &Column, start: i32, length: Option<i32>) -> Result<Column, JsValue> {
        use arrow_array::{Array, StringArray};
        use arrow_schema::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
        use arrow_array::RecordBatch;
        use std::sync::Arc;
        
        crate::core::with_table_registry(|registry| {
            if let Some(batch) = registry.get(column.table_handle) {
                if column.column_index < batch.num_columns() {
                    let array = batch.column(column.column_index);
                    let schema = batch.schema();
                    let field = schema.field(column.column_index);
                    
                    match field.data_type() {
                        ArrowDataType::Utf8 => {
                            if let Some(string_array) = array.as_any().downcast_ref::<StringArray>() {
                                // Apply substring to all strings
                                let substring_values: Vec<Option<String>> = (0..string_array.len())
                                    .map(|i| {
                                        if string_array.is_null(i) {
                                            None
                                        } else {
                                            let original = string_array.value(i);
                                            let start_pos = if start < 0 { 0 } else { start as usize };
                                            
                                            if start_pos >= original.len() {
                                                Some(String::new())
                                            } else {
                                                let substring = if let Some(len) = length {
                                                    if len <= 0 {
                                                        String::new()
                                                    } else {
                                                        let end_pos = std::cmp::min(start_pos + len as usize, original.len());
                                                        original[start_pos..end_pos].to_string()
                                                    }
                                                } else {
                                                    original[start_pos..].to_string()
                                                };
                                                Some(substring)
                                            }
                                        }
                                    })
                                    .collect();
                                
                                // Create new array
                                let new_array = StringArray::from(substring_values);
                                
                                // Create new schema with single field
                                let new_field = ArrowField::new("substring", ArrowDataType::Utf8, field.is_nullable());
                                let new_schema = ArrowSchema::new(vec![new_field]);
                                
                                // Create new batch
                                let new_batch = RecordBatch::try_new(Arc::new(new_schema), vec![Arc::new(new_array)])
                                    .map_err(|e| JsValue::from_str(&format!("Failed to create batch: {}", e)))?;
                                
                                // Register and return
                                let handle = crate::core::with_table_registry(|reg| {
                                    reg.insert(new_batch)
                                });
                                
                                Ok(Column::from_table_column(handle, 0))
                            } else {
                                Err(JsValue::from_str("Failed to cast to string array"))
                            }
                        },
                        _ => Err(JsValue::from_str("Substring operation only supported for string columns"))
                    }
                } else {
                    Err(JsValue::from_str("Column index out of bounds"))
                }
            } else {
                Err(JsValue::from_str("Table not found"))
            }
        })
    }
}