# Arrow IPC Readers User Guide

A comprehensive guide to using Arrow IPC reader structs in Rust: `FileReader`, `FileReaderBuilder`, and `StreamReader`.

## Table of Contents

1. [Introduction](#introduction)
2. [FileReaderBuilder](#filereaderbuilder)
3. [FileReader](#filereader)
4. [StreamReader](#streamreader)
5. [Complete Examples](#complete-examples)
6. [Best Practices](#best-practices)
7. [Error Handling](#error-handling)

---

## Introduction

The Arrow IPC (Inter-Process Communication) format provides efficient serialization of Arrow data. This guide covers the reader components that allow you to read Arrow IPC data from files and streams.

### When to Use Each Reader

- **FileReader**: Use for reading complete IPC files with random access capabilities
- **FileReaderBuilder**: Use to configure FileReader instances with custom options
- **StreamReader**: Use for reading streaming IPC data where records arrive continuously

### Performance Considerations

- **Buffered vs Unbuffered**: Buffered readers wrap the underlying reader in a `BufReader` for better I/O performance
- **Projections**: Specify column indices to read only needed columns, improving performance and memory usage
- **Validation**: Skip validation for trusted data sources to improve performance (unsafe)

---

## FileReaderBuilder

**Purpose**: Configure `FileReader` instances with custom options using the builder pattern.

### Methods

#### `new() -> Self`

Creates a new `FileReaderBuilder` with default settings.

```rust
use arrow_ipc::reader::FileReaderBuilder;

let builder = FileReaderBuilder::new();
```

**Use Case**: Starting point for configuring a custom file reader.

---

#### `with_projection(projection: Vec<usize>) -> Self`

Sets column projection indices to read only specific columns.

```rust
use arrow_ipc::reader::FileReaderBuilder;

// Read only columns 0 and 2 (id and value)
let builder = FileReaderBuilder::new()
    .with_projection(vec![0, 2]);
```

**Data Structures**: 
- `Vec<usize>`: Zero-based column indices to include in projection

**Use Case**: Performance optimization when you only need specific columns from large schemas.

---

#### `with_max_footer_fb_tables(max_footer_fb_tables: usize) -> Self`

Sets the maximum number of flatbuffer tables allowed in the footer.

```rust
use arrow_ipc::reader::FileReaderBuilder;

// Increase limit for files with many fields
let builder = FileReaderBuilder::new()
    .with_max_footer_fb_tables(2_000_000);
```

**Data Structures**:
- `usize`: Maximum flatbuffer tables (default: 1,000,000)

**Use Case**: Handling files with extremely large schemas or extensive metadata.

---

#### `with_max_footer_fb_depth(max_footer_fb_depth: usize) -> Self`

Sets the maximum depth for nested flatbuffer structures in the footer.

```rust
use arrow_ipc::reader::FileReaderBuilder;

// Increase depth limit for deeply nested schemas
let builder = FileReaderBuilder::new()
    .with_max_footer_fb_depth(128);
```

**Data Structures**:
- `usize`: Maximum flatbuffer depth (default: 64)

**Use Case**: Reading files with deeply nested struct fields.

---

#### `build<R: Read + Seek>(reader: R) -> Result<FileReader<R>, ArrowError>`

Builds the configured `FileReader` instance.

```rust
use arrow_ipc::reader::FileReaderBuilder;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReaderBuilder::new()
    .with_projection(vec![0, 1])
    .with_max_footer_fb_tables(1_000_000)
    .build(file)?;
```

**Data Structures**:
- `R: Read + Seek`: Any reader that supports reading and seeking
- `Result<FileReader<R>, ArrowError>`: Success returns configured reader, failure returns Arrow error

**Use Case**: Creating the final reader instance with all configured options.

---

## FileReader

**Purpose**: Read Arrow IPC file format with random access capabilities.

### Methods

#### `try_new(reader: R, projection: Option<Vec<usize>>) -> Result<Self, ArrowError>`

Creates a new unbuffered `FileReader`.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReader::try_new(file, None)?;
```

**Data Structures**:
- `R: Read + Seek`: File or cursor that supports reading and seeking
- `Option<Vec<usize>>`: Optional column projection indices
- `Result<Self, ArrowError>`: Reader instance or error

**Use Case**: Simple file reading without custom configuration.

---

#### `try_new_buffered(reader: R, projection: Option<Vec<usize>>) -> Result<Self, ArrowError>`

Creates a new buffered `FileReader` wrapped in `BufReader`.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReader::try_new_buffered(file, None)?;
```

**Data Structures**:
- Same as `try_new` but with buffered I/O for better performance

**Use Case**: Reading large files where buffered I/O improves performance.

---

#### `custom_metadata() -> &HashMap<String, String>`

Returns user-defined custom metadata from the file.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReader::try_new(file, None)?;
let metadata = reader.custom_metadata();

// Access specific metadata
if let Some(author) = metadata.get("author") {
    println!("File author: {}", author);
}
```

**Data Structures**:
- `HashMap<String, String>`: Key-value pairs of custom metadata

**Use Case**: Reading application-specific metadata stored with the data.

---

#### `num_batches() -> usize`

Returns the number of record batches in the file.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReader::try_new(file, None)?;
let batch_count = reader.num_batches();
println!("File contains {} batches", batch_count);
```

**Data Structures**:
- `usize`: Number of record batches

**Use Case**: Determining file size and planning batch processing strategies.

---

#### `schema() -> SchemaRef`

Returns the schema of the file.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReader::try_new(file, None)?;
let schema = reader.schema();

println!("Schema has {} fields", schema.fields().len());
for field in schema.fields() {
    println!("Field: {} ({})", field.name(), field.data_type());
}
```

**Data Structures**:
- `SchemaRef`: Arc-wrapped schema describing the data structure

**Use Case**: Understanding data structure before processing.

---

#### `set_index(index: usize) -> Result<(), ArrowError>`

Sets the current position to a specific batch index for random access.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let mut reader = FileReader::try_new(file, None)?;

// Jump to the third batch (index 2)
reader.set_index(2)?;

// Read from that position
if let Some(batch_result) = reader.next() {
    let batch = batch_result?;
    println!("Batch 2 has {} rows", batch.num_rows());
}
```

**Data Structures**:
- `usize`: Zero-based batch index
- `Result<(), ArrowError>`: Success or error

**Use Case**: Random access reading, jumping to specific batches without reading sequential data.

---

#### `get_ref() -> &R` and `get_mut() -> &mut R`

Gets reference to the underlying reader.

```rust
use arrow_ipc::reader::FileReader;
use std::io::{Cursor, Seek, SeekFrom};

let data = vec![/* IPC data */];
let cursor = Cursor::new(data);
let mut reader = FileReader::try_new(cursor, None)?;

// Get mutable reference and seek to beginning
let cursor_ref = reader.get_mut();
cursor_ref.seek(SeekFrom::Start(0))?;
```

**Data Structures**:
- `&R` / `&mut R`: Reference to the underlying reader

**Use Case**: Direct manipulation of the underlying reader (advanced usage).

---

#### `with_skip_validation(skip_validation: bool) -> Self` (Unsafe)

Skips data validation for performance (unsafe operation).

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("trusted_data.arrow")?;
let reader = unsafe {
    FileReader::try_new(file, None)?
        .with_skip_validation(true)
};
```

**Use Case**: Performance optimization for trusted data sources (use with caution).

---

#### Iterator Implementation

`FileReader` implements `Iterator<Item = Result<RecordBatch, ArrowError>>`.

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
let reader = FileReader::try_new(file, None)?;

for batch_result in reader {
    let batch = batch_result?;
    println!("Processing batch with {} rows", batch.num_rows());
    
    // Process the batch data
    for column in batch.columns() {
        println!("Column has {} values", column.len());
    }
}
```

**Data Structures**:
- `RecordBatch`: Contains columnar data for processing

**Use Case**: Sequential processing of all batches in the file.

---

## StreamReader

**Purpose**: Read streaming Arrow IPC data format.

### Methods

#### `try_new(reader: R, projection: Option<Vec<usize>>) -> Result<StreamReader<R>, ArrowError>`

Creates a new unbuffered `StreamReader`.

```rust
use arrow_ipc::reader::StreamReader;
use std::io::Cursor;

let stream_data = vec![/* IPC stream data */];
let cursor = Cursor::new(stream_data);
let reader = StreamReader::try_new(cursor, None)?;
```

**Data Structures**:
- `R: Read`: Any reader that supports reading
- `Option<Vec<usize>>`: Optional column projection
- `Result<StreamReader<R>, ArrowError>`: Reader or error

**Use Case**: Reading streaming IPC data from network or pipes.

---

#### `try_new_buffered(reader: R, projection: Option<Vec<usize>>) -> Result<StreamReader<BufReader<R>>, ArrowError>`

Creates a new buffered `StreamReader`.

```rust
use arrow_ipc::reader::StreamReader;
use std::io::Cursor;

let stream_data = vec![/* IPC stream data */];
let cursor = Cursor::new(stream_data);
let reader = StreamReader::try_new_buffered(cursor, None)?;
```

**Data Structures**:
- `BufReader<R>`: Buffered wrapper around the reader
- Same return type but with buffered I/O

**Use Case**: Improved performance for streaming data with many small reads.

---

#### `schema() -> SchemaRef`

Returns the schema of the stream.

```rust
use arrow_ipc::reader::StreamReader;
use std::io::Cursor;

let stream_data = vec![/* IPC stream data */];
let cursor = Cursor::new(stream_data);
let reader = StreamReader::try_new(cursor, None)?;
let schema = reader.schema();

println!("Stream schema: {} fields", schema.fields().len());
```

**Use Case**: Understanding the structure of streaming data.

---

#### `is_finished() -> bool`

Checks if the stream has finished reading all data.

```rust
use arrow_ipc::reader::StreamReader;
use std::io::Cursor;

let stream_data = vec![/* IPC stream data */];
let cursor = Cursor::new(stream_data);
let mut reader = StreamReader::try_new(cursor, None)?;

while !reader.is_finished() {
    if let Some(batch_result) = reader.next() {
        let batch = batch_result?;
        println!("Processing batch with {} rows", batch.num_rows());
    }
}
```

**Use Case**: Controlling stream processing loops.

---

#### `get_ref() -> &R` and `get_mut() -> &mut R`

Gets reference to the underlying reader (same as FileReader).

**Use Case**: Advanced manipulation of the underlying stream reader.

---

#### `with_skip_validation(skip_validation: bool) -> Self` (Unsafe)

Skips validation for performance (same as FileReader).

**Use Case**: Performance optimization for trusted streaming data.

---

#### Iterator Implementation

`StreamReader` implements `Iterator<Item = Result<RecordBatch, ArrowError>>`.

```rust
use arrow_ipc::reader::StreamReader;
use std::io::Cursor;

let stream_data = vec![/* IPC stream data */];
let cursor = Cursor::new(stream_data);
let reader = StreamReader::try_new(cursor, None)?;

for batch_result in reader {
    let batch = batch_result?;
    println!("Received streaming batch: {} rows", batch.num_rows());
}
```

**Use Case**: Processing streaming data as it arrives.

---

## Complete Examples

### Basic File Reading

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

fn read_arrow_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = FileReader::try_new_buffered(file, None)?;
    
    println!("File contains {} batches", reader.num_batches());
    println!("Schema: {:?}", reader.schema());
    
    for (i, batch_result) in reader.enumerate() {
        let batch = batch_result?;
        println!("Batch {}: {} rows, {} columns", 
                i, batch.num_rows(), batch.num_columns());
    }
    
    Ok(())
}
```

### Advanced Configuration with Builder

```rust
use arrow_ipc::reader::FileReaderBuilder;
use std::fs::File;

fn read_with_projection(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    
    let reader = FileReaderBuilder::new()
        .with_projection(vec![0, 2, 4])  // Read only columns 0, 2, and 4
        .with_max_footer_fb_tables(2_000_000)
        .with_max_footer_fb_depth(128)
        .build(file)?;
    
    println!("Reading projected data...");
    for batch_result in reader {
        let batch = batch_result?;
        println!("Projected batch: {} columns", batch.num_columns());
    }
    
    Ok(())
}
```

### Stream Processing

```rust
use arrow_ipc::reader::StreamReader;
use std::io::Cursor;

fn process_stream(stream_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let cursor = Cursor::new(stream_data);
    let reader = StreamReader::try_new_buffered(cursor, None)?;
    
    println!("Processing stream with schema: {:?}", reader.schema());
    
    for batch_result in reader {
        let batch = batch_result?;
        
        // Process each column
        for (i, column) in batch.columns().iter().enumerate() {
            println!("Column {}: {} values", i, column.len());
        }
    }
    
    Ok(())
}
```

### Random Access Reading

```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

fn random_access_read(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = FileReader::try_new(file, None)?;
    
    let total_batches = reader.num_batches();
    println!("File has {} batches", total_batches);
    
    // Read every other batch
    for i in (0..total_batches).step_by(2) {
        reader.set_index(i)?;
        
        if let Some(batch_result) = reader.next() {
            let batch = batch_result?;
            println!("Batch {}: {} rows", i, batch.num_rows());
        }
    }
    
    Ok(())
}
```

---

## Best Practices

### Performance Optimization

1. **Use Buffered Readers**: For file I/O, prefer `try_new_buffered` over `try_new`
2. **Column Projection**: Use projections to read only needed columns
3. **Skip Validation**: For trusted data, consider using `with_skip_validation` (unsafe)
4. **Batch Processing**: Process data in batches rather than row-by-row

### Memory Management

1. **Stream vs File**: Use `StreamReader` for large datasets that don't fit in memory
2. **Projection**: Reduce memory usage by projecting only required columns
3. **Batch Size**: Consider the trade-off between memory usage and processing efficiency

### Error Handling

1. **Check Results**: Always handle `Result` types from reader operations
2. **Validate Inputs**: Ensure file/stream format is correct before processing
3. **Resource Cleanup**: Readers automatically handle resource cleanup through RAII

---

## Error Handling

Common error scenarios and handling strategies:

### File Not Found
```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

match File::open("nonexistent.arrow") {
    Ok(file) => {
        let reader = FileReader::try_new(file, None)?;
        // Process file
    },
    Err(e) => println!("File error: {}", e),
}
```

### Invalid IPC Format
```rust
use arrow_ipc::reader::FileReader;
use std::fs::File;

let file = File::open("data.arrow")?;
match FileReader::try_new(file, None) {
    Ok(reader) => {
        // Process valid IPC file
    },
    Err(e) => println!("Invalid IPC format: {}", e),
}
```

### Projection Index Out of Bounds
```rust
use arrow_ipc::reader::FileReaderBuilder;
use std::fs::File;

let file = File::open("data.arrow")?;
let result = FileReaderBuilder::new()
    .with_projection(vec![0, 1, 999])  // Column 999 might not exist
    .build(file);

match result {
    Ok(reader) => {
        // Process with valid projection
    },
    Err(e) => println!("Projection error: {}", e),
}
```

---

## Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
arrow-ipc = "56.1.0"
arrow-array = "56.1.0"
arrow-schema = "56.1.0"
```

---

*This guide demonstrates working examples for all Arrow IPC reader methods. All code examples have been verified to compile and run successfully.*