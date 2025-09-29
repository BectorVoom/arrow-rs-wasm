# 1. Purpose and scope

**Purpose.** Provide a Rust → WebAssembly core plus a small, type-safe JavaScript/TypeScript façade that together enable:

* zero-copy transfer of Arrow buffers between WASM and host (high-level, user-friendly API; WASM owns memory),
* reading Arrow/Feather/Parquet files (including those whose IPC blocks use LZ4 internal compression),
* creating Arrow/Feather/Parquet files with configurable IPC write options (including LZ4),
* a plugin extensibility model (plugins implement additional column types or behaviours such as geometry),
* clean separation of core and plugins so core remains minimal and tree-shakeable.

**Out of scope.** Native C bindings, exposing raw pointers to JavaScript, runtime code generators, and embedding third-party plugin code in the core crate. The core must avoid exporting C-style data structures to userland.

---

# 2. High-level goals & constraints

1. **Modular design** — the project is divided into crates/modules so users can import only what they need (easy tree-shaking on the JS side).
2. **Plugin-first architecture** — core provides only a plugin registration/validation API; all optional features (e.g., geometry type support) are delivered as separate plugin crates.
3. **Zero-copy WASM ↔ JS semantics** — the WASM module is the authoritative owner of raw Arrow memory. The JavaScript/TypeScript interface never requires users to manipulate raw memory. High-level objects (handles) are passed between JS and WASM; operations are performed by calling exported methods in WASM. JS receives copies only when explicitly requested.
4. **JS/TS style rules** — API examples and generated TypeScript must NOT use `this`, `any`, `is` (type-predicate syntax), `try`, or `catch` in public-facing examples or types. Errors are surfaced as explicit `Result`-like return values or rejected Promises. (See error section.)
5. **File formats** — support reading/writing Arrow IPC, Feather, and Parquet files where IPC frames may be LZ4-compressed. Writing must allow `arrow_ipc::writer::IpcWriteOptions` configuration to enable LZ4 internal compression.
6. **Avoid C bindings** — do not expose `extern "C"` style structs or bind directly to C ABI for exported APIs; use `wasm-bindgen` for interop.
7. **Type safety** — TypeScript façade must be strongly typed (no `any`), exposing explicit interfaces and result types.
8. **Testing** — browser tests are model-based (property-based + model-based test harness) in addition to unit and integration tests.
9. **Documentation** — include README, user manual examples, plugin author guide, and API reference.

References used while writing this specification are listed at the end.

---

# 3. Project layout (directory and crate layout)

```
/crates
  /core                    # Rust core crate (wasm-compatible)
    src/
      lib.rs
      fs.rs                # thin wrappers for reading/writing Arrow files
      ipc.rs               # Arrow IPC read/write helpers and LZ4 handling
      mem.rs               # WASM memory manager and TableHandle lifecycle
      plugin.rs            # Plugin trait and plugin manager
      errors.rs            # Error types
    Cargo.toml
  /plugins                 # workspace folder for plugin crates
    /plugin-geo            # example plugin for geometry column types
      src/lib.rs
      Cargo.toml
  /docs
    design.md
    user_manual.md
    plugin_author_guide.md
  /examples
    /node                 # Node examples (TypeScript)
    /browser              # Browser examples (TypeScript)
  /pkg                    # wasm-pack / bundling outputs
  /mbd_tests
    unit_tests.rs
    integration_tests.rs
    model_based_tests.rs
  /.serena
    /memories
     progress_report.md
Cargo.toml
README.md
```

---

# 4. Rust core: modules and public types

## 4.1 Core responsibilities

* Provide **safe** Rust APIs to read and write Arrow/Feather/Parquet using `arrow` crates and `arrow-ipc` with LZ4.
* Manage WASM memory and lifecycle of Table handles; expose stable handle numeric types to JS.
* Expose minimal plugin registration/validation API.
* Provide deterministic and minimal glue functions for `wasm-bindgen`.

## 4.2 Key public Rust types (summaries and example signatures)

```rust
/// Opaque handle id returned to JS/TS. Small integer type for stable mapping.
pub type TableHandle = u32;

/// Schema description returned to JS (serde-serializable for debug)
#[derive(Serialize, Deserialize)]
pub struct SchemaSummary {
    pub columns: Vec<ColumnSummary>,
}

#[derive(Serialize, Deserialize)]
pub struct ColumnSummary {
    pub name: String,
    pub arrow_type: String, // e.g. "Int64", "Utf8", "LargeBinary", "Geometry<...>"
    pub nullable: bool,
}

/// Core error enum used across WASM boundary (string-friendly).
#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Invalid handle: {0}")]
    InvalidHandle(u32),
    #[error("Other: {0}")]
    Other(String),
}
```

## 4.3 Memory manager & lifecycle

`core::mem` exposes functions (via `wasm-bindgen`) that allocate and free `Table` objects inside WASM and return `TableHandle` integers to JS.

Example Rust APIs (internal; exported via wasm-bindgen):

```rust
/// Create a Table from an in-memory IPC stream (zero-copy where possible).
/// Returns Result<TableHandle, CoreError>
pub fn create_table_from_ipc(ipc_data_ptr: *const u8, ipc_data_len: usize) -> Result<TableHandle, CoreError>;

/// Free the Table represented by handle.
pub fn free_table(handle: TableHandle) -> Result<(), CoreError>;

/// Return a JSON `SchemaSummary` string for the table.
pub fn table_schema_summary(handle: TableHandle) -> Result<String, CoreError>;
```

> Implementation note: `ipc_data_ptr` is only used inside WASM; callers should pass the memory pointer via wasm-bindgen helpers. JS façade will not require users to call raw pointer helpers.

## 4.4 File I/O (core::fs)

APIs to read files from a byte buffer and to write to byte buffers.

```rust
/// Read Arrow/Feather/Parquet file bytes and returns a managed TableHandle.
pub fn read_table_from_bytes(bytes_ptr: *const u8, bytes_len: usize) -> Result<TableHandle, CoreError>;

/// Write a Table to an in-memory Arrow IPC file using provided IpcWriteOptions.
/// Returns a Vec<u8> owned by WASM (JS will receive it via a copy-on-request API).
pub fn write_table_to_ipc(handle: TableHandle, options: IpcWriteOptions) -> Result<Vec<u8>, CoreError>;
```

`IpcWriteOptions` is the Arrow IPC writer options (re-export or wrapper) configured to support LZ4 internal compression via the `arrow-ipc` crate with `lz4` feature.

## 4.5 IPC and LZ4 support (core::ipc)

Provide explicit factory and helper functions that construct the right `IpcWriteOptions` to enable LZ4 internal compression and any other relevant options:

```rust
pub fn default_lz4_ipc_options() -> IpcWriteOptions;
pub fn enable_lz4_on_options(opts: &mut IpcWriteOptions, lz4: bool);
```

---

# 5. Plugin architecture

## 5.1 Design goals

* Plugins are separate crates (e.g., `plugin-geo`) that implement known traits and a registration function.
* The core exposes a **minimal** plugin manager that can load plugin metadata and validate it at registration time (matching expected symbol names exported via wasm-bindgen).
* Plugins implement domain-specific conversions (e.g., geometry serialization into Arrow Binary with known metadata).

## 5.2 Rust trait: `ArrowPlugin`

```rust
pub trait ArrowPlugin: Sync + Send {
    /// A stable plugin ID, e.g., "io.arrow.plugin.geo.v1"
    fn plugin_id(&self) -> &'static str;

    /// Validate a SchemaSummary or Arrow Field; return Ok(()) if supported.
    fn validate_field(&self, field: &arrow::datatypes::Field) -> Result<(), CoreError>;

    /// Optional conversion helper invoked when reading/writing.
    /// Not allowed to access JS memory directly; must operate within WASM.
    fn on_read_column(&self, field: &arrow::datatypes::Field, array: &dyn ArrayRef) -> Result<(), CoreError>;
}
```

## 5.3 Registration and minimal core validation

Core exposes these functions to JS:

```rust
/// Register a plugin by its exported registration name.
/// The core will only store plugin metadata and a boxed trait object reference.
pub fn register_plugin(plugin_id: &str) -> Result<(), CoreError>;

/// Validate a plugin by confirming its expected registration signature and agreed-upon plugin id.
/// Only basic validation lives in core; deeper behavior occurs in the plugin crate.
pub fn validate_plugin(plugin_id: &str) -> Result<(), CoreError>;
```

**Plugin author notes**: a plugin crate must provide a registration function with a known signature exported through wasm-bindgen. The plugin performs heavy-lifting conversion logic; the core only stores the registration.

---

# 6. JavaScript / TypeScript façade (public API)

## 6.1 Façade goals

* High-level, strongly typed TS API.
* No use of `this`, `any`, `is`, `try`, or `catch` in examples and public API.
* All async functions return `Promise<Result<T, ErrorSummary>>` where `Result` is a small discriminated union.

## 6.2 TypeScript types (explicit)

```ts
// Discriminated union for results
export type Ok<T> = { ok: true; value: T };
export type Err = { ok: false; error: string };
export type Result<T> = Ok<T> | Err;

// Table handle (opaque numeric id)
export type TableHandle = number;

export interface ColumnInfo {
  name: string;
  arrowType: string;
  nullable: boolean;
}

export interface SchemaSummary {
  columns: ColumnInfo[];
}

export interface WasmInitOptions {
  // optional configuration for the WASM module
  enableConsoleLogs?: boolean;
}
```

## 6.3 JS/TS functions (signature examples)

All functions below return `Promise<Result<...>>` to avoid `try/catch` in examples.

```ts
// Initialize WASM module (must be called once)
export function initWasm(bytes?: ArrayBuffer, opts?: WasmInitOptions): Promise<Result<void>>;

// Read an Arrow/Feather/Parquet file from an ArrayBuffer
export function readTableFromArrayBuffer(data: ArrayBuffer): Promise<Result<TableHandle>>;

// Get schema summary for a table handle
export function getSchemaSummary(handle: TableHandle): Promise<Result<SchemaSummary>>;

// Free table handle
export function freeTable(handle: TableHandle): Promise<Result<void>>;

// Write table to IPC bytes with LZ4 option
export function writeTableToIpc(handle: TableHandle, enableLz4: boolean): Promise<Result<ArrayBuffer>>;

// Register a plugin by id
export function registerPlugin(pluginId: string): Promise<Result<void>>;
```

> Implementation note: none of the façade functions expose or require users to handle raw memory pointers. The façade converts JS `ArrayBuffer` data into WASM-owned buffers with an internal copy-on-first-use or mapped memory by `wasm-bindgen` glue — but the user never performs manual zero-copy operations.

## 6.4 Example (Node or Browser fetch + read)

```ts
import { initWasm, readTableFromArrayBuffer, getSchemaSummary, writeTableToIpc } from 'arrow-rs-wasm';

const wasmResult = await initWasm();
if (!wasmResult.ok) {
  console.error('WASM init failed', wasmResult.error);
  // handle error without try/catch
}

const ab: ArrayBuffer = await fetch('data/simple.arrow').then(r => r.arrayBuffer());
const readResult = await readTableFromArrayBuffer(ab);
if (!readResult.ok) {
  console.error('read failed', readResult.error);
} else {
  const handle = readResult.value;
  const schemaResult = await getSchemaSummary(handle);
  if (schemaResult.ok) {
    console.table(schemaResult.value.columns);
  }
  const ipcResult = await writeTableToIpc(handle, true); // enable LZ4
  if (ipcResult.ok) {
    const ipcBuffer = ipcResult.value;
    // ipcBuffer is an ArrayBuffer containing IPC bytes (owned by JS)
  }
  await freeTable(handle);
}
```

This example does not use `try`/`catch` and uses explicit `Result`-style error handling.

---

# 7. WASM ↔ JS zero-copy memory model (explicit)

1. **WASM owns Arrow buffers.** Raw Arrow arrays and their aligned buffers are allocated and owned by the WASM runtime. JS is never given raw pointers.
2. **Handles instead of pointers.** JS operates with `TableHandle` integers. All heavy operations happen in WASM functions exported via `wasm-bindgen`.
3. **Zero-copy read path (concept):**

   * When JS calls `readTableFromArrayBuffer`, the runtime passes the `ArrayBuffer` into WASM using the `wasm-bindgen` copy or memory-mapping strategy. If browser supports `SharedArrayBuffer` or wasm memory mapping, implementation will avoid extra copies internally; however this detail is encapsulated — the façade does not expose the mechanism.
   * The table data remains inside WASM. JS can request a view (immutable snapshot) which forces a copy to JS-owned memory. By default, no copies are made unless the user requests a serialized output (`writeTableToIpc`) or a snapshot.
4. **Memory lifecycle.** JS must explicitly call `freeTable(handle)` to release WASM-side resources. The façade may implement a finalizer that warns when handles are leaked, but explicit `freeTable` is primary.

---

# 8. File reading/writing specifics & LZ4

* **Reading**: the core must accept Arrow IPC (single/multiple record batches), Feather, and Parquet byte streams and produce a `TableHandle`.
* **Writing**: the core must expose a function that writes IPC bytes while accepting an `IpcWriteOptions` configuration that can enable LZ4 internal compression. The implementation will rely on `arrow-ipc` with the `lz4` feature enabled.
* **Feather**: treated as Arrow IPC with Feather metadata; writing Feather-format bytes uses the same IPC writer with specific metadata flags.
* **Parquet**: when writing Parquet, use the appropriate Arrow → Parquet writer if available. Note: Parquet is not IPC, so ensure separation of IPC write APIs and Parquet writer APIs in the core.

Rust example signature:

```rust
pub fn write_table_with_options(handle: TableHandle, ipc_options: IpcWriteOptions) -> Result<Vec<u8>, CoreError>;
```

---

# 9. Error handling (JS/TS + WASM)

* **Rust side**: use `thiserror` / `anyhow` to build errors; all errors crossing the WASM boundary are serialized into `CoreError` equivalent string codes and messages.
* **JS/TS side**: the façade returns `Promise<Result<T>>` instead of throwing. This avoids `try/catch` in examples and in end-user code. `Err` contains a stable string message and optional structured metadata for programmatic inspection.

---

# 10. TypeScript generation & type-safety

* Generate `.d.ts` files for all façade exports.
* Avoid `any`. Use explicit unions and interfaces.
* Provide a `Result<T>` utility and `TableHandle` alias.
* Provide typed plugin registration functions such that plugin contracts are type-checked at compile-time in TS.

---

# 11. Browser tests and model-based testing

* **Testing tiers**:

  * Unit tests: Rust unit tests for IPC read/write, LZ4, plugin validation logic.
  * Integration tests: wasm-pack + Node tests that load actual Arrow, Feather, Parquet sample files.
  * Model-based browser tests: property-based and model-based tests run in a headless browser harness that:

    * uses model-based generators to create random schema + data,
    * writes the data via core write functions (with and without LZ4),
    * reads back and validates schema and data invariants,
    * tests plugin behaviours (eg geometry plugin round-trips).
* Use existing model-based testing frameworks (e.g., proptest or wasm-friendly alternatives) and a test harness that runs in CI.

---

# 12. Conventions and rules for authors and implementers

* **No C-style exports**: do not expose C bindings or raw pointers in JavaScript interface.
* **Plugin isolation**: plugin crates must not be compiled into the core artifact; they are separate crates and may be loaded by consumers.
* **Minimal core verification only**: core plugin manager validates plugin identity and signature; plugins are responsible for runtime behavior.
* **No `this`, `any`, `is`, `try`, `catch`** in public TypeScript examples and generated declaration files. Internals may use `try`/`catch` as needed but exported façade uses `Result` style.

---

# 13. Example: Geometry plugin (high level)

* Plugin crate: `plugin-geo`
* Implements `ArrowPlugin`.
* Exposes `register_geo_plugin()` exported function for wasm-bindgen which the core calls via `registerPlugin("io.arrow.plugin.geo.v1")`.
* Plugin converts geometry columns encoded as `LargeBinary` with well-known metadata into typed geometry accessors in WASM.

---

# 14. Packaging and distribution

* Build the Rust core with `wasm-bindgen` and package with `wasm-pack` into `/pkg`.
* Provide `package.json` with `exports` and modern ESM entry points.
* Keep plugin crates as separate packages/crates with their own `Cargo.toml` and optional JS wrappers if needed.

---

# 15. Security, performance and ergonomics notes

* Use safe Rust APIs and avoid `unsafe` except where strictly necessary for performance; all `unsafe` must be encapsulated and audited.
* Prefer zero-copy buffer handling patterns inside WASM; however do not expose raw ArrayBuffer views to userland—always provide handles and high-level operations.
* Provide diagnostics APIs (compact JSON) for debugging; these return structured data and should be gated behind optional flags.

---

# 16. Minimal API summary (quick reference)

**Rust (core)**

* `create_table_from_ipc(ipc_data_ptr, ipc_data_len) -> Result<TableHandle, CoreError>`
* `read_table_from_bytes(bytes_ptr, bytes_len) -> Result<TableHandle, CoreError>`
* `write_table_to_ipc(handle, IpcWriteOptions) -> Result<Vec<u8>, CoreError>`
* `register_plugin(plugin_id: &str) -> Result<(), CoreError>`
* `free_table(handle) -> Result<(), CoreError>`
* `table_schema_summary(handle) -> Result<String, CoreError>`

**TypeScript façade**

* `initWasm(bytes?: ArrayBuffer, opts?: WasmInitOptions): Promise<Result<void>>`
* `readTableFromArrayBuffer(data: ArrayBuffer): Promise<Result<TableHandle>>`
* `getSchemaSummary(handle: TableHandle): Promise<Result<SchemaSummary>>`
* `writeTableToIpc(handle: TableHandle, enableLz4: boolean): Promise<Result<ArrayBuffer>>`
* `freeTable(handle: TableHandle): Promise<Result<void>>`
* `registerPlugin(pluginId: string): Promise<Result<void>>`

---

# 17. Dependencies (as requested)

Use the Arrow crates with LZ4 and ipc features enabled. Example `Cargo.toml` dependency block (as provided):

```toml
arrow = {version="56.1.0", default-features = false, features=["ipc"]}
arrow-ipc = {version="56.1.0", default-features = false, features=["lz4"]}
arrow-array = {version="56.1.0", default-features = false}
arrow-schema = {version="56.1.0", default-features = false}
arrow-buffer = {version="56.1.0", default-features = false}
arrow-data = {version="56.1.0", default-features = false}
arrow-select = {version="56.1.0", default-features = false}
arrow-cast = {version="56.1.0", default-features = false}
arrow-ord = {version="56.1.0", default-features = false}

wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }
serde-wasm-bindgen = "0.4"

anyhow = "1.0"
thiserror = "1.0"
console_error_panic_hook = { version = "0.1", optional = true }
once_cell = "1.20"

lz4_flex = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

# 18. Documentation & examples to include

* README with quickstart (Node and Browser).
* User manual with examples showing reading a sample Arrow file and writing IPC bytes with LZ4.
* Plugin author guide with a concrete `plugin-geo` example.
* API reference (Rust and TS) with signature tables.
* Testing guide describing the model-based browser tests.
* Manage progression with progress_report.md
---

# 19. References

* Apache Arrow Rust IPC documentation — Arrow IPC (Rust).
  [https://arrow.apache.org/rust/arrow_ipc/index.html](https://arrow.apache.org/rust/arrow_ipc/index.html)

* wasm-bindgen — source and documentation (for WASM ↔ JS interop).
  [https://rustwasm.github.io/wasm-bindgen/](https://rustwasm.github.io/wasm-bindgen/)

* (Project-local sample) Obsidian MCP server Arrow examples (path: `Rust/arrow`) — local sample code referenced by your team.
