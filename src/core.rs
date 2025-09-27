//! Core functionality and initialization for the WASM Arrow library.
//!
//! This module provides fundamental initialization and lifecycle management.

use wasm_bindgen::prelude::*;
use crate::{VersionInfo, console_log};
use std::sync::Once;

// Global initialization flag
static CORE_INIT: Once = Once::new();

/// Core initialization function for the WASM module
/// 
/// Sets up the WASM linear memory, initializes panic hooks,
/// and prepares the plugin registry.
pub fn init_core() {
    CORE_INIT.call_once(|| {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        
        console_log!("Arrow WASM core initialized");
    });
}

/// Memory allocation utilities for WASM
pub mod memory {
    use crate::error::{ArrowError, ErrorCode};

    /// Allocate aligned memory in WASM linear memory
    pub fn allocate_aligned(size: usize, alignment: usize) -> Result<usize, ArrowError> {
        // For now, use simple alignment - this can be optimized later
        if !alignment.is_power_of_two() {
            return Err(ArrowError::new(
                ErrorCode::InvalidFormat,
                "Alignment must be a power of two"
            ));
        }

        // TODO: Implement proper aligned allocation
        // For now, return a placeholder
        Ok(0)
    }

    /// Check if a pointer is properly aligned
    pub fn is_aligned(ptr: usize, alignment: usize) -> bool {
        ptr % alignment == 0
    }

    /// Validate memory bounds
    pub fn validate_bounds(ptr: usize, len: usize, total_size: usize) -> Result<(), ArrowError> {
        if ptr.saturating_add(len) > total_size {
            return Err(ArrowError::new(
                ErrorCode::OutOfBounds,
                "Memory access out of bounds"
            ));
        }
        Ok(())
    }
}

/// Handle registry for managing Arrow objects in WASM memory
pub mod handles {
    use std::collections::HashMap;
    use std::sync::{Mutex, Arc};
    use once_cell::sync::Lazy;

    /// Handle ID type
    pub type HandleId = u32;

    /// Handle registry for managing object lifetimes
    pub struct HandleRegistry<T> {
        next_id: HandleId,
        objects: HashMap<HandleId, Arc<T>>,
    }

    impl<T> HandleRegistry<T> {
        pub fn new() -> Self {
            HandleRegistry {
                next_id: 1, // Start from 1, 0 is reserved for null
                objects: HashMap::new(),
            }
        }

        /// Insert an object and return its handle ID
        pub fn insert(&mut self, object: T) -> HandleId {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            self.objects.insert(id, Arc::new(object));
            id
        }

        /// Get a reference to an object by handle ID
        pub fn get(&self, id: HandleId) -> Option<Arc<T>> {
            self.objects.get(&id).cloned()
        }

        /// Remove an object by handle ID
        pub fn remove(&mut self, id: HandleId) -> Option<Arc<T>> {
            self.objects.remove(&id)
        }

        /// Get the number of registered objects
        pub fn len(&self) -> usize {
            self.objects.len()
        }
    }

    impl<T> Default for HandleRegistry<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    // Global registries for different object types
    pub type TableRegistry = HandleRegistry<arrow_array::RecordBatch>;
    pub type ColumnRegistry = HandleRegistry<Box<dyn arrow_array::Array>>;
    pub type SchemaRegistry = HandleRegistry<arrow_schema::Schema>;

    // Lazy initialization of global registries
    static TABLE_REGISTRY: Lazy<Mutex<TableRegistry>> = 
        Lazy::new(|| Mutex::new(TableRegistry::new()));
    
    static COLUMN_REGISTRY: Lazy<Mutex<ColumnRegistry>> = 
        Lazy::new(|| Mutex::new(ColumnRegistry::new()));
        
    static SCHEMA_REGISTRY: Lazy<Mutex<SchemaRegistry>> = 
        Lazy::new(|| Mutex::new(SchemaRegistry::new()));

    /// Get access to the global table registry
    pub fn with_table_registry<F, R>(f: F) -> R 
    where 
        F: FnOnce(&mut TableRegistry) -> R,
    {
        let mut registry = TABLE_REGISTRY.lock().unwrap();
        f(&mut registry)
    }

    /// Get access to the global column registry  
    pub fn with_column_registry<F, R>(f: F) -> R
    where
        F: FnOnce(&mut ColumnRegistry) -> R,
    {
        let mut registry = COLUMN_REGISTRY.lock().unwrap();
        f(&mut registry)
    }

    /// Get access to the global schema registry
    pub fn with_schema_registry<F, R>(f: F) -> R
    where
        F: FnOnce(&mut SchemaRegistry) -> R,
    {
        let mut registry = SCHEMA_REGISTRY.lock().unwrap();
        f(&mut registry)
    }

    /// Statistics about handle usage
    #[derive(Debug)]
    pub struct HandleStats {
        pub tables: usize,
        pub columns: usize, 
        pub schemas: usize,
    }

    /// Get statistics about current handle usage
    pub fn get_handle_stats() -> HandleStats {
        HandleStats {
            tables: with_table_registry(|r| r.len()),
            columns: with_column_registry(|r| r.len()),
            schemas: with_schema_registry(|r| r.len()),
        }
    }
}

/// Plugin registry for managing extensions
pub mod plugin_registry {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    /// Plugin trait for extensions
    pub trait Plugin: Send + Sync {
        fn name(&self) -> &str;
        fn version(&self) -> &str;
        fn initialize(&self) -> Result<(), crate::error::ArrowError>;
        fn dispose(&self);
    }

    /// Plugin registry (placeholder implementation for now)
    /// TODO: Implement proper plugin system when needed
    
    /// Register a plugin (placeholder)
    pub fn register_plugin(_plugin: Box<dyn Plugin>) -> Result<(), crate::error::ArrowError> {
        // TODO: Implement plugin registration
        Ok(())
    }

    /// Unregister a plugin (placeholder)
    pub fn unregister_plugin(_name: &str) -> Option<Box<dyn Plugin>> {
        // TODO: Implement plugin unregistration
        None
    }

    /// Get a plugin by name (placeholder)
    pub fn get_plugin(_name: &str) -> Option<String> {
        // TODO: Implement plugin lookup
        None
    }

    /// List all registered plugins (placeholder)
    pub fn list_plugins() -> Vec<String> {
        // TODO: Implement plugin listing
        Vec::new()
    }
}

// Re-export commonly used items
pub use handles::{HandleId, with_table_registry, with_column_registry, with_schema_registry};
pub use memory::{allocate_aligned, is_aligned, validate_bounds};
pub use plugin_registry::{Plugin, register_plugin, unregister_plugin, get_plugin, list_plugins};