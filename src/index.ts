/**
 * Arrow WebAssembly Library - TypeScript Façade
 * 
 * A strongly-typed TypeScript interface for the Arrow WASM library
 * with zero-copy semantics and LZ4 compression support.
 */

// Re-export the WASM module initialization function
import init, * as wasm from '../pkg/arrow_wasm';

// Type definitions following the API specification

/**
 * Discriminated union for results - avoids try/catch in user code
 */
export type Ok<T> = { ok: true; value: T };
export type Err = { ok: false; error: string };
export type Result<T> = Ok<T> | Err;

/**
 * Table handle - opaque numeric id for referencing tables in WASM
 */
export type TableHandle = number;

/**
 * Column information in a schema
 */
export interface ColumnInfo {
  name: string;
  arrowType: string;
  nullable: boolean;
}

/**
 * Schema summary for a table
 */
export interface SchemaSummary {
  columns: ColumnInfo[];
  metadata: Record<string, string>;
}

/**
 * Memory usage statistics
 */
export interface MemoryStats {
  activeTables: number;
  totalRows: number;
  totalBatches: number;
}

/**
 * Compression configuration options
 */
export interface CompressionConfig {
  enabled: boolean;
  compressionType: 'LZ4_FRAME' | 'ZSTD' | 'None';
  preserveDictId: boolean;
}

/**
 * Plugin metadata information
 */
export interface PluginMetadata {
  id: string;
  name: string;
  version: string;
  description: string;
  registeredAt: number;
}

/**
 * Build information
 */
export interface BuildInfo {
  version: string;
  name: string;
  description: string;
  lz4Supported: boolean;
  compressionTypes: string[];
}

/**
 * WASM initialization options
 */
export interface WasmInitOptions {
  enableConsoleLogs?: boolean;
}

// Helper functions for creating Result types
function createOk<T>(value: T): Ok<T> {
  return { ok: true, value };
}

function createErr(error: string): Err {
  return { ok: false, error };
}

// WASM module state
let wasmModule: typeof wasm | null = null;
let isInitialized = false;

/**
 * Initialize the WASM module
 * Must be called once before using any other functions
 */
export async function initWasm(
  wasmBytes?: ArrayBuffer,
  opts?: WasmInitOptions
): Promise<Result<void>> {
  try {
    if (wasmBytes) {
      await init(wasmBytes);
    } else {
      await init();
    }
    
    wasmModule = wasm;
    
    // Initialize with options
    if (opts?.enableConsoleLogs !== undefined) {
      wasm.init_with_options(opts.enableConsoleLogs);
    }
    
    isInitialized = true;
    return createOk(undefined);
  } catch (error) {
    return createErr(`WASM initialization failed: ${error}`);
  }
}

/**
 * Check if WASM module is initialized
 */
export function isWasmInitialized(): boolean {
  return isInitialized && wasmModule !== null;
}

/**
 * Ensure WASM is initialized - throws if not
 */
function ensureInitialized(): typeof wasm {
  if (!wasmModule || !isInitialized) {
    throw new Error('WASM module not initialized. Call initWasm() first.');
  }
  return wasmModule;
}

/**
 * Read an Arrow/Feather/Parquet file from an ArrayBuffer
 */
export async function readTableFromArrayBuffer(
  data: ArrayBuffer
): Promise<Result<TableHandle>> {
  try {
    const wasm = ensureInitialized();
    const handle = wasm.read_table_from_array_buffer(data);
    return createOk(handle);
  } catch (error) {
    return createErr(`Failed to read table: ${error}`);
  }
}

/**
 * Get schema summary for a table handle
 */
export async function getSchemaSummary(
  handle: TableHandle
): Promise<Result<SchemaSummary>> {
  try {
    const wasm = ensureInitialized();
    const summaryJson = wasm.table_schema_summary(handle);
    const summary = JSON.parse(summaryJson) as SchemaSummary;
    return createOk(summary);
  } catch (error) {
    return createErr(`Failed to get schema: ${error}`);
  }
}

/**
 * Write table to IPC bytes with LZ4 option
 */
export async function writeTableToIpc(
  handle: TableHandle,
  enableLz4: boolean
): Promise<Result<ArrayBuffer>> {
  try {
    const wasm = ensureInitialized();
    const buffer = wasm.write_table_to_array_buffer(handle, enableLz4);
    return createOk(buffer);
  } catch (error) {
    return createErr(`Failed to write table: ${error}`);
  }
}

/**
 * Free table handle and release its memory
 */
export async function freeTable(handle: TableHandle): Promise<Result<void>> {
  try {
    const wasm = ensureInitialized();
    wasm.free_table(handle);
    return createOk(undefined);
  } catch (error) {
    return createErr(`Failed to free table: ${error}`);
  }
}

/**
 * Register a plugin by id
 */
export async function registerPlugin(pluginId: string): Promise<Result<void>> {
  try {
    const wasm = ensureInitialized();
    wasm.register_plugin(pluginId);
    return createOk(undefined);
  } catch (error) {
    return createErr(`Plugin registration failed: ${error}`);
  }
}

// Types are already exported above with their declarations