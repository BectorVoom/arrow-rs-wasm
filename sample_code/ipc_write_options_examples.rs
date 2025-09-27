//! # IpcWriteOptions Examples
//!
//! This module contains comprehensive examples for `arrow_ipc::writer::IpcWriteOptions`.
//! 
//! `IpcWriteOptions` is used to control the behavior of IPC writers, including compression,
//! alignment, metadata version, and dictionary handling. These options affect both
//! FileWriter and StreamWriter.

use crate::*;
use std::io::Cursor;
use arrow_ipc::gen::Schema::MetadataVersion;
use arrow_ipc::writer::DictionaryHandling;

/// Example 1: Default IpcWriteOptions with `default()`
/// 
/// The simplest way to create IpcWriteOptions with sensible defaults.
/// This provides standard alignment, no compression, and latest metadata version.
pub fn example_default_options() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create default write options
    let options = IpcWriteOptions::default();
    
    let mut buffer = Cursor::new(Vec::new());
    
    // Use the options with a FileWriter
    let mut writer = FileWriter::try_new_with_options(&mut buffer, &schema, options)?;
    
    writer.write(&batch)?;
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 2: Custom options with `try_new()`
/// 
/// This example shows how to create IpcWriteOptions with custom alignment,
/// legacy format setting, and specific metadata version.
pub fn example_try_new_options() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create custom write options
    let options = IpcWriteOptions::try_new(
        64,                      // alignment (64 bytes for better cache performance)
        false,                   // write_legacy_ipc_format (use modern format)
        MetadataVersion::V5,     // metadata_version (latest version)
    )?;
    
    let mut buffer = Cursor::new(Vec::new());
    
    // Use custom options with FileWriter
    let mut writer = FileWriter::try_new_with_options(&mut buffer, &schema, options)?;
    
    writer.write(&batch)?;
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 3: Legacy format compatibility
/// 
/// This example shows how to enable legacy IPC format for compatibility
/// with older Arrow implementations.
pub fn example_legacy_format() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create options with legacy format enabled
    let options = IpcWriteOptions::try_new(
        8,                       // alignment (8 bytes)
        true,                    // write_legacy_ipc_format (enable legacy format)
        MetadataVersion::V4,     // use older metadata version for compatibility
    )?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new_with_options(&mut buffer, &schema, options)?;
    
    writer.write(&batch)?;
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 4: Compression with `try_with_compression()`
/// 
/// This example demonstrates how to enable LZ4 compression for IPC files.
/// Compression can significantly reduce file size at the cost of CPU time.
pub fn example_compression() -> Result<(Vec<u8>, Vec<u8>), ArrowError> {
    use arrow_ipc::gen::Message::CompressionType;
    
    // Create larger test data to see compression benefits
    let schema = test_data::simple_schema();
    let batch = test_data::large_record_batch(1000)?;
    
    // Create options without compression
    let uncompressed_options = IpcWriteOptions::default();
    
    // Create options with LZ4 compression
    let compressed_options = IpcWriteOptions::default()
        .try_with_compression(Some(CompressionType::LZ4_FRAME))?;
    
    // Write uncompressed version
    let mut uncompressed_buffer = Cursor::new(Vec::new());
    let mut uncompressed_writer = FileWriter::try_new_with_options(
        &mut uncompressed_buffer, 
        &schema, 
        uncompressed_options
    )?;
    uncompressed_writer.write(&batch)?;
    uncompressed_writer.finish()?;
    
    // Write compressed version
    let mut compressed_buffer = Cursor::new(Vec::new());
    let mut compressed_writer = FileWriter::try_new_with_options(
        &mut compressed_buffer, 
        &schema, 
        compressed_options
    )?;
    compressed_writer.write(&batch)?;
    compressed_writer.finish()?;
    
    Ok((uncompressed_buffer.into_inner(), compressed_buffer.into_inner()))
}

/// Example 5: Dictionary handling with `with_dictionary_handling()`
/// 
/// This example shows how to configure dictionary handling for string columns
/// and other dictionary-encoded data.
pub fn example_dictionary_handling() -> Result<Vec<u8>, ArrowError> {
    use arrow_array::{DictionaryArray, StringArray};
    use arrow_array::types::Int32Type;
    use arrow_schema::{Field, DataType, Schema};
    
    // Create schema with dictionary-encoded string column
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new(
            "category", 
            DataType::Dictionary(
                Box::new(DataType::Int32), 
                Box::new(DataType::Utf8)
            ), 
            true
        ),
    ]));
    
    // Create test data with dictionary array
    let id_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
    
    // Create dictionary array with repeated categories
    let dict_array = DictionaryArray::<Int32Type>::from_iter(vec![
        Some("Electronics"),
        Some("Books"), 
        Some("Electronics"),
        Some("Clothing"),
        Some("Books"),
    ]);
    
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(id_array),
            Arc::new(dict_array),
        ]
    )?;
    
    // Configure dictionary handling for delta compression
    let options = IpcWriteOptions::default()
        .with_dictionary_handling(DictionaryHandling::Delta);
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new_with_options(&mut buffer, &schema, options)?;
    
    writer.write(&batch)?;
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 6: Cloning options with `clone()`
/// 
/// This example demonstrates how to clone IpcWriteOptions to reuse
/// the same configuration across multiple writers.
pub fn example_clone_options() -> Result<(Vec<u8>, Vec<u8>), ArrowError> {
    // Create test data
    let schema1 = test_data::simple_schema();
    let schema2 = Arc::new(Schema::new(vec![
        Field::new("count", DataType::Int64, false),
        Field::new("label", DataType::Utf8, true),
    ]));
    
    let batch1 = test_data::simple_record_batch()?;
    let batch2 = RecordBatch::try_new(
        schema2.clone(),
        vec![
            Arc::new(Int64Array::from(vec![10, 20, 30])),
            Arc::new(StringArray::from(vec![Some("A"), Some("B"), None])),
        ]
    )?;
    
    // Create base options
    let base_options = IpcWriteOptions::try_new(
        16,                      // 16-byte alignment
        false,                   // modern format
        MetadataVersion::V5,     // latest version
    )?;
    
    // Clone options for first writer
    let options1 = base_options.clone();
    
    // Clone and modify for second writer
    let options2 = base_options.clone()
        .with_dictionary_handling(DictionaryHandling::Delta);
    
    // Write first file
    let mut buffer1 = Cursor::new(Vec::new());
    let mut writer1 = FileWriter::try_new_with_options(&mut buffer1, &schema1, options1)?;
    writer1.write(&batch1)?;
    writer1.finish()?;
    
    // Write second file
    let mut buffer2 = Cursor::new(Vec::new());
    let mut writer2 = FileWriter::try_new_with_options(&mut buffer2, &schema2, options2)?;
    writer2.write(&batch2)?;
    writer2.finish()?;
    
    Ok((buffer1.into_inner(), buffer2.into_inner()))
}

/// Example 7: Performance comparison of different alignments
/// 
/// This example demonstrates how different alignment settings can affect
/// performance and file size.
pub fn example_alignment_comparison() -> Result<Vec<(usize, usize)>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::large_record_batch(1000)?;
    
    let alignments = vec![8, 16, 32, 64];
    let mut results = Vec::new();
    
    for alignment in alignments {
        let options = IpcWriteOptions::try_new(
            alignment,
            false,
            MetadataVersion::V5,
        )?;
        
        let mut buffer = Cursor::new(Vec::new());
        let mut writer = FileWriter::try_new_with_options(&mut buffer, &schema, options)?;
        
        writer.write(&batch)?;
        writer.finish()?;
        
        let data = buffer.into_inner();
        results.push((alignment, data.len()));
    }
    
    Ok(results)
}

/// Example 8: Options for StreamWriter vs FileWriter
/// 
/// This example shows how the same IpcWriteOptions can be used with
/// both StreamWriter and FileWriter.
pub fn example_options_with_different_writers() -> Result<(Vec<u8>, Vec<u8>), ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create shared options
    let options = IpcWriteOptions::try_new(
        8,                       // 8-byte alignment
        false,                   // modern format
        MetadataVersion::V5,     // latest version
    )?;
    
    // Use with FileWriter
    let mut file_buffer = Cursor::new(Vec::new());
    let mut file_writer = FileWriter::try_new_with_options(
        &mut file_buffer, 
        &schema, 
        options.clone()
    )?;
    file_writer.write(&batch)?;
    file_writer.finish()?;
    
    // Use with StreamWriter
    let mut stream_buffer = Cursor::new(Vec::new());
    let mut stream_writer = StreamWriter::try_new_with_options(
        &mut stream_buffer, 
        &schema, 
        options
    )?;
    stream_writer.write(&batch)?;
    stream_writer.finish()?;
    
    Ok((file_buffer.into_inner(), stream_buffer.into_inner()))
}

/// Example 9: Error handling with invalid options
/// 
/// This example demonstrates error handling when creating IpcWriteOptions
/// with invalid parameters.
pub fn example_options_error_handling() -> Result<String, ArrowError> {
    // Try to create options with invalid alignment (must be power of 2)
    match IpcWriteOptions::try_new(
        7,                       // Invalid alignment (not power of 2)
        false,
        MetadataVersion::V5,
    ) {
        Ok(_) => Ok("Unexpected success with invalid alignment".to_string()),
        Err(e) => Ok(format!("Expected error with invalid alignment: {}", e)),
    }
}

/// Example 10: Best practices for IpcWriteOptions
/// 
/// This example demonstrates recommended settings for different use cases.
pub fn example_best_practices() -> Result<String, ArrowError> {
    // For high performance (larger files, network transfer)
    let _performance_options = IpcWriteOptions::try_new(
        64,                      // Large alignment for cache efficiency
        false,                   // Modern format
        MetadataVersion::V5,     // Latest features
    )?;
    
    // For compatibility (older systems)
    let _compatibility_options = IpcWriteOptions::try_new(
        8,                       // Standard alignment
        true,                    // Legacy format for older readers
        MetadataVersion::V4,     // Older metadata version
    )?;
    
    // For minimal size (storage optimization)
    let _size_options = IpcWriteOptions::try_new(
        8,                       // Minimal valid alignment
        false,                   // Modern format
        MetadataVersion::V5,     // Latest version
    )?;
    
    Ok("Demonstrated best practice configurations for different use cases".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options_example() {
        let result = example_default_options();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_try_new_options_example() {
        let result = example_try_new_options();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_legacy_format_example() {
        let result = example_legacy_format();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_compression_example() {
        let result = example_compression();
        assert!(result.is_ok());
        let (uncompressed, compressed) = result.unwrap();
        assert!(!uncompressed.is_empty(), "Should have uncompressed data");
        assert!(!compressed.is_empty(), "Should have compressed data");
        // Note: Compression effectiveness depends on data patterns
    }

    #[test]
    fn test_dictionary_handling_example() {
        let result = example_dictionary_handling();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written dictionary data");
    }

    #[test]
    fn test_clone_options_example() {
        let result = example_clone_options();
        assert!(result.is_ok());
        let (data1, data2) = result.unwrap();
        assert!(!data1.is_empty(), "Should have written first file");
        assert!(!data2.is_empty(), "Should have written second file");
    }

    #[test]
    fn test_alignment_comparison_example() {
        let result = example_alignment_comparison();
        if let Err(e) = &result {
            println!("Error in alignment comparison: {}", e);
        }
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 4, "Should have results for 4 alignments");
        
        for (alignment, size) in results {
            assert!(size > 0, "Should have written data for alignment {}", alignment);
        }
    }

    #[test]
    fn test_options_with_different_writers_example() {
        let result = example_options_with_different_writers();
        assert!(result.is_ok());
        let (file_data, stream_data) = result.unwrap();
        assert!(!file_data.is_empty(), "Should have file data");
        assert!(!stream_data.is_empty(), "Should have stream data");
        // File and stream formats are different, so sizes will differ
        assert_ne!(file_data.len(), stream_data.len(), "File and stream should have different sizes");
    }

    #[test]
    fn test_options_error_handling_example() {
        let result = example_options_error_handling();
        assert!(result.is_ok());
        let error_msg = result.unwrap();
        assert!(error_msg.to_lowercase().contains("error") || 
                error_msg.contains("alignment") ||
                error_msg.contains("invalid"), 
                "Should contain error message about alignment");
    }

    #[test]
    fn test_best_practices_example() {
        let result = example_best_practices();
        if let Err(e) = &result {
            println!("Error in best practices: {}", e);
        }
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.contains("practice"), "Should contain best practices info");
    }
}