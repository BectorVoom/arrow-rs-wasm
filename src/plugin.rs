//! Plugin architecture for Arrow WASM library
//! 
//! This module provides a minimal plugin system that allows registration
//! and validation of plugins for extending Arrow functionality.

use crate::errors::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use arrow_array::Array;

/// Macro for console logging in WASM
macro_rules! console_log {
    ($($t:tt)*) => (
        web_sys::console::log_1(&format!($($t)*).into())
    );
}

/// Plugin trait for extending Arrow functionality
pub trait ArrowPlugin: Send + Sync {
    /// A stable plugin ID, e.g., "io.arrow.plugin.geo.v1"
    fn plugin_id(&self) -> &'static str;
    
    /// Plugin name for display purposes
    fn plugin_name(&self) -> &'static str;
    
    /// Plugin version
    fn plugin_version(&self) -> &'static str;
    
    /// Validate a field type; return Ok(()) if supported
    fn validate_field(&self, field: &arrow_schema::Field) -> CoreResult<()>;
    
    /// Optional conversion helper invoked when reading/writing
    /// Not allowed to access JS memory directly; must operate within WASM
    fn on_read_column(
        &self,
        field: &arrow_schema::Field,
        array: &dyn arrow_array::Array,
    ) -> CoreResult<()>;
}

/// Plugin metadata for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub registered_at: std::time::SystemTime,
}

/// Global plugin registry
static PLUGIN_REGISTRY: Lazy<Mutex<PluginRegistry>> = Lazy::new(|| {
    Mutex::new(PluginRegistry::new())
});

/// Registry for managing plugins
struct PluginRegistry {
    /// Map from plugin ID to plugin trait object
    plugins: HashMap<String, Box<dyn ArrowPlugin>>,
    /// Plugin metadata
    metadata: HashMap<String, PluginMetadata>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    fn register(&mut self, plugin: Box<dyn ArrowPlugin>) -> CoreResult<()> {
        let id = plugin.plugin_id().to_string();
        
        if self.plugins.contains_key(&id) {
            return Err(CoreError::plugin(format!(
                "Plugin already registered: {}", id
            )));
        }
        
        let metadata = PluginMetadata {
            id: id.clone(),
            name: plugin.plugin_name().to_string(),
            version: plugin.plugin_version().to_string(),
            description: format!("Plugin: {}", plugin.plugin_name()),
            registered_at: std::time::SystemTime::now(),
        };
        
        self.metadata.insert(id.clone(), metadata);
        self.plugins.insert(id, plugin);
        
        Ok(())
    }
    
    fn get_plugin(&self, plugin_id: &str) -> Option<&dyn ArrowPlugin> {
        self.plugins.get(plugin_id).map(|p| p.as_ref())
    }
    
    fn get_metadata(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        self.metadata.get(plugin_id)
    }
    
    fn list_plugins(&self) -> Vec<&PluginMetadata> {
        self.metadata.values().collect()
    }

    
    fn is_registered(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }
    
    fn validate_plugin(&self, plugin_id: &str) -> CoreResult<()> {
        if self.plugins.contains_key(plugin_id) {
            Ok(())
        } else {
            Err(CoreError::plugin(format!(
                "Plugin not found: {}", plugin_id
            )))
        }
    }
}

/// Register a plugin instance
pub fn register_plugin_instance(plugin: Box<dyn ArrowPlugin>) -> CoreResult<()> {
    let mut registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    registry.register(plugin)
}

/// Register a plugin by its exported registration name
/// 
/// This implementation supports a flexible plugin factory system that allows
/// plugins to be registered dynamically at runtime while maintaining WASM compatibility.
pub fn register_plugin(plugin_id: &str) -> CoreResult<()> {
    // Validate plugin ID format
    if plugin_id.is_empty() {
        return Err(CoreError::plugin("Plugin ID cannot be empty"));
    }
    
    if !plugin_id.contains('.') {
        return Err(CoreError::plugin(
            "Plugin ID should follow format 'domain.plugin.name.version'"
        ));
    }
    
    // Check if plugin is already registered
    let registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    if registry.is_registered(plugin_id) {
        return Err(CoreError::plugin(format!("Plugin '{}' is already registered", plugin_id)));
    }
    drop(registry); // Release lock before potential plugin creation
    
    // Use the plugin factory system to create the plugin
    create_and_register_plugin(plugin_id)
}

/// Plugin factory function that creates plugins dynamically using the factory system
/// This allows for extensible plugin creation without hardcoding all types
fn create_and_register_plugin(plugin_id: &str) -> CoreResult<()> {
    // Extract plugin type from the plugin_id
    let plugin_type = extract_plugin_type(plugin_id)?;
    
    // Get available factories
    let factories = get_builtin_factories();
    
    // Find and use the appropriate factory
    if let Some(factory) = factories.get(plugin_type.as_str()) {
        console_log!("Creating plugin '{}' using factory for type '{}'", plugin_id, plugin_type);
        let plugin = factory.create_plugin(plugin_id)?;
        register_plugin_instance(plugin)
    } else {
        let available_types: Vec<&str> = factories.keys().cloned().collect();
        Err(CoreError::plugin(format!(
            "Unknown plugin type '{}' in plugin ID '{}'. Available types: {}", 
            plugin_type, 
            plugin_id,
            available_types.join(", ")
        )))
    }
}

/// Extract plugin type from plugin ID
/// Expected format: domain.plugin.TYPE.version (e.g., "io.arrow.plugin.geo.v1")
fn extract_plugin_type(plugin_id: &str) -> CoreResult<String> {
    let parts: Vec<&str> = plugin_id.split('.').collect();
    
    // Handle different ID formats
    match parts.len() {
        2 => {
            // Simple format: "geometry" or "geo.v1"
            Ok(parts[0].to_string())
        }
        3 => {
            // Format: "plugin.geo.v1" or "arrow.geometry.v1"
            Ok(parts[1].to_string())
        }
        4 => {
            // Format: "io.arrow.geo.v1"
            Ok(parts[2].to_string())
        }
        5 => {
            // Full format: "io.arrow.plugin.geo.v1"
            Ok(parts[3].to_string())
        }
        _ => {
            Err(CoreError::plugin(format!(
                "Invalid plugin ID format: '{}'. Expected formats: 'type', 'type.version', 'domain.type.version', etc.",
                plugin_id
            )))
        }
    }
}

/// Plugin factory trait for creating plugins dynamically
/// This allows for future extensibility without modifying core code
pub trait PluginFactory: Send + Sync {
    /// Create a plugin instance with the given ID
    fn create_plugin(&self, plugin_id: &str) -> CoreResult<Box<dyn ArrowPlugin>>;
    
    /// Get the plugin type this factory handles
    fn plugin_type(&self) -> &'static str;
}

/// Geometry plugin factory
pub struct GeometryPluginFactory;

impl PluginFactory for GeometryPluginFactory {
    fn create_plugin(&self, plugin_id: &str) -> CoreResult<Box<dyn ArrowPlugin>> {
        Ok(Box::new(GeometryPlugin::new_with_id(plugin_id)))
    }
    
    fn plugin_type(&self) -> &'static str {
        "geo"
    }
}

/// Demo plugin factory
pub struct DemoPluginFactory;

impl PluginFactory for DemoPluginFactory {
    fn create_plugin(&self, plugin_id: &str) -> CoreResult<Box<dyn ArrowPlugin>> {
        Ok(Box::new(DummyPlugin::new(plugin_id)))
    }
    
    fn plugin_type(&self) -> &'static str {
        "demo"
    }
}

/// Enhanced plugin registration that could support external factories
/// For WASM environment, we use a static registry but designed for extensibility
fn get_builtin_factories() -> HashMap<&'static str, Box<dyn PluginFactory>> {
    let mut factories = HashMap::new();
    factories.insert("geo", Box::new(GeometryPluginFactory) as Box<dyn PluginFactory>);
    factories.insert("geometry", Box::new(GeometryPluginFactory) as Box<dyn PluginFactory>);
    factories.insert("demo", Box::new(DemoPluginFactory) as Box<dyn PluginFactory>);
    factories.insert("dummy", Box::new(DemoPluginFactory) as Box<dyn PluginFactory>);
    factories
}

/// Get list of available plugin types that can be dynamically loaded
pub fn get_available_plugin_types() -> CoreResult<Vec<String>> {
    let factories = get_builtin_factories();
    let types: Vec<String> = factories.keys().map(|k| k.to_string()).collect();
    Ok(types)
}

/// Enhanced plugin discovery - list all plugins that could be registered
pub fn discover_available_plugins() -> CoreResult<Vec<String>> {
    let mut available_plugins = Vec::new();
    
    // Add well-known plugin IDs for each type
    let factories = get_builtin_factories();
    
    for plugin_type in factories.keys() {
        match *plugin_type {
            "geo" | "geometry" => {
                available_plugins.push("io.arrow.plugin.geo.v1".to_string());
                available_plugins.push("geometry".to_string());
            }
            "demo" | "dummy" => {
                available_plugins.push("io.arrow.plugin.demo.v1".to_string());
                available_plugins.push("demo".to_string());
            }
            _ => {
                available_plugins.push(format!("io.arrow.plugin.{}.v1", plugin_type));
            }
        }
    }
    
    Ok(available_plugins)
}

/// Validate that a plugin ID format is correct and plugin type is available
pub fn validate_plugin_id_format(plugin_id: &str) -> CoreResult<String> {
    let plugin_type = extract_plugin_type(plugin_id)?;
    let factories = get_builtin_factories();
    
    if !factories.contains_key(plugin_type.as_str()) {
        let available_types: Vec<&str> = factories.keys().cloned().collect();
        return Err(CoreError::plugin(format!(
            "Plugin type '{}' is not available. Available types: {}",
            plugin_type,
            available_types.join(", ")
        )));
    }
    
    Ok(plugin_type)
}

// Future extension point for WASM dynamic loading
// This would be implemented when WASM supports dynamic module loading
#[allow(dead_code)]
fn load_external_plugin_module(_plugin_id: &str, _wasm_bytes: &[u8]) -> CoreResult<Box<dyn ArrowPlugin>> {
    // Placeholder for future dynamic WASM plugin loading
    // This would:
    // 1. Load the WASM module from bytes
    // 2. Validate the module exports the required plugin interface
    // 3. Instantiate the plugin and return it
    Err(CoreError::plugin("Dynamic WASM plugin loading not yet supported"))
}

/// Validate a plugin by confirming its expected registration signature
pub fn validate_plugin(plugin_id: &str) -> CoreResult<()> {
    let registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    registry.validate_plugin(plugin_id)
}

/// Get information about all registered plugins
pub fn get_plugin_info() -> CoreResult<Vec<PluginMetadata>> {
    let registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    Ok(registry.list_plugins().into_iter().cloned().collect())
}

/// Get information about a specific plugin
pub fn get_plugin_metadata(plugin_id: &str) -> CoreResult<PluginMetadata> {
    let registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    registry
        .get_metadata(plugin_id)
        .cloned()
        .ok_or_else(|| CoreError::plugin(format!("Plugin not found: {}", plugin_id)))
}

/// Validate a field using registered plugins
pub fn validate_field_with_plugins(field: &arrow_schema::Field) -> CoreResult<()> {
    let registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    // Try validation with each registered plugin
    for plugin in registry.plugins.values() {
        if let Ok(()) = plugin.validate_field(field) {
            return Ok(());
        }
    }
    
    // If no plugin can validate, it's still valid (core types)
    Ok(())
}

/// Process a column with registered plugins
pub fn process_column_with_plugins(
    field: &arrow_schema::Field,
    array: &dyn arrow_array::Array,
) -> CoreResult<()> {
    let registry = PLUGIN_REGISTRY
        .lock()
        .map_err(|e| CoreError::memory(format!("Failed to lock plugin registry: {}", e)))?;
    
    // Process with each applicable plugin
    for plugin in registry.plugins.values() {
        if plugin.validate_field(field).is_ok() {
            plugin.on_read_column(field, array)?;
        }
    }
    
    Ok(())
}

/// Dummy plugin implementation for testing
struct DummyPlugin {
    id: String,
    name: String,
    version: String,
}

/// Geometry Plugin implementation for Arrow WASM library
/// Handles geometry columns encoded as LargeBinary with well-known metadata
pub struct GeometryPlugin;

impl GeometryPlugin {
    /// Create a new geometry plugin instance
    pub fn new() -> Self {
        Self
    }
    
    /// Create a new geometry plugin instance with specific plugin ID
    /// This supports the dynamic plugin factory system
    pub fn new_with_id(_plugin_id: &str) -> Self {
        // For now, GeometryPlugin doesn't need ID-specific configuration
        // In future, different geometry plugin variants could use the ID
        // to configure specific behaviors (e.g., different CRS handling)
        Self
    }
    
    /// Check if a field represents a geometry column
    fn is_geometry_field(field: &arrow_schema::Field) -> bool {
        // Check if the field is LargeBinary with geometry metadata
        matches!(field.data_type(), arrow_schema::DataType::LargeBinary) &&
        field.metadata().get("ARROW:extension:name").map_or(false, |name| {
            name.starts_with("geo.") || name == "geometry" || name == "wkb"
        })
    }
    
    /// Extract geometry type from field metadata
    fn get_geometry_type(field: &arrow_schema::Field) -> Option<String> {
        field.metadata().get("ARROW:extension:name").cloned()
    }
    
    /// Validate Well-Known Binary (WKB) geometry data
    fn validate_wkb_data(data: &[u8]) -> CoreResult<GeometryInfo> {
        if data.len() < 9 {
            return Err(CoreError::plugin("Invalid WKB: too short".to_string()));
        }
        
        // Parse WKB header (simplified)
        let byte_order = data[0];
        if byte_order != 1 && byte_order != 0 {
            return Err(CoreError::plugin("Invalid WKB: bad byte order".to_string()));
        }
        
        // Extract geometry type (bytes 1-4, little-endian assumed)
        let geom_type = if byte_order == 1 {
            u32::from_le_bytes([data[1], data[2], data[3], data[4]])
        } else {
            u32::from_be_bytes([data[1], data[2], data[3], data[4]])
        };
        
        let geometry_type = match geom_type & 0xFF {
            1 => "Point",
            2 => "LineString", 
            3 => "Polygon",
            4 => "MultiPoint",
            5 => "MultiLineString",
            6 => "MultiPolygon",
            7 => "GeometryCollection",
            _ => "Unknown",
        };
        
        Ok(GeometryInfo {
            geometry_type: geometry_type.to_string(),
            dimension: if geom_type & 0x80000000 != 0 { 3 } else { 2 },
            srid: None, // Could be extracted from extended WKB
        })
    }
}

impl ArrowPlugin for GeometryPlugin {
    fn plugin_id(&self) -> &'static str {
        "io.arrow.plugin.geo.v1"
    }
    
    fn plugin_name(&self) -> &'static str {
        "Geometry Plugin"
    }
    
    fn plugin_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn validate_field(&self, field: &arrow_schema::Field) -> CoreResult<()> {
        if !Self::is_geometry_field(field) {
            return Err(CoreError::plugin(format!(
                "Field '{}' is not a valid geometry field", field.name()
            )));
        }
        
        // Validate required metadata
        let extension_name = field.metadata().get("ARROW:extension:name")
            .ok_or_else(|| CoreError::plugin("Missing ARROW:extension:name metadata".to_string()))?;
        
        let valid_geometry_types = ["geo.point", "geo.linestring", "geo.polygon", 
                                  "geo.multipoint", "geo.multilinestring", "geo.multipolygon",
                                  "geometry", "wkb"];
        
        if !valid_geometry_types.contains(&extension_name.as_str()) {
            return Err(CoreError::plugin(format!(
                "Unsupported geometry type: {}", extension_name
            )));
        }
        
        // Validate that data type is LargeBinary
        if !matches!(field.data_type(), arrow_schema::DataType::LargeBinary) {
            return Err(CoreError::plugin(
                "Geometry fields must use LargeBinary data type".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn on_read_column(
        &self,
        field: &arrow_schema::Field,
        array: &dyn arrow_array::Array,
    ) -> CoreResult<()> {
        if !Self::is_geometry_field(field) {
            return Ok(()); // Not a geometry field, skip
        }
        
        // Cast to LargeBinaryArray for processing
        let binary_array = array
            .as_any()
            .downcast_ref::<arrow_array::LargeBinaryArray>()
            .ok_or_else(|| CoreError::plugin("Expected LargeBinaryArray for geometry field".to_string()))?;
        
        let geometry_type = Self::get_geometry_type(field)
            .unwrap_or_else(|| "unknown".to_string());
        
        console_log!("Processing geometry column '{}' of type '{}' with {} values", 
                    field.name(), geometry_type, binary_array.len());
        
        // Validate a sample of the geometry data
        let sample_size = std::cmp::min(5, binary_array.len());
        let mut valid_geometries = 0;
        
        for i in 0..sample_size {
            let wkb_data = binary_array.value(i);
            if wkb_data.len() > 0 {
                match Self::validate_wkb_data(wkb_data) {
                    Ok(geom_info) => {
                        console_log!("  Geometry {}: {} ({}D)", 
                                    i, geom_info.geometry_type, geom_info.dimension);
                        valid_geometries += 1;
                    }
                    Err(e) => {
                        console_log!("  Geometry {}: Invalid WKB - {}", i, e);
                    }
                }
            }
        }
        
        console_log!("Validated {}/{} geometries in sample", valid_geometries, sample_size);
        
        Ok(())
    }
}

/// Information about a parsed geometry
#[derive(Debug, Clone)]
pub struct GeometryInfo {
    pub geometry_type: String,
    pub dimension: u8,
    pub srid: Option<u32>,
}

/// Register the geometry plugin
pub fn register_geometry_plugin() -> CoreResult<()> {
    let plugin = GeometryPlugin::new();
    register_plugin_instance(Box::new(plugin))?;
    console_log!("Geometry plugin registered successfully");
    Ok(())
}

/// Create a sample geometry field for testing
pub fn create_sample_geometry_field(name: &str, geometry_type: &str) -> arrow_schema::Field {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("ARROW:extension:name".to_string(), geometry_type.to_string());
    metadata.insert("ARROW:extension:metadata".to_string(), "{}".to_string());
    
    arrow_schema::Field::new(name, arrow_schema::DataType::LargeBinary, true)
        .with_metadata(metadata)
}

/// Create sample WKB point data for testing
pub fn create_sample_point_wkb(x: f64, y: f64) -> Vec<u8> {
    let mut wkb = Vec::new();
    
    // Byte order (little endian)
    wkb.push(1);
    
    // Geometry type (Point = 1)
    wkb.extend_from_slice(&1u32.to_le_bytes());
    
    // X coordinate
    wkb.extend_from_slice(&x.to_le_bytes());
    
    // Y coordinate 
    wkb.extend_from_slice(&y.to_le_bytes());
    
    wkb
}

/// Example function demonstrating geometry plugin usage
pub fn demonstrate_geometry_plugin() -> CoreResult<()> {
    console_log!("=== Geometry Plugin Demonstration ===");
    
    // Register the geometry plugin
    register_geometry_plugin()?;
    
    // Create a sample geometry field
    let point_field = create_sample_geometry_field("location", "geo.point");
    
    // Validate the field with the plugin
    validate_field_with_plugins(&point_field)?;
    console_log!("✅ Point field validation passed");
    
    // Create sample WKB data
    let point1_wkb = create_sample_point_wkb(1.0, 2.0);
    let point2_wkb = create_sample_point_wkb(3.0, 4.0);
    let point3_wkb = create_sample_point_wkb(5.0, 6.0);
    
    // Create a LargeBinaryArray with the sample data
    let binary_builder = arrow_array::builder::LargeBinaryBuilder::new();
    let mut builder = binary_builder;
    
    builder.append_value(&point1_wkb);
    builder.append_value(&point2_wkb);
    builder.append_value(&point3_wkb);
    
    let binary_array = builder.finish();
    
    // Process the column with plugins
    process_column_with_plugins(&point_field, &binary_array)?;
    
    console_log!("✅ Geometry plugin demonstration completed successfully");
    
    Ok(())
}

impl DummyPlugin {
    fn new(plugin_id: &str) -> Self {
        Self {
            id: plugin_id.to_string(),
            name: format!("Dummy Plugin ({})", plugin_id),
            version: "0.1.0".to_string(),
        }
    }
}

impl ArrowPlugin for DummyPlugin {
    fn plugin_id(&self) -> &'static str {
        // Note: This is a limitation - we need a static string
        // In a real implementation, this would be handled differently
        "io.arrow.plugin.dummy.v1"
    }
    
    fn plugin_name(&self) -> &'static str {
        "Dummy Plugin"
    }
    
    fn plugin_version(&self) -> &'static str {
        "0.1.0"
    }
    
    fn validate_field(&self, _field: &arrow_schema::Field) -> CoreResult<()> {
        // Dummy plugin accepts all fields
        Ok(())
    }
    
    fn on_read_column(
        &self,
        _field: &arrow_schema::Field,
        _array: &dyn arrow_array::Array,
    ) -> CoreResult<()> {
        // Dummy plugin does nothing
        Ok(())
    }
}

/// Clear all registered plugins (for testing)
pub fn clear_all_plugins() {
    if let Ok(mut registry) = PLUGIN_REGISTRY.lock() {
        registry.plugins.clear();
        registry.metadata.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_schema::{DataType, Field};
    
    #[test]
    fn test_plugin_registration() {
        clear_all_plugins();
        
        // Test registering a plugin
        let result = register_plugin("io.test.plugin.v1");
        assert!(result.is_ok());
        
        // Test validation
        let validation = validate_plugin("io.arrow.plugin.dummy.v1");
        assert!(validation.is_ok());
        
        // Test invalid plugin
        let invalid = validate_plugin("nonexistent.plugin");
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_plugin_info() {
        clear_all_plugins();
        
        register_plugin("io.test.plugin1.v1").unwrap();
        register_plugin("io.test.plugin2.v1").unwrap();
        
        let info = get_plugin_info().unwrap();
        assert_eq!(info.len(), 2);
        
        // Check that we can get metadata for specific plugins
        let metadata = get_plugin_metadata("io.arrow.plugin.dummy.v1");
        assert!(metadata.is_ok());
    }
    
    #[test]
    fn test_field_validation() {
        clear_all_plugins();
        
        register_plugin("io.test.plugin.v1").unwrap();
        
        let field = Field::new("test", DataType::Int32, false);
        let result = validate_field_with_plugins(&field);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_plugin_id() {
        let result = register_plugin("");
        assert!(result.is_err());
        
        let result = register_plugin("invalid_plugin_id");
        assert!(result.is_err());
    }
}