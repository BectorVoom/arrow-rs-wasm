# wasm-arrow Library — API Specification (Markdown — English Delivery)

## 0. Purpose

This document defines the API specification and implementation requirements for a WebAssembly (wasm) version of the Apache Arrow library ("the Library"). The specification is written in formal, public-facing English and is intended as the authoritative reference for design, implementation, verification, testing, and acceptance.

---

## 1. Scope

This specification covers the following:

* wasm-enabled APIs for reading, creating, and manipulating Arrow / IPC, Feather, and Parquet format files.
* a Rust core implementation and a high-level JavaScript/TypeScript facade accessible from JS/TS environments.
* a zero-copy memory transfer design between wasm and JS, and verification of that design.
* unit tests, integration tests, and end-to-end tests in both browser and Node.js environments.

**Exclusions**: Low-level C bindings (C structures and functions) must not be exposed directly to consumers. See §7.

---

## 2. Definitions

* **Table**: A columnar data structure equivalent to an Apache Arrow Table.
* **IPC**: Arrow IPC format (commonly `.arrow`) including write options such as `IpcWriteOptions`.
* **zero-copy**: A design in which wasm memory is referenced by JS without copying. The user must not be required to perform memory operations directly.
* **High-level API**: An abstraction that hides low-level memory handling; users only perform initialization and disposal, not manual memory manipulation.

---

## 3. Overall Design Principles

1. **Modular separation**: Clearly separate the core (Rust + wasm) from the JS/TS facade. Provide high-level wrappers on the JS/TS side.
2. **Tree-shaking friendly**: Implement the JS/TS facade as side-effect-free ES modules, organized to allow efficient tree-shaking.
3. **Usability**: Use predictable naming, complete TypeScript types, comprehensive examples, and clear documentation.
4. **High-level memory handling**: Only initialization (`initialize`) and release (`freeTable`) are explicit to users; all other memory management occurs inside wasm.
5. **Avoid direct C bindings exposure**: Keep C bindings internal; expose only high-level APIs.

---

## 4. JavaScript / TypeScript Constraints

In all public-facing JS/TS code, samples, and documentation, the following must be avoided:

* `this`
* `any`
* `is`
* `try` / `catch`

**Alternatives**:


* Avoid `any` by providing detailed generic types or union types where appropriate.
* Avoid `this` by using factory functions or explicit context objects.

> Note: These constraints apply to public SDK code. Build scripts and internal tooling may be exempt but must be documented if so.

---

## 5. Functional Requirements

### 5.1 File Import (Read)

**Conceptual signature**:

```ts
export function readTableFromBuffer(buffer: Uint8Array): TableHandle;
```

Requirements:

* Support Arrow (IPC / `.arrow`), Feather, and Parquet file formats.
* Accept input types appropriate for both Node.js and browser environments (Buffer, ArrayBuffer, Uint8Array, File-like objects).
* Return a high-level Table object or handle on success.

### 5.2 File Export (Write)

**Conceptual signature**:

```ts
export function writeTableToArrowIPC(handle: TableHandle, opts?: IpcWriteOptions): Promise<Uint8Array>;
```

Requirements:

* Support writing to Arrow IPC with options equivalent to `arrow_ipc::writer::IpcWriteOptions` available from JS/TS.
* Provide write APIs for Feather and Parquet formats. Parquet output must accept options for chunking and compression.

### 5.3 Table Manipulation APIs

Required APIs:

**Rust (core)**

* `create_table_from_ipc(ipc_data_ptr, ipc_data_len) -> Result<TableHandle, CoreError>`
* `read_table_from_bytes(bytes_ptr, bytes_len) -> Result<TableHandle, CoreError>`
* `write_table_to_ipc(handle, IpcWriteOptions) -> Result<Vec<u8>, CoreError>`
* `free_table(handle) -> Result<(), CoreError>`


**TypeScript façade**
* `getColumnByName(table: Table, name: string): ColumnData` — retrieve a column by its name.
* `toArray(table: Table): any[]` — convert a Table to a JS-friendly array representation.
* `table.schema: SchemaDescriptor` — obtain schema from a table.
* `schema.fields: Fields` — obtain fields from a schema.
* `initialize(bytes?: ArrayBuffer, opts?: WasmInitOptions): Promise<Result<void>>`
* `readTableFromArrayBuffer(data: ArrayBuffer): TableHandle`
* `writeTableToIpc(handle: TableHandle, enableLz4: boolean): ArrayBuffer`
* `freeTable(handle: TableHandle): Promise<Result<void>>`

### 5.4 Zero-copy and Memory Management

* Implement zero-copy between wasm and JS wherever practical.
* Users must not perform memory operations except for explicit initialization and disposal.
* Provide explicit lifecycle APIs that return handles for resource management. Example:

```ts
export function initializeWasm(options?: WasmInitOptions): Promise<void>;
export function disposeWasm(): Promise<void>;
export function freeTable(handle: TableHandle): Promise<void>;
```

### 5.5 Compression Support

* LZ4 compression must be supported.
* Additional codecs (e.g., Zstd) may be added later, but direct exposure of C bindings is not permitted.

---

## 6. Non-functional Requirements

1. **Type safety**: Public APIs must include TypeScript declarations (`.d.ts`) and avoid `any`.
2. **Modularity**: Components should be separated to allow tree-shaking and minimal bundle sizes.
3. **Compatibility**: Support major browsers and Node.js. Use `wasm-pack` / `wasm-bindgen` for builds and optimize for bundlers (vite, rollup).
4. **Security**: Validate external inputs to prevent invalid memory access and DoS vectors.
5. **Performance**: Minimize copies via zero-copy design and efficient data paths.

---

## 7. Implementation Constraints

* Do not expose C-language bindings or structures directly in the JS/TS API surface.
* C bindings may be used internally but must be wrapped by high-level, safe interfaces.
* Adhere to the crate selection in §9; any additional crates must be reviewed for compatibility and security.

---

## 8. API Examples (Conceptual Signatures)

```ts
// Initialization / teardown
export function initialize(options?: WasmInitOptions): Promise<void>;
export function dispose(): Promise<void>;

// Reading
export function readTableFromBuffer(buffer: Uint8Array): TableHandle;

// Writing
export function writeTableToArrowIPC(handle: TableHandle, opts?: IpcWriteOptions): Uint8Array;

// Schema / column operations

export function getColumnByName(handle: TableHandle, name: string): ColumnData;

// Resource management
export function freeTable(handle: TableHandle): Promise<void>;
```

> Error handling: rely on Promise rejections or Result-style wrappers rather than `try` / `catch` in the public examples.

---

## 9. Specified Crates

```toml
arrow = { version = "56.1.0", default-features = false, features = ["ipc"] }
arrow-ipc = { version = "56.1.0", default-features = false, features = ["lz4"] }
arrow-array = { version = "56.1.0", default-features = false }
arrow-schema = { version = "56.1.0", default-features = false }
arrow-buffer = { version = "56.1.0", default-features = false }
arrow-data = { version = "56.1.0", default-features = false }
arrow-select = { version = "56.1.0", default-features = false }
arrow-cast = { version = "56.1.0", default-features = false }
arrow-ord = { version = "56.1.0", default-features = false }
serde-wasm-bindgen = "0.4"

# Error handling and utilities
anyhow = "1.0"
thiserror = "1.0"
console_error_panic_hook = { version = "0.1", optional = true }
once_cell = "1.20"

# LZ4 compression support
lz4_flex = "0.11"

# For Parquet ChunkReader support
bytes = "1.0"

# Serialization support for debugging/diagnostics
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2.104"
js-sys = "0.3.81"
wasm-bindgen-futures = "0.4.54"
```

---

## 10. Testing Requirements

1. **Model-Based Design (MBD)**: Test design and generation must follow model-based principles. Tests should be derivable from API models and state-transition diagrams where feasible.
2. **Unit / Integration tests**: Include round-trip tests using sample Arrow, Feather, and Parquet files (read → transform → write → verify).
3. **Zero-copy verification**: Provide tests that demonstrate zero-copy behavior (measure copy counts, validate handle-backed references).
4. **E2E tests**: Run end-to-end tests in major browsers and in Node.js with bundlers (vite, rollup).
5. **Acceptance criteria for tests**:

   * Round-trip read/write succeeds for Arrow / Feather / Parquet.
   * Zero-copy mode does not perform unnecessary memory copies (measurable evidence).
   * TypeScript compilation (`tsc`) passes and `any` is not present in public APIs.

---

## 11. Documentation and Deliverables

* API specification (English / Japanese versions).
* TypeScript declaration files (`.d.ts`) and usage examples for browser and Node.js.
* Rust crate(s) for core implementation and wasm build artifacts.
* Test suites (unit, integration, E2E) and test run instructions.
* Zero-copy verification report including performance and memory measurements.

---

## 12. Acceptance Criteria

1. Arrow / Feather / Parquet read and write functionality operates as specified.
2. Consumers can obtain schema, field names, and extract columns via the public JS/TS API.
3. Zero-copy memory transfer is implemented so users do not need to manipulate memory directly.
4. Public SDK code is free of `this`, `any`, `is`, and `try` / `catch` (statistically verifiable).
5. All unit, integration, and E2E tests pass.

---

## 13. Additional Notes and Roadmap Items

* Parquet-specific strategies for compression and chunking will be phased in according to implementation maturity.
* Additional compression codecs (e.g., Zstd) are candidates for future addition; such additions must preserve the rule of not exposing raw C bindings.
* The ban on `try` / `catch` in public code may be unconventional for some JS developers; provide a migration guide and examples to ease adoption.

# 14. Project layout (directory and crate layout)

```
/crates
  /core                    # Rust core crate (wasm-compatible)
    src/
      lib.rs
      fs.rs                # thin wrappers for reading/writing Arrow files
      ipc.rs               # Arrow IPC read/write helpers and LZ4 handling
      mem.rs               # WASM memory manager and TableHandle lifecycle
      errors.rs            # Error types
    Cargo.toml

  /docs
    design.md
    user_manual.md

  /examples
    /node                 # Node examples (TypeScript)
    /browser              # Browser examples (TypeScript)
  /pkg                    # wasm-pack / bundling outputs
  /tests_rs
    unit_tests.rs
    integration_tests.rs
  /mbd_tests
    model_based_tests
    vite_node test

  /.serena
    /memories
     progress_report.md
Cargo.toml
README.md
```


# 15. References

* Apache Arrow Rust IPC documentation — Arrow IPC (Rust).
  [https://arrow.apache.org/rust/arrow_ipc/index.html](https://arrow.apache.org/rust/arrow_ipc/index.html)

* wasm-bindgen — source and documentation (for WASM ↔ JS interop).
  [https://rustwasm.github.io/wasm-bindgen/](https://rustwasm.github.io/wasm-bindgen/)

* (Project-local sample) Obsidian MCP server Arrow examples (path: `Rust/arrow`) — local sample code referenced by your team.
