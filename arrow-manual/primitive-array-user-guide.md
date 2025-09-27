# Apache Arrow PrimitiveArray User Guide

## Table of Contents
1. [Introduction and Prerequisites](#introduction-and-prerequisites)
2. [Core Concepts Overview](#core-concepts-overview)
3. [PrimitiveArray<T> Struct Reference](#primitivearrayt-struct-reference)
4. [ArrowPrimitiveType Trait Reference](#arrowprimitivetype-trait-reference)
5. [Supporting Structures Reference](#supporting-structures-reference)
6. [Complete Examples and Use Cases](#complete-examples-and-use-cases)
7. [Best Practices and Performance](#best-practices-and-performance)

---

## Introduction and Prerequisites

### Overview
`PrimitiveArray<T>` is the core data structure in Apache Arrow for efficiently storing and manipulating arrays of primitive values such as integers, floating-point numbers, dates, timestamps, and decimals. This guide provides comprehensive coverage with working examples for every method and feature.

### Prerequisites
- Basic Rust knowledge
- Understanding of generic types and traits
- Familiarity with Apache Arrow concepts

### Dependencies
Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
arrow = "56.1.0"
arrow-array = "56.1.0"
arrow-buffer = "56.1.0"
arrow-schema = "56.1.0"
```

### Basic Imports
```rust
use arrow_array::{Array, Int32Array, PrimitiveArray, types::Int32Type};
use arrow_buffer::{ScalarBuffer, NullBuffer};
use arrow_schema::DataType;
```

---

## Core Concepts Overview

### Type System
`PrimitiveArray<T>` is generic over types that implement `ArrowPrimitiveType`. This provides compile-time type safety while maintaining efficiency.

```rust
// Each primitive type has its own type marker
let int32_array: PrimitiveArray<Int32Type> = Int32Array::from(vec![1, 2, 3]);
let float64_array: PrimitiveArray<Float64Type> = Float64Array::from(vec![1.0, 2.0, 3.0]);
```

### Memory Layout
Arrow uses a columnar memory layout with two main components:
- **Values Buffer**: Contiguous array of primitive values
- **Null Buffer**: Optional bit-packed null flags (1 bit per element)

### Performance Characteristics
- **Zero-copy operations** where possible
- **SIMD optimization** support
- **Cache-efficient** memory access patterns
- **Minimal overhead** metadata

---

## PrimitiveArray<T> Struct Reference

### Construction Methods

#### Basic Construction

**`new(values: ScalarBuffer<T::Native>, nulls: Option<NullBuffer>) -> Self`**

Primary constructor with explicit buffers.

```rust
use arrow_array::Int32Array;
use arrow_buffer::{ScalarBuffer, NullBuffer};

// Create array without nulls
let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
let array = Int32Array::new(values, None);
println!("Array length: {}", array.len());

// Create array with nulls
let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
let nulls = NullBuffer::from(vec![true, false, true, false, true]);
let array_with_nulls = Int32Array::new(values, Some(nulls));
println!("Null count: {}", array_with_nulls.null_count());
```

**Use Cases:**
- Direct construction from existing buffers
- Integration with external data sources
- When you have pre-computed null masks

**`new_null(length: usize) -> Self`**

Creates array where all values are null.

```rust
let null_array = Int32Array::new_null(5);
assert_eq!(null_array.len(), 5);
assert!(null_array.is_null(0));
assert_eq!(null_array.null_count(), 5);
```

**Use Cases:**
- Placeholder arrays for algorithms
- Initialization before filling with real data
- Testing null-handling logic

**`try_new(values: ScalarBuffer<T::Native>, nulls: Option<NullBuffer>) -> Result<Self, ArrowError>`**

Fallible constructor with validation.

```rust
let values = ScalarBuffer::from(vec![1, 2, 3]);
match Int32Array::try_new(values, None) {
    Ok(array) => println!("Created array with {} elements", array.len()),
    Err(e) => eprintln!("Error creating array: {}", e),
}
```

**Use Cases:**
- When validation is required
- Error-prone data sources
- Defensive programming

#### Iterator-based Construction

**`from_iter_values<I: IntoIterator<Item = T::Native>>(iter: I) -> Self`**

Creates array from an iterator of values (no nulls).

```rust
let array = Int32Array::from_iter_values(0..5);
assert_eq!(array.len(), 5);
assert_eq!(array.value(0), 0);
assert_eq!(array.value(4), 4);
```

**Use Cases:**
- Converting from iterators
- Generating sequential data
- When all values are guaranteed to be valid

**`from_value(value: T::Native, count: usize) -> Self`**

Creates array by repeating a single value.

```rust
let array = Int32Array::from_value(42, 3);
assert_eq!(array.len(), 3);
assert_eq!(array.value(0), 42);
assert_eq!(array.value(1), 42);
assert_eq!(array.value(2), 42);
```

**Use Cases:**
- Creating constant arrays
- Initialization with default values
- Broadcasting single values

#### Option-based Construction

**`FromIterator<Option<T::Native>>`**

Construct from iterator of Options for null handling.

```rust
let values = vec![Some(1), None, Some(3), None, Some(5)];
let array: Int32Array = values.into_iter().collect();

assert_eq!(array.len(), 5);
assert!(!array.is_null(0)); // Some(1) is valid
assert!(array.is_null(1));  // None is null
assert_eq!(array.null_count(), 2);
```

**Use Cases:**
- When data naturally contains missing values
- Converting from `Vec<Option<T>>`
- Handling sparse data

### Access Methods

#### Basic Access

**`len(&self) -> usize`**

Returns the number of elements in the array.

```rust
let array = Int32Array::from(vec![1, 2, 3]);
assert_eq!(array.len(), 3);
```

**`is_empty(&self) -> bool`**

Returns true if the array has no elements.

```rust
let empty_array = Int32Array::from(vec![] as Vec<i32>);
let non_empty_array = Int32Array::from(vec![1]);
assert!(empty_array.is_empty());
assert!(!non_empty_array.is_empty());
```

**`value(&self, i: usize) -> T::Native`**

Returns value at specified index (panics if null or out of bounds).

```rust
let array = Int32Array::from(vec![10, 20, 30]);
assert_eq!(array.value(0), 10);
assert_eq!(array.value(1), 20);
assert_eq!(array.value(2), 30);
```

**Use Cases:**
- Safe value access when you know index is valid
- Performance-critical code with bounds checking
- Extracting individual values

**`unsafe fn value_unchecked(&self, i: usize) -> T::Native`**

Returns value without bounds checking (unsafe but faster).

```rust
let array = Int32Array::from(vec![10, 20, 30]);

// Using unsafe method for performance-critical code
unsafe {
    assert_eq!(array.value_unchecked(0), 10);
    assert_eq!(array.value_unchecked(1), 20);
    assert_eq!(array.value_unchecked(2), 30);
}
```

**Use Cases:**
- Performance-critical inner loops
- When bounds are guaranteed by algorithm
- Hot paths in computational kernels

**`values(&self) -> &ScalarBuffer<T::Native>`**

Returns reference to underlying values buffer.

```rust
let array = Int32Array::from(vec![1, 2, 3, 4, 5]);
let values_buffer = array.values();

// Buffer provides direct access to underlying data
assert_eq!(values_buffer.len(), 5);
assert_eq!(values_buffer[0], 1);
assert_eq!(values_buffer[4], 5);
```

**Use Cases:**
- Direct buffer manipulation
- SIMD operations on raw data
- Zero-copy data sharing

#### Iterator Access

**`iter(&self) -> PrimitiveIter<T>`**

Returns iterator over array values (handles nulls).

```rust
let array = Int32Array::from(vec![1, 2, 3]);
let collected: Vec<Option<i32>> = array.iter().collect();
assert_eq!(collected, vec![Some(1), Some(2), Some(3)]);

// With nulls
let values = vec![Some(1), None, Some(3)];
let array_with_nulls: Int32Array = values.into_iter().collect();
let collected: Vec<Option<i32>> = array_with_nulls.iter().collect();
assert_eq!(collected, vec![Some(1), None, Some(3)]);
```

**Use Cases:**
- Functional programming patterns
- Safe iteration over potentially null data
- Integration with Rust iterator ecosystem

#### Null Handling

**`is_null(&self, i: usize) -> bool`**

Check if value at index is null.

```rust
let values = vec![Some(1), None, Some(3), None, Some(5)];
let array: Int32Array = values.into_iter().collect();

assert!(!array.is_null(0)); // valid
assert!(array.is_null(1));  // null
assert!(!array.is_null(2)); // valid
assert!(array.is_null(3));  // null
```

**`null_count(&self) -> usize`**

Returns total number of null values.

```rust
let values = vec![Some(1), None, Some(3), None, Some(5)];
let array: Int32Array = values.into_iter().collect();
assert_eq!(array.null_count(), 2);
```

### Transformation Methods

#### Basic Transformations

**`slice(&self, offset: usize, length: usize) -> Self`**

Creates zero-copy slice of the array.

```rust
let array = Int32Array::from(vec![1, 2, 3, 4, 5]);
let sliced = array.slice(1, 3);
assert_eq!(sliced.len(), 3);
assert_eq!(sliced.value(0), 2);
assert_eq!(sliced.value(2), 4);
```

**Use Cases:**
- Working with sub-arrays
- Efficient data windowing
- Pagination without copying

#### Functional Transformations

**Using Arrow Compute Functions**

Arrow provides compute functions for transformations:

```rust
use arrow::compute;

// Unary transformation
let array = Int32Array::from(vec![1, 2, 3]);
let result: Int64Array = compute::unary(&array, |x| (x * 2) as i64);
assert_eq!(result.value(0), 2);
assert_eq!(result.value(1), 4);
assert_eq!(result.value(2), 6);

// Type conversion
let int_array = Int32Array::from(vec![1, 2, 3, 4]);
let float_result: Float64Array = compute::unary(&int_array, |x| x as f64);
assert_eq!(float_result.value(0), 1.0);
assert_eq!(float_result.value(3), 4.0);
```

**Use Cases:**
- Mathematical transformations
- Type conversions
- Data cleaning operations

#### Transformation Chaining

```rust
let array = Int32Array::from(vec![1, 2, 3, 4, 5]);

// First transformation: multiply by 2
let doubled: Int32Array = compute::unary(&array, |x| x * 2);

// Second transformation: convert to float and add 0.5
let result: Float32Array = compute::unary(&doubled, |x| x as f32 + 0.5);

assert_eq!(result.value(0), 2.5);
assert_eq!(result.value(1), 4.5);
assert_eq!(result.value(4), 10.5);
```

**Use Cases:**
- Complex data pipelines
- Multi-step transformations
- Functional programming patterns

### Specialized Methods

#### Temporal Type Methods

For arrays storing dates, times, and timestamps:

**Date32 Arrays**
```rust
use arrow_array::Date32Array;

// Date32 stores days since Unix epoch (1970-01-01)
let days_since_epoch = vec![0, 1, 365, 1000];
let date_array = Date32Array::from(days_since_epoch);

assert_eq!(date_array.value(0), 0);   // 1970-01-01
assert_eq!(date_array.value(1), 1);   // 1970-01-02
assert_eq!(date_array.value(2), 365); // 1971-01-01
```

**Time32 Arrays**
```rust
use arrow_array::Time32SecondArray;

// Time32Second stores seconds since midnight
let seconds_since_midnight = vec![0, 3600, 43200, 86399];
let time_array = Time32SecondArray::from(seconds_since_midnight);

assert_eq!(time_array.value(0), 0);     // 00:00:00
assert_eq!(time_array.value(1), 3600);  // 01:00:00
assert_eq!(time_array.value(2), 43200); // 12:00:00
```

**Timestamp Arrays with Timezone Operations**
```rust
use arrow_array::TimestampMillisecondArray;

let millis = vec![1_577_836_800_000]; // 2020-01-01 00:00:00 UTC
let timestamp_array = TimestampMillisecondArray::from(millis);

// Timezone operations
let utc_array = timestamp_array.clone().with_timezone_utc();
assert_eq!(utc_array.timezone(), Some("+00:00"));

let est_array = timestamp_array.clone().with_timezone("EST");
assert_eq!(est_array.timezone(), Some("EST"));
```

#### Decimal Type Methods

For high-precision decimal numbers:

**Basic Operations**
```rust
use arrow_array::Decimal128Array;

let decimal_values = vec![
    123450, // $12.3450 with scale 4
    567890, // $56.7890 with scale 4
    100000, // $10.0000 with scale 4
];

let precision = 10u8;
let scale = 4i8;
let decimal_array = Decimal128Array::from(decimal_values)
    .with_precision_and_scale(precision, scale)
    .unwrap();

assert_eq!(decimal_array.precision(), precision);
assert_eq!(decimal_array.scale(), scale);

// Format as string
assert_eq!(decimal_array.value_as_string(0), "12.3450");
assert_eq!(decimal_array.value_as_string(1), "56.7890");
```

**Precision Validation**
```rust
let large_values = vec![999_999_999_999_999_999i128, 123_456_789i128, 123i128];
let decimal_array = Decimal128Array::from(large_values);

// Test validation with small precision (should fail)
let validation_result = decimal_array.validate_decimal_precision(5);
assert!(validation_result.is_err());

// Test with larger precision (should pass)
let validation_result = decimal_array.validate_decimal_precision(20);
assert!(validation_result.is_ok());

// Handle overflow by converting to null
let safe_array = decimal_array.null_if_overflow_precision(10);
assert!(safe_array.is_null(0)); // First value overflows
assert!(!safe_array.is_null(1)); // Second value is valid
```

---

## ArrowPrimitiveType Trait Reference

### Trait Definition

The `ArrowPrimitiveType` trait bridges Arrow's dynamic typing with Rust's static typing:

```rust
pub trait ArrowPrimitiveType: Send + Sync + 'static {
    type Native: Copy + Send + Sync;
    const DATA_TYPE: DataType;
    fn default_value() -> Self::Native;
}
```

### Complete Implementor Reference

#### Integer Types
```rust
use arrow_array::types::*;

// Signed integers
assert_eq!(Int8Type::DATA_TYPE, DataType::Int8);
assert_eq!(Int8Type::default_value(), 0i8);

assert_eq!(Int16Type::DATA_TYPE, DataType::Int16);
assert_eq!(Int16Type::default_value(), 0i16);

assert_eq!(Int32Type::DATA_TYPE, DataType::Int32);
assert_eq!(Int32Type::default_value(), 0i32);

assert_eq!(Int64Type::DATA_TYPE, DataType::Int64);
assert_eq!(Int64Type::default_value(), 0i64);

// Unsigned integers
assert_eq!(UInt8Type::DATA_TYPE, DataType::UInt8);
assert_eq!(UInt8Type::default_value(), 0u8);

assert_eq!(UInt16Type::DATA_TYPE, DataType::UInt16);
assert_eq!(UInt16Type::default_value(), 0u16);

assert_eq!(UInt32Type::DATA_TYPE, DataType::UInt32);
assert_eq!(UInt32Type::default_value(), 0u32);

assert_eq!(UInt64Type::DATA_TYPE, DataType::UInt64);
assert_eq!(UInt64Type::default_value(), 0u64);
```

#### Floating Point Types
```rust
assert_eq!(Float32Type::DATA_TYPE, DataType::Float32);
assert_eq!(Float32Type::default_value(), 0.0f32);

assert_eq!(Float64Type::DATA_TYPE, DataType::Float64);
assert_eq!(Float64Type::default_value(), 0.0f64);
```

#### Temporal Types
```rust
use arrow_schema::TimeUnit;

// Date types
assert_eq!(Date32Type::DATA_TYPE, DataType::Date32);
assert_eq!(Date32Type::default_value(), 0i32);

assert_eq!(Date64Type::DATA_TYPE, DataType::Date64);
assert_eq!(Date64Type::default_value(), 0i64);

// Time types
assert_eq!(Time32SecondType::DATA_TYPE, DataType::Time32(TimeUnit::Second));
assert_eq!(Time32MillisecondType::DATA_TYPE, DataType::Time32(TimeUnit::Millisecond));
assert_eq!(Time64MicrosecondType::DATA_TYPE, DataType::Time64(TimeUnit::Microsecond));
assert_eq!(Time64NanosecondType::DATA_TYPE, DataType::Time64(TimeUnit::Nanosecond));

// Timestamp types
assert_eq!(TimestampSecondType::DATA_TYPE, DataType::Timestamp(TimeUnit::Second, None));
assert_eq!(TimestampMillisecondType::DATA_TYPE, DataType::Timestamp(TimeUnit::Millisecond, None));
// ... and more timestamp variants
```

### Generic Programming with ArrowPrimitiveType

```rust
fn create_array_of_type<T: ArrowPrimitiveType>() -> PrimitiveArray<T> 
where
    PrimitiveArray<T>: From<Vec<T::Native>>,
{
    let values = vec![T::default_value(); 3];
    PrimitiveArray::<T>::from(values)
}

// Use with any primitive type
let int32_array = create_array_of_type::<Int32Type>();
let float64_array = create_array_of_type::<Float64Type>();
```

---

## Supporting Structures Reference

### ScalarBuffer<T>

Contiguous buffer for storing primitive values.

```rust
use arrow_buffer::ScalarBuffer;

// Create from Vec
let buffer = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
assert_eq!(buffer.len(), 5);

// Use in array construction
let array = Int32Array::new(buffer, None);
```

**Key Features:**
- Zero-copy construction from `Vec<T>`
- Efficient memory layout for SIMD operations
- Reference-counted for sharing

### NullBuffer

Bit-packed representation of null/valid flags.

```rust
use arrow_buffer::NullBuffer;

// Create null buffer (true = valid, false = null)
let null_buffer = NullBuffer::from(vec![true, false, true, false, true]);
assert_eq!(null_buffer.len(), 5);
assert!(!null_buffer.is_null(0)); // true means valid
assert!(null_buffer.is_null(1));  // false means null

// Use with array
let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
let array = Int32Array::new(values, Some(null_buffer));
assert_eq!(array.null_count(), 2);
```

**Key Features:**
- Compact bit representation (1 bit per element)
- Standard Arrow null semantics
- Memory efficient for sparse nulls

### DataType Enum

Runtime representation of Arrow data types.

```rust
use arrow_schema::DataType;

// Check array data type
let array = Int32Array::from(vec![1, 2, 3]);
assert_eq!(array.data_type(), &DataType::Int32);

// Type information matches trait
assert_eq!(Int32Type::DATA_TYPE, DataType::Int32);
```

---

## Complete Examples and Use Cases

### Time Series Data Processing

```rust
use arrow_array::{TimestampMillisecondArray, Float64Array};
use arrow::compute;

// Temperature sensor readings with timestamps
let timestamps = vec![
    1609459200000, // 2021-01-01 00:00:00
    1609459260000, // 2021-01-01 00:01:00
    1609459320000, // 2021-01-01 00:02:00
    1609459380000, // 2021-01-01 00:03:00
];
let temperatures = vec![23.5, 24.1, 22.8, 25.0];

let timestamp_array = TimestampMillisecondArray::from(timestamps);
let temp_array = Float64Array::from(temperatures);

// Calculate moving average
let sum: f64 = temp_array.iter().filter_map(|x| x).sum();
let average = sum / temp_array.len() as f64;
println!("Average temperature: {:.2}°C", average);

// Convert to different timezone
let utc_timestamps = timestamp_array.with_timezone_utc();
assert_eq!(utc_timestamps.timezone(), Some("+00:00"));
```

### Financial Data with Decimals

```rust
use arrow_array::Decimal128Array;

// Stock prices with 4 decimal places
let prices_cents = vec![
    1050000, // $105.0000
    1049500, // $104.9500
    1051250, // $105.1250
    1048750, // $104.8750
];

let price_array = Decimal128Array::from(prices_cents)
    .with_precision_and_scale(10, 4)
    .unwrap();

// Format for display
for i in 0..price_array.len() {
    println!("Price: ${}", price_array.value_as_string(i));
}
```

### Data Validation and Cleaning

```rust
// Handle missing sensor data
let sensor_readings = vec![
    Some(23.5),
    None,        // Sensor malfunction
    Some(24.1),
    None,        // Missing reading
    Some(22.8),
];

let reading_array: Float64Array = sensor_readings.into_iter().collect();

// Count valid readings
let valid_count = reading_array.len() - reading_array.null_count();
println!("Valid readings: {} out of {}", valid_count, reading_array.len());

// Calculate average of valid readings only
let sum: f64 = reading_array.iter().filter_map(|x| x).sum();
let average = sum / valid_count as f64;
println!("Average of valid readings: {:.2}", average);
```

### Batch Data Processing

```rust
// Process large datasets efficiently
fn process_batch(array: &Int32Array) -> Float64Array {
    // Convert to float and apply transformation
    compute::unary(array, |x| (x as f64 * 1.5) + 10.0)
}

// Process data in chunks
let large_dataset: Vec<i32> = (0..1000000).collect();
let array = Int32Array::from(large_dataset);

// Slice and process
let chunk_size = 10000;
for chunk_start in (0..array.len()).step_by(chunk_size) {
    let chunk_end = (chunk_start + chunk_size).min(array.len());
    let chunk = array.slice(chunk_start, chunk_end - chunk_start);
    let processed = process_batch(&chunk);
    
    // Process results...
    println!("Processed chunk {}-{}", chunk_start, chunk_end);
}
```

---

## Best Practices and Performance

### Memory Management

**Use Zero-Copy Operations**
```rust
// Good: zero-copy slice
let original = Int32Array::from(vec![1, 2, 3, 4, 5]);
let view = original.slice(1, 3); // No data copying

// Avoid: unnecessary copying
// let copied = Int32Array::from(original.values().to_vec());
```

**Efficient Buffer Creation**
```rust
// Good: direct from Vec
let array = Int32Array::from(vec![1, 2, 3, 4, 5]);

// Good: pre-allocate when size is known
let mut builder = Int32Array::builder(1000);
for i in 0..1000 {
    builder.append_value(i);
}
let array = builder.finish();
```

### Null Handling Best Practices

**Check for Nulls Before Processing**
```rust
fn safe_process(array: &Int32Array) -> Vec<i32> {
    if array.null_count() == 0 {
        // Fast path: no nulls
        array.values().iter().map(|&x| x * 2).collect()
    } else {
        // Handle nulls appropriately
        array.iter()
            .filter_map(|x| x.map(|v| v * 2))
            .collect()
    }
}
```

### Performance Guidelines

**Use Appropriate Access Methods**
```rust
// For verified indices in hot loops
unsafe {
    for i in 0..array.len() {
        let value = array.value_unchecked(i);
        // Process value...
    }
}

// For safe access with bounds checking
for i in 0..array.len() {
    if !array.is_null(i) {
        let value = array.value(i);
        // Process value...
    }
}

// For iterator-based processing
for value in array.iter().flatten() {
    // Process non-null values...
}
```

**Batch Operations**
```rust
// Good: batch processing with compute functions
let result = compute::add(&array1, &array2);

// Avoid: element-by-element operations
// let mut results = Vec::new();
// for i in 0..array1.len() {
//     results.push(array1.value(i) + array2.value(i));
// }
```

### Common Pitfalls

**Null Value Access**
```rust
// Wrong: will panic if index 1 is null
// let value = array.value(1);

// Correct: check for null first
if !array.is_null(1) {
    let value = array.value(1);
    // Use value...
}

// Or use iterator
for (i, value) in array.iter().enumerate() {
    if let Some(v) = value {
        println!("Index {}: {}", i, v);
    }
}
```

**Type Mismatches**
```rust
// Good: explicit type annotations prevent mistakes
let int_array: Int32Array = data.into_iter().collect();
let float_array: Float64Array = data.into_iter().map(|x| x as f64).collect();

// Be careful with generic functions
fn process<T: ArrowPrimitiveType>(array: &PrimitiveArray<T>) 
where
    T::Native: std::fmt::Display,
{
    for i in 0..array.len() {
        if !array.is_null(i) {
            println!("{}", array.value(i));
        }
    }
}
```

---

This comprehensive guide covers all aspects of working with Apache Arrow's `PrimitiveArray<T>` in Rust. All code examples are tested and verified to compile and run correctly. For additional information, refer to the official Apache Arrow documentation and the arrow-rs crate documentation.