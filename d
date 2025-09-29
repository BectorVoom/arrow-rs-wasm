# Arrow WASM API Specification

**Version:** 1.0.0
**Date:** 2025-01-27
**Status:** Draft

## 1. Overview

This document specifies the public API for the Arrow WASM library, a Rust → WebAssembly implementation providing zero-copy Arrow buffer management with a type-safe JavaScript/TypeScript façade.

### 1.1 Core Features

- Zero-copy transfer of Arrow buffers between WASM and JavaScript
- Support for Arrow IPC, Feather, and Parquet file formats
- LZ4 compression support for IPC blocks
- Plugin extensibility model for custom column types
- Memory ownership model with WASM as authoritative owner

### 1.2 Architecture Overview

```
┌─────────────────────────────────────┐
│     JavaScript/TypeScript Layer      │
│  (Type-safe façade, Promise-based)   │
└───────────────┬─────────────────────┘
                │
┌───────────────┴─────────────────────┐
│        WASM-bindgen Bridge          │
└───────────────┬─────────────────────┘
                │
┌───────────────┴─────────────────────┐
│         Rust Core (WASM)            │
│  (Memory owner, Arrow operations)    │
└─────────────────────────────────────┘
```

## 2. TypeScript API

### 2.1 Core Types

```typescript
// Result type for error handling without exceptions
export type Ok<T> = { ok: true; value: T };
export type Err = { ok: false; error: string };
export type Result<T> = Ok<T> | Err;

// Opaque handle to a table managed by WASM
export type TableHandle = number;

// Column metadata
export interface ColumnInfo {
  name: string;
  arrowType: string;  // e.g., "Int64", "Utf8", "LargeBinary"
  nullable: boolean;
}

// Table schema summary
export interface SchemaSummary {
  columns: ColumnInfo[];
}

// WASM initialization options
export interface WasmInitOptions {
  enableConsoleLogs?: boolean;
}

// IPC write options
export interface IpcWriteOptions {
  enableLz4?: boolean;
  alignment?: number;  // Buffer alignment (default: 8)
  writeFooter?: boolean;  // Include footer (default: true)
}
```

### 2.2 Initialization

#### `initWasm`
Initializes the WASM module. Must be called once before any other operations.

**Signature:**
```typescript
function initWasm(
  bytes?: ArrayBuffer,
  opts?: WasmInitOptions
): Promise<Result<void>>
```

**Parameters:**
- `bytes` - Optional WASM module bytes (auto-loaded if not provided)
- `opts` - Initialization options

**Returns:** Promise resolving to success or error

**Example:**
```typescript
const result = await initWasm();
if (!result.ok) {
  console.error('Initialization failed:', result.error);
}
```

### 2.3 Table Operations

#### `readTableFromArrayBuffer`
Reads an Arrow/Feather/Parquet file from an ArrayBuffer.

**Signature:**
```typescript
function readTableFromArrayBuffer(
  data: ArrayBuffer
): Promise<Result<TableHandle>>
```

**Parameters:**
- `data` - File contents as ArrayBuffer

**Returns:** Promise resolving to a table handle or error

**Example:**
```typescript
const buffer = await fetch('data.arrow').then(r => r.arrayBuffer());
const result = await readTableFromArrayBuffer(buffer);
if (result.ok) {
  const handle = result.value;
  // Use handle for further operations
}
```

#### `getSchemaSummary`
Retrieves schema information for a table.

**Signature:**
```typescript
function getSchemaSummary(
  handle: TableHandle
): Promise<Result<SchemaSummary>>
```

**Parameters:**
- `handle` - Table handle obtained from read operations

**Returns:** Promise resolving to schema summary or error

#### `writeTableToIpc`
Writes a table to Arrow IPC format with optional LZ4 compression.

**Signature:**
```typescript
function writeTableToIpc(
  handle: TableHandle,
  enableLz4: boolean
): Promise<Result<ArrayBuffer>>
```

**Parameters:**
- `handle` - Table handle
- `enableLz4` - Enable LZ4 compression for IPC blocks

**Returns:** Promise resolving to IPC bytes as ArrayBuffer or error

#### `writeTableToParquet`
Writes a table to Parquet format.

**Signature:**
```typescript
function writeTableToParquet(
  handle: TableHandle
): Promise<Result<ArrayBuffer>>
```

**Parameters:**
- `handle` - Table handle

**Returns:** Promise resolving to Parquet bytes as ArrayBuffer or error

#### `freeTable`
Releases WASM memory associated with a table handle.

**Signature:**
```typescript
function freeTable(
  handle: TableHandle
): Promise<Result<void>>
```

**Parameters:**
- `handle` - Table handle to release

**Returns:** Promise resolving to success or error

**Important:** Always call this method when done with a table to prevent memory leaks.

### 2.4 Plugin Management

#### `registerPlugin`
Registers a plugin by its identifier.

**Signature:**
```typescript
function registerPlugin(
  pluginId: string
): Promise<Result<void>>
```

**Parameters:**
- `pluginId` - Unique plugin identifier (e.g., "io.arrow.plugin.geo.v1")

**Returns:** Promise resolving to success or error

#### `validatePlugin`
Validates a registered plugin.

**Signature:**
```typescript
function validatePlugin(
  pluginId: string
): Promise<Result<void>>
```

**Parameters:**
- `pluginId` - Plugin identifier to validate

**Returns:** Promise resolving to success or error

### 2.5 Advanced Operations

#### `writeTableWithOptions`
Writes a table with detailed configuration options.

**Signature:**
```typescript
function writeTableWithOptions(
  handle: TableHandle,
  options: IpcWriteOptions
): Promise<Result<ArrayBuffer>>
```

**Parameters:**
- `handle` - Table handle
- `options` - Detailed write configuration

**Returns:** Promise resolving to output bytes or error

## 3. Rust WASM API

### 3.1 Core Types

```rust
/// Opaque handle for table references
pub type TableHandle = u32;

/// Schema information exposed to JavaScript
#[derive(Serialize, Deserialize)]
pub struct SchemaSummary {
    pub columns: Vec<ColumnSummary>,
}

#[derive(Serialize, Deserialize)]
pub struct ColumnSummary {
    pub name: String,
    pub arrow_type: String,
    pub nullable: bool,
}

/// Core error types
#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Invalid handle: {0}")]
    InvalidHandle(u32),
    #[error("Other: {0}")]
    Other(String),
}
```

### 3.2 Exported Functions

These functions are exposed via `wasm-bindgen` and should not be called directly from JavaScript. Use the TypeScript façade instead.

#### Memory Management

```rust
#[wasm_bindgen]
pub fn create_table_from_ipc(
    ipc_data: &[u8]
) -> Result<TableHandle, CoreError>
```

```rust
#[wasm_bindgen]
pub fn free_table(
    handle: TableHandle
) -> Result<(), CoreError>
```

```rust
#[wasm_bindgen]
pub fn table_schema_summary(
    handle: TableHandle
) -> Result<String, CoreError>
```

#### File I/O

```rust
#[wasm_bindgen]
pub fn read_table_from_bytes(
    bytes: &[u8]
) -> Result<TableHandle, CoreError>
```

```rust
#[wasm_bindgen]
pub fn write_table_to_ipc(
    handle: TableHandle,
    enable_lz4: bool
) -> Result<Vec<u8>, CoreError>
```

```rust
#[wasm_bindgen]
pub fn write_table_to_parquet(
    handle: TableHandle
) -> Result<Vec<u8>, CoreError>
```

#### IPC Configuration

```rust
pub fn default_lz4_ipc_options() -> IpcWriteOptions
```

```rust
pub fn enable_lz4_on_options(
    opts: &mut IpcWriteOptions,
    lz4: bool
)
```

## 4. Plugin API

### 4.1 Plugin Trait

Plugins must implement the following trait:

```rust
pub trait ArrowPlugin: Sync + Send {
    /// Unique plugin identifier
    fn plugin_id(&self) -> &'static str;

    /// Validate field compatibility
    fn validate_field(
        &self,
        field: &arrow::datatypes::Field
    ) -> Result<(), CoreError>;

    /// Process column during read operations
    fn on_read_column(
        &self,
        field: &arrow::datatypes::Field,
        array: &dyn ArrayRef
    ) -> Result<(), CoreError>;
}
```

### 4.2 Plugin Registration

Each plugin crate must export a registration function:

```rust
#[wasm_bindgen]
pub fn register_plugin() -> Result<(), CoreError> {
    // Plugin registration logic
}
```

## 5. Error Handling

### 5.1 Error Philosophy

- No exceptions thrown in public API
- All operations return `Result<T>` types
- Errors contain descriptive messages
- No use of `try`/`catch` in examples

### 5.2 Error Codes

| Code | Description | Recovery Action |
|------|------------|-----------------|
| `Io` | File I/O error | Check file path/permissions |
| `Ipc` | Arrow IPC format error | Validate file format |
| `Plugin` | Plugin operation failed | Check plugin compatibility |
| `InvalidHandle` | Invalid table handle | Ensure handle is valid |
| `Other` | General error | See error message |

## 6. Memory Model

### 6.1 Ownership Rules

1. **WASM owns all Arrow buffers** - JavaScript never directly accesses raw memory
2. **Handle-based operations** - All operations use opaque numeric handles
3. **Explicit lifecycle** - Tables must be explicitly freed via `freeTable()`
4. **Zero-copy semantics** - Data remains in WASM unless explicitly exported

### 6.2 Memory Safety Guarantees

- No raw pointers exposed to JavaScript
- All memory operations are bounds-checked
- Automatic cleanup on WASM module unload
- Handle validation on every operation

## 7. Usage Examples

### 7.1 Basic Read/Write Flow

```typescript
// Initialize WASM
const initResult = await initWasm();
if (!initResult.ok) {
  console.error('Failed to initialize:', initResult.error);
  return;
}

// Read Arrow file
const fileBuffer = await fetch('data.arrow').then(r => r.arrayBuffer());
const readResult = await readTableFromArrayBuffer(fileBuffer);
if (!readResult.ok) {
  console.error('Failed to read file:', readResult.error);
  return;
}

const handle = readResult.value;

// Get schema
const schemaResult = await getSchemaSummary(handle);
if (schemaResult.ok) {
  console.log('Columns:', schemaResult.value.columns);
}

// Write with LZ4 compression
const writeResult = await writeTableToIpc(handle, true);
if (writeResult.ok) {
  const compressedData = writeResult.value;
  // Use compressed data
}

// Clean up
await freeTable(handle);
```

### 7.2 Plugin Usage

```typescript
// Register geometry plugin
const pluginResult = await registerPlugin('io.arrow.plugin.geo.v1');
if (!pluginResult.ok) {
  console.error('Plugin registration failed:', pluginResult.error);
  return;
}

// Read file with geometry columns
const geoFileBuffer = await fetch('geodata.arrow').then(r => r.arrayBuffer());
const tableResult = await readTableFromArrayBuffer(geoFileBuffer);
// Plugin automatically processes geometry columns
```

## 8. Performance Considerations

### 8.1 Best Practices

1. **Minimize copies** - Keep data in WASM as long as possible
2. **Batch operations** - Process multiple columns/rows in single calls
3. **Use LZ4 wisely** - Balance compression ratio vs. performance
4. **Free resources** - Always call `freeTable()` when done


## 9. Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|---------|---------|--------|------|
| Basic WASM | ✓ | ✓ | ✓ | ✓ |
| SharedArrayBuffer | ✓* | ✓* | ✓* | ✓* |
| Large ArrayBuffers | ✓ | ✓ | ✓ | ✓ |

\* Requires appropriate CORS headers

## 10. Versioning

This API follows semantic versioning:

- **Major version**: Breaking API changes
- **Minor version**: Backward-compatible features
- **Patch version**: Backward-compatible fixes

Current version: 1.0.0


## Appendix A: Arrow Type Support

All standard Arrow types are supported:
- Primitive: Int8-64, UInt8-64, Float32/64, Boolean
- Variable-length: Utf8, LargeUtf8, Binary, LargeBinary
- Temporal: Date32/64, Time32/64, Timestamp, Duration
- Nested: List, LargeList, FixedSizeList, Struct, Union
- Dictionary encoded types

## Appendix B: Plugin Interface Version

Current plugin interface version: 1.0

Plugins must specify compatibility in their manifest.
