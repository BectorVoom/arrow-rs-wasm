# WASM-Arrow Library API Definition Document

## 1. Executive Summary

This document defines the Application Programming Interface (API) for the **wasm-arrow** library, which enables reading, writing, and various operations on Apache Arrow format files in WebAssembly environments. The library provides a zero-copy, type-safe interface between Rust/WebAssembly and JavaScript/TypeScript, with support for modular design and extensible plugin architecture.

## 2. Architecture Overview

### 2.1 Core Principles

1. **Modular Design**: The library shall be organized into distinct modules for different functionalities
2. **Plugin Architecture**: Extensible system allowing future additions, particularly for geometry column support
3. **Zero-Copy Data Transfer**: Data moves between WASM and JavaScript without duplication
4. **WASM-Side Memory Management**: All memory allocation and deallocation handled within WebAssembly
5. **Type Safety**: Full TypeScript type definitions ensuring compile-time safety

### 2.2 Module Structure

```
wasm-arrow/
├── core/           # Core functionality for Arrow operations
├── io/             # File reading and writing operations
├── table/          # Table manipulation and operations
├── schema/         # Schema definition and management
├── compute/        # Computational operations on arrays
├── plugin/         # Plugin system interface
└── types/          # Type definitions and conversions
```

## 3. Core Module API

### 3.1 Initialization

The core module provides fundamental initialization and lifecycle management for the library.

#### 3.1.1 `initialize()`

**Description**: Initializes the WASM module and sets up the memory allocator.

**TypeScript Signature**:
```typescript
function initialize(): Promise<void>
```

**Returns**: A Promise that resolves when initialization is complete.

**Business Logic**:
- Sets up the WASM linear memory
- Initializes the panic hook for error handling
- Prepares the plugin registry

#### 3.1.2 `getVersion()`

**Description**: Returns the library version information.

**TypeScript Signature**:
```typescript
function getVersion(): VersionInfo

interface VersionInfo {
  major: number;
  minor: number;
  patch: number;
  arrow_version: string;
}
```

## 4. Table Module API

### 4.1 Table Creation and Loading

#### 4.1.1 `tableFromIPC()`

**Description**: Creates a Table instance from IPC-formatted Arrow data.

**TypeScript Signature**:
```typescript
function tableFromIPC(buffer: ArrayBuffer): Table
```

**Parameters**:
- `buffer`: ArrayBuffer containing Arrow IPC format data

**Returns**: A Table instance

**Business Logic**:
1. Validates the buffer contains valid Arrow IPC format
2. Parses the schema from the IPC metadata
3. Creates internal column arrays without copying data
4. Returns a Table handle referencing WASM memory

**Implementation Requirements**:
- Data remains in WASM linear memory
- JavaScript receives only a handle/pointer
- Zero-copy operation

#### 4.1.2 `tableFromJSON()`

**Description**: Creates a Table from JSON data with schema inference.

**TypeScript Signature**:
```typescript
function tableFromJSON(
  data: Record<string, unknown>[],
  schema?: Schema
): Table
```

**Parameters**:
- `data`: Array of objects representing rows
- `schema`: Optional schema specification

**Business Logic**:
1. If schema not provided, infers from data
2. Validates data against schema
3. Converts JavaScript values to Arrow arrays
4. Constructs Table in WASM memory

### 4.2 Table Operations

#### 4.2.1 `Table` Class

**TypeScript Interface**:
```typescript
interface Table {
  readonly numRows: number;
  readonly numColumns: number;
  readonly schema: Schema;

  getColumn(name: string): Column;
  getColumnAt(index: number): Column;
  slice(offset: number, length: number): Table;
  select(columns: string[]): Table;
  filter(predicate: (row: Row) => boolean): Table;
  toArray(): Record<string, unknown>[];
  toIPC(options?: WriteOptions): ArrayBuffer;
  dispose(): void;
}
```

**Key Methods**:

##### `getColumn(name: string): Column`
- Returns a Column reference by name
- Throws if column doesn't exist
- Column data remains in WASM memory

##### `slice(offset: number, length: number): Table`
- Creates a zero-copy slice of the table
- Original data unchanged
- Returns new Table handle

##### `filter(predicate: (row: Row) => boolean): Table`
- Filters rows based on predicate
- Creates new Table with filtered data
- Predicate executed in JavaScript

##### `toIPC(options?: WriteOptions): ArrayBuffer`
- Serializes Table to Arrow IPC format
- Uses IpcWriteOptions configuration
- Returns ArrayBuffer with IPC data

##### `dispose(): void`
- Explicitly releases WASM memory
- Must be called to prevent memory leaks
- Invalidates the Table handle

### 4.3 Row Interface

**TypeScript Interface**:
```typescript
interface Row {
  get(column: string): unknown;
  getAt(index: number): unknown;
  toObject(): Record<string, unknown>;
}
```

**Implementation Note**: Row is a virtual interface - data is accessed directly from columnar storage without materializing full rows.

## 5. Schema Module API

### 5.1 Schema Definition

#### 5.1.1 `Schema` Interface

**TypeScript Interface**:
```typescript
interface Schema {
  readonly fields: Field[];
  readonly metadata: Map<string, string>;

  getField(name: string): Field;
  getFieldIndex(name: string): number;
  equals(other: Schema): boolean;
  toJSON(): SchemaJSON;
}
```

#### 5.1.2 `Field` Interface

**TypeScript Interface**:
```typescript
interface Field {
  readonly name: string;
  readonly dataType: DataType;
  readonly nullable: boolean;
  readonly metadata: Map<string, string>;
}
```

### 5.2 Data Types

**TypeScript Type Definitions**:
```typescript
type DataType =
  | { type: "null" }
  | { type: "bool" }
  | { type: "int8" }
  | { type: "int16" }
  | { type: "int32" }
  | { type: "int64" }
  | { type: "uint8" }
  | { type: "uint16" }
  | { type: "uint32" }
  | { type: "uint64" }
  | { type: "float32" }
  | { type: "float64" }
  | { type: "utf8" }
  | { type: "binary" }
  | { type: "date32" }
  | { type: "date64" }
  | { type: "timestamp"; unit: TimeUnit; timezone?: string }
  | { type: "list"; elementType: DataType }
  | { type: "struct"; fields: Field[] }
  | { type: "dictionary"; indexType: DataType; valueType: DataType }
  | { type: "decimal128"; precision: number; scale: number }
  | { type: "decimal256"; precision: number; scale: number };

type TimeUnit = "second" | "millisecond" | "microsecond" | "nanosecond";
```

## 6. IO Module API

### 6.1 File Operations

#### 6.1.1 `readFile()`

**Description**: Reads an Arrow file from provided data.

**TypeScript Signature**:
```typescript
function readFile(data: ArrayBuffer): Promise<Table>
```

**Business Logic**:
1. Validates Arrow file format (magic bytes)
2. Parses file footer for metadata
3. Loads record batches into memory
4. Constructs Table from batches

#### 6.1.2 `writeFile()`

**Description**: Writes a Table to Arrow file format.

**TypeScript Signature**:
```typescript
function writeFile(table: Table, options?: WriteOptions): Promise<ArrayBuffer>
```

### 6.2 Write Options

#### 6.2.1 `WriteOptions` Interface

**TypeScript Interface**:
```typescript
interface WriteOptions {
  compression?: CompressionType;
  alignment?: number;
  metadataVersion?: MetadataVersion;
  dictionaryHandling?: DictionaryHandling;
  metadata?: Record<string, string>;
}

type CompressionType = "none" | "lz4" | "zstd";
type MetadataVersion = "V4" | "V5";
type DictionaryHandling = "replace" | "delta";
```

**Implementation Requirements**:
- Maps directly to Rust's `IpcWriteOptions`
- Default alignment: 8 bytes
- Default version: V5
- Default compression: none

## 7. Column Module API

### 7.1 Column Interface

**TypeScript Interface**:
```typescript
interface Column {
  readonly name: string;
  readonly dataType: DataType;
  readonly length: number;
  readonly nullCount: number;

  get(index: number): unknown;
  getValue(index: number): unknown;
  isNull(index: number): boolean;
  isValid(index: number): boolean;
  slice(offset: number, length: number): Column;
  toArray(): unknown[];
  statistics(): ColumnStatistics;
}

interface ColumnStatistics {
  min?: unknown;
  max?: unknown;
  nullCount: number;
  distinctCount?: number;
}
```

### 7.2 Array Builders

**TypeScript Interface**:
```typescript
interface ArrayBuilder<T> {
  append(value: T): void;
  appendNull(): void;
  appendValues(values: T[]): void;
  finish(): Column;
  clear(): void;
}

function createBuilder(dataType: DataType, capacity?: number): ArrayBuilder<unknown>;
```

## 8. Compute Module API

### 8.1 Aggregation Functions

**TypeScript Signatures**:
```typescript
function sum(column: Column): number;
function mean(column: Column): number;
function min(column: Column): unknown;
function max(column: Column): unknown;
function count(column: Column): number;
function countDistinct(column: Column): number;
```

### 8.2 Transformation Functions

**TypeScript Signatures**:
```typescript
function cast(column: Column, targetType: DataType): Column;
function take(column: Column, indices: number[]): Column;
function filter(column: Column, mask: boolean[]): Column;
function sort(column: Column, descending?: boolean): Column;
```

## 9. Plugin System API

### 9.1 Plugin Interface

**TypeScript Interface**:
```typescript
interface Plugin {
  readonly name: string;
  readonly version: string;
  readonly supportedTypes: DataType[];

  initialize(): Promise<void>;
  processColumn(column: Column): Column;
  dispose(): void;
}
```

### 9.2 Plugin Registration

**TypeScript Signatures**:
```typescript
function registerPlugin(plugin: Plugin): void;
function unregisterPlugin(name: string): void;
function getPlugin(name: string): Plugin | undefined;
function listPlugins(): string[];
```

### 9.3 Geometry Plugin (Future)

**Planned Interface**:
```typescript
interface GeometryPlugin extends Plugin {
  readonly name: "geometry";

  parseWKT(column: Column): GeometryColumn;
  parseWKB(column: Column): GeometryColumn;
  toGeoJSON(column: GeometryColumn): string;
}

interface GeometryColumn extends Column {
  readonly geometryType: GeometryType;
  readonly srid?: number;

  area(): Column;
  length(): Column;
  centroid(): GeometryColumn;
  buffer(distance: number): GeometryColumn;
}
```

## 10. Memory Management

### 10.1 Ownership Model

All data structures follow a clear ownership model:

1. **Tables own their data**: When a Table is created, it owns the underlying Arrow data
2. **Columns are references**: Columns reference data owned by Tables
3. **Slices are zero-copy**: Slicing operations create new references, not copies
4. **Explicit disposal**: Tables must be explicitly disposed to free WASM memory

### 10.2 Memory Handle System

**Internal Structure** (not exposed to JavaScript):
```rust
pub struct TableHandle {
    ptr: *mut Table,
    id: u32,
}

pub struct ColumnHandle {
    table_id: u32,
    column_index: usize,
}
```

JavaScript receives opaque numeric handles that map to WASM memory pointers.

## 11. Error Handling

### 11.1 Error Types

**TypeScript Definitions**:
```typescript
class ArrowError extends Error {
  readonly code: ErrorCode;
  readonly details?: string;
}

enum ErrorCode {
  InvalidFormat = "INVALID_FORMAT",
  SchemaMismatch = "SCHEMA_MISMATCH",
  OutOfBounds = "OUT_OF_BOUNDS",
  TypeMismatch = "TYPE_MISMATCH",
  MemoryError = "MEMORY_ERROR",
  IOError = "IO_ERROR",
  NotImplemented = "NOT_IMPLEMENTED",
}
```

### 11.2 Error Handling Pattern

Since `try/catch` is prohibited, errors are handled through:

1. **Result types** for operations that can fail:
```typescript
type Result<T> = { ok: true; value: T } | { ok: false; error: ArrowError };
```

2. **Error callbacks**:
```typescript
function onError(handler: (error: ArrowError) => void): void;
```

## 12. JavaScript/TypeScript Constraints

### 12.1 Prohibited Constructs

The following JavaScript/TypeScript constructs are **NOT** used:
- `this` keyword
- `any` type
- `is` type guards
- `try/catch` blocks

### 12.2 Alternative Patterns

#### Instead of `this`:
```typescript
// BAD
class Table {
  getData() { return this.data; }
}

// GOOD
interface Table {
  getData: () => unknown[];
}

function createTable(): Table {
  const data = [];
  return {
    getData: () => data
  };
}
```

#### Instead of `any`:
```typescript
// BAD
function process(data: any): void

// GOOD
function process(data: unknown): void
function process<T>(data: T): void
```

#### Instead of `is`:
```typescript
// BAD
function isTable(obj: unknown): obj is Table

// GOOD
function checkTable(obj: unknown): boolean
```

#### Instead of `try/catch`:
```typescript
// BAD
try {
  const table = readFile(data);
} catch (e) {
  console.error(e);
}

// GOOD
const result = readFile(data);
if (result.ok) {
  const table = result.value;
} else {
  console.error(result.error);
}
```

## 13. Usage Examples

### 13.1 Basic Table Loading

```javascript
import { tableFromIPC, initialize } from 'wasm-arrow';

// Initialize the library
await initialize();

// Load Arrow file
const response = await fetch('data.arrow');
const buffer = await response.arrayBuffer();
const table = tableFromIPC(buffer);

// Access table data
console.log(`Rows: ${table.numRows}, Columns: ${table.numColumns}`);

// Get column
const column = table.getColumn('temperature');
console.log(`Temperature range: ${column.statistics().min} - ${column.statistics().max}`);

// Convert to JavaScript array
const data = table.toArray();
console.table(data);

// Clean up
table.dispose();
```

### 13.2 Creating and Writing Tables

```javascript
import { tableFromJSON, writeFile } from 'wasm-arrow';

// Create table from JSON
const data = [
  { id: 1, name: 'Alice', value: 100 },
  { id: 2, name: 'Bob', value: 200 },
  { id: 3, name: 'Charlie', value: 300 }
];

const table = tableFromJSON(data);

// Write with compression
const options = {
  compression: 'lz4',
  metadata: {
    'created_by': 'wasm-arrow',
    'version': '1.0.0'
  }
};

const result = await writeFile(table, options);
if (result.ok) {
  const buffer = result.value;
  // Save buffer to file
} else {
  console.error(result.error);
}

table.dispose();
```

### 13.3 Table Operations

```javascript
// Filter and select
const filtered = table.filter(row => row.get('value') > 100);
const selected = filtered.select(['id', 'name']);

// Slice
const subset = selected.slice(0, 10);

// Compute aggregations
import { sum, mean } from 'wasm-arrow/compute';

const valueColumn = table.getColumn('value');
const total = sum(valueColumn);
const average = mean(valueColumn);

// Clean up all tables
subset.dispose();
selected.dispose();
filtered.dispose();
```

## 14. Build Configuration

### 14.1 Rust Crate Dependencies

As specified in requirements:
```toml
[dependencies]
arrow = { version = "56.1.0", default-features = false, features = ["ipc", "ipc_compression"] }
arrow-ipc = { version = "56.1.0", default-features = false, features = ["lz4"] }
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }
anyhow = "1.0"
thiserror = "1.0"
console_error_panic_hook = { version = "0.1", optional = true }
once_cell = "1.20"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 14.2 WebAssembly Build Target

```toml
[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "z"
lto = true
```

## 15. Testing Strategy

### 15.1 Unit Tests

Each module includes comprehensive unit tests:
- Schema validation
- Data type conversions
- Memory management
- Error conditions

### 15.2 Integration Tests

End-to-end tests covering:
- File reading and writing
- Table operations
- Memory lifecycle
- Plugin integration

### 15.3 Browser Tests

Tests run in actual browser environment:
- WebAssembly loading
- Memory limits
- Performance benchmarks
- Type safety verification

## 16. Performance Considerations

### 16.1 Zero-Copy Operations

All operations designed to minimize data copying:
- Slicing creates views, not copies
- Column access returns references
- Data transfer uses SharedArrayBuffer where available

### 16.2 Memory Pooling

WASM-side memory management uses pooling:
- Reuses allocated buffers
- Reduces allocation overhead
- Configurable pool sizes

### 16.3 Lazy Evaluation

Operations evaluated only when necessary:
- Filter predicates applied on-demand
- Statistics computed once and cached
- Schema inference deferred until needed

## 17. Future Extensions

### 17.1 Planned Features

1. **Geometry Plugin**: Full support for geometric data types
2. **Streaming API**: Process large files in chunks
3. **Parallel Operations**: Web Workers for compute operations
4. **SQL Interface**: Query tables with SQL syntax
5. **Format Converters**: CSV, Parquet, JSON converters

### 17.2 API Stability

The API follows semantic versioning:
- Major version: Breaking changes
- Minor version: New features, backward compatible
- Patch version: Bug fixes

## 18. Conclusion

This API definition provides a comprehensive, type-safe, and efficient interface for Arrow operations in WebAssembly environments. The design prioritizes:

- Zero-copy data transfer
- Type safety without runtime overhead
- Modular, extensible architecture
- Clear memory ownership model
- JavaScript/TypeScript best practices
