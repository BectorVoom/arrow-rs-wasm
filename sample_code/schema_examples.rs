//! # Arrow Schema Examples
//!
//! This module demonstrates the usage of `arrow_schema::Schema` and related structures.
//! The Schema represents the metadata of an ordered sequence of fields.

use arrow_schema::{DataType, Field, Schema, ArrowError};
use std::collections::HashMap;
use std::sync::Arc;

/// Demonstrates creating an empty Schema
/// 
/// # Use Cases
/// - Initializing schemas that will be built incrementally
/// - Creating placeholder schemas for dynamic table structures
/// - Starting point for schema builders
pub fn create_empty_schema() -> Schema {
    let schema = Schema::empty();
    println!("Empty schema: {}", schema);
    schema
}

/// Demonstrates creating a new Schema from fields
/// 
/// # Use Cases
/// - Defining table structures for data processing
/// - Creating schemas for CSV/JSON data ingestion
/// - Establishing data contracts between systems
pub fn create_schema_from_fields() -> Schema {
    let field_a = Field::new("id", DataType::Int64, false);
    let field_b = Field::new("name", DataType::Utf8, true);
    let field_c = Field::new("active", DataType::Boolean, false);
    
    let schema = Schema::new(vec![field_a, field_b, field_c]);
    println!("Schema from fields: {}", schema);
    schema
}

/// Demonstrates creating a Schema with metadata
/// 
/// # Use Cases
/// - Adding table-level metadata like descriptions, ownership
/// - Storing data lineage information
/// - Including data quality constraints
/// - Adding versioning information
pub fn create_schema_with_metadata() -> Schema {
    let field_a = Field::new("user_id", DataType::Int64, false);
    let field_b = Field::new("email", DataType::Utf8, false);
    
    let mut metadata = HashMap::new();
    metadata.insert("table_name".to_string(), "users".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("description".to_string(), "User account information".to_string());
    metadata.insert("owner".to_string(), "data_team".to_string());
    
    let schema = Schema::new_with_metadata(vec![field_a, field_b], metadata);
    println!("Schema with metadata: {}", schema);
    schema
}

/// Demonstrates adding metadata to an existing Schema
/// 
/// # Use Cases
/// - Updating schema metadata without recreating the schema
/// - Adding runtime metadata like processing timestamps
/// - Chaining metadata operations in builder patterns
pub fn add_metadata_to_schema() -> Schema {
    let field = Field::new("data", DataType::Utf8, true);
    let schema = Schema::new(vec![field]);
    
    let mut metadata = HashMap::new();
    metadata.insert("processed_at".to_string(), "2024-01-01T00:00:00Z".to_string());
    metadata.insert("processor".to_string(), "etl_pipeline".to_string());
    
    let schema_with_metadata = schema.with_metadata(metadata);
    println!("Schema with added metadata: {}", schema_with_metadata);
    schema_with_metadata
}

/// Demonstrates projecting a Schema to a subset of columns
/// 
/// # Use Cases
/// - Selecting specific columns for query optimization
/// - Creating views with subset of original table columns
/// - Data privacy: excluding sensitive columns
/// - Reducing memory usage by working with fewer columns
pub fn project_schema() -> Result<Schema, ArrowError> {
    let fields = vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, true),
        Field::new("email", DataType::Utf8, true),
        Field::new("age", DataType::Int32, true),
        Field::new("salary", DataType::Float64, true),
    ];
    let schema = Schema::new(fields);
    
    // Project to only id, name, and email (indices 0, 1, 2)
    let projected_schema = schema.project(&[0, 1, 2])?;
    println!("Original schema: {}", schema);
    println!("Projected schema: {}", projected_schema);
    
    Ok(projected_schema)
}

/// Demonstrates merging multiple Schemas
/// 
/// # Use Cases
/// - Combining schemas from different data sources
/// - Union operations on tables
/// - Merging schemas when joining datasets
/// - Schema evolution: adding fields from newer versions
pub fn merge_schemas() -> Result<Schema, ArrowError> {
    let schema1 = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]);
    
    let schema2 = Schema::new(vec![
        Field::new("id", DataType::Int64, true), // more nullable
        Field::new("name", DataType::Utf8, false),
        Field::new("email", DataType::Utf8, true), // new field
    ]);
    
    let merged_schema = Schema::try_merge(vec![schema1, schema2])?;
    println!("Merged schema: {}", merged_schema);
    
    Ok(merged_schema)
}

/// Demonstrates accessing fields from a Schema
/// 
/// # Use Cases
/// - Introspecting schema structure
/// - Validating expected fields exist
/// - Dynamic column processing
/// - Building data transformation pipelines
pub fn access_schema_fields() -> Schema {
    let schema = Schema::new(vec![
        Field::new("timestamp", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), false),
        Field::new("sensor_id", DataType::Utf8, false),
        Field::new("temperature", DataType::Float32, true),
        Field::new("humidity", DataType::Float32, true),
    ]);
    
    // Access all fields
    let fields = schema.fields();
    println!("All fields: {:?}", fields);
    
    // Access specific field by index
    let first_field = schema.field(0);
    println!("First field: {}", first_field);
    
    // Access field by name
    match schema.field_with_name("temperature") {
        Ok(temp_field) => println!("Temperature field: {}", temp_field),
        Err(e) => println!("Field not found: {}", e),
    }
    
    schema
}

/// Demonstrates getting flattened fields (including nested)
/// 
/// # Use Cases
/// - Analyzing nested schema structures
/// - Flattening complex data for analytics
/// - Schema documentation and visualization
/// - Validation of deeply nested structures
pub fn get_flattened_fields() -> Schema {
    use arrow_schema::Fields;
    
    let inner_struct_fields = Fields::from(vec![
        Arc::new(Field::new("street", DataType::Utf8, false)),
        Arc::new(Field::new("city", DataType::Utf8, false)),
        Arc::new(Field::new("zip", DataType::Utf8, true)),
    ]);
    
    let list_field = Arc::new(Field::new("item", DataType::Utf8, true));
    
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("address", DataType::Struct(inner_struct_fields), true),
        Field::new("tags", DataType::List(list_field), true),
    ]);
    
    let flattened = schema.flattened_fields();
    println!("Flattened fields ({} total):", flattened.len());
    for (i, field) in flattened.iter().enumerate() {
        println!("  {}: {}", i, field.name());
    }
    
    schema
}

/// Demonstrates finding field index by name
/// 
/// # Use Cases
/// - Column indexing for array operations
/// - Mapping field names to positions
/// - Dynamic field access in processing pipelines
/// - Validation that required fields exist
pub fn find_field_index() -> Result<(), ArrowError> {
    let schema = Schema::new(vec![
        Field::new("user_id", DataType::Int64, false),
        Field::new("username", DataType::Utf8, false),
        Field::new("created_at", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), false),
    ]);
    
    // Find index by name
    let username_index = schema.index_of("username")?;
    println!("Username field is at index: {}", username_index);
    
    // Try to find non-existent field
    match schema.index_of("non_existent") {
        Ok(index) => println!("Found at index: {}", index),
        Err(e) => println!("Field not found: {}", e),
    }
    
    Ok(())
}

/// Demonstrates accessing Schema metadata
/// 
/// # Use Cases
/// - Reading table-level configuration
/// - Extracting data lineage information
/// - Accessing processing hints and optimization flags
/// - Reading schema versioning information
pub fn access_schema_metadata() -> Schema {
    let field = Field::new("data", DataType::Utf8, true);
    
    let mut metadata = HashMap::new();
    metadata.insert("format_version".to_string(), "2.0".to_string());
    metadata.insert("encoding".to_string(), "utf8".to_string());
    metadata.insert("compression".to_string(), "snappy".to_string());
    metadata.insert("created_by".to_string(), "arrow_schema_examples".to_string());
    
    let schema = Schema::new_with_metadata(vec![field], metadata);
    
    // Access metadata
    let metadata = schema.metadata();
    println!("Schema metadata:");
    for (key, value) in metadata.iter() {
        println!("  {}: {}", key, value);
    }
    
    schema
}

/// Demonstrates normalizing a Schema (flattening nested structures)
/// 
/// # Use Cases
/// - Converting nested data to flat tables for SQL databases
/// - Preparing data for systems that don't support nested types
/// - Creating flat views of complex nested data
/// - Data warehouse ETL operations
pub fn normalize_schema() -> Result<Schema, ArrowError> {
    use arrow_schema::Fields;
    
    let address_fields = Fields::from(vec![
        Arc::new(Field::new("street", DataType::Utf8, false)),
        Arc::new(Field::new("city", DataType::Utf8, false)),
        Arc::new(Field::new("country", DataType::Utf8, false)),
    ]);
    
    let nested_schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("address", DataType::Struct(address_fields), true),
    ]);
    
    println!("Original nested schema: {}", nested_schema);
    
    // Normalize with dot separator
    let normalized = nested_schema.normalize(".", None)?;
    println!("Normalized schema: {}", normalized);
    
    // Normalize with underscore separator and max depth
    let normalized_limited = nested_schema.normalize("_", Some(1))?;
    println!("Normalized with limit: {}", normalized_limited);
    
    Ok(normalized)
}

/// Demonstrates finding column by name with index
/// 
/// # Use Cases
/// - Combined field lookup and indexing
/// - Efficient field access in processing loops
/// - Building field mapping dictionaries
/// - Validation with position information
pub fn find_column_with_name() -> Schema {
    let schema = Schema::new(vec![
        Field::new("product_id", DataType::Utf8, false),
        Field::new("category", DataType::Utf8, true),
        Field::new("price", DataType::Decimal128(10, 2), false),
        Field::new("in_stock", DataType::Boolean, false),
    ]);
    
    // Find column by name
    if let Some((index, field)) = schema.column_with_name("price") {
        println!("Found field '{}' at index {}: {}", field.name(), index, field.data_type());
    } else {
        println!("Field 'price' not found");
    }
    
    // Try with non-existent field
    if let Some((index, field)) = schema.column_with_name("discount") {
        println!("Found field '{}' at index {}: {}", field.name(), index, field.data_type());
    } else {
        println!("Field 'discount' not found");
    }
    
    schema
}

/// Demonstrates checking if one Schema contains another
/// 
/// # Use Cases
/// - Schema compatibility validation
/// - Checking if data conforms to expected schema
/// - Schema evolution validation
/// - API compatibility checking
pub fn check_schema_contains() -> Schema {
    let base_schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, true), // nullable
        Field::new("email", DataType::Utf8, false),
        Field::new("age", DataType::Int32, true),
    ]);
    
    let subset_schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false), // less nullable
        Field::new("email", DataType::Utf8, false),
    ]);
    
    let incompatible_schema = Schema::new(vec![
        Field::new("id", DataType::Utf8, false), // different type
        Field::new("name", DataType::Utf8, false),
    ]);
    
    // Check if base schema contains subset
    if base_schema.contains(&subset_schema) {
        println!("Base schema contains subset schema");
    } else {
        println!("Base schema does not contain subset schema");
    }
    
    // Check incompatible schema
    if base_schema.contains(&incompatible_schema) {
        println!("Base schema contains incompatible schema");
    } else {
        println!("Base schema does not contain incompatible schema");
    }
    
    base_schema
}

/// Demonstrates deprecated fields_with_dict_id method
/// 
/// # Use Cases
/// - Legacy dictionary field management (deprecated)
/// - Migration from old dictionary APIs
/// - Backward compatibility with older Arrow versions
#[allow(deprecated)]
pub fn find_fields_with_dict_id() -> Schema {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new_dict("category", DataType::Dictionary(
            Box::new(DataType::Int32), 
            Box::new(DataType::Utf8)
        ), true, 1, false),
        Field::new_dict("status", DataType::Dictionary(
            Box::new(DataType::Int32), 
            Box::new(DataType::Utf8)
        ), true, 1, false),
    ]);
    
    // Find fields with dictionary ID 1
    let dict_fields = schema.fields_with_dict_id(1);
    println!("Fields with dictionary ID 1: {}", dict_fields.len());
    for field in dict_fields {
        println!("  {}: {:?}", field.name(), field.data_type());
    }
    
    schema
}

/// Comprehensive example demonstrating Schema usage patterns
/// 
/// # Use Cases
/// - Real-world schema management workflows
/// - Building schema transformation pipelines
/// - Creating reusable schema components
/// - Demonstrating best practices
pub fn comprehensive_schema_example() -> Result<(), ArrowError> {
    println!("=== Comprehensive Schema Example ===");
    
    // 1. Create base schema
    let base_schema = Schema::new(vec![
        Field::new("transaction_id", DataType::Utf8, false),
        Field::new("user_id", DataType::Int64, false),
        Field::new("amount", DataType::Decimal128(12, 2), false),
        Field::new("currency", DataType::Utf8, false),
        Field::new("timestamp", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, Some("UTC".into())), false),
    ]);
    
    // 2. Add metadata
    let mut metadata = HashMap::new();
    metadata.insert("table".to_string(), "transactions".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("owner".to_string(), "finance_team".to_string());
    
    let schema_with_meta = base_schema.clone().with_metadata(metadata);
    
    // 3. Create projection for reporting
    let reporting_schema = schema_with_meta.project(&[1, 2, 4])?; // user_id, amount, timestamp
    
    // 4. Create extended schema for analytics
    let analytics_fields = vec![
        Field::new("transaction_id", DataType::Utf8, false),
        Field::new("user_id", DataType::Int64, false),
        Field::new("amount", DataType::Decimal128(12, 2), false),
        Field::new("currency", DataType::Utf8, false),
        Field::new("timestamp", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, Some("UTC".into())), false),
        Field::new("merchant_id", DataType::Utf8, true),
        Field::new("category", DataType::Utf8, true),
    ];
    let analytics_schema = Schema::new(analytics_fields);
    
    // 5. Merge schemas
    let merged_schema = Schema::try_merge(vec![schema_with_meta, analytics_schema])?;
    
    println!("Base schema fields: {}", base_schema.fields().len());
    println!("Reporting schema fields: {}", reporting_schema.fields().len());
    println!("Merged schema fields: {}", merged_schema.fields().len());
    
    // 6. Validate schema compatibility
    let test_schema = Schema::new(vec![
        Field::new("transaction_id", DataType::Utf8, false),
        Field::new("user_id", DataType::Int64, true), // more nullable
        Field::new("amount", DataType::Decimal128(12, 2), false),
    ]);
    
    if merged_schema.contains(&test_schema) {
        println!("Test schema is compatible with merged schema");
    } else {
        println!("Test schema is not compatible with merged schema");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_empty_schema() {
        let schema = create_empty_schema();
        assert_eq!(schema.fields().len(), 0);
    }
    
    #[test]
    fn test_create_schema_from_fields() {
        let schema = create_schema_from_fields();
        assert_eq!(schema.fields().len(), 3);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "name");
        assert_eq!(schema.field(2).name(), "active");
    }
    
    #[test]
    fn test_create_schema_with_metadata() {
        let schema = create_schema_with_metadata();
        assert_eq!(schema.fields().len(), 2);
        assert!(!schema.metadata().is_empty());
        assert_eq!(schema.metadata().get("table_name"), Some(&"users".to_string()));
    }
    
    #[test]
    fn test_project_schema() {
        let result = project_schema();
        assert!(result.is_ok());
        let projected = result.unwrap();
        assert_eq!(projected.fields().len(), 3);
    }
    
    #[test]
    fn test_merge_schemas() {
        let result = merge_schemas();
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.fields().len(), 3); // id, name, email
    }
    
    #[test]
    fn test_find_field_index() {
        let result = find_field_index();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_normalize_schema() {
        let result = normalize_schema();
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert!(normalized.fields().len() > 3); // Should have flattened fields
    }
    
    #[test]
    fn test_comprehensive_example() {
        let result = comprehensive_schema_example();
        assert!(result.is_ok());
    }
}