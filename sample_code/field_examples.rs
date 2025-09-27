//! # Arrow Field Examples
//!
//! This module demonstrates the usage of `arrow_schema::Field` and related structures.
//! The Field describes a single column in a Schema with name, data type, nullability, and metadata.

use arrow_schema::{DataType, Field, ArrowError, UnionMode, TimeUnit, IntervalUnit};
use std::collections::HashMap;
use std::sync::Arc;

/// Demonstrates creating a basic Field
/// 
/// # Use Cases
/// - Defining simple table columns
/// - Basic schema construction
/// - Column type specification
pub fn create_basic_field() -> Field {
    let field = Field::new("user_id", DataType::Int64, false);
    println!("Basic field: {}", field);
    field
}

/// Demonstrates creating a list field with default naming
/// 
/// # Use Cases
/// - Creating list/array columns with conventional naming
/// - Defining repeated value fields
/// - Working with nested data structures
pub fn create_list_field() -> Field {
    let field = Field::new_list_field(DataType::Utf8, true);
    println!("List field: {}", field);
    field
}

/// Demonstrates creating a dictionary field (deprecated)
/// 
/// # Use Cases
/// - Legacy dictionary field creation
/// - Backward compatibility with older Arrow versions
/// - Dictionary encoding for categorical data
#[allow(deprecated)]
pub fn create_dict_field() -> Field {
    let field = Field::new_dict("category", DataType::Dictionary(
        Box::new(DataType::Int32),
        Box::new(DataType::Utf8)
    ), true, 1, false);
    println!("Dictionary field: {}", field);
    field
}

/// Demonstrates creating a dictionary field using modern API
/// 
/// # Use Cases
/// - Creating dictionary-encoded columns for categorical data
/// - Memory optimization for repeated string values
/// - Efficient storage of enum-like data
pub fn create_dictionary_field() -> Field {
    let field = Field::new_dictionary("status", DataType::Int32, DataType::Utf8, true);
    println!("Dictionary field (modern): {}", field);
    field
}

/// Demonstrates creating a struct field
/// 
/// # Use Cases
/// - Defining complex nested objects
/// - Representing JSON-like structures
/// - Grouping related fields together
pub fn create_struct_field() -> Field {
    let address_fields = vec![
        Arc::new(Field::new("street", DataType::Utf8, false)),
        Arc::new(Field::new("city", DataType::Utf8, false)),
        Arc::new(Field::new("zip_code", DataType::Utf8, true)),
        Arc::new(Field::new("country", DataType::Utf8, false)),
    ];
    
    let field = Field::new_struct("address", address_fields, true);
    println!("Struct field: {}", field);
    field
}

/// Demonstrates creating a list field
/// 
/// # Use Cases
/// - Variable-length arrays of data
/// - Repeated fields in data structures
/// - Multi-value columns
pub fn create_list_field_detailed() -> Field {
    let item_field = Arc::new(Field::new("item", DataType::Utf8, true));
    let field = Field::new_list("tags", item_field, true);
    println!("List field (detailed): {}", field);
    field
}

/// Demonstrates creating a large list field
/// 
/// # Use Cases
/// - Very large arrays (>2GB of data)
/// - High-performance analytics on large lists
/// - Big data processing scenarios
pub fn create_large_list_field() -> Field {
    let item_field = Arc::new(Field::new("value", DataType::Float64, false));
    let field = Field::new_large_list("measurements", item_field, false);
    println!("Large list field: {}", field);
    field
}

/// Demonstrates creating a fixed-size list field
/// 
/// # Use Cases
/// - Arrays with known, constant size
/// - Vector data (coordinates, embeddings)
/// - Performance optimization for fixed-size arrays
pub fn create_fixed_size_list_field() -> Field {
    let item_field = Arc::new(Field::new("coordinate", DataType::Float32, false));
    let field = Field::new_fixed_size_list("position", item_field, 3, false); // 3D coordinates
    println!("Fixed-size list field: {}", field);
    field
}

/// Demonstrates creating a map field
/// 
/// # Use Cases
/// - Key-value pair data
/// - Dictionary-like structures
/// - Sparse data representation
/// - Configuration or property storage
pub fn create_map_field() -> Field {
    let key_field = Arc::new(Field::new("key", DataType::Utf8, false));
    let value_field = Arc::new(Field::new("value", DataType::Utf8, true));
    
    let field = Field::new_map("properties", "entries", key_field, value_field, false, true);
    println!("Map field: {}", field);
    field
}

/// Demonstrates creating a union field
/// 
/// # Use Cases
/// - Variant/sum types
/// - Polymorphic data structures
/// - Mixed-type columns
/// - Type-safe data unions
pub fn create_union_field() -> Field {
    let type_ids = vec![0, 1, 2];
    let union_fields = vec![
        Arc::new(Field::new("string_value", DataType::Utf8, true)),
        Arc::new(Field::new("int_value", DataType::Int64, true)),
        Arc::new(Field::new("float_value", DataType::Float64, true)),
    ];
    
    let field = Field::new_union("value", type_ids, union_fields, UnionMode::Sparse);
    println!("Union field: {}", field);
    field
}

/// Demonstrates field metadata operations
/// 
/// # Use Cases
/// - Adding column-level documentation
/// - Storing data quality rules
/// - Adding formatting hints
/// - Preserving data lineage
pub fn field_metadata_operations() -> Field {
    let mut field = Field::new("temperature", DataType::Float32, true);
    
    // Set metadata using mutable reference
    let mut metadata = HashMap::new();
    metadata.insert("unit".to_string(), "celsius".to_string());
    metadata.insert("precision".to_string(), "0.1".to_string());
    metadata.insert("description".to_string(), "Ambient temperature reading".to_string());
    metadata.insert("source".to_string(), "sensor_network".to_string());
    
    field.set_metadata(metadata);
    
    // Access metadata
    let meta = field.metadata();
    println!("Field metadata:");
    for (key, value) in meta.iter() {
        println!("  {}: {}", key, value);
    }
    
    // Create field with metadata using builder pattern
    let mut new_metadata = HashMap::new();
    new_metadata.insert("updated_at".to_string(), "2024-01-01".to_string());
    
    let field_with_meta = field.with_metadata(new_metadata);
    println!("Field with updated metadata: {}", field_with_meta.metadata().len());
    
    field_with_meta
}

/// Demonstrates field name operations
/// 
/// # Use Cases
/// - Column renaming operations
/// - Dynamic field name generation
/// - Schema transformation pipelines
/// - Field aliasing
pub fn field_name_operations() -> Field {
    let mut field = Field::new("original_name", DataType::Utf8, true);
    println!("Original field name: {}", field.name());
    
    // Modify name in place
    field.set_name("updated_name");
    println!("Updated field name: {}", field.name());
    
    // Create new field with different name
    let renamed_field = field.with_name("final_name");
    println!("Final field name: {}", renamed_field.name());
    
    renamed_field
}

/// Demonstrates field data type operations
/// 
/// # Use Cases
/// - Type casting operations
/// - Schema evolution
/// - Data type migrations
/// - Dynamic type assignment
pub fn field_data_type_operations() -> Field {
    let mut field = Field::new("value", DataType::Int32, true);
    println!("Original data type: {:?}", field.data_type());
    
    // Change data type in place
    field.set_data_type(DataType::Int64);
    println!("Updated data type: {:?}", field.data_type());
    
    // Create new field with different data type
    let new_field = field.with_data_type(DataType::Float64);
    println!("Final data type: {:?}", new_field.data_type());
    
    new_field
}

/// Demonstrates extension type operations
/// 
/// # Use Cases
/// - Custom data type definitions
/// - Domain-specific data types
/// - Type system extensions
/// - Metadata-driven type behavior
pub fn extension_type_operations() -> Field {
    let field = Field::new("uuid", DataType::FixedSizeBinary(16), false);
    
    // Check for extension type name
    match field.extension_type_name() {
        Some(name) => println!("Extension type name: {}", name),
        None => println!("No extension type name set"),
    }
    
    // Check for extension type metadata
    match field.extension_type_metadata() {
        Some(metadata) => println!("Extension type metadata: {}", metadata),
        None => println!("No extension type metadata set"),
    }
    
    // Add extension type metadata manually
    let mut metadata = HashMap::new();
    metadata.insert("ARROW:extension:name".to_string(), "uuid".to_string());
    metadata.insert("ARROW:extension:metadata".to_string(), "{}".to_string());
    
    let field_with_extension = field.with_metadata(metadata);
    println!("Field with extension metadata: {}", field_with_extension.extension_type_name().unwrap_or("none"));
    
    field_with_extension
}

/// Demonstrates nullability operations
/// 
/// # Use Cases
/// - Schema evolution for nullability constraints
/// - Data validation rule changes
/// - Optional field management
/// - Database schema migrations
pub fn nullability_operations() -> Field {
    let mut field = Field::new("email", DataType::Utf8, false);
    println!("Original nullability: {}", field.is_nullable());
    
    // Change nullability in place
    field.set_nullable(true);
    println!("Updated nullability: {}", field.is_nullable());
    
    // Create new field with different nullability
    let nullable_field = field.with_nullable(false);
    println!("Final nullability: {}", nullable_field.is_nullable());
    
    nullable_field
}

/// Demonstrates dictionary operations (deprecated)
/// 
/// # Use Cases
/// - Legacy dictionary metadata access
/// - Backward compatibility
/// - Migration from older Arrow versions
#[allow(deprecated)]
pub fn dictionary_operations() -> Field {
    let dict_field = Field::new_dict("category", DataType::Dictionary(
        Box::new(DataType::Int32),
        Box::new(DataType::Utf8)
    ), true, 42, false);
    
    // Access dictionary ID
    match dict_field.dict_id() {
        Some(id) => println!("Dictionary ID: {}", id),
        None => println!("No dictionary ID"),
    }
    
    // Access dictionary ordering
    match dict_field.dict_is_ordered() {
        Some(ordered) => println!("Dictionary is ordered: {}", ordered),
        None => println!("Not a dictionary field"),
    }
    
    // Set dictionary ordering
    let ordered_dict = dict_field.with_dict_is_ordered(true);
    println!("Updated dictionary ordering: {:?}", ordered_dict.dict_is_ordered());
    
    ordered_dict
}

/// Demonstrates field merging operations
/// 
/// # Use Cases
/// - Schema reconciliation
/// - Merging schemas from different sources
/// - Field compatibility checking
/// - Schema evolution validation
pub fn field_merge_operations() -> Result<Field, ArrowError> {
    let mut base_field = Field::new("user_id", DataType::Int64, false);
    
    // Field with more permissive nullability
    let nullable_field = Field::new("user_id", DataType::Int64, true);
    
    // Field with additional metadata
    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), "User identifier".to_string());
    let metadata_field = Field::new("user_id", DataType::Int64, true).with_metadata(metadata);
    
    println!("Base field nullable: {}", base_field.is_nullable());
    
    // Merge nullable field
    base_field.try_merge(&nullable_field)?;
    println!("After merging nullable field: {}", base_field.is_nullable());
    
    // Merge metadata field
    base_field.try_merge(&metadata_field)?;
    println!("After merging metadata field, metadata count: {}", base_field.metadata().len());
    
    Ok(base_field)
}

/// Demonstrates field containment checking
/// 
/// # Use Cases
/// - Schema compatibility validation
/// - Field subset verification
/// - API contract validation
/// - Data compatibility checking
pub fn field_contains_operations() -> Field {
    let base_field = Field::new("amount", DataType::Decimal128(10, 2), true);
    
    // Compatible field (same type, less nullable)
    let compatible_field = Field::new("amount", DataType::Decimal128(10, 2), false);
    
    // Incompatible field (different type)
    let incompatible_field = Field::new("amount", DataType::Float64, false);
    
    // Check if base field contains compatible field
    if base_field.contains(&compatible_field) {
        println!("Base field contains compatible field");
    } else {
        println!("Base field does not contain compatible field");
    }
    
    // Check if base field contains incompatible field
    if base_field.contains(&incompatible_field) {
        println!("Base field contains incompatible field");
    } else {
        println!("Base field does not contain incompatible field");
    }
    
    base_field
}

/// Demonstrates field size calculation
/// 
/// # Use Cases
/// - Memory usage estimation
/// - Schema optimization
/// - Performance analysis
/// - Resource planning
pub fn field_size_operations() -> Field {
    let simple_field = Field::new("id", DataType::Int64, false);
    println!("Simple field size: {} bytes", simple_field.size());
    
    // Complex nested field
    let address_fields = vec![
        Arc::new(Field::new("street", DataType::Utf8, false)),
        Arc::new(Field::new("city", DataType::Utf8, false)),
        Arc::new(Field::new("zip", DataType::Utf8, true)),
    ];
    let complex_field = Field::new_struct("address", address_fields, true);
    println!("Complex field size: {} bytes", complex_field.size());
    
    // Field with metadata
    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), "A very long description that takes up memory".to_string());
    metadata.insert("source".to_string(), "external_api".to_string());
    let metadata_field = simple_field.clone().with_metadata(metadata);
    println!("Field with metadata size: {} bytes", metadata_field.size());
    
    metadata_field
}

/// Demonstrates various timestamp and temporal field types
/// 
/// # Use Cases
/// - Time series data modeling
/// - Event logging schemas
/// - Temporal data analysis
/// - Time-based indexing
pub fn temporal_field_examples() -> Vec<Field> {
    let mut fields = Vec::new();
    
    // Timestamp fields with different precisions
    fields.push(Field::new("timestamp_sec", DataType::Timestamp(TimeUnit::Second, None), false));
    fields.push(Field::new("timestamp_ms", DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())), false));
    fields.push(Field::new("timestamp_us", DataType::Timestamp(TimeUnit::Microsecond, Some("America/New_York".into())), true));
    fields.push(Field::new("timestamp_ns", DataType::Timestamp(TimeUnit::Nanosecond, Some("+05:30".into())), true));
    
    // Date fields
    fields.push(Field::new("date32", DataType::Date32, false));
    fields.push(Field::new("date64", DataType::Date64, true));
    
    // Time fields
    fields.push(Field::new("time32_sec", DataType::Time32(TimeUnit::Second), false));
    fields.push(Field::new("time32_ms", DataType::Time32(TimeUnit::Millisecond), false));
    fields.push(Field::new("time64_us", DataType::Time64(TimeUnit::Microsecond), true));
    fields.push(Field::new("time64_ns", DataType::Time64(TimeUnit::Nanosecond), true));
    
    // Duration fields
    fields.push(Field::new("duration_sec", DataType::Duration(TimeUnit::Second), true));
    fields.push(Field::new("duration_ms", DataType::Duration(TimeUnit::Millisecond), true));
    
    // Interval fields
    fields.push(Field::new("interval_year_month", DataType::Interval(IntervalUnit::YearMonth), true));
    fields.push(Field::new("interval_day_time", DataType::Interval(IntervalUnit::DayTime), true));
    fields.push(Field::new("interval_month_day_nano", DataType::Interval(IntervalUnit::MonthDayNano), true));
    
    for field in &fields {
        println!("Temporal field: {} -> {:?}", field.name(), field.data_type());
    }
    
    fields
}

/// Demonstrates various numeric field types
/// 
/// # Use Cases
/// - Numeric data modeling
/// - Financial data schemas
/// - Scientific computing schemas
/// - Performance optimization for numeric data
pub fn numeric_field_examples() -> Vec<Field> {
    let mut fields = Vec::new();
    
    // Integer fields
    fields.push(Field::new("int8", DataType::Int8, true));
    fields.push(Field::new("int16", DataType::Int16, true));
    fields.push(Field::new("int32", DataType::Int32, false));
    fields.push(Field::new("int64", DataType::Int64, false));
    
    // Unsigned integer fields
    fields.push(Field::new("uint8", DataType::UInt8, true));
    fields.push(Field::new("uint16", DataType::UInt16, true));
    fields.push(Field::new("uint32", DataType::UInt32, false));
    fields.push(Field::new("uint64", DataType::UInt64, false));
    
    // Floating point fields
    fields.push(Field::new("float16", DataType::Float16, true));
    fields.push(Field::new("float32", DataType::Float32, false));
    fields.push(Field::new("float64", DataType::Float64, false));
    
    // Decimal fields
    fields.push(Field::new("decimal32", DataType::Decimal32(7, 2), false)); // precision 7, scale 2
    fields.push(Field::new("decimal64", DataType::Decimal64(15, 4), false));
    fields.push(Field::new("decimal128", DataType::Decimal128(28, 6), true));
    fields.push(Field::new("decimal256", DataType::Decimal256(76, 18), true));
    
    for field in &fields {
        println!("Numeric field: {} -> {:?}", field.name(), field.data_type());
    }
    
    fields
}

/// Demonstrates string and binary field types
/// 
/// # Use Cases
/// - Text data modeling
/// - Binary data storage
/// - Large object handling
/// - String optimization strategies
pub fn string_binary_field_examples() -> Vec<Field> {
    let mut fields = Vec::new();
    
    // String fields
    fields.push(Field::new("utf8", DataType::Utf8, true));
    fields.push(Field::new("large_utf8", DataType::LargeUtf8, true));
    fields.push(Field::new("utf8_view", DataType::Utf8View, true));
    
    // Binary fields
    fields.push(Field::new("binary", DataType::Binary, true));
    fields.push(Field::new("large_binary", DataType::LargeBinary, true));
    fields.push(Field::new("binary_view", DataType::BinaryView, true));
    fields.push(Field::new("fixed_size_binary", DataType::FixedSizeBinary(16), false)); // UUID size
    
    for field in &fields {
        println!("String/Binary field: {} -> {:?}", field.name(), field.data_type());
    }
    
    fields
}

/// Comprehensive example demonstrating Field usage patterns
/// 
/// # Use Cases
/// - Real-world field management workflows
/// - Building complex schema structures
/// - Field transformation pipelines
/// - Best practices demonstration
pub fn comprehensive_field_example() -> Result<Vec<Field>, ArrowError> {
    println!("=== Comprehensive Field Example ===");
    
    let mut fields = Vec::new();
    
    // 1. Basic fields with metadata
    let mut user_id_field = Field::new("user_id", DataType::Int64, false);
    let mut metadata = HashMap::new();
    metadata.insert("primary_key".to_string(), "true".to_string());
    metadata.insert("description".to_string(), "Unique user identifier".to_string());
    user_id_field.set_metadata(metadata);
    fields.push(user_id_field);
    
    // 2. Complex nested structure
    let profile_fields = vec![
        Arc::new(Field::new("first_name", DataType::Utf8, false)),
        Arc::new(Field::new("last_name", DataType::Utf8, false)),
        Arc::new(Field::new("avatar_url", DataType::Utf8, true)),
    ];
    let profile_field = Field::new_struct("profile", profile_fields, true);
    fields.push(profile_field);
    
    // 3. List of tags
    let tag_item = Arc::new(Field::new("tag", DataType::Utf8, false));
    let tags_field = Field::new_list("tags", tag_item, true);
    fields.push(tags_field);
    
    // 4. Map for preferences
    let pref_key = Arc::new(Field::new("key", DataType::Utf8, false));
    let pref_value = Arc::new(Field::new("value", DataType::Utf8, true));
    let preferences_field = Field::new_map("preferences", "entries", pref_key, pref_value, false, true);
    fields.push(preferences_field);
    
    // 5. Union for flexible value storage
    let type_ids = vec![0, 1, 2];
    let union_fields = vec![
        Arc::new(Field::new("string_val", DataType::Utf8, true)),
        Arc::new(Field::new("number_val", DataType::Float64, true)),
        Arc::new(Field::new("bool_val", DataType::Boolean, true)),
    ];
    let dynamic_value_field = Field::new_union("dynamic_value", type_ids, union_fields, UnionMode::Sparse);
    fields.push(dynamic_value_field);
    
    // 6. Temporal fields
    let created_at_field = Field::new("created_at", DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())), false);
    fields.push(created_at_field);
    
    // 7. Test field operations
    for (i, field) in fields.iter().enumerate() {
        println!("Field {}: {} ({:?})", i, field.name(), field.data_type());
        println!("  Nullable: {}", field.is_nullable());
        println!("  Metadata entries: {}", field.metadata().len());
        println!("  Size: {} bytes", field.size());
    }
    
    // 8. Test field merging
    let mut base_field = Field::new("status", DataType::Utf8, false);
    let nullable_field = Field::new("status", DataType::Utf8, true);
    base_field.try_merge(&nullable_field)?;
    
    println!("Merged field nullability: {}", base_field.is_nullable());
    fields.push(base_field);
    
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_basic_field() {
        let field = create_basic_field();
        assert_eq!(field.name(), "user_id");
        assert_eq!(field.data_type(), &DataType::Int64);
        assert!(!field.is_nullable());
    }
    
    #[test]
    fn test_create_list_field() {
        let field = create_list_field();
        assert_eq!(field.name(), "item");
        assert!(field.is_nullable());
    }
    
    #[test]
    fn test_create_dictionary_field() {
        let field = create_dictionary_field();
        assert_eq!(field.name(), "status");
        assert!(field.is_nullable());
        match field.data_type() {
            DataType::Dictionary(key, value) => {
                assert_eq!(**key, DataType::Int32);
                assert_eq!(**value, DataType::Utf8);
            }
            _ => panic!("Expected dictionary type"),
        }
    }
    
    #[test]
    fn test_create_struct_field() {
        let field = create_struct_field();
        assert_eq!(field.name(), "address");
        assert!(field.is_nullable());
        match field.data_type() {
            DataType::Struct(fields) => {
                assert_eq!(fields.len(), 4);
            }
            _ => panic!("Expected struct type"),
        }
    }
    
    #[test]
    fn test_field_metadata_operations() {
        let field = field_metadata_operations();
        assert!(!field.metadata().is_empty());
    }
    
    #[test]
    fn test_field_name_operations() {
        let field = field_name_operations();
        assert_eq!(field.name(), "final_name");
    }
    
    #[test]
    fn test_field_data_type_operations() {
        let field = field_data_type_operations();
        assert_eq!(field.data_type(), &DataType::Float64);
    }
    
    #[test]
    fn test_nullability_operations() {
        let field = nullability_operations();
        assert!(!field.is_nullable());
    }
    
    #[test]
    fn test_field_merge_operations() {
        let result = field_merge_operations();
        assert!(result.is_ok());
        let field = result.unwrap();
        assert!(field.is_nullable());
        assert!(!field.metadata().is_empty());
    }
    
    #[test]
    fn test_field_contains_operations() {
        let _field = field_contains_operations();
        // Test passes if no panic occurs
    }
    
    #[test]
    fn test_field_size_operations() {
        let field = field_size_operations();
        assert!(field.size() > 0);
    }
    
    #[test]
    fn test_temporal_field_examples() {
        let fields = temporal_field_examples();
        assert!(!fields.is_empty());
        assert!(fields.len() >= 10);
    }
    
    #[test]
    fn test_numeric_field_examples() {
        let fields = numeric_field_examples();
        assert!(!fields.is_empty());
        assert!(fields.len() >= 10);
    }
    
    #[test]
    fn test_string_binary_field_examples() {
        let fields = string_binary_field_examples();
        assert!(!fields.is_empty());
        assert!(fields.len() >= 7);
    }
    
    #[test]
    fn test_comprehensive_field_example() {
        let result = comprehensive_field_example();
        assert!(result.is_ok());
        let fields = result.unwrap();
        assert!(fields.len() >= 6);
    }
}