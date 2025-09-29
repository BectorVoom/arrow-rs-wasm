/**
 * Arrow WebAssembly Library - TypeScript Façade
 *
 * A strongly-typed TypeScript interface for the Arrow WASM library
 * with zero-copy semantics and LZ4 compression support.
 */
/**
 * Discriminated union for results - avoids try/catch in user code
 */
export type Ok<T> = {
    ok: true;
    value: T;
};
export type Err = {
    ok: false;
    error: string;
};
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
/**
 * Initialize the WASM module
 * Must be called once before using any other functions
 */
export declare function initWasm(wasmBytes?: ArrayBuffer, opts?: WasmInitOptions): Promise<Result<void>>;
/**
 * Check if WASM module is initialized
 */
export declare function isWasmInitialized(): boolean;
/**
 * Read an Arrow/Feather/Parquet file from an ArrayBuffer
 */
export declare function readTableFromArrayBuffer(data: ArrayBuffer): Promise<Result<TableHandle>>;
/**
 * Get schema summary for a table handle
 */
export declare function getSchemaSummary(handle: TableHandle): Promise<Result<SchemaSummary>>;
/**
 * Write table to IPC bytes with LZ4 option
 */
export declare function writeTableToIpc(handle: TableHandle, enableLz4: boolean): Promise<Result<ArrayBuffer>>;
/**
 * Free table handle and release its memory
 */
export declare function freeTable(handle: TableHandle): Promise<Result<void>>;
/**
 * Register a plugin by id
 */
export declare function registerPlugin(pluginId: string): Promise<Result<void>>;
//# sourceMappingURL=index.d.ts.map