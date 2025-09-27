# Arrow WASM Browser Testing

This directory contains browser testing infrastructure for the Arrow WASM library.

## Files

- `index.html` - Main test page with interactive tests
- `server.py` - HTTP server with proper WASM MIME type support
- `run-tests.sh` - Convenience script to start the server

## Running Tests

1. **Start the server:**
   ```bash
   python3 server.py
   ```
   Or use the convenience script:
   ```bash
   ./run-tests.sh
   ```

2. **Open the test page:**
   Navigate to `http://localhost:8080/browser/index.html` in your web browser

3. **Run the tests:**
   - Click individual test buttons to run specific tests
   - Click "Run All Tests" to run the complete test suite
   - View results in the output console

## Test Coverage

The test suite covers:
- WASM module loading
- Core API initialization
- Version information retrieval  
- Basic table operations (placeholder)

## Requirements

- Python 3.x for the HTTP server
- Modern web browser with WebAssembly support
- Arrow WASM package built in `../pkg/` directory

## Troubleshooting

- **Module loading errors**: Ensure the WASM package is built (`wasm-pack build`)
- **CORS errors**: Use the provided server (don't open HTML directly)
- **Port conflicts**: Use `python3 server.py 8081` for a different port