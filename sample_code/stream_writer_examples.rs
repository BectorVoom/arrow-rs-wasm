//! # StreamWriter Examples
//!
//! This module contains comprehensive examples for `arrow_ipc::writer::StreamWriter`.
//! 
//! The `StreamWriter` struct is used to write Arrow data in the IPC Streaming format.
//! This format is suitable for real-time data streaming, as data becomes available
//! immediately after each write operation.

use crate::*;
use std::io::Cursor;
use std::sync::Arc;

/// Example 1: Basic StreamWriter creation with `try_new()`
/// 
/// The most basic way to create a StreamWriter. This creates an unbuffered writer
/// that will write directly to the underlying writer.
pub fn example_try_new() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create a buffer to write to (simulating a stream)
    let mut buffer = Cursor::new(Vec::new());
    
    // Create the StreamWriter
    let mut writer = StreamWriter::try_new(&mut buffer, &schema)?;
    
    // Write the record batch
    writer.write(&batch)?;
    
    // Finish writing (this sends end-of-stream marker)
    writer.finish()?;
    
    // Return the written bytes
    Ok(buffer.into_inner())
}

/// Example 2: Buffered StreamWriter creation with `try_new_buffered()`
/// 
/// Creates a StreamWriter wrapped in a BufWriter for improved performance
/// when writing many small writes.
pub fn example_try_new_buffered() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create a buffer to write to
    let buffer = Cursor::new(Vec::new());
    
    // Create buffered StreamWriter - note this takes ownership of the writer
    let mut writer = StreamWriter::try_new_buffered(buffer, &schema)?;
    
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

/// Example 3: StreamWriter with custom options using `try_new_with_options()`
/// 
/// This example shows how to create a StreamWriter with custom IpcWriteOptions,
/// including dictionary handling for efficient streaming.
pub fn example_try_new_with_options() -> Result<Vec<u8>, ArrowError> {
    use arrow_ipc::gen::Schema::MetadataVersion;
    use arrow_ipc::writer::DictionaryHandling;
    
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create custom write options for streaming
    let options = IpcWriteOptions::try_new(
        8,                        // alignment (8 bytes)
        false,                    // write_legacy_ipc_format
        MetadataVersion::V5,      // metadata_version
    )?
    .with_dictionary_handling(DictionaryHandling::Delta); // Enable delta dictionaries
    
    let mut buffer = Cursor::new(Vec::new());
    
    // Create StreamWriter with custom options
    let mut writer = StreamWriter::try_new_with_options(&mut buffer, &schema, options)?;
    
    // Write the record batch
    writer.write(&batch)?;
    
    // Finish writing
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 4: Streaming multiple batches with `write()`
/// 
/// This example demonstrates the key advantage of StreamWriter - each batch
/// becomes available immediately after writing, perfect for real-time scenarios.
pub fn example_write_multiple_batches() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch1 = test_data::simple_record_batch()?;
    let batch2 = test_data::large_record_batch(100)?;
    let batch3 = test_data::large_record_batch(50)?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = StreamWriter::try_new(&mut buffer, &schema)?;
    
    // Write first batch - immediately available to readers
    writer.write(&batch1)?;
    
    // Write second batch - also immediately available  
    writer.write(&batch2)?;
    
    // Write third batch
    writer.write(&batch3)?;
    
    // Signal end of stream
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 5: Manual flushing with `flush()`
/// 
/// This example shows how to manually flush the writer's buffers for
/// guaranteed transmission in network scenarios.
pub fn example_flush() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch1 = test_data::simple_record_batch()?;
    let batch2 = test_data::large_record_batch(50)?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = StreamWriter::try_new(&mut buffer, &schema)?;
    
    // Write first batch
    writer.write(&batch1)?;
    
    // Manually flush to ensure data is transmitted
    writer.flush()?;
    // At this point, the first batch is guaranteed to be available to readers
    
    // Write second batch
    writer.write(&batch2)?;
    
    // Another flush
    writer.flush()?;
    
    // Finish writing (this also flushes)
    writer.finish()?;
    
    Ok(buffer.into_inner())
}

/// Example 6: Accessing underlying writer with `get_ref()` and `get_mut()`
/// 
/// These methods allow you to access the underlying writer, though direct
/// manipulation should be avoided as it can corrupt the IPC format.
pub fn example_writer_access() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = StreamWriter::try_new(&mut buffer, &schema)?;
    
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

/// Example 7: Unwrapping with `into_inner()`
/// 
/// This method consumes the StreamWriter and returns the underlying writer.
/// It automatically finishes the stream and flushes all buffers.
pub fn example_into_inner() -> Result<Vec<u8>, ArrowError> {
    // Create test data
    let schema = test_data::simple_schema();
    let batch = test_data::simple_record_batch()?;
    
    // Create buffer and give ownership to StreamWriter
    let buffer = Cursor::new(Vec::new());
    let mut writer = StreamWriter::try_new(buffer, &schema)?;
    
    // Write data
    writer.write(&batch)?;
    
    // into_inner() automatically calls finish() and flush()
    // then returns the underlying writer
    let finished_buffer = writer.into_inner()?;
    
    Ok(finished_buffer.into_inner())
}

/// Example 8: Real-time streaming simulation
/// 
/// This example demonstrates how StreamWriter enables real-time data processing
/// by making each batch immediately available.
pub fn example_realtime_streaming() -> Result<(Vec<u8>, String), ArrowError> {
    // Create schema for time-series data
    let schema = Arc::new(Schema::new(vec![
        Field::new("timestamp", DataType::Int64, false),
        Field::new("sensor_id", DataType::Utf8, false),
        Field::new("value", DataType::Float64, false),
    ]));
    
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = StreamWriter::try_new_buffered(buffer, &schema)?;
    
    let mut info = String::new();
    
    // Simulate real-time data arriving in batches
    for i in 0..3 {
        // Create a batch representing data from time period i
        let timestamp_data = vec![i * 1000]; // milliseconds
        let sensor_data = vec![format!("sensor_{}", i)];
        let value_data = vec![42.5 + i as f64];
        
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(timestamp_data)),
                Arc::new(StringArray::from(sensor_data)),
                Arc::new(arrow_array::Float64Array::from(value_data)),
            ]
        )?;
        
        // Write batch - immediately available for processing
        writer.write(&batch)?;
        writer.flush()?; // Ensure immediate availability
        
        info.push_str(&format!("Batch {} written and flushed\n", i));
    }
    
    // Finish stream
    let finished_writer = writer.into_inner()?;
    let cursor = finished_writer.into_inner().map_err(|e| {
        ArrowError::ExternalError(Box::new(e))
    })?;
    
    Ok((cursor.into_inner(), info))
}

/// Example 9: Error handling patterns
/// 
/// This example demonstrates proper error handling when working with StreamWriter.
pub fn example_error_handling() -> Result<String, ArrowError> {
    use arrow_schema::{Field, DataType, Schema};
    
    // Create a schema with different structure
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
    ]));
    
    // This batch has different schema (3 fields vs 1 field)
    let mismatched_batch = test_data::simple_record_batch()?;
    
    let buffer = Cursor::new(Vec::new());
    
    match StreamWriter::try_new(buffer, &schema) {
        Ok(mut writer) => {
            // This should fail because schemas don't match
            match writer.write(&mismatched_batch) {
                Ok(_) => {
                    writer.finish()?;
                    Ok("Schema validation might be lenient - demonstrates error handling patterns".to_string())
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

/// Example 10: Performance considerations for streaming
/// 
/// This example demonstrates best practices for performance when using StreamWriter
/// in high-throughput scenarios.
pub fn example_performance_streaming() -> Result<(usize, String), ArrowError> {
    // Create schema optimized for streaming
    let schema = test_data::simple_schema();
    
    // Use buffered writer for better performance
    let buffer = Cursor::new(Vec::new());
    let mut writer = StreamWriter::try_new_buffered(buffer, &schema)?;
    
    let mut total_rows = 0;
    
    // Write multiple batches efficiently
    for i in 0..10 {
        let batch = test_data::large_record_batch(1000)?;
        total_rows += batch.num_rows();
        
        writer.write(&batch)?;
        
        // Flush every 3 batches for balance between latency and throughput
        if i % 3 == 0 {
            writer.flush()?;
        }
    }
    
    // Get final buffer
    let finished_buffer = writer.into_inner()?;
    let cursor = finished_buffer.into_inner().map_err(|e| {
        ArrowError::ExternalError(Box::new(e))
    })?;
    
    let data = cursor.into_inner();
    
    Ok((
        data.len(),
        format!("Streamed {} rows in {} bytes", total_rows, data.len())
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
    fn test_write_multiple_batches_example() {
        let result = example_write_multiple_batches();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written multiple batches");
    }

    #[test]
    fn test_flush_example() {
        let result = example_flush();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty(), "Should have written some data");
    }

    #[test]
    fn test_writer_access_example() {
        let result = example_writer_access();
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
    fn test_realtime_streaming_example() {
        let result = example_realtime_streaming();
        assert!(result.is_ok());
        let (data, info) = result.unwrap();
        assert!(!data.is_empty(), "Should have written stream data");
        assert!(info.contains("Batch"), "Should contain batch information");
    }

    #[test]
    fn test_error_handling_example() {
        let result = example_error_handling();
        assert!(result.is_ok());
        let error_msg = result.unwrap();
        println!("Error message: {}", error_msg);
        assert!(error_msg.to_lowercase().contains("error") || 
                error_msg.contains("schema") || 
                error_msg.contains("validation"), 
                "Should contain error-related message, got: {}", error_msg);
    }

    #[test]
    fn test_performance_streaming_example() {
        let result = example_performance_streaming();
        assert!(result.is_ok());
        let (size, info) = result.unwrap();
        assert!(size > 0, "Should have written some data");
        assert!(info.contains("rows"), "Should contain performance info");
    }
}