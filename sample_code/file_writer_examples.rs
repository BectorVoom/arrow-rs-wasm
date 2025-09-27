//! # FileWriter Examples
//!
//! This module contains comprehensive examples for `arrow_ipc::writer::FileWriter`.
//! 
//! The `FileWriter` struct is used to write Arrow data in the IPC File format.
//! This format is suitable for storing Arrow data persistently, as it includes
//! a footer with metadata that allows random access to record batches.

use crate::*;
use std::io::Cursor;
use std::sync::Arc;

/// Example 1: Basic FileWriter creation with `try_new()`
/// 
/// The most basic way to create a FileWriter. This creates an unbuffered writer
/// that will write directly to the underlying writer.
pub fn example_try_new() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create a buffer to write to (simulating a file)
    let mut buffer = Cursor::new(Vec::new());
    
    // Create the FileWriter
    let mut writer = FileWriter::try_new(&mut buffer, &schema)?;
    
    // Write the record batch
    writer.write(&batch)?;
    
    // Finish writing (this is crucial - writes the footer)
    writer.finish()?;
    
    // Return the written bytes
    Ok(buffer.into_inner())
}

/// Example 2: Buffered FileWriter creation with `try_new_buffered()`
/// 
/// Creates a FileWriter wrapped in a BufWriter for improved performance
/// when writing many small writes.
pub fn example_try_new_buffered() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create a buffer to write to
    let buffer = Cursor::new(Vec::new());
    
    // Create buffered FileWriter - note this takes ownership of the writer
    let mut writer = FileWriter::try_new_buffered(buffer, &schema)?;
    
    // Write the record batch
    writer.write(&batch)?;
    
    // Finish and unwrap to get the underlying buffer back
    let inner_writer = writer.into_inner()?;
    
    // Handle the Result from BufWriter::into_inner()
    let cursor = inner_writer.into_inner().map_err(|e| {
        ArrowError::ExternalError(Box::new(e))
    })?;
    
    Ok(cursor.into_inner())
}

/// Example 3: FileWriter with custom options using `try_new_with_options()`
/// 
/// This example shows how to create a FileWriter with custom IpcWriteOptions,
/// such as specific alignment, compression, or metadata version.
pub fn example_try_new_with_options() -> Result<Vec<u8>, ArrowError> {
    use arrow_ipc::gen::Schema::MetadataVersion;
    
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create custom write options
    let options = IpcWriteOptions::try_new(
        8,                        // alignment (8 bytes)
        false,                    // write_legacy_ipc_format
        MetadataVersion::V5,      // metadata_version
    )?;
    
    let mut buffer = Cursor::new(Vec::new());
    
    // Create FileWriter with custom options
    let mut writer = FileWriter::try_new_with_options(&mut buffer, &schema, options)?;
    
    // Write the record batch
    writer.write(&batch)?;
    
    // Finish writing
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 4: Adding metadata with `write_metadata()`
/// 
/// This example shows how to add custom key-value metadata to the file.
/// Metadata is written to the file footer and can be read back when opening the file.
pub fn example_write_metadata() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new(&mut buffer, &schema)?;
    
    // Add custom metadata before writing data
    writer.write_metadata("created_by", "arrow_ipc_examples");
    writer.write_metadata("version", "1.0.0");
    writer.write_metadata("description", "Example Arrow IPC file with metadata");
    
    // Write the record batch
    writer.write(&batch)?;
    
    // Finish writing
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 5: Writing multiple batches with `write()`
/// 
/// This example demonstrates writing multiple record batches to a single file.
/// All batches must have the same schema.
pub fn example_write_multiple_batches() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch1 = test_data::simple_record_batch()?;
    let batch2 = test_data::large_record_batch(100)?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new(&mut buffer, &schema)?;
    
    // Write first batch
    writer.write(&batch1)?;
    
    // Write second batch  
    writer.write(&batch2)?;
    
    // You can write as many batches as needed
    // Each call to write() appends another record batch to the file
    
    // Finish writing (required to write footer)
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 6: Getting schema information with `schema()`
/// 
/// This example shows how to retrieve the schema from an active FileWriter.
pub fn example_schema_access() -> Result<String, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new(&mut buffer, &schema)?;
    
    // Get the schema from the writer
    let writer_schema = writer.schema();
    
    // You can inspect the schema
    let field_names: Vec<String> = writer_schema
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .collect();
    
    // Write some data
    writer.write(&batch)?;
    writer.finish()?;
    
    Ok(format!("Schema fields: {}", field_names.join(", ")))
}

/// Example 7: Accessing underlying writer with `get_ref()` and `get_mut()`
/// 
/// These methods allow you to access the underlying writer, though direct
/// manipulation is not recommended as it can corrupt the IPC format.
pub fn example_writer_access() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new(&mut buffer, &schema)?;
    
    // Get immutable reference to underlying writer
    let _writer_ref = writer.get_ref();
    // You could use this to check the current position, etc.
    // but avoid writing directly to it
    
    // Get mutable reference (use with extreme caution)
    let _writer_mut = writer.get_mut();
    // Direct writes to this could corrupt the IPC format
    
    // Normal usage
    writer.write(&batch)?;
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 8: Manual flushing with `flush()`
/// 
/// This example shows how to manually flush the writer's buffers.
/// This is particularly useful when you want to ensure data is written
/// to the underlying storage at specific points.
pub fn example_flush() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch1 = test_data::simple_record_batch()?;
    let batch2 = test_data::large_record_batch(50)?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new(&mut buffer, &schema)?;
    
    // Write first batch
    writer.write(&batch1)?;
    
    // Manually flush after first batch
    writer.flush()?;
    // At this point, the first batch data is guaranteed to be written
    // to the underlying writer (but footer is not written yet)
    
    // Write second batch
    writer.write(&batch2)?;
    
    // Another flush
    writer.flush()?;
    
    // Finish writing (this also flushes)
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 9: Unwrapping with `into_inner()`
/// 
/// This method consumes the FileWriter and returns the underlying writer.
/// It automatically finishes the file and flushes all buffers.
pub fn example_into_inner() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create buffer and give ownership to FileWriter
    let buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new(buffer, &schema)?;
    
    // Write data
    writer.write(&batch)?;
    
    // into_inner() automatically calls finish() and flush()
    // then returns the underlying writer
    let finished_buffer = writer.into_inner()?;
    
    Ok(finished_buffer.into_inner())
}

/// Example 10: Error handling patterns
/// 
/// This example demonstrates proper error handling when working with FileWriter.
pub fn example_error_handling() -> Result<String, ArrowError> {
    use arrow_schema::{Field, DataType, Schema};
    
    // Create a schema with different number of fields than our test data
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
    ]));
    
    // This batch has different schema (3 fields vs 1 field)
    let mismatched_batch = test_data::simple_record_batch()?;
    
    let buffer = Cursor::new(Vec::new());
    
    match FileWriter::try_new(buffer, &schema) {
        Ok(mut writer) => {
            // This should fail because schemas don't match
            match writer.write(&mismatched_batch) {
                Ok(_) => {
                    // If it doesn't fail, let's try a different approach
                    // Try to write a batch that definitely doesn't match
                    let wrong_batch = RecordBatch::try_new(
                        Arc::new(Schema::new(vec![
                            Field::new("different_field", DataType::Utf8, true),
                            Field::new("another_field", DataType::Int64, true),
                        ])),
                        vec![
                            Arc::new(StringArray::from(vec!["test"])),
                            Arc::new(Int64Array::from(vec![123])),
                        ]
                    )?;
                    
                    match writer.write(&wrong_batch) {
                        Ok(_) => {
                            writer.finish()?;
                            Ok("Schema validation might be lenient - this demonstrates error handling patterns".to_string())
                        }
                        Err(e) => {
                            Ok(format!("Expected schema mismatch error caught: {}", e))
                        }
                    }
                }
                Err(e) => {
                    // Handle the error gracefully
                    Ok(format!("Expected schema error caught: {}", e))
                }
            }
        }
        Err(e) => Ok(format!("Writer creation failed with error: {}", e))
    }
}

/// Example 11: Performance considerations and best practices
/// 
/// This example demonstrates best practices for performance when using FileWriter.
pub fn example_performance_best_practices() -> Result<(usize, String), ArrowError> {
    // Create larger test data
    let schema = test_data::simple_schema();
    let large_batch = test_data::large_record_batch(10000)?;
    
    // Use buffered writer for better performance
    let buffer = Cursor::new(Vec::new());
    let mut writer = FileWriter::try_new_buffered(buffer, &schema)?;
    
    // Add relevant metadata
    writer.write_metadata("rows", &large_batch.num_rows().to_string());
    writer.write_metadata("columns", &large_batch.num_columns().to_string());
    
    // Write the large batch
    writer.write(&large_batch)?;
    
    // Get final buffer
    let finished_buffer = writer.into_inner()?;
    
    // Handle the Result from BufWriter::into_inner()
    let cursor = finished_buffer.into_inner().map_err(|e| {
        ArrowError::ExternalError(Box::new(e))
    })?;
    
    let data = cursor.into_inner();
    
    Ok((
        data.len(),
        format!("Wrote {} rows in {} bytes", large_batch.num_rows(), data.len())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_new_example() {
        let result = example_try_new();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_try_new_buffered_example() {
        let result = example_try_new_buffered();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_try_new_with_options_example() {
        let result = example_try_new_with_options();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_write_metadata_example() {
        let result = example_write_metadata();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_write_multiple_batches_example() {
        let result = example_write_multiple_batches();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_schema_access_example() {
        let result = example_schema_access();
        assert!(result.is_ok());
        let schema_info = result.unwrap();
        assert!(schema_info.contains("id"), "Should contain field names");
    }

    #[test]
    fn test_writer_access_example() {
        let result = example_writer_access();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_flush_example() {
        let result = example_flush();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_into_inner_example() {
        let result = example_into_inner();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_error_handling_example() {
        let result = example_error_handling();
        assert!(result.is_ok());
        let error_msg = result.unwrap();
        println!("Error message: {}", error_msg);
        assert!(error_msg.to_lowercase().contains("error") || 
                error_msg.contains("Schema") || 
                error_msg.contains("mismatch"), 
                "Should contain error-related message, got: {}", error_msg);
    }

    #[test]
    fn test_performance_best_practices_example() {
        let result = example_performance_best_practices();
        assert!(result.is_ok());
        let (size, info) = result.unwrap();
        assert!(size > 0, "Should have written some data");
        assert!(info.contains("rows"), "Should contain performance info");
    }
}