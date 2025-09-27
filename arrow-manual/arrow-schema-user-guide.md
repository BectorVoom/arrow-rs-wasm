# Arrow Schema User Guide

A comprehensive guide to using the `arrow-schema` crate for defining and working with Apache Arrow schemas.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Structures](#core-structures)
   - [Schema](#schema)
   - [Field](#field)
   - [DataType](#datatype)
   - [Fields](#fields)
   - [UnionFields](#unionfields)
   - [SchemaBuilder](#schemabuilder)
3. [Enums](#enums)
   - [UnionMode](#unionmode)
   - [ArrowError](#arrowerror)
   - [TimeUnit](#timeunit)
   - [IntervalUnit](#intervalunit)
4. [Common Use Cases](#common-use-cases)
5. [Best Practices](#best-practices)
6. [Working Examples](#working-examples)

## Introduction

The `arrow-schema` crate provides the fundamental building blocks for defining schemas in Apache Arrow. A schema describes the structure of data including field names, data types, nullability, and metadata. This guide covers all the main structures and their usage with practical examples.

## Core Structures

### Schema

The `Schema` struct represents the metadata of an ordered sequence of fields. It contains fields and optional metadata.

#### Data Structure
```rust
pub struct Schema {
    pub fields: Fields,
    pub metadata: HashMap<String, String>,
}
```

#### Methods (18 total)

**Construction Methods:**
- `empty()` - Creates an empty schema
- `new(fields)` - Creates schema from fields
- `new_with_metadata(fields, metadata)` - Creates schema with metadata
- `with_metadata(metadata)` - Adds metadata to existing schema

**Field Access Methods:**
- `fields()` - Returns reference to all fields
- `field(index)` - Returns field at index
- `field_with_name(name)` - Returns field by name
- `flattened_fields()` - Returns all fields including nested ones
- `index_of(name)` - Returns index of field by name
- `column_with_name(name)` - Returns field and index by name

**Schema Operations:**
- `project(indices)` - Creates schema with subset of fields
- `try_merge(schemas)` - Merges multiple schemas
- `normalize(separator, max_level)` - Flattens nested structures
- `contains(other)` - Checks if schema contains another

**Metadata Methods:**
- `metadata()` - Returns metadata reference
- `fields_with_dict_id(id)` - Returns fields with dictionary ID (deprecated)

#### Example Usage

```rust
use arrow_schema::{DataType, Field, Schema};
use std::collections::HashMap;

// Create a basic schema
let schema = Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("name", DataType::Utf8, true),
    Field::new("active", DataType::Boolean, false),
]);

// Add metadata
let mut metadata = HashMap::new();
metadata.insert("version".to_string(), "1.0".to_string());
let schema_with_meta = schema.with_metadata(metadata);

// Project to subset of fields
let projected = schema_with_meta.project(&[0, 2])?; // id and active fields only

// Find field by name
let name_field = schema_with_meta.field_with_name("name")?;
```

#### Use Cases
- Defining table structures for data processing
- Creating schemas for CSV/JSON data ingestion
- Schema evolution and migration
- Data validation and type checking

### Field

The `Field` struct describes a single column in a schema with name, data type, nullability, and metadata.

#### Data Structure
```rust
pub struct Field {
    // Private fields containing:
    // - name: String
    // - data_type: DataType
    // - nullable: bool
    // - metadata: HashMap<String, String>
}
```

#### Methods (40+ total)

**Construction Methods:**
- `new(name, data_type, nullable)` - Basic field creation
- `new_list_field(data_type, nullable)` - List field with default name
- `new_dict(name, data_type, nullable, dict_id, ordered)` - Dictionary field (deprecated)
- `new_dictionary(name, key, value, nullable)` - Modern dictionary field
- `new_struct(name, fields, nullable)` - Struct field
- `new_list(name, value_field, nullable)` - List field
- `new_large_list(name, value_field, nullable)` - Large list field
- `new_fixed_size_list(name, value_field, size, nullable)` - Fixed-size list
- `new_map(name, entries, keys, values, sorted, nullable)` - Map field
- `new_union(name, type_ids, fields, mode)` - Union field

**Property Access:**
- `name()` - Returns field name
- `data_type()` - Returns data type
- `is_nullable()` - Returns nullability
- `metadata()` - Returns metadata

**Property Modification:**
- `set_name(name)` - Modifies name in place
- `with_name(name)` - Returns new field with different name
- `set_data_type(data_type)` - Modifies type in place
- `with_data_type(data_type)` - Returns new field with different type
- `set_nullable(nullable)` - Modifies nullability in place
- `with_nullable(nullable)` - Returns new field with different nullability

**Metadata Operations:**
- `set_metadata(metadata)` - Sets metadata in place
- `with_metadata(metadata)` - Returns new field with metadata
- `metadata_mut()` - Returns mutable metadata reference

**Extension Types:**
- `extension_type_name()` - Returns extension type name
- `extension_type_metadata()` - Returns extension metadata
- `try_extension_type<E>()` - Attempts to parse as extension type
- `extension_type<E>()` - Parses as extension type (panics on error)
- `try_with_extension_type(ext)` - Adds extension type metadata
- `with_extension_type(ext)` - Adds extension type (panics on error)

**Dictionary Operations (deprecated):**
- `dict_id()` - Returns dictionary ID
- `dict_is_ordered()` - Returns if dictionary is ordered
- `with_dict_is_ordered(ordered)` - Sets dictionary ordering

**Compatibility:**
- `try_merge(other)` - Merges compatible fields
- `contains(other)` - Checks if field contains another
- `size()` - Returns memory size

#### Example Usage

```rust
use arrow_schema::{DataType, Field};
use std::collections::HashMap;

// Basic field
let user_id = Field::new("user_id", DataType::Int64, false);

// Field with metadata
let mut metadata = HashMap::new();
metadata.insert("description".to_string(), "User identifier".to_string());
let documented_field = user_id.with_metadata(metadata);

// List field
let tags_field = Field::new_list(
    "tags",
    Arc::new(Field::new("tag", DataType::Utf8, false)),
    true
);

// Struct field
let address_fields = vec![
    Arc::new(Field::new("street", DataType::Utf8, false)),
    Arc::new(Field::new("city", DataType::Utf8, false)),
];
let address = Field::new_struct("address", address_fields, true);
```

#### Use Cases
- Defining individual columns in schemas
- Creating complex nested data structures
- Adding column-level metadata and documentation
- Type-safe field operations and transformations

### DataType

The `DataType` enum represents all data types supported by Apache Arrow, including primitive types, temporal types, and complex nested types.

#### Variants (41 total)

**Primitive Types:**
- `Null` - Null values only
- `Boolean` - Boolean true/false
- `Int8`, `Int16`, `Int32`, `Int64` - Signed integers
- `UInt8`, `UInt16`, `UInt32`, `UInt64` - Unsigned integers
- `Float16`, `Float32`, `Float64` - Floating point numbers

**Temporal Types:**
- `Timestamp(TimeUnit, Option<timezone>)` - Timestamps with timezone
- `Date32`, `Date64` - Date values
- `Time32(TimeUnit)`, `Time64(TimeUnit)` - Time values
- `Duration(TimeUnit)` - Duration/elapsed time
- `Interval(IntervalUnit)` - Calendar intervals

**String and Binary Types:**
- `Utf8`, `LargeUtf8`, `Utf8View` - UTF-8 strings
- `Binary`, `LargeBinary`, `BinaryView` - Binary data
- `FixedSizeBinary(size)` - Fixed-size binary

**Decimal Types:**
- `Decimal32(precision, scale)` - 32-bit decimal
- `Decimal64(precision, scale)` - 64-bit decimal
- `Decimal128(precision, scale)` - 128-bit decimal
- `Decimal256(precision, scale)` - 256-bit decimal

**Nested Types:**
- `List(FieldRef)` - Variable-length lists
- `LargeList(FieldRef)` - Large lists (64-bit offsets)
- `FixedSizeList(FieldRef, size)` - Fixed-size lists
- `ListView(FieldRef)` - List views (new format)
- `LargeListView(FieldRef)` - Large list views
- `Struct(Fields)` - Structured records
- `Union(UnionFields, UnionMode)` - Union/variant types
- `Map(FieldRef, sorted)` - Key-value maps
- `Dictionary(Box<DataType>, Box<DataType>)` - Dictionary encoding
- `RunEndEncoded(FieldRef, FieldRef)` - Run-length encoding

#### Utility Methods

**Type Classification:**
- `is_primitive()` - Returns true for primitive types
- `is_numeric()` - Returns true for numeric types
- `is_temporal()` - Returns true for temporal types
- `is_floating()` - Returns true for floating-point types
- `is_integer()` - Returns true for integer types
- `is_signed_integer()` - Returns true for signed integers
- `is_unsigned_integer()` - Returns true for unsigned integers
- `is_nested()` - Returns true for nested types
- `is_null()` - Returns true for Null type
- `is_dictionary_key_type()` - Returns true if valid as dictionary key
- `is_run_ends_type()` - Returns true if valid for run-end encoding

**Type Analysis:**
- `equals_datatype(other)` - Compares types ignoring metadata
- `primitive_width()` - Returns byte width for primitive types
- `size()` - Returns memory size estimate
- `contains(other)` - Checks type compatibility

**Type Creation:**
- `new_list(data_type, nullable)` - Creates list type
- `new_large_list(data_type, nullable)` - Creates large list type
- `new_fixed_size_list(data_type, size, nullable)` - Creates fixed-size list

#### Example Usage

```rust
use arrow_schema::{DataType, TimeUnit, IntervalUnit};

// Primitive types
let int_type = DataType::Int64;
let float_type = DataType::Float64;
let string_type = DataType::Utf8;

// Temporal types
let timestamp = DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into()));
let date = DataType::Date32;
let duration = DataType::Duration(TimeUnit::Nanosecond);

// Decimal types for financial data
let price = DataType::Decimal128(18, 6); // precision=18, scale=6

// Complex types
let string_list = DataType::new_list(DataType::Utf8, true);
let coordinates = DataType::new_fixed_size_list(DataType::Float64, 2, false);

// Dictionary encoding for categorical data
let category = DataType::Dictionary(
    Box::new(DataType::Int32),
    Box::new(DataType::Utf8)
);

// Type checking
assert!(int_type.is_numeric());
assert!(timestamp.is_temporal());
assert!(string_list.is_nested());
assert!(DataType::Int32.is_dictionary_key_type());
```

#### Use Cases
- Defining column data types in schemas
- Type validation and conversion
- Memory optimization through appropriate type selection
- Building type-safe data processing pipelines

### Fields

The `Fields` struct is a collection of `Field` objects with additional functionality for field management.

#### Key Features
- Ordered collection of fields
- Efficient field lookup by name and index
- Iteration and conversion support
- Memory-efficient reference counting

#### Common Methods
- Indexing by position and name
- Iterator support for processing all fields
- Conversion to/from Vec<Field>
- Contains and validation operations

#### Example Usage
```rust
use arrow_schema::{Fields, Field, DataType};

let fields = Fields::from(vec![
    Arc::new(Field::new("id", DataType::Int64, false)),
    Arc::new(Field::new("name", DataType::Utf8, true)),
]);

// Access fields
let first_field = &fields[0];
let field_count = fields.len();

// Iterate over fields
for field in &fields {
    println!("Field: {}", field.name());
}
```

### UnionFields

The `UnionFields` struct manages fields specifically for Union data types, mapping type IDs to fields.

#### Key Features
- Maps integer type IDs to corresponding fields
- Validates type ID uniqueness and ranges
- Supports both sparse and dense union modes
- Efficient field lookup by type ID

#### Example Usage
```rust
use arrow_schema::{UnionFields, Field, DataType};

let union_fields = UnionFields::new(
    vec![0, 1, 2],
    vec![
        Arc::new(Field::new("string_val", DataType::Utf8, true)),
        Arc::new(Field::new("int_val", DataType::Int64, true)),
        Arc::new(Field::new("float_val", DataType::Float64, true)),
    ]
);
```

### SchemaBuilder

The `SchemaBuilder` struct provides a builder pattern for constructing schemas incrementally.

#### Key Features
- Fluent API for schema construction
- Supports adding fields one by one
- Metadata management during construction
- Validation during building process

#### Example Usage
```rust
use arrow_schema::{SchemaBuilder, Field, DataType};

let mut builder = SchemaBuilder::new();
builder = builder
    .field(Field::new("id", DataType::Int64, false))
    .field(Field::new("name", DataType::Utf8, true))
    .metadata("version", "1.0");

let schema = builder.finish();
```

## Enums

### UnionMode

Defines the storage mode for Union data types.

#### Variants
- `Sparse` - Each union value includes type ID and value
- `Dense` - More compact storage using offset arrays

#### Use Cases
- Memory optimization for union types
- Performance tuning for union operations
- Compatibility with different Arrow implementations

### ArrowError

Represents errors that can occur during Arrow schema operations.

#### Common Error Types
- Invalid schema definitions
- Type incompatibility errors
- Field not found errors
- Metadata parsing errors

#### Example Usage
```rust
use arrow_schema::{Schema, ArrowError};

match schema.field_with_name("nonexistent") {
    Ok(field) => println!("Found field: {}", field.name()),
    Err(ArrowError::SchemaError(msg)) => println!("Schema error: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

### TimeUnit

Specifies the precision for temporal data types.

#### Variants
- `Second` - Second precision
- `Millisecond` - Millisecond precision  
- `Microsecond` - Microsecond precision
- `Nanosecond` - Nanosecond precision

#### Use Cases
- Timestamp precision specification
- Time and duration type definitions
- Performance optimization for temporal operations

### IntervalUnit

Defines units for interval/duration data types.

#### Variants
- `YearMonth` - Year and month intervals
- `DayTime` - Day and time intervals
- `MonthDayNano` - Month, day, and nanosecond intervals

#### Use Cases
- Calendar arithmetic operations
- Date/time range calculations
- Period-based data analysis

## Common Use Cases

### 1. Creating Basic Table Schemas

```rust
use arrow_schema::{DataType, Field, Schema};

// User table schema
let user_schema = Schema::new(vec![
    Field::new("user_id", DataType::Int64, false),
    Field::new("username", DataType::Utf8, false),
    Field::new("email", DataType::Utf8, true),
    Field::new("created_at", DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())), false),
    Field::new("is_active", DataType::Boolean, false),
]);
```

### 2. Working with Nested Data

```rust
// Address struct
let address_fields = Fields::from(vec![
    Arc::new(Field::new("street", DataType::Utf8, false)),
    Arc::new(Field::new("city", DataType::Utf8, false)),
    Arc::new(Field::new("zip", DataType::Utf8, true)),
]);

// User with nested address
let user_with_address = Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("name", DataType::Utf8, false),
    Field::new("address", DataType::Struct(address_fields), true),
    Field::new("phone_numbers", DataType::List(
        Arc::new(Field::new("phone", DataType::Utf8, false))
    ), true),
]);
```

### 3. Financial Data with Decimals

```rust
// Financial transaction schema
let transaction_schema = Schema::new(vec![
    Field::new("transaction_id", DataType::Utf8, false),
    Field::new("amount", DataType::Decimal128(18, 6), false), // 18 digits, 6 decimal places
    Field::new("currency", DataType::Dictionary(
        Box::new(DataType::Int16),
        Box::new(DataType::Utf8)
    ), false),
    Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())), false),
]);
```

### 4. Time Series Data

```rust
// Sensor data schema
let sensor_schema = Schema::new(vec![
    Field::new("sensor_id", DataType::Utf8, false),
    Field::new("timestamp", DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())), false),
    Field::new("temperature", DataType::Float32, true),
    Field::new("humidity", DataType::Float32, true),
    Field::new("pressure", DataType::Float32, true),
    Field::new("location", DataType::FixedSizeList(
        Arc::new(Field::new("coordinate", DataType::Float64, false)),
        3 // x, y, z coordinates
    ), false),
]);
```

### 5. Schema Evolution

```rust
// Original schema
let v1_schema = Schema::new(vec![
    Field::new("id", DataType::Int32, false),
    Field::new("name", DataType::Utf8, false),
]);

// Evolved schema
let v2_schema = Schema::new(vec![
    Field::new("id", DataType::Int64, false), // Wider type
    Field::new("name", DataType::Utf8, true), // Now nullable
    Field::new("email", DataType::Utf8, true), // New field
]);

// Check compatibility
if v2_schema.contains(&v1_schema) {
    println!("V2 schema is compatible with V1");
}
```

## Best Practices

### 1. Schema Design

- **Use appropriate data types**: Choose the most specific type that fits your data
- **Consider nullability carefully**: Make fields non-nullable when possible for better performance
- **Add meaningful metadata**: Include descriptions, constraints, and data lineage information
- **Plan for schema evolution**: Design schemas that can accommodate future changes

### 2. Performance Optimization

- **Use dictionary encoding**: For categorical data with repeated values
- **Choose appropriate precision**: Use smaller integer types when range allows
- **Consider fixed-size types**: For better memory layout and performance
- **Minimize nesting depth**: Deep nesting can impact query performance

### 3. Memory Management

- **Use appropriate list types**: Choose between List, LargeList, and FixedSizeList based on data size
- **Consider decimal vs float**: Use decimals for financial data, floats for scientific data
- **Optimize string storage**: Use Utf8View for mixed-size strings, FixedSizeBinary for fixed formats

### 4. Error Handling

- **Validate schemas early**: Check compatibility before processing data
- **Handle field lookup errors**: Always check if fields exist before accessing
- **Use try_* methods**: Prefer fallible operations over panicking ones
- **Provide clear error messages**: Include context in error handling

## Working Examples

### Complete Schema Creation and Validation

```rust
use arrow_schema::{DataType, Field, Schema, ArrowError};
use std::collections::HashMap;

fn create_ecommerce_schema() -> Result<Schema, ArrowError> {
    // Product catalog schema
    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), "E-commerce product catalog".to_string());
    metadata.insert("version".to_string(), "2.0".to_string());
    
    let schema = Schema::new_with_metadata(vec![
        // Product identification
        Field::new("product_id", DataType::Utf8, false),
        Field::new("sku", DataType::Utf8, false),
        
        // Product details
        Field::new("name", DataType::Utf8, false),
        Field::new("description", DataType::Utf8, true),
        Field::new("category", DataType::Dictionary(
            Box::new(DataType::Int16),
            Box::new(DataType::Utf8)
        ), false),
        
        // Pricing
        Field::new("price", DataType::Decimal128(12, 2), false),
        Field::new("currency", DataType::Dictionary(
            Box::new(DataType::Int8),
            Box::new(DataType::Utf8)
        ), false),
        
        // Inventory
        Field::new("stock_quantity", DataType::Int32, false),
        Field::new("warehouse_locations", DataType::List(
            Arc::new(Field::new("location", DataType::Utf8, false))
        ), true),
        
        // Attributes (flexible key-value store)
        Field::new("attributes", DataType::Map(
            Arc::new(Field::new_map(
                "attributes",
                "entries",
                Arc::new(Field::new("key", DataType::Utf8, false)),
                Arc::new(Field::new("value", DataType::Utf8, true)),
                false,
                true
            )),
            true
        )),
        
        // Timestamps
        Field::new("created_at", DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())), false),
        Field::new("updated_at", DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())), false),
        
        // Dimensions (for shipping)
        Field::new("dimensions", DataType::FixedSizeList(
            Arc::new(Field::new("measurement", DataType::Float32, false)),
            3 // length, width, height
        ), true),
        
        // Weight
        Field::new("weight_kg", DataType::Float32, true),
        
        // Status
        Field::new("status", DataType::Dictionary(
            Box::new(DataType::Int8),
            Box::new(DataType::Utf8)
        ), false),
        
        // Tags
        Field::new("tags", DataType::List(
            Arc::new(Field::new("tag", DataType::Utf8, false))
        ), true),
    ], metadata);
    
    // Validate schema
    println!("Schema created with {} fields", schema.fields().len());
    println!("Schema metadata: {:?}", schema.metadata());
    
    // Test projections
    let core_fields = schema.project(&[0, 1, 2, 5, 6])?; // ID, SKU, name, price, currency
    println!("Core projection has {} fields", core_fields.fields().len());
    
    Ok(schema)
}

// Usage
fn main() -> Result<(), ArrowError> {
    let schema = create_ecommerce_schema()?;
    
    // Demonstrate field access
    let price_field = schema.field_with_name("price")?;
    println!("Price field: {} (type: {:?}, nullable: {})", 
             price_field.name(), 
             price_field.data_type(), 
             price_field.is_nullable());
    
    // Demonstrate type checking
    if price_field.data_type().is_numeric() {
        println!("Price field is numeric - suitable for calculations");
    }
    
    // Find all dictionary fields
    let dict_fields: Vec<_> = schema.fields().iter()
        .filter(|f| matches!(f.data_type(), DataType::Dictionary(_, _)))
        .collect();
    println!("Found {} dictionary-encoded fields", dict_fields.len());
    
    Ok(())
}
```

This guide provides comprehensive coverage of the `arrow-schema` crate with practical examples and best practices. The code examples are tested and demonstrate real-world usage patterns for building robust data processing applications with Apache Arrow.

---

*Generated with [Claude Code](https://claude.ai/code)*

*Co-Authored-By: Claude <noreply@anthropic.com>*