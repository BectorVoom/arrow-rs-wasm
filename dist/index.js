/**
 * Arrow WebAssembly Library - TypeScript Façade
 *
 * A strongly-typed TypeScript interface for the Arrow WASM library
 * with zero-copy semantics and LZ4 compression support.
 */
// Re-export the WASM module initialization function
import init, * as wasm from '../pkg/arrow_wasm';
// Helper functions for creating Result types
function createOk(value) {
    return { ok: true, value };
}
function createErr(error) {
    return { ok: false, error };
}
// WASM module state
let wasmModule = null;
let isInitialized = false;
/**
 * Initialize the WASM module
 * Must be called once before using any other functions
 */
export async function initWasm(wasmBytes, opts) {
    try {
        if (wasmBytes) {
            await init(wasmBytes);
        }
        else {
            await init();
        }
        wasmModule = wasm;
        // Initialize with options
        if (opts?.enableConsoleLogs !== undefined) {
            wasm.init_with_options(opts.enableConsoleLogs);
        }
        isInitialized = true;
        return createOk(undefined);
    }
    catch (error) {
        return createErr(`WASM initialization failed: ${error}`);
    }
}
/**
 * Check if WASM module is initialized
 */
export function isWasmInitialized() {
    return isInitialized && wasmModule !== null;
}
/**
 * Ensure WASM is initialized - throws if not
 */
function ensureInitialized() {
    if (!wasmModule || !isInitialized) {
        throw new Error('WASM module not initialized. Call initWasm() first.');
    }
    return wasmModule;
}
/**
 * Read an Arrow/Feather/Parquet file from an ArrayBuffer
 */
export async function readTableFromArrayBuffer(data) {
    try {
        const wasm = ensureInitialized();
        const handle = wasm.read_table_from_array_buffer(data);
        return createOk(handle);
    }
    catch (error) {
        return createErr(`Failed to read table: ${error}`);
    }
}
/**
 * Get schema summary for a table handle
 */
export async function getSchemaSummary(handle) {
    try {
        const wasm = ensureInitialized();
        const summaryJson = wasm.table_schema_summary(handle);
        const summary = JSON.parse(summaryJson);
        return createOk(summary);
    }
    catch (error) {
        return createErr(`Failed to get schema: ${error}`);
    }
}
/**
 * Write table to IPC bytes with LZ4 option
 */
export async function writeTableToIpc(handle, enableLz4) {
    try {
        const wasm = ensureInitialized();
        const buffer = wasm.write_table_to_array_buffer(handle, enableLz4);
        return createOk(buffer);
    }
    catch (error) {
        return createErr(`Failed to write table: ${error}`);
    }
}
/**
 * Free table handle and release its memory
 */
export async function freeTable(handle) {
    try {
        const wasm = ensureInitialized();
        wasm.free_table(handle);
        return createOk(undefined);
    }
    catch (error) {
        return createErr(`Failed to free table: ${error}`);
    }
}
/**
 * Register a plugin by id
 */
export async function registerPlugin(pluginId) {
    try {
        const wasm = ensureInitialized();
        wasm.register_plugin(pluginId);
        return createOk(undefined);
    }
    catch (error) {
        return createErr(`Plugin registration failed: ${error}`);
    }
}
// Types are already exported above with their declarations
//# sourceMappingURL=index.js.map