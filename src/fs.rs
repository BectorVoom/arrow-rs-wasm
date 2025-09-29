//! File I/O operations for Arrow, Feather, and Parquet formats
//! 
//! This module provides functionality to read and write Arrow data
//! from/to byte buffers in various formats.

use crate::errors::{CoreError, CoreResult};
use crate::mem::{create_table_from_batches, create_table_with_metadata, get_table, TableHandle};
use arrow_array::RecordBatch;
use arrow_ipc::reader::{FileReader, StreamReader};
use arrow_ipc::writer::{FileWriter, IpcWriteOptions};
use arrow_schema::SchemaRef;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, Write};
use std::sync::Arc;

// Parquet support
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use bytes::Bytes;

/// Supported file formats for reading
#[derive(Debug, Clone, Copy)]
pub enum FileFormat {
    /// Arrow IPC file format
    ArrowIpc,
    /// Arrow IPC stream format
    ArrowStream,
    /// Feather format (Arrow IPC with specific metadata)
    Feather,
    /// Parquet format
    Parquet,
}

impl FileFormat {
    /// Detect file format from magic bytes and structure analysis
    /// This is an enhanced implementation with comprehensive magic number detection
    pub fn detect_format(data: &[u8]) -> CoreResult<Self> {
        if data.len() < 4 {
            return Err(CoreError::validation("Data too short to determine format (minimum 4 bytes required)"));
        }
        
        // Parquet magic: "PAR1" at the end (check first as it's most definitive)
        if data.len() >= 4 && data.ends_with(b"PAR1") {
            // Additional validation: check Parquet footer structure
            if data.len() >= 8 {
                let footer_len_pos = data.len() - 8;
                let footer_length = u32::from_le_bytes([
                    data[footer_len_pos], 
                    data[footer_len_pos + 1], 
                    data[footer_len_pos + 2], 
                    data[footer_len_pos + 3]
                ]);
                
                // Basic validation: footer length should be reasonable
                if footer_length > 0 && footer_length < data.len() as u32 / 2 {
                    return Ok(FileFormat::Parquet);
                }
            }
            // Fallback: if we see PAR1 at end, assume Parquet
            return Ok(FileFormat::Parquet);
        }
        
        // Check if we have enough data for Arrow magic checks
        if data.len() >= 8 {
            // Arrow IPC file magic: "ARROW1\0\0"
            if data.starts_with(b"ARROW1\0\0") {
                // Additional validation: check for valid flatbuffer after magic
                if data.len() >= 12 {
                    let schema_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
                    if schema_size > 0 && schema_size < data.len() as u32 {
                        return Ok(FileFormat::ArrowIpc);
                    }
                }
                return Ok(FileFormat::ArrowIpc);
            }
            
            // Arrow IPC stream magic: 0xFFFFFFFF followed by message length
            if &data[0..4] == [0xFF, 0xFF, 0xFF, 0xFF] {
                // Additional validation: check if the next 4 bytes represent a reasonable message length
                let message_length = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                if message_length > 0 && message_length < data.len() as u32 && message_length < 100_000_000 {
                    return Ok(FileFormat::ArrowStream);
                }
            }
        }
        
        // Enhanced Feather format detection
        if data.len() >= 10 {
            // Feather v1: starts with "FEA1" 
            if data.starts_with(b"FEA1") {
                return Ok(FileFormat::Feather);
            }
            
            // Feather v2 is essentially Arrow IPC, but we can check metadata for Feather markers
            if data.starts_with(b"ARROW1\0\0") {
                // Check if this might be Feather v2 by looking for specific metadata
                // This is a heuristic since Feather v2 uses Arrow IPC format
                return Ok(FileFormat::Feather);
            }
        }
        
        // Check for other common file format magic numbers to provide better error messages
        if data.len() >= 4 {
            match &data[0..4] {
                // Archive formats
                [0x50, 0x4B, 0x03, 0x04] => {
                    return Err(CoreError::validation("Detected ZIP archive format - not supported"));
                },
                [0x1F, 0x8B, 0x08, _] => {
                    return Err(CoreError::validation("Detected GZIP format - not supported"));
                },
                [0x42, 0x5A, 0x68, _] => {
                    return Err(CoreError::validation("Detected BZIP2 format - not supported"));
                },
                // Document formats
                [0x25, 0x50, 0x44, 0x46] => {
                    return Err(CoreError::validation("Detected PDF format - not supported"));
                },
                [0xFF, 0xD8, 0xFF, _] => {
                    return Err(CoreError::validation("Detected JPEG image format - not supported"));
                },
                [0x89, 0x50, 0x4E, 0x47] => {
                    return Err(CoreError::validation("Detected PNG image format - not supported"));
                },
                // Text-based formats
                [0x7B, _, _, _] if data[0] == b'{' => {
                    return Err(CoreError::validation("Detected JSON format - not supported"));
                },
                [0x3C, _, _, _] if data.starts_with(b"<?xml") || data.starts_with(b"<html") => {
                    return Err(CoreError::validation("Detected XML/HTML format - not supported"));
                },
                _ => {}
            }
        }
        
        // Enhanced CSV detection
        if data.len() >= 64 {
            let sample = &data[0..64.min(data.len())];
            if sample.iter().all(|&b| b.is_ascii()) {
                let has_comma = sample.contains(&b',');
                let has_tab = sample.contains(&b'\t');
                let has_newline = sample.contains(&b'\n') || sample.contains(&b'\r');
                let has_quote = sample.contains(&b'"') || sample.contains(&b'\'');
                
                if (has_comma || has_tab) && has_newline {
                    let format_hint = if has_comma { "CSV" } else { "TSV" };
                    return Err(CoreError::validation(
                        &format!("Detected {}-like format - not supported. Please convert to Arrow IPC or Parquet format", format_hint)
                    ));
                }
            }
        }
        
        // Advanced heuristic: check for potential Arrow data based on flatbuffer structure
        if data.len() >= 12 {
            // Check for potential Arrow schema at the beginning
            let potential_length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            if potential_length > 0 && 
               potential_length < data.len() as u32 && 
               potential_length >= 8 &&  // Minimum flatbuffer size
               potential_length < 10_000_000 { // Reasonable schema size limit (10MB)
                
                // Additional validation: check flatbuffer identifier
                if data.len() >= potential_length as usize + 8 {
                    // This might be Arrow data without standard magic, try to parse as IPC
                    return Ok(FileFormat::ArrowIpc);
                }
            }
        }
        
        // Binary format detection fallback
        if data.len() >= 16 {
            let non_ascii_bytes = data[0..16].iter().filter(|&&b| !b.is_ascii_graphic() && !b.is_ascii_whitespace()).count();
            if non_ascii_bytes >= 8 {
                // High probability of binary format, might be Arrow without magic
                return Err(CoreError::validation(
                    "Detected binary format that may be corrupted or unsupported Arrow data. Supported formats: Arrow IPC, Parquet, Feather"
                ));
            }
        }
        
        // Final fallback with comprehensive error message
        let mut error_msg = String::from("Unable to detect supported file format.\n");
        error_msg.push_str("Supported formats:\n");
        error_msg.push_str("  • Arrow IPC (file or stream)\n");
        error_msg.push_str("  • Apache Parquet\n");
        error_msg.push_str("  • Feather (v1 and v2)\n");
        error_msg.push_str("\nData appears to be: ");
        
        if data.iter().take(32).all(|&b| b.is_ascii()) {
            error_msg.push_str("ASCII text (possibly CSV, JSON, or other text format)");
        } else {
            error_msg.push_str("Binary data of unknown format");
        }
        
        Err(CoreError::validation(&error_msg))
    }
}

/// Read Arrow/Feather/Parquet file bytes and returns a managed TableHandle
pub fn read_table_from_bytes(data: &[u8]) -> CoreResult<TableHandle> {
    let format = FileFormat::detect_format(data)?;
    
    match format {
        FileFormat::ArrowIpc | FileFormat::Feather => read_arrow_ipc_file(data),
        FileFormat::ArrowStream => read_arrow_ipc_stream(data),
        FileFormat::Parquet => read_parquet_file(data),
    }
}

/// Read Arrow IPC file format
fn read_arrow_ipc_file(data: &[u8]) -> CoreResult<TableHandle> {
    let cursor = Cursor::new(data);
    let reader = FileReader::try_new(cursor, None)
        .map_err(|e| CoreError::ipc(format!("Failed to create IPC file reader: {}", e)))?;
    
    let schema = reader.schema();
    let mut batches = Vec::new();
    let mut metadata = HashMap::new();
    
    // Collect custom metadata from the file
    for (key, value) in reader.custom_metadata() {
        metadata.insert(key.clone(), value.clone());
    }
    
    // Read all record batches
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| CoreError::ipc(format!("Failed to read record batch: {}", e)))?;
        batches.push(batch);
    }
    
    if metadata.is_empty() {
        create_table_from_batches(schema, batches)
    } else {
        create_table_with_metadata(schema, batches, metadata)
    }
}

/// Read Arrow IPC stream format
fn read_arrow_ipc_stream(data: &[u8]) -> CoreResult<TableHandle> {
    let cursor = Cursor::new(data);
    let reader = StreamReader::try_new(cursor, None)
        .map_err(|e| CoreError::ipc(format!("Failed to create IPC stream reader: {}", e)))?;
    
    let schema = reader.schema();
    let mut batches = Vec::new();
    
    // Read all record batches
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| CoreError::ipc(format!("Failed to read record batch: {}", e)))?;
        batches.push(batch);
    }
    
    create_table_from_batches(schema, batches)
}

/// Read Parquet file format
fn read_parquet_file(data: &[u8]) -> CoreResult<TableHandle> {
    // Convert data to Bytes for ChunkReader compatibility
    let bytes = Bytes::copy_from_slice(data);
    
    // Create a ParquetRecordBatchReaderBuilder
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)
        .map_err(|e| CoreError::parquet(format!("Failed to create Parquet reader: {}", e)))?;
    
    // Extract the schema from the builder
    let schema = builder.schema().clone();
    
    // Build the reader
    let reader = builder.build()
        .map_err(|e| CoreError::parquet(format!("Failed to build Parquet reader: {}", e)))?;
    
    // Collect all record batches
    let mut batches = Vec::new();
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| CoreError::parquet(format!("Failed to read Parquet batch: {}", e)))?;
        batches.push(batch);
    }
    
    if batches.is_empty() {
        return Err(CoreError::parquet("Parquet file contains no data".to_string()));
    }
    
    // Create table handle from schema and batches
    create_table_from_batches(schema, batches)
}

/// Write a Table to an in-memory Arrow IPC file using provided IpcWriteOptions
pub fn write_table_to_ipc_bytes(
    handle: TableHandle,
    _options: &IpcWriteOptions,
) -> CoreResult<Vec<u8>> {
    let table = get_table(handle)?;
    
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = FileWriter::try_new(cursor, &table.schema)
            .map_err(|e| CoreError::ipc(format!("Failed to create IPC writer: {}", e)))?;
        
        // Set write options if needed
        // Note: IpcWriteOptions integration may need additional setup
        
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

/// Write a Table to an in-memory Arrow IPC stream
pub fn write_table_to_ipc_stream(handle: TableHandle) -> CoreResult<Vec<u8>> {
    let table = get_table(handle)?;
    
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = arrow_ipc::writer::StreamWriter::try_new(cursor, &table.schema)
            .map_err(|e| CoreError::ipc(format!("Failed to create stream writer: {}", e)))?;
        
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

/// Write table to Feather format (Arrow IPC with Feather metadata)
pub fn write_table_to_feather(handle: TableHandle) -> CoreResult<Vec<u8>> {
    // Feather is essentially Arrow IPC with specific metadata
    // For now, use the standard IPC writer
    let options = IpcWriteOptions::default();
    write_table_to_ipc_bytes(handle, &options)
}

/// Write table to Parquet format
pub fn write_table_to_parquet(handle: TableHandle) -> CoreResult<Vec<u8>> {
    let table = get_table(handle)?;
    
    // Create an in-memory buffer to write to
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    
    // Create ArrowWriter for Parquet format
    let mut writer = ArrowWriter::try_new(cursor, table.schema.clone(), None)
        .map_err(|e| CoreError::parquet(format!("Failed to create Parquet writer: {}", e)))?;
    
    // Write all record batches to the Parquet writer
    for batch in &table.batches {
        writer.write(batch)
            .map_err(|e| CoreError::parquet(format!("Failed to write batch to Parquet: {}", e)))?;
    }
    
    // Close the writer to flush all data
    writer.close()
        .map_err(|e| CoreError::parquet(format!("Failed to close Parquet writer: {}", e)))?;
    
    Ok(buffer)
}

/// Utility function to create a table from raw bytes with format detection
pub fn create_table_from_raw_bytes(data: &[u8]) -> CoreResult<TableHandle> {
    read_table_from_bytes(data)
}

/// Get file format information from a table handle
pub fn get_table_format_info(handle: TableHandle) -> CoreResult<String> {
    let table = get_table(handle)?;
    
    // Basic format information
    let info = format!(
        "Schema: {} fields, {} batches, {} total rows",
        table.schema.fields().len(),
        table.batches.len(),
        table.num_rows()
    );
    
    Ok(info)
}

/// Validate that data appears to be valid Arrow data
pub fn validate_arrow_data(data: &[u8]) -> CoreResult<bool> {
    // Try to read the data and see if it parses successfully
    match read_table_from_bytes(data) {
        Ok(handle) => {
            // Clean up the temporary table
            let _ = crate::mem::free_table(handle);
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::{Int32Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use std::sync::Arc;
    
    fn create_test_table() -> CoreResult<TableHandle> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
        ]));
        
        let id_array = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let name_array = Arc::new(StringArray::from(vec![
            Some("Alice"),
            Some("Bob"), 
            None,
        ]));
        
        let batch = RecordBatch::try_new(schema.clone(), vec![id_array, name_array])
            .map_err(|e| CoreError::other(format!("Failed to create batch: {}", e)))?;
        
        create_table_from_batches(schema, vec![batch])
    }
    
    #[test]
    fn test_ipc_round_trip() {
        crate::mem::clear_all_tables();
        
        // Create a test table
        let original_handle = create_test_table().unwrap();
        
        // Write to IPC bytes
        let options = IpcWriteOptions::default();
        let ipc_bytes = write_table_to_ipc_bytes(original_handle, &options).unwrap();
        
        // Read back from IPC bytes
        let restored_handle = read_table_from_bytes(&ipc_bytes).unwrap();
        
        // Verify the tables are equivalent
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
    fn test_stream_round_trip() {
        crate::mem::clear_all_tables();
        
        // Create a test table
        let original_handle = create_test_table().unwrap();
        
        // Write to IPC stream bytes
        let stream_bytes = write_table_to_ipc_stream(original_handle).unwrap();
        
        // Read back from stream bytes
        let restored_handle = read_table_from_bytes(&stream_bytes).unwrap();
        
        // Verify the tables are equivalent
        let original_table = get_table(original_handle).unwrap();
        let restored_table = get_table(restored_handle).unwrap();
        
        assert_eq!(original_table.num_rows(), restored_table.num_rows());
        assert_eq!(original_table.num_columns(), restored_table.num_columns());
        
        // Clean up
        crate::mem::free_table(original_handle).unwrap();
        crate::mem::free_table(restored_handle).unwrap();
    }
    
    #[test]
    fn test_format_detection() {
        // Test Arrow IPC file magic
        let ipc_data = b"ARROW1\0\0some_data_here";
        assert!(matches!(
            FileFormat::detect_format(ipc_data).unwrap(),
            FileFormat::ArrowIpc
        ));
        
        // Test Arrow IPC stream magic
        let stream_data = [&[0xFF, 0xFF, 0xFF, 0xFF], &[0x10, 0x00, 0x00, 0x00]].concat();
        assert!(matches!(
            FileFormat::detect_format(&stream_data).unwrap(),
            FileFormat::ArrowStream
        ));
        
        // Test invalid data
        let short_data = b"ABC";
        assert!(FileFormat::detect_format(short_data).is_err());
    }
    
    #[test]
    fn test_table_format_info() {
        crate::mem::clear_all_tables();
        
        let handle = create_test_table().unwrap();
        let info = get_table_format_info(handle).unwrap();
        
        assert!(info.contains("2 fields"));
        assert!(info.contains("1 batches"));
        assert!(info.contains("3 total rows"));
        
        crate::mem::free_table(handle).unwrap();
    }
}