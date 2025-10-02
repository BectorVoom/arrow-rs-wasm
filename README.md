# arrow-rs-wasm

High-performance WebAssembly library for Apache Arrow, Feather, and Parquet data with zero-copy semantics, LZ4 compression, and comprehensive model-based testing.

## Installation

```bash
npm install arrow-rs-wasm
```

## Quick Start

### Node.js

```javascript
import init, {
  createTestTable,
  writeTableToIpc,
  readTableFromBytes,
  getTableInfo,
  freeTable,
  initWithOptions,
  initSync
} from 'arrow-rs-wasm';
import { readFileSync } from 'fs';

async function example() {
  // Initialize WASM module
  const wasmBytes = readFileSync('./node_modules/arrow-rs-wasm/arrow_rs_wasm_bg.wasm');
  initSync(wasmBytes);
  initWithOptions(true); // Enable console logs

  // Create test table
  const tableHandle = createTestTable();
  console.log(`Created table with handle: ${tableHandle}`);

  // Get table information
  const tableInfo = getTableInfo(tableHandle);
  console.log(`Table: ${tableInfo.row_count} rows, ${tableInfo.column_count} columns`);
  console.log(`Columns: ${tableInfo.column_names.join(', ')}`);

  // Serialize to Arrow IPC format with LZ4 compression
  const ipcData = writeTableToIpc(tableHandle, true);
  console.log(`Serialized: ${ipcData.length} bytes`);

  // Read data back
  const newTableHandle = readTableFromBytes(ipcData);
  const newTableInfo = getTableInfo(newTableHandle);
  console.log(`Round-trip: ${newTableInfo.row_count} rows, ${newTableInfo.column_count} columns`);

  // Clean up memory
  freeTable(tableHandle);
  freeTable(newTableHandle);
}

example().catch(console.error);
```

### Browser

```html
<!DOCTYPE html>
<html>
<head>
    <title>Arrow WASM Example</title>
</head>
<body>
    <script type="module">
        import init, {
            createTestTable,
            writeTableToIpc,
            readTableFromBytes,
            getTableInfo,
            freeTable,
            initWithOptions
        } from './node_modules/arrow-rs-wasm/arrow_rs_wasm.js';

        async function runExample() {
            // Initialize WASM module
            await init();
            initWithOptions(true);

            // Create test table
            const tableHandle = createTestTable();
            console.log(`Created table with handle: ${tableHandle}`);

            // Get table information
            const tableInfo = getTableInfo(tableHandle);
            console.log(`Table: ${tableInfo.row_count} rows, ${tableInfo.column_count} columns`);

            // Serialize with compression
            const ipcData = writeTableToIpc(tableHandle, true);
            console.log(`Serialized: ${ipcData.length} bytes`);

            // Round-trip test
            const newTableHandle = readTableFromBytes(ipcData);
            const newTableInfo = getTableInfo(newTableHandle);
            console.log(`Round-trip: ${newTableInfo.row_count} rows, ${newTableInfo.column_count} columns`);

            // Clean up
            freeTable(tableHandle);
            freeTable(newTableHandle);

            console.log('✅ All tests passed!');
        }

        runExample().catch(console.error);
    </script>
</body>
</html>
```

### Vite + React

```javascript
import { useState, useEffect } from 'react'
import init, {
  createTestTable,
  writeTableToIpc,
  readTableFromBytes,
  getTableInfo,
  freeTable,
  initWithOptions
} from 'arrow-rs-wasm'

function App() {
  const [wasmLoaded, setWasmLoaded] = useState(false)
  const [testResults, setTestResults] = useState([])

  // Initialize WASM module
  useEffect(() => {
    const initWasm = async () => {
      try {
        await init()
        initWithOptions(true)
        setWasmLoaded(true)
        addResult('✅ WASM module loaded successfully!')
      } catch (error) {
        addResult(`❌ Failed to load WASM module: ${error}`)
      }
    }
    initWasm()
  }, [])

  const addResult = (message) => {
    setTestResults(prev => [...prev, `${new Date().toLocaleTimeString()}: ${message}`])
  }

  const runTest = async () => {
    if (!wasmLoaded) return

    try {
      // Create test table
      const tableHandle = createTestTable()
      addResult(`✅ Test table created (handle: ${tableHandle})`)

      // Get table info
      const tableInfo = getTableInfo(tableHandle)
      addResult(`✅ Table: ${tableInfo.row_count} rows, ${tableInfo.column_count} columns`)

      // Serialize with LZ4 compression
      const ipcData = writeTableToIpc(tableHandle, true)
      const uncompressedData = writeTableToIpc(tableHandle, false)
      const ratio = ((ipcData.length / uncompressedData.length) * 100).toFixed(1)
      addResult(`✅ Serialized: ${ipcData.length} bytes (${ratio}% compression)`)

      // Round-trip test
      const newTableHandle = readTableFromBytes(ipcData)
      const newTableInfo = getTableInfo(newTableHandle)
      addResult(`✅ Round-trip: ${newTableInfo.row_count} rows, ${newTableInfo.column_count} columns`)

      // Clean up
      freeTable(tableHandle)
      freeTable(newTableHandle)
      addResult('🎉 All tests passed!')
    } catch (error) {
      addResult(`❌ Test failed: ${error}`)
    }
  }

  return (
    <div>
      <h1>Arrow WASM + Vite + React</h1>
      <p>WASM Status: {wasmLoaded ? '✅ Loaded' : '❌ Not Loaded'}</p>
      <button onClick={runTest} disabled={!wasmLoaded}>
        Run Arrow WASM Test
      </button>
      <div style={{ marginTop: '20px', fontFamily: 'monospace' }}>
        {testResults.map((result, index) => (
          <div key={index}>{result}</div>
        ))}
      </div>
    </div>
  )
}

export default App
```

For Vite projects, add this to your `vite.config.js`:

```javascript
export default defineConfig({
  // ... other config
  assetsInclude: ['**/*.wasm'],
  optimizeDeps: {
    exclude: ['arrow-rs-wasm']
  }
})
```

## Key Features

- **Zero-copy semantics** - Direct memory access without data copying
- **LZ4 compression** - Fast compression for reduced memory usage
- **Cross-platform** - Works in Node.js, browsers, and bundlers like Vite
- **Apache Arrow support** - Full Arrow IPC format compatibility
- **Memory management** - Explicit memory control with cleanup functions
- **TypeScript support** - Comprehensive type definitions included

## API Overview

### Core Functions

- `createTestTable()` - Creates a sample table for testing
- `writeTableToIpc(handle, enableLz4)` - Serializes table to Arrow IPC format
- `readTableFromBytes(data)` - Reads table from Arrow IPC bytes
- `getTableInfo(handle)` - Returns table metadata (rows, columns, names)
- `freeTable(handle)` - Releases table memory

### Initialization

- `init()` - Initialize WASM module (async)
- `initSync(wasmBytes)` - Initialize WASM module synchronously (Node.js only)
- `initWithOptions(enableConsoleLogs)` - Configure logging options

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
