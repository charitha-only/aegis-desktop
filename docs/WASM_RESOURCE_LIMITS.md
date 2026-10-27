# Wasm resource accounting and module reuse

The interpreter accepts validated WebAssembly binaries up to 1 MiB, without host imports. Each execution receives a fresh store/instance with 100,000 fuel, at most one instance, one linear memory capped at 16 MiB, and one table capped at 4,096 elements. Growth failure traps. GPU execution remains unavailable.

## v0.25.13 corrections

`memory_used_bytes` previously returned serialized JSON input length, not guest memory. It now reports allocated guest linear-memory bytes, including unexported memory and permitted growth, through the same resource limiter that enforces the bound. A regression reproduced a two-page guest incorrectly reporting 2 bytes; it now reports 131,072 bytes. `output_payload.memoryMeasurement` is `guestLinearMemoryBytes`; `inputBytes` separately reports serialized input length. A guest without linear memory reports zero.

This measure is **not process RSS or a total host-memory budget**. Interpreter/compiler allocations, bytecode/data structures, stacks, tables and host input/output buffers are outside that measurement. Fuel meters guest execution, not parsing/validation/translation time or a strict wall-clock deadline. Process isolation, hard host memory/time budgets and physical low-resource acceptance remain separate work.

Previously metadata listing cloned the raw binary of every module, and execution reparsed the binary every time. The registry now stores the validated module and reuses its shared compiled representation. Metadata contains no binary copies. A fresh store is still created per execution: guest globals/memory are not shared between calls. Replacement swaps the registered module after successful validation; in-flight calls keep their selected module snapshot.

The legacy public Rust `WasmModuleInfo.bytecode` field remains for source compatibility but returned metadata always leaves it empty. IPC already omitted this field. Hosts needing original bytes should retain the bytes they supply to `load_wasm_binary` rather than retrieving them from metadata. The registry remains capped at 32 modules, including built-ins; full-registry rejection happens before validating another new module. Module IDs are 1–128 ASCII letters/digits/dots/underscores/hyphens. Display names are 1–1,024 UTF-8 bytes without NUL. Rejected loads do not add an entry.

For `wasm_execute`, omitted tier selects AUTO. Explicit tiers must be `AUTO`, `INTERPRETER` or `WGPU_JIT`; unknown strings, null and non-string values now fail before execution. AUTO uses the interpreter and WGPU_JIT returns unavailable. `bytecodeVerified` means Wasm validation succeeded; it does not mean publisher-signature verification, remote attestation or an independent audit.

## Verification

Run `cargo test --locked --manifest-path src-core/Cargo.toml wasm`. Public module/IPC tests cover unexported memory growth, zero-memory guests, memory/table/fuel traps, empty metadata bytecode, replacement and fresh guest state, oversized identity rejection and malformed execution tiers. After building the release core, `node aegis-cli/bin/portable-smoke.js` checks actual IPC memory accounting and invalid-tier rejection under owned temporary platform roots. These are hosted/API checks, not low-end physical-device performance claims.
