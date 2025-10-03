import { useState, useEffect } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'

// Import arrow-rs-wasm
import init, { 
  create_test_table,
  write_table_to_ipc,
  read_table_from_bytes,
  get_table_info,
  free_table,
  init_with_options
} from 'arrow-rs-wasm'

function App() {
  const [count, setCount] = useState(0)
  const [wasmLoaded, setWasmLoaded] = useState(false)
  const [testResults, setTestResults] = useState<string[]>([])
  const [isRunning, setIsRunning] = useState(false)

  // Initialize WASM module on component mount
  useEffect(() => {
    const initWasm = async () => {
      try {
        await init()
        init_with_options(true)
        setWasmLoaded(true)
        addResult('✅ WASM module loaded successfully!')
      } catch (error) {
        addResult(`❌ Failed to load WASM module: ${error}`)
      }
    }
    initWasm()
  }, [])

  const addResult = (message: string) => {
    setTestResults(prev => [...prev, `${new Date().toLocaleTimeString()}: ${message}`])
  }

  const runArrowTest = async () => {
    if (!wasmLoaded || isRunning) return
    
    setIsRunning(true)
    setTestResults([])
    
    try {
      addResult('🧪 Starting Arrow WASM test in Vite + React...')
      
      // Create test table
      addResult('1. Creating test table...')
      const tableHandle = create_test_table()
      addResult(`✅ Test table created (handle: ${tableHandle})`)

      // Get table info
      addResult('2. Getting table information...')
      const tableInfo = get_table_info(tableHandle)
      addResult(`✅ Table: ${tableInfo.row_count} rows, ${tableInfo.column_count} columns`)
      addResult(`   Columns: ${tableInfo.column_names.join(', ')}`)

      // Write to IPC format
      addResult('3. Serializing to Arrow IPC format...')
      const ipcData = write_table_to_ipc(tableHandle, true)
      const uncompressedData = write_table_to_ipc(tableHandle, false)
      const ratio = ((ipcData.length / uncompressedData.length) * 100).toFixed(1)
      addResult(`✅ Serialized: ${ipcData.length} bytes (${ratio}% compression)`)

      // Read data back
      addResult('4. Reading data back...')
      const newTableHandle = read_table_from_bytes(ipcData)
      const newTableInfo = get_table_info(newTableHandle)
      addResult(`✅ Round-trip successful: ${newTableInfo.row_count} rows, ${newTableInfo.column_count} columns`)

      // Clean up
      addResult('5. Cleaning up memory...')
      free_table(tableHandle)
      free_table(newTableHandle)
      addResult('✅ Memory cleaned up')

      addResult('🎉 All Vite + React tests passed!')
      
    } catch (error) {
      addResult(`❌ Test failed: ${error}`)
    } finally {
      setIsRunning(false)
    }
  }

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React + Arrow WASM</h1>
      
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        
        <div style={{ margin: '20px 0' }}>
          <h3>Arrow WASM Test</h3>
          <p>WASM Status: <span style={{ color: wasmLoaded ? 'green' : 'red' }}>
            {wasmLoaded ? '✅ Loaded' : '❌ Not Loaded'}
          </span></p>
          
          <button 
            onClick={runArrowTest} 
            disabled={!wasmLoaded || isRunning}
            style={{ 
              backgroundColor: wasmLoaded && !isRunning ? '#007cba' : '#ccc',
              cursor: wasmLoaded && !isRunning ? 'pointer' : 'not-allowed'
            }}
          >
            {isRunning ? 'Running Test...' : 'Run Arrow WASM Test'}
          </button>
          
          {testResults.length > 0 && (
            <div style={{ 
              marginTop: '10px', 
              padding: '10px', 
              backgroundColor: '#000', 
              color: '#00ff00', 
              fontFamily: 'monospace',
              fontSize: '12px',
              borderRadius: '5px',
              height: '200px',
              overflowY: 'auto'
            }}>
              {testResults.map((result, index) => (
                <div key={index}>{result}</div>
              ))}
            </div>
          )}
        </div>
        
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  )
}

export default App
