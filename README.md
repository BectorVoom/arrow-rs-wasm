# arrow-rs-wasm

[![npm version](https://badge.fury.io/js/arrow-rs-wasm.svg)](https://www.npmjs.com/package/arrow-rs-wasm)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/BectorVoom/arrow-rs-wasm#license)

A high-performance WebAssembly library for reading, writing, and processing Apache Arrow, Feather, and Parquet data with zero-copy semantics and LZ4 compression support.

## 🚀 Features

- **🔥 Zero-Copy Performance**: Efficient data processing with minimal memory overhead
- **📊 Multiple File Formats**: Native support for Arrow IPC, Feather, and Parquet files
- **🗜️ LZ4 Compression**: Built-in compression support for optimized storage
- **🧩 Plugin Architecture**: Extensible system for custom data types (geometry support included)
- **🔒 Type Safety**: Complete TypeScript definitions with Result-based error handling
- **🌍 Universal**: Works in browsers and Node.js environments
- **⚡ Production Ready**: Optimized WebAssembly build with comprehensive testing

## 📦 Installation

```bash
npm install arrow-rs-wasm
```

## 🏃 Quick Start

### Basic Usage (Browser)

```typescript
import { 
  initWasm, 
  readTableFromArrayBuffer, 
  getSchemaSummary, 
  writeTableToIpc,
  freeTable 
} from 'arrow-rs-wasm';

// Initialize the WASM module
const initResult = await initWasm();
if (!initResult.ok) {
  console.error('Failed to initialize WASM:', initResult.error);
  return;
}

// Load an Arrow file
const response = await fetch('data/sample.arrow');
const arrayBuffer = await response.arrayBuffer();

// Read the table
const readResult = await readTableFromArrayBuffer(arrayBuffer);
if (!readResult.ok) {
  console.error('Failed to read table:', readResult.error);
  return;
}

const tableHandle = readResult.value;

// Inspect the schema
const schemaResult = await getSchemaSummary(tableHandle);
if (schemaResult.ok) {
  console.log('Table Schema:');
  schemaResult.value.columns.forEach(col => {
    console.log(`- ${col.name}: ${col.arrowType} (nullable: ${col.nullable})`);
  });
}

// Write back with LZ4 compression
const writeResult = await writeTableToIpc(tableHandle, true);
if (writeResult.ok) {
  const compressedData = writeResult.value;
  console.log(`Compressed size: ${compressedData.byteLength} bytes`);
}

// Clean up memory
await freeTable(tableHandle);
```

### Node.js Usage

```typescript
import { readFileSync } from 'fs';
import { 
  initWasm, 
  readTableFromArrayBuffer, 
  getSchemaSummary 
} from 'arrow-rs-wasm';

// Initialize WASM
await initWasm();

// Read file from disk
const fileData = readFileSync('data/sample.parquet');
const arrayBuffer = fileData.buffer.slice(
  fileData.byteOffset, 
  fileData.byteOffset + fileData.byteLength
);

// Process the file
const result = await readTableFromArrayBuffer(arrayBuffer);
if (result.ok) {
  const schema = await getSchemaSummary(result.value);
  console.log('Columns:', schema.value?.columns.length);
  
  await freeTable(result.value);
}
```

## 📚 File Format Examples

### Reading Different Formats

```typescript
// The library automatically detects file formats
async function processFile(arrayBuffer: ArrayBuffer) {
  const result = await readTableFromArrayBuffer(arrayBuffer);
  
  if (!result.ok) {
    console.error('Read failed:', result.error);
    return;
  }
  
  // Works with:
  // - Arrow IPC files (.arrow)
  // - Feather files (.feather, .fea)
  // - Parquet files (.parquet)
  
  const handle = result.value;
  const schema = await getSchemaSummary(handle);
  
  console.log('Successfully read file with', schema.value?.columns.length, 'columns');
  await freeTable(handle);
}

// Usage with different file types
await processFile(await fetch('data.arrow').then(r => r.arrayBuffer()));
await processFile(await fetch('data.feather').then(r => r.arrayBuffer()));
await processFile(await fetch('data.parquet').then(r => r.arrayBuffer()));
```

### Writing with Compression

```typescript
async function saveWithCompression(tableHandle: number) {
  // Write with LZ4 compression enabled
  const compressedResult = await writeTableToIpc(tableHandle, true);
  
  // Write without compression
  const uncompressedResult = await writeTableToIpc(tableHandle, false);
  
  if (compressedResult.ok && uncompressedResult.ok) {
    const compressed = compressedResult.value;
    const uncompressed = uncompressedResult.value;
    
    console.log(`Compression ratio: ${(compressed.byteLength / uncompressed.byteLength * 100).toFixed(1)}%`);
    
    // Save compressed version
    const blob = new Blob([compressed], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = url;
    link.download = 'data_compressed.arrow';
    link.click();
  }
}
```

## 🧩 Plugin System

### Using the Geometry Plugin

```typescript
import { registerPlugin } from 'arrow-rs-wasm';

// Register the built-in geometry plugin
const pluginResult = await registerPlugin('geometry');
if (pluginResult.ok) {
  console.log('Geometry plugin registered successfully');
  
  // Now you can process Arrow files with geometry columns
  // The plugin handles Well-Known Binary (WKB) geometry data
}

// Register by full plugin ID
await registerPlugin('io.arrow.plugin.geo.v1');
```

### Plugin Discovery

```typescript
import { discoverAvailablePlugins, getPluginInfo } from 'arrow-rs-wasm';

// Discover available plugins
const pluginsResult = await discoverAvailablePlugins();
if (pluginsResult.ok) {
  console.log('Available plugins:', pluginsResult.value);
}

// Get information about registered plugins
const infoResult = await getPluginInfo();
if (infoResult.ok) {
  console.log('Registered plugins:', infoResult.value);
}
```

## 🔧 Advanced Usage

### Memory Management

```typescript
import { 
  getMemoryStats, 
  clearAllTables, 
  isValidHandle 
} from 'arrow-rs-wasm';

async function demonstrateMemoryManagement() {
  // Check memory usage
  const statsResult = await getMemoryStats();
  if (statsResult.ok) {
    const stats = statsResult.value;
    console.log(`Active tables: ${stats.activeTables}`);
    console.log(`Total rows: ${stats.totalRows}`);
    console.log(`Total batches: ${stats.totalBatches}`);
  }
  
  // Validate handles
  const handle = 123;
  if (isValidHandle(handle)) {
    console.log('Handle is valid');
    await freeTable(handle);
  }
  
  // Clear all tables (useful for testing)
  clearAllTables();
}
```

### Error Handling Patterns

```typescript
// The library uses Result types instead of throwing exceptions
async function safeProcessing(data: ArrayBuffer) {
  const readResult = await readTableFromArrayBuffer(data);
  
  // Pattern 1: Early return on error
  if (!readResult.ok) {
    console.error('Failed to read:', readResult.error);
    return;
  }
  
  const handle = readResult.value;
  
  // Pattern 2: Nested result handling
  const schemaResult = await getSchemaSummary(handle);
  if (schemaResult.ok) {
    const columns = schemaResult.value.columns;
    
    // Process columns...
    const writeResult = await writeTableToIpc(handle, true);
    if (writeResult.ok) {
      console.log('Successfully processed and wrote data');
      return writeResult.value;
    } else {
      console.error('Write failed:', writeResult.error);
    }
  } else {
    console.error('Schema read failed:', schemaResult.error);
  }
  
  // Always clean up
  await freeTable(handle);
}
```

### Batch Processing

```typescript
async function processManyFiles(files: File[]) {
  const results = [];
  
  for (const file of files) {
    const arrayBuffer = await file.arrayBuffer();
    const readResult = await readTableFromArrayBuffer(arrayBuffer);
    
    if (readResult.ok) {
      const handle = readResult.value;
      const schemaResult = await getSchemaSummary(handle);
      
      if (schemaResult.ok) {
        results.push({
          filename: file.name,
          columns: schemaResult.value.columns.length,
          handle: handle
        });
      }
    }
  }
  
  // Process results...
  console.log(`Successfully loaded ${results.length} files`);
  
  // Clean up all handles
  for (const result of results) {
    await freeTable(result.handle);
  }
  
  return results;
}
```

## 📖 API Reference

### Core Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `initWasm(wasmBytes?, opts?)` | Initialize the WASM module | `Promise<Result<void>>` |
| `readTableFromArrayBuffer(data)` | Read Arrow/Feather/Parquet data | `Promise<Result<TableHandle>>` |
| `getSchemaSummary(handle)` | Get table schema information | `Promise<Result<SchemaSummary>>` |
| `writeTableToIpc(handle, enableLz4)` | Write table as Arrow IPC | `Promise<Result<ArrayBuffer>>` |
| `freeTable(handle)` | Release table memory | `Promise<Result<void>>` |
| `registerPlugin(pluginId)` | Register a plugin | `Promise<Result<void>>` |

### Utility Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `isWasmInitialized()` | Check if WASM is ready | `boolean` |
| `isValidHandle(handle)` | Validate table handle | `boolean` |
| `getMemoryStats()` | Get memory usage statistics | `Promise<Result<MemoryStats>>` |
| `clearAllTables()` | Clear all tables (testing) | `void` |

### TypeScript Types

```typescript
// Result type for error handling
export type Result<T> = 
  | { ok: true; value: T }
  | { ok: false; error: string };

// Table handle (opaque reference)
export type TableHandle = number;

// Schema information
export interface SchemaSummary {
  columns: ColumnInfo[];
  metadata: Record<string, string>;
}

export interface ColumnInfo {
  name: string;
  arrowType: string;
  nullable: boolean;
}

// Memory statistics
export interface MemoryStats {
  activeTables: number;
  totalRows: number;
  totalBatches: number;
}

// Initialization options
export interface WasmInitOptions {
  enableConsoleLogs?: boolean;
}
```

## ⚡ Performance Considerations

### Memory Management Best Practices

1. **Always free table handles** when done processing
2. **Use `getMemoryStats()`** to monitor memory usage
3. **Process files in batches** for large datasets
4. **Enable LZ4 compression** for network transfers

### Optimization Tips

```typescript
// Good: Process and immediately free
async function efficientProcessing(data: ArrayBuffer) {
  const result = await readTableFromArrayBuffer(data);
  if (result.ok) {
    const schema = await getSchemaSummary(result.value);
    const output = await writeTableToIpc(result.value, true);
    
    await freeTable(result.value); // Free immediately
    return output;
  }
}

// Avoid: Accumulating handles without cleanup
const handles = []; // This will leak memory!
for (const file of files) {
  const result = await readTableFromArrayBuffer(await file.arrayBuffer());
  if (result.ok) {
    handles.push(result.value); // Don't do this
  }
}
```

## 🌟 Browser vs Node.js

### Browser-Specific Features
- Automatic WASM loading from CDN
- File drag-and-drop processing
- Blob/URL creation for downloads
- Worker thread support

### Node.js-Specific Features
- File system integration
- Stream processing
- Buffer handling
- Command-line tools

## 🤝 Contributing

Contributions are welcome! Please read our [contributing guidelines](CONTRIBUTING.md) and ensure all tests pass before submitting a pull request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/BectorVoom/arrow-rs-wasm.git
cd arrow-rs-wasm

# Install dependencies
npm install

# Build the project
npm run build

# Run tests
npm test
```

## 📄 License

This project is dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 🔗 Links

- [npm package](https://www.npmjs.com/package/arrow-rs-wasm)
- [GitHub repository](https://github.com/BectorVoom/arrow-rs-wasm)
- [Apache Arrow project](https://arrow.apache.org/)
- [WebAssembly](https://webassembly.org/)

---

**Built with ❤️ using Rust and WebAssembly**