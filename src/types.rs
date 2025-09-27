//! Type definitions and conversions for the WASM Arrow library.
//!
//! This module provides TypeScript-compatible type definitions that follow
//! the API specification without using prohibited constructs.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use arrow_schema::DataType as ArrowDataType;
use arrow_schema::TimeUnit as ArrowTimeUnit;

/// Library version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub arrow_version: String,
}

/// WASM wrapper for VersionInfo
#[wasm_bindgen]
pub struct VersionInfoWasm {
    major: u32,
    minor: u32,
    patch: u32,
    arrow_version: String,
}

#[wasm_bindgen]
impl VersionInfoWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(major: u32, minor: u32, patch: u32, arrow_version: &str) -> VersionInfoWasm {
        VersionInfoWasm {
            major,
            minor,
            patch,
            arrow_version: arrow_version.to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn major(&self) -> u32 {
        self.major
    }

    #[wasm_bindgen(getter)]
    pub fn minor(&self) -> u32 {
        self.minor
    }

    #[wasm_bindgen(getter)]
    pub fn patch(&self) -> u32 {
        self.patch
    }

    #[wasm_bindgen(getter, js_name = "arrowVersion")]
    pub fn arrow_version(&self) -> String {
        self.arrow_version.clone()
    }
}

impl From<VersionInfo> for VersionInfoWasm {
    fn from(version: VersionInfo) -> Self {
        VersionInfoWasm {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            arrow_version: version.arrow_version,
        }
    }
}

/// Simple data type representation
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DataType {
    type_id: u32,
}

#[wasm_bindgen]
impl DataType {
    #[wasm_bindgen(js_name = "newNull")]
    pub fn new_null() -> DataType {
        DataType { type_id: 0 }
    }

    #[wasm_bindgen(js_name = "newBool")]
    pub fn new_bool() -> DataType {
        DataType { type_id: 1 }
    }

    #[wasm_bindgen(js_name = "newInt32")]
    pub fn new_int32() -> DataType {
        DataType { type_id: 2 }
    }

    #[wasm_bindgen(js_name = "newInt64")]
    pub fn new_int64() -> DataType {
        DataType { type_id: 3 }
    }

    #[wasm_bindgen(js_name = "newFloat32")]
    pub fn new_float32() -> DataType {
        DataType { type_id: 4 }
    }

    #[wasm_bindgen(js_name = "newFloat64")]
    pub fn new_float64() -> DataType {
        DataType { type_id: 5 }
    }

    #[wasm_bindgen(js_name = "newUtf8")]
    pub fn new_utf8() -> DataType {
        DataType { type_id: 6 }
    }

    #[wasm_bindgen(getter, js_name = "typeId")]
    pub fn type_id(&self) -> u32 {
        self.type_id
    }
}

impl From<&ArrowDataType> for DataType {
    fn from(arrow_type: &ArrowDataType) -> Self {
        let type_id = match arrow_type {
            ArrowDataType::Null => 0,
            ArrowDataType::Boolean => 1,
            ArrowDataType::Int32 => 2,
            ArrowDataType::Int64 => 3,
            ArrowDataType::Float32 => 4,
            ArrowDataType::Float64 => 5,
            ArrowDataType::Utf8 => 6,
            _ => 0, // Default to null for unsupported types
        };
        DataType { type_id }
    }
}

impl TryFrom<&DataType> for ArrowDataType {
    type Error = crate::error::ArrowError;

    fn try_from(data_type: &DataType) -> Result<Self, Self::Error> {
        match data_type.type_id {
            0 => Ok(ArrowDataType::Null),
            1 => Ok(ArrowDataType::Boolean),
            2 => Ok(ArrowDataType::Int32),
            3 => Ok(ArrowDataType::Int64),
            4 => Ok(ArrowDataType::Float32),
            5 => Ok(ArrowDataType::Float64),
            6 => Ok(ArrowDataType::Utf8),
            _ => Err(crate::arrow_error!(
                crate::error::ErrorCode::NotImplemented,
                &format!("Data type with ID {} not implemented", data_type.type_id)
            )),
        }
    }
}

/// Compression type enumeration
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None = 0,
    LZ4 = 1, 
    ZSTD = 2,
}

/// Metadata version enumeration
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataVersion {
    V4 = 4,
    V5 = 5,
}

/// Dictionary handling enumeration
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DictionaryHandling {
    Replace = 0,
    Delta = 1,
}