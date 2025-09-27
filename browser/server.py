#!/usr/bin/env python3
"""
Simple HTTP server for testing Arrow WASM in browser.
Serves files with proper MIME types for WASM.
"""

import http.server
import socketserver
import mimetypes
import os
import sys

# Add WASM and Arrow file MIME type support
mimetypes.add_type('application/wasm', '.wasm')
mimetypes.add_type('application/vnd.apache.arrow.file', '.arrow')
mimetypes.add_type('application/vnd.apache.arrow.stream', '.feather')
mimetypes.add_type('application/octet-stream', '.ipc')

class WasmHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Enable CORS for all requests
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        
        # Enable SharedArrayBuffer support (needed for some WASM features)
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        
        super().end_headers()
    
    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

def run_server(port=8080, directory=None):
    if directory:
        os.chdir(directory)
    
    handler = WasmHTTPRequestHandler
    
    try:
        with socketserver.TCPServer(("", port), handler) as httpd:
            print(f"Server running at http://localhost:{port}")
            print(f"Serving directory: {os.getcwd()}")
            print("Press Ctrl+C to stop the server")
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped by user")
    except OSError as e:
        if "Address already in use" in str(e):
            print(f"Port {port} is already in use. Try a different port.")
            print(f"Usage: python3 server.py [port] [directory]")
        else:
            print(f"Error starting server: {e}")

if __name__ == "__main__":
    port = 8080
    directory = None
    
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            print("Invalid port number. Using default port 8080.")
    
    if len(sys.argv) > 2:
        directory = sys.argv[2]
        if not os.path.exists(directory):
            print(f"Directory '{directory}' does not exist. Using current directory.")
            directory = None
    
    run_server(port, directory)