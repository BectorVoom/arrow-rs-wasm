/* tslint:disable */
/* eslint-disable */
export function set_panic_hook(): void;
export function table_row_count(handle: number): number;
export function table_column_count(handle: number): number;
export function get_column_names(handle: number): string[];
export function export_column_by_name(handle: number, column_name: string): Uint8Array;
export function free_table(handle: number): void;
export function get_memory_info(): any;
export function init(): void;
export function init_with_options(enable_console_logs: boolean): void;
export function read_table_from_bytes(data: Uint8Array): number;
export function write_table_to_ipc(handle: number, enable_lz4: boolean): Uint8Array;
export function create_test_table(): number;
export function get_table_schema_json(handle: number): string;
export function export_column_with_type(handle: number, column_name: string): any;
export function get_table_info(handle: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly table_row_count: (a: number, b: number) => void;
  readonly table_column_count: (a: number, b: number) => void;
  readonly get_column_names: (a: number, b: number) => void;
  readonly export_column_by_name: (a: number, b: number, c: number, d: number) => void;
  readonly free_table: (a: number, b: number) => void;
  readonly get_memory_info: () => number;
  readonly init_with_options: (a: number) => void;
  readonly read_table_from_bytes: (a: number, b: number, c: number) => void;
  readonly write_table_to_ipc: (a: number, b: number, c: number) => void;
  readonly create_test_table: (a: number) => void;
  readonly get_table_schema_json: (a: number, b: number) => void;
  readonly export_column_with_type: (a: number, b: number, c: number, d: number) => void;
  readonly get_table_info: (a: number, b: number) => void;
  readonly set_panic_hook: () => void;
  readonly init: () => void;
  readonly __wbindgen_export_0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_1: (a: number) => void;
  readonly __wbindgen_export_2: (a: number, b: number) => number;
  readonly __wbindgen_export_3: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
