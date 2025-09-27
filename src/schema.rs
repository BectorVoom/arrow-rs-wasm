//! Schema definition and management for the WASM Arrow library.
//!
//! This module provides schema and field definitions following the API specification.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use arrow_schema::{Schema as ArrowSchema, Field as ArrowField};
use crate::{DataType, error::ArrowError, core::HandleId};
use std::collections::HashMap;
use std::sync::Arc;

/// Field interface for column definitions
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    name: String,
    data_type: DataType,
    nullable: bool,
    metadata: HashMap<String, String>,
}

#[wasm_bindgen]
impl Field {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, data_type: DataType, nullable: bool) -> Field {
        Field {
            name: name.to_string(),
            data_type,
            nullable,
            metadata: HashMap::new(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter, js_name = "dataType")]
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nullable(&self) -> bool {
        self.nullable
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.metadata).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = "withMetadata")]
    pub fn with_metadata(&self, metadata: JsValue) -> Result<Field, JsValue> {
        let metadata_map: HashMap<String, String> = serde_wasm_bindgen::from_value(metadata)
            .map_err(|e| JsValue::from_str(&format!("Invalid metadata: {}", e)))?;
        
        let mut field = self.clone();
        field.metadata = metadata_map;
        Ok(field)
    }
}

impl From<&ArrowField> for Field {
    fn from(field: &ArrowField) -> Self {
        let metadata = field.metadata().iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Field {
            name: field.name().clone(),
            data_type: field.data_type().into(),
            nullable: field.is_nullable(),
            metadata,
        }
    }
}

impl TryFrom<&Field> for ArrowField {
    type Error = ArrowError;

    fn try_from(field: &Field) -> Result<Self, Self::Error> {
        let arrow_type = (&field.data_type).try_into()?;
        Ok(ArrowField::new(&field.name, arrow_type, field.nullable))
    }
}

/// Schema interface for table structure definition
#[wasm_bindgen]
pub struct Schema {
    handle: HandleId,
}

#[wasm_bindgen]
impl Schema {
    /// Get field by name
    #[wasm_bindgen(js_name = "getField")]
    pub fn get_field(&self, name: &str) -> Option<Field> {
        crate::core::with_schema_registry(|registry| {
            registry.get(self.handle).and_then(|schema| {
                schema.field_with_name(name).ok().map(|field| field.into())
            })
        })
    }

    /// Get field index by name
    #[wasm_bindgen(js_name = "getFieldIndex")]
    pub fn get_field_index(&self, name: &str) -> Option<usize> {
        crate::core::with_schema_registry(|registry| {
            registry.get(self.handle).and_then(|schema| {
                schema.index_of(name).ok().map(|index| index)
            })
        })
    }

    /// Get all fields
    #[wasm_bindgen(getter)]
    pub fn fields(&self) -> JsValue {
        crate::core::with_schema_registry(|registry| {
            if let Some(schema) = registry.get(self.handle) {
                let fields: Vec<Field> = schema.fields().iter().map(|f| f.as_ref().into()).collect();
                serde_wasm_bindgen::to_value(&fields).unwrap_or(JsValue::NULL)
            } else {
                JsValue::NULL
            }
        })
    }

    /// Get metadata
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        crate::core::with_schema_registry(|registry| {
            if let Some(schema) = registry.get(self.handle) {
                let metadata: HashMap<String, String> = schema.metadata().iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                serde_wasm_bindgen::to_value(&metadata).unwrap_or(JsValue::NULL)
            } else {
                JsValue::NULL
            }
        })
    }

    /// Check if schemas are equal
    #[wasm_bindgen]
    pub fn equals(&self, other: &Schema) -> bool {
        crate::core::with_schema_registry(|registry| {
            if let (Some(schema1), Some(schema2)) = (registry.get(self.handle), registry.get(other.handle)) {
                schema1.as_ref() == schema2.as_ref()
            } else {
                false
            }
        })
    }

    /// Convert to JSON representation
    #[wasm_bindgen(js_name = "toJSON")]
    pub fn to_json(&self) -> JsValue {
        crate::core::with_schema_registry(|registry| {
            if let Some(schema) = registry.get(self.handle) {
                // Create a simplified JSON representation
                let fields: Vec<Field> = schema.fields().iter().map(|f| f.as_ref().into()).collect();
                let metadata: HashMap<String, String> = schema.metadata().iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                
                let json_obj = serde_json::json!({
                    "fields": fields,
                    "metadata": metadata
                });
                
                serde_wasm_bindgen::to_value(&json_obj).unwrap_or(JsValue::NULL)
            } else {
                JsValue::NULL
            }
        })
    }

    /// Dispose of the schema handle
    #[wasm_bindgen]
    pub fn dispose(&self) {
        crate::core::with_schema_registry(|registry| {
            registry.remove(self.handle);
        });
    }
}

/// Create a schema from fields
#[wasm_bindgen(js_name = "createSchema")]
pub fn create_schema(fields: JsValue) -> Result<Schema, JsValue> {
    let field_list: Vec<Field> = serde_wasm_bindgen::from_value(fields)
        .map_err(|e| JsValue::from_str(&format!("Invalid fields: {}", e)))?;

    let arrow_fields: Result<Vec<ArrowField>, _> = field_list.iter()
        .map(|f| f.try_into())
        .collect();

    match arrow_fields {
        Ok(fields) => {
            let schema = ArrowSchema::new(fields);
            let handle = crate::core::with_schema_registry(|registry| {
                registry.insert(schema)
            });
            
            Ok(Schema { handle })
        }
        Err(e) => Err(JsValue::from_str(&format!("Schema creation failed: {}", e)))
    }
}

/// Create a schema from an Arrow schema
pub fn create_schema_from_arrow(schema: ArrowSchema) -> Schema {
    let handle = crate::core::with_schema_registry(|registry| {
        registry.insert(schema)
    });
    
    Schema { handle }
}