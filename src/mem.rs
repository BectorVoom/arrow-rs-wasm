//! Memory management for Arrow tables in WASM
//! 
//! This module manages the lifecycle of Arrow tables in WASM memory,
//! providing safe handle-based access from JavaScript.

use crate::errors::{CoreError, CoreResult};
use arrow_array::RecordBatch;
use arrow_schema::{Schema, SchemaRef};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Opaque handle id returned to JS/TS. Small integer type for stable mapping.
pub type TableHandle = u32;

/// Managed table containing Arrow data and metadata
#[derive(Debug, Clone)]
pub struct ManagedTable {
    /// Schema of the table
    pub schema: SchemaRef,
    /// Record batches that make up the table
    pub batches: Vec<RecordBatch>,
    /// Custom metadata associated with the table
    pub metadata: HashMap<String, String>,
    /// Creation timestamp for debugging
    pub created_at: std::time::SystemTime,
}

impl ManagedTable {
    /// Create a new managed table
    pub fn new(schema: SchemaRef, batches: Vec<RecordBatch>) -> Self {
        Self {
            schema,
            batches,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now(),
        }
    }
    
    /// Create a managed table with metadata
    pub fn with_metadata(
        schema: SchemaRef, 
        batches: Vec<RecordBatch>, 
        metadata: HashMap<String, String>
    ) -> Self {
        Self {
            schema,
            batches,
            metadata,
            created_at: std::time::SystemTime::now(),
        }
    }
    
    /// Get the total number of rows across all batches
    pub fn num_rows(&self) -> usize {
        self.batches.iter().map(|batch| batch.num_rows()).sum()
    }
    
    /// Get the number of columns
    pub fn num_columns(&self) -> usize {
        self.schema.fields().len()
    }
    
    /// Get the number of batches
    pub fn num_batches(&self) -> usize {
        self.batches.len()
    }
}

/// Schema description returned to JS (serde-serializable for debug)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaSummary {
    pub columns: Vec<ColumnSummary>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnSummary {
    pub name: String,
    pub arrow_type: String, // e.g. "Int64", "Utf8", "LargeBinary", "Geometry<...>"
    pub nullable: bool,
}

impl SchemaSummary {
    /// Create a schema summary from an Arrow schema
    pub fn from_schema(schema: &Schema) -> Self {
        let columns = schema
            .fields()
            .iter()
            .map(|field| ColumnSummary {
                name: field.name().clone(),
                arrow_type: format!("{:?}", field.data_type()),
                nullable: field.is_nullable(),
            })
            .collect();
            
        let metadata = schema.metadata().clone();
            
        Self { columns, metadata }
    }
}

/// Global table registry
static TABLE_REGISTRY: Lazy<Mutex<TableRegistry>> = Lazy::new(|| {
    Mutex::new(TableRegistry::new())
});

/// Registry for managing table handles and their associated data
struct TableRegistry {
    /// Map from handle to managed table
    tables: HashMap<TableHandle, ManagedTable>,
    /// Next available handle ID
    next_handle: TableHandle,
}

impl TableRegistry {
    /// Create a new empty registry
    fn new() -> Self {
        Self {
            tables: HashMap::new(),
            next_handle: 1, // Start from 1, reserve 0 for invalid handle
        }
    }
    
    /// Register a new table and return its handle
    fn register_table(&mut self, table: ManagedTable) -> TableHandle {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.tables.insert(handle, table);
        handle
    }
    
    /// Get a table by handle
    fn get_table(&self, handle: TableHandle) -> CoreResult<&ManagedTable> {
        self.tables
            .get(&handle)
            .ok_or_else(|| CoreError::invalid_handle(handle))
    }
    
    /// Get a mutable table by handle
    fn get_table_mut(&mut self, handle: TableHandle) -> CoreResult<&mut ManagedTable> {
        self.tables
            .get_mut(&handle)
            .ok_or_else(|| CoreError::invalid_handle(handle))
    }
    
    /// Remove a table by handle
    fn remove_table(&mut self, handle: TableHandle) -> CoreResult<ManagedTable> {
        self.tables
            .remove(&handle)
            .ok_or_else(|| CoreError::invalid_handle(handle))
    }
    
    /// Get the number of registered tables
    fn len(&self) -> usize {
        self.tables.len()
    }
    
    /// Check if a handle exists
    fn contains_handle(&self, handle: TableHandle) -> bool {
        self.tables.contains_key(&handle)
    }
    
    /// Clear all tables (for cleanup)
    fn clear(&mut self) {
        self.tables.clear();
    }
}

/// Create a table from an Arrow schema and record batches
pub fn create_table_from_batches(
    schema: SchemaRef, 
    batches: Vec<RecordBatch>
) -> CoreResult<TableHandle> {
    // Validate that all batches have the same schema
    for batch in &batches {
        if batch.schema() != schema {
            return Err(CoreError::schema("Batch schema does not match table schema"));
        }
    }
    
    let table = ManagedTable::new(schema, batches);
    
    let mut registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    Ok(registry.register_table(table))
}

/// Create a table with custom metadata
pub fn create_table_with_metadata(
    schema: SchemaRef, 
    batches: Vec<RecordBatch>,
    metadata: HashMap<String, String>
) -> CoreResult<TableHandle> {
    // Validate that all batches have the same schema
    for batch in &batches {
        if batch.schema() != schema {
            return Err(CoreError::schema("Batch schema does not match table schema"));
        }
    }
    
    let table = ManagedTable::with_metadata(schema, batches, metadata);
    
    let mut registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    Ok(registry.register_table(table))
}

/// Get a table's schema summary as JSON string
pub fn get_table_schema_summary(handle: TableHandle) -> CoreResult<String> {
    let registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    let table = registry.get_table(handle)?;
    let summary = SchemaSummary::from_schema(&table.schema);
    
    serde_json::to_string(&summary)
        .map_err(|e| CoreError::other(format!("Failed to serialize schema: {}", e)))
}

/// Get a table by handle (for internal use)
pub fn get_table(handle: TableHandle) -> CoreResult<ManagedTable> {
    let registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    Ok(registry.get_table(handle)?.clone())
}

/// Get table metadata
pub fn get_table_metadata(handle: TableHandle) -> CoreResult<HashMap<String, String>> {
    let registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    let table = registry.get_table(handle)?;
    Ok(table.metadata.clone())
}

/// Free a table handle and release its memory
pub fn free_table(handle: TableHandle) -> CoreResult<()> {
    let mut registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    registry.remove_table(handle)?;
    Ok(())
}

/// Check if a handle is valid
pub fn is_valid_handle(handle: TableHandle) -> bool {
    if let Ok(registry) = TABLE_REGISTRY.lock() {
        registry.contains_handle(handle)
    } else {
        false
    }
}

/// Get the number of currently registered tables
pub fn get_table_count() -> usize {
    if let Ok(registry) = TABLE_REGISTRY.lock() {
        registry.len()
    } else {
        0
    }
}

/// Clear all tables (for testing and cleanup)
pub fn clear_all_tables() {
    if let Ok(mut registry) = TABLE_REGISTRY.lock() {
        registry.clear();
    }
}

/// Get statistics about memory usage
#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryStats {
    pub active_tables: usize,
    pub total_rows: usize,
    pub total_batches: usize,
}

/// Get memory usage statistics
pub fn get_memory_stats() -> CoreResult<MemoryStats> {
    let registry = TABLE_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock registry: {}", e)))?;
    
    let mut total_rows = 0;
    let mut total_batches = 0;
    
    for table in registry.tables.values() {
        total_rows += table.num_rows();
        total_batches += table.num_batches();
    }
    
    Ok(MemoryStats {
        active_tables: registry.len(),
        total_rows,
        total_batches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::{Int32Array, StringArray};
    use arrow_schema::{DataType, Field};
    use std::sync::Arc;
    
    fn create_test_schema() -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
        ]))
    }
    
    fn create_test_batch(schema: SchemaRef) -> RecordBatch {
        let id_array = Arc::new(Int32Array::from(vec![1, 2, 3]));
        let name_array = Arc::new(StringArray::from(vec![
            Some("Alice"),
            Some("Bob"),
            None,
        ]));
        
        RecordBatch::try_new(schema, vec![id_array, name_array]).unwrap()
    }
    
    #[test]
    fn test_table_creation_and_retrieval() {
        clear_all_tables();
        
        let schema = create_test_schema();
        let batch = create_test_batch(schema.clone());
        
        let handle = create_table_from_batches(schema, vec![batch]).unwrap();
        assert!(is_valid_handle(handle));
        
        let table = get_table(handle).unwrap();
        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.num_columns(), 2);
        assert_eq!(table.num_batches(), 1);
        
        free_table(handle).unwrap();
        assert!(!is_valid_handle(handle));
    }
    
    #[test]
    fn test_schema_summary() {
        clear_all_tables();
        
        let schema = create_test_schema();
        let batch = create_test_batch(schema.clone());
        
        let handle = create_table_from_batches(schema, vec![batch]).unwrap();
        let summary_json = get_table_schema_summary(handle).unwrap();
        let summary: SchemaSummary = serde_json::from_str(&summary_json).unwrap();
        
        assert_eq!(summary.columns.len(), 2);
        assert_eq!(summary.columns[0].name, "id");
        assert_eq!(summary.columns[1].name, "name");
        assert!(!summary.columns[0].nullable);
        assert!(summary.columns[1].nullable);
        
        free_table(handle).unwrap();
    }
    
    #[test]
    fn test_invalid_handle() {
        clear_all_tables();
        
        let result = get_table(999);
        assert!(matches!(result, Err(CoreError::InvalidHandle(999))));
        
        let result = free_table(999);
        assert!(matches!(result, Err(CoreError::InvalidHandle(999))));
    }
    
    #[test]
    fn test_memory_stats() {
        clear_all_tables();
        
        let schema = create_test_schema();
        let batch = create_test_batch(schema.clone());
        
        let _handle1 = create_table_from_batches(schema.clone(), vec![batch.clone()]).unwrap();
        let _handle2 = create_table_from_batches(schema, vec![batch]).unwrap();
        
        let stats = get_memory_stats().unwrap();
        assert_eq!(stats.active_tables, 2);
        assert_eq!(stats.total_rows, 6); // 3 rows per table
        assert_eq!(stats.total_batches, 2);
        
        clear_all_tables();
        
        let stats = get_memory_stats().unwrap();
        assert_eq!(stats.active_tables, 0);
        assert_eq!(stats.total_rows, 0);
        assert_eq!(stats.total_batches, 0);
    }
}