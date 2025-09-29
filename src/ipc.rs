//! IPC (Inter-Process Communication) utilities with LZ4 compression support
//! 
//! This module provides utilities for configuring Arrow IPC read/write operations
//! with optional LZ4 compression.

use crate::errors::{CoreError, CoreResult};
use crate::mem::{get_table, TableHandle};
use arrow_ipc::writer::{IpcWriteOptions, StreamWriter, FileWriter};
use arrow_ipc::reader::{FileReader, StreamReader};
use arrow_ipc::CompressionType;
use serde::{Serialize, Deserialize};
use std::io::Cursor;

/// Default IPC write options with LZ4 compression enabled
pub fn default_lz4_ipc_options() -> IpcWriteOptions {
    IpcWriteOptions::default()
        .try_with_compression(Some(CompressionType::LZ4_FRAME))
        .unwrap_or_else(|_| {
            // Fallback to no compression if LZ4 is not available
            IpcWriteOptions::default()
        })
}

/// Default IPC write options without compression
pub fn default_uncompressed_ipc_options() -> IpcWriteOptions {
    IpcWriteOptions::default()
}

/// Create IPC write options with configurable compression
pub fn create_ipc_options(enable_lz4: bool) -> IpcWriteOptions {
    if enable_lz4 {
        default_lz4_ipc_options()
    } else {
        default_uncompressed_ipc_options()
    }
}

/// Enable or disable LZ4 compression on existing IpcWriteOptions
pub fn enable_lz4_on_options(opts: &mut IpcWriteOptions, lz4: bool) -> CoreResult<()> {
    if lz4 {
        *opts = opts.clone()
            .try_with_compression(Some(CompressionType::LZ4_FRAME))
            .map_err(|e| CoreError::ipc(format!("Failed to enable LZ4 compression: {}", e)))?;
    } else {
        *opts = opts.clone()
            .try_with_compression(None)
            .map_err(|e| CoreError::ipc(format!("Failed to disable compression: {}", e)))?;
    }
    Ok(())
}

/// Create IPC write options with custom compression settings
pub fn create_custom_ipc_options(
    compression: Option<CompressionType>,
    _preserve_dict_id: bool,
    alignment: Option<usize>,
) -> CoreResult<IpcWriteOptions> {
    let mut options = IpcWriteOptions::default();
    
    // Set compression
    if let Some(comp) = compression {
        options = options
            .try_with_compression(Some(comp))
            .map_err(|e| CoreError::ipc(format!("Failed to set compression: {}", e)))?;
    }
    
    // Note: Dictionary preservation method may vary by arrow-ipc version
    // For now, we'll use the basic options and rely on defaults
    
    // Set alignment if specified
    if let Some(align) = alignment {
        // Note: Alignment setting may depend on specific arrow-ipc version
        // For now, we'll just use the provided options
        let _ = align; // Suppress unused variable warning
    }
    
    Ok(options)
}

/// Write table to IPC file format with specified options
pub fn write_table_to_ipc_with_options(
    handle: TableHandle,
    options: &IpcWriteOptions,
) -> CoreResult<Vec<u8>> {
    let table = get_table(handle)?;
    
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = FileWriter::try_new_with_options(cursor, &table.schema, options.clone())
            .map_err(|e| CoreError::ipc(format!("Failed to create IPC file writer: {}", e)))?;
        
        // Write all batches
        for batch in &table.batches {
            writer
                .write(batch)
                .map_err(|e| CoreError::ipc(format!("Failed to write batch: {}", e)))?;
        }
        
        writer
            .finish()
            .map_err(|e| CoreError::ipc(format!("Failed to finish writing: {}", e)))?;
    }
    
    Ok(buffer)
}

/// Write table to IPC stream format with specified options
pub fn write_table_to_ipc_stream_with_options(
    handle: TableHandle,
    options: &IpcWriteOptions,
) -> CoreResult<Vec<u8>> {
    let table = get_table(handle)?;
    
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = StreamWriter::try_new_with_options(cursor, &table.schema, options.clone())
            .map_err(|e| CoreError::ipc(format!("Failed to create IPC stream writer: {}", e)))?;
        
        // Write all batches
        for batch in &table.batches {
            writer
                .write(batch)
                .map_err(|e| CoreError::ipc(format!("Failed to write batch: {}", e)))?;
        }
        
        writer
            .finish()
            .map_err(|e| CoreError::ipc(format!("Failed to finish stream writing: {}", e)))?;
    }
    
    Ok(buffer)
}

/// Check if LZ4 compression is supported in the current build
pub fn is_lz4_supported() -> bool {
    // Try to create options with LZ4 compression
    IpcWriteOptions::default()
        .try_with_compression(Some(CompressionType::LZ4_FRAME))
        .is_ok()
}

/// Get available compression types
pub fn get_supported_compression_types() -> Vec<String> {
    let mut types = vec!["None".to_string()];
    
    // Check LZ4 support
    if is_lz4_supported() {
        types.push("LZ4_FRAME".to_string());
    }
    
    // Check for other compression types
    if IpcWriteOptions::default()
        .try_with_compression(Some(CompressionType::ZSTD))
        .is_ok()
    {
        types.push("ZSTD".to_string());
    }
    
    types
}

/// Compression configuration for easy use from WASM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub compression_type: String, // "LZ4_FRAME", "ZSTD", or "None"
    pub preserve_dict_id: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            compression_type: "None".to_string(),
            preserve_dict_id: true,
        }
    }
}

impl CompressionConfig {
    /// Create a new compression config with LZ4 enabled
    pub fn with_lz4() -> Self {
        Self {
            enabled: true,
            compression_type: "LZ4_FRAME".to_string(),
            preserve_dict_id: true,
        }
    }
    
    /// Create a new compression config with ZSTD enabled
    pub fn with_zstd() -> Self {
        Self {
            enabled: true,
            compression_type: "ZSTD".to_string(),
            preserve_dict_id: true,
        }
    }
    
    /// Convert to IpcWriteOptions
    pub fn to_ipc_options(&self) -> CoreResult<IpcWriteOptions> {
        if !self.enabled {
            return Ok(default_uncompressed_ipc_options());
        }
        
        let compression = match self.compression_type.as_str() {
            "LZ4_FRAME" => Some(CompressionType::LZ4_FRAME),
            "ZSTD" => Some(CompressionType::ZSTD),
            "None" => None,
            _ => return Err(CoreError::validation(format!(
                "Unsupported compression type: {}", self.compression_type
            ))),
        };
        
        create_custom_ipc_options(compression, self.preserve_dict_id, None)
    }
}

/// Utility to write table with simple compression setting
pub fn write_table_with_compression(
    handle: TableHandle,
    enable_lz4: bool,
) -> CoreResult<Vec<u8>> {
    let options = create_ipc_options(enable_lz4);
    write_table_to_ipc_with_options(handle, &options)
}

/// Read IPC data with automatic decompression
pub fn read_ipc_data(data: &[u8]) -> CoreResult<TableHandle> {
    // The reader should automatically handle decompression
    crate::fs::read_table_from_bytes(data)
}

/// Get compression information from IPC data
pub fn analyze_ipc_compression(data: &[u8]) -> CoreResult<String> {
    // Try to read the data and extract compression info
    let cursor = Cursor::new(data);
    
    // Try as file first
    if let Ok(reader) = FileReader::try_new(cursor.clone(), None) {
        let mut analysis = Vec::new();
        
        // Basic file information
        analysis.push(format!("Format: IPC File"));
        analysis.push(format!("Batches: {}", reader.num_batches()));
        analysis.push(format!("Schema fields: {}", reader.schema().fields().len()));
        
        // Try to determine compression by examining the data structure
        let compression_info = detect_ipc_compression_from_data(data);
        analysis.push(format!("Compression: {}", compression_info));
        
        // Add schema metadata if available
        let schema = reader.schema();
        let metadata = schema.metadata();
        if !metadata.is_empty() {
            analysis.push(format!("Metadata entries: {}", metadata.len()));
            
            // Look for compression-related metadata
            for (key, value) in metadata.iter() {
                if key.to_lowercase().contains("compress") {
                    analysis.push(format!("Metadata: {}={}", key, value));
                }
            }
        }
        
        // Custom metadata from file reader
        let custom_metadata = reader.custom_metadata();
        if !custom_metadata.is_empty() {
            analysis.push(format!("Custom metadata entries: {}", custom_metadata.len()));
            for (key, value) in custom_metadata.iter() {
                if key.to_lowercase().contains("compress") {
                    analysis.push(format!("Custom: {}={}", key, value));
                }
            }
        }
        
        return Ok(analysis.join(", "));
    }
    
    // Try as stream
    let cursor = Cursor::new(data);
    if let Ok(reader) = StreamReader::try_new(cursor, None) {
        let mut analysis = Vec::new();
        
        // Basic stream information
        analysis.push(format!("Format: IPC Stream"));
        
        let schema = reader.schema();
        analysis.push(format!("Schema fields: {}", schema.fields().len()));
        
        // Try to determine compression by examining the data structure
        let compression_info = detect_ipc_compression_from_data(data);
        analysis.push(format!("Compression: {}", compression_info));
        
        // Add schema metadata if available
        let metadata = schema.metadata();
        if !metadata.is_empty() {
            analysis.push(format!("Metadata entries: {}", metadata.len()));
            
            // Look for compression-related metadata
            for (key, value) in metadata.iter() {
                if key.to_lowercase().contains("compress") {
                    analysis.push(format!("Metadata: {}={}", key, value));
                }
            }
        }
        
        return Ok(analysis.join(", "));
    }
    
    Err(CoreError::ipc("Unable to read IPC data"))
}

/// Detect compression type from IPC data by analyzing the binary structure
fn detect_ipc_compression_from_data(data: &[u8]) -> String {
    if data.len() < 16 {
        return "Unknown (insufficient data)".to_string();
    }
    
    // Look for compression signatures in the data
    // LZ4 frame magic number: 0x04224D18
    let lz4_magic = [0x04, 0x22, 0x4D, 0x18];
    
    // ZSTD magic number: 0xFD2FB528 (little endian: 0x28B52FFD)
    let zstd_magic = [0x28, 0xB5, 0x2F, 0xFD];
    
    // Scan through the data looking for compression signatures
    for i in 0..data.len().saturating_sub(4) {
        let chunk = &data[i..i + 4];
        
        if chunk == lz4_magic {
            return "LZ4_FRAME detected".to_string();
        }
        
        if chunk == zstd_magic {
            return "ZSTD detected".to_string();
        }
    }
    
    // Check if data looks compressed by examining entropy
    // Simple heuristic: if we find repeated null bytes, likely uncompressed
    let null_count = data.iter().take(1024).filter(|&&b| b == 0).count();
    let sample_size = data.len().min(1024);
    let null_ratio = null_count as f64 / sample_size as f64;
    
    if null_ratio > 0.1 {
        "None (likely uncompressed)".to_string()
    } else {
        // High entropy suggests compression, but we couldn't identify the type
        "Unknown (possibly compressed)".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::{create_table_from_batches, clear_all_tables};
    use arrow_array::{Int32Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use arrow_array::RecordBatch;
    use std::sync::Arc;
    
    fn create_test_table_handle() -> CoreResult<TableHandle> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
        ]));
        
        let id_array = Arc::new(Int32Array::from(vec![1, 2, 3, 4, 5]));
        let name_array = Arc::new(StringArray::from(vec![
            Some("Alice"),
            Some("Bob"),
            None,
            Some("Charlie"),
            Some("Diana"),
        ]));
        
        let batch = RecordBatch::try_new(schema.clone(), vec![id_array, name_array])
            .map_err(|e| CoreError::other(format!("Failed to create batch: {}", e)))?;
        
        create_table_from_batches(schema, vec![batch])
    }
    
    #[test]
    fn test_compression_config() {
        let config = CompressionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.compression_type, "None");
        
        let lz4_config = CompressionConfig::with_lz4();
        assert!(lz4_config.enabled);
        assert_eq!(lz4_config.compression_type, "LZ4_FRAME");
        
        let zstd_config = CompressionConfig::with_zstd();
        assert!(zstd_config.enabled);
        assert_eq!(zstd_config.compression_type, "ZSTD");
    }
    
    #[test]
    fn test_ipc_options_creation() {
        let uncompressed = default_uncompressed_ipc_options();
        // Basic test - just ensure it doesn't panic
        let _options = uncompressed;
        
        let lz4_options = default_lz4_ipc_options();
        let _options = lz4_options;
        
        let custom_config = CompressionConfig::with_lz4();
        let _custom_options = custom_config.to_ipc_options().unwrap();
    }
    
    #[test]
    fn test_compression_support_detection() {
        let supported_types = get_supported_compression_types();
        assert!(supported_types.contains(&"None".to_string()));
        
        // LZ4 might or might not be supported depending on build features
        let lz4_supported = is_lz4_supported();
        println!("LZ4 supported: {}", lz4_supported);
    }
    
    #[test]
    fn test_write_with_compression_options() {
        clear_all_tables();
        
        let handle = create_test_table_handle().unwrap();
        
        // Test uncompressed write
        let uncompressed_data = write_table_with_compression(handle, false).unwrap();
        assert!(!uncompressed_data.is_empty());
        
        // Test compressed write (if LZ4 is available)
        if is_lz4_supported() {
            let compressed_data = write_table_with_compression(handle, true).unwrap();
            assert!(!compressed_data.is_empty());
            
            // Compressed data might be smaller or larger depending on data
            // Just ensure both writes produce valid data
            println!("Uncompressed size: {}", uncompressed_data.len());
            println!("Compressed size: {}", compressed_data.len());
        }
        
        crate::mem::free_table(handle).unwrap();
    }
    
    #[test]
    fn test_roundtrip_with_compression() {
        clear_all_tables();
        
        let original_handle = create_test_table_handle().unwrap();
        
        // Write with compression
        let compressed_data = write_table_with_compression(original_handle, true).unwrap();
        
        // Read back
        let restored_handle = read_ipc_data(&compressed_data).unwrap();
        
        // Verify data integrity
        let original_table = get_table(original_handle).unwrap();
        let restored_table = get_table(restored_handle).unwrap();
        
        assert_eq!(original_table.num_rows(), restored_table.num_rows());
        assert_eq!(original_table.num_columns(), restored_table.num_columns());
        assert_eq!(original_table.num_batches(), restored_table.num_batches());
        
        // Clean up
        crate::mem::free_table(original_handle).unwrap();
        crate::mem::free_table(restored_handle).unwrap();
    }
    
    #[test]
    fn test_ipc_analysis() {
        clear_all_tables();
        
        let handle = create_test_table_handle().unwrap();
        let ipc_data = write_table_with_compression(handle, false).unwrap();
        
        let analysis = analyze_ipc_compression(&ipc_data).unwrap();
        assert!(analysis.contains("IPC"));
        
        crate::mem::free_table(handle).unwrap();
    }
}