#!/bin/bash

# Arrow WASM Browser Test Server
# This script starts the HTTP server and provides instructions for testing

echo "Arrow WASM Browser Test Server"
echo "=============================="
echo

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is not installed or not in PATH"
    echo "Please install Python 3 to run the test server"
    exit 1
fi

# Check if WASM package exists
if [ ! -d "../pkg" ]; then
    echo "Warning: WASM package not found in ../pkg/"
    echo "Please run 'wasm-pack build --target web --out-dir pkg' from the project root"
    echo
fi

# Change to project root directory to serve files correctly
cd "$(dirname "$0")/.."

echo "Starting HTTP server on port 8080..."
echo "Serving from: $(pwd)"
echo
echo "Open your browser and navigate to:"
echo "  http://localhost:8080/browser/index.html"
echo
echo "Press Ctrl+C to stop the server"
echo

# Start the server from the project root
python3 browser/server.py 8080