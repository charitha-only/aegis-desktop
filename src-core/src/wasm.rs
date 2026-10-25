//! Validated WebAssembly execution with no host imports, bounded memory and fuel.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Mutex, time::Instant};
use wasmi::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleInfo {
    pub module_id: String,
    pub name: String,
    pub version: String,
    pub sha256_hash: String,
    pub memory_limit_mb: u32,
    pub instruction_limit: u64,
    pub allowed_imports: Vec<String>,
    pub status: String,
    pub execution_count: u64,
    #[serde(skip_serializing, default)]
    pub bytecode: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmExecutionResult {
    pub module_id: String,
    pub exit_code: i32,
    pub gas_consumed: u64,
    pub memory_used_bytes: usize,
    pub output_payload: Value,
    pub duration_us: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionTier {
    Auto,
    Interpreter,
    WgpuJit,
}
pub struct WasmSandbox {
    modules: Mutex<HashMap<String, WasmModuleInfo>>,
    engine: Engine,
}
impl WasmSandbox {
    pub fn new() -> Self {
        let mut config = Config::default();
        config.consume_fuel(true);
        let vm = Self {
            modules: Mutex::new(HashMap::new()),
            engine: Engine::new(&config),
        };
        vm.load_wasm_binary(
            "wasm-crypto-accelerator",
            "Integer arithmetic demo (not cryptography)",
            vec![
                0x00, 0x61, 0x73, 0x6D, // \0asm
                0x01, 0x00, 0x00, 0x00, // version 1
                // Section 1: Type
                0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F,
                // Section 3: Function
                0x03, 0x02, 0x01, 0x00, // Section 7: Export
                0x07, 0x0B, 0x01, 0x07, 0x65, 0x78, 0x65, 0x63, 0x75, 0x74, 0x65, 0x00, 0x00,
                // Section 10: Code
                0x0A, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B,
            ],
        )
        .expect("valid built-in module");
        vm.load_wasm_binary(
            "wasm-schema-validator",
            "Non-empty object demo (not JSON Schema)",
            vec![
                0x00, 0x61, 0x73, 0x6D, // \0asm
                0x01, 0x00, 0x00, 0x00, // version 1
                // Section 1: Type
                0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F,
                // Section 3: Function
                0x03, 0x02, 0x01, 0x00, // Section 7: Export "validate"
                0x07, 0x0C, 0x01, 0x08, 0x76, 0x61, 0x6C, 0x69, 0x64, 0x61, 0x74, 0x65, 0x00, 0x00,
                // Section 10: Code
                0x0A, 0x0F, 0x01, 0x0D, 0x00, 0x20, 0x01, 0x45, 0x04, 0x7F, 0x41, 0x7F, 0x05, 0x41,
                0x00, 0x0B, 0x0B,
            ],
        )
        .expect("valid built-in module");
        vm
    }
    pub fn list_modules(&self) -> Vec<WasmModuleInfo> {
        self.modules.lock().unwrap().values().cloned().collect()
    }
    pub fn load_wasm_binary(
        &self,
        id: &str,
        name: &str,
        bytes: Vec<u8>,
    ) -> Result<WasmModuleInfo, String> {
        if bytes.len() > 1024 * 1024 || !bytes.starts_with(b"\0asm") {
            return Err("Invalid binary or module exceeds 1 MiB".into());
        }
        let m = Module::new(&self.engine, &bytes[..])
            .map_err(|e| format!("Wasm validation failed: {e}"))?;
        if m.imports().next().is_some() {
            return Err("Wasm host imports are disabled".into());
        }
        let info = WasmModuleInfo {
            module_id: id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            sha256_hash: hex::encode(Sha256::digest(&bytes)),
            memory_limit_mb: 16,
            instruction_limit: 100_000,
            allowed_imports: vec![],
            status: "VALIDATED".into(),
            execution_count: 0,
            bytecode: bytes,
        };
        let mut modules = self.modules.lock().unwrap();
        if modules.len() >= 32 && !modules.contains_key(id) {
            return Err("Module registry quota exceeded".into());
        }
        modules.insert(id.into(), info.clone());
        Ok(info)
    }
    pub fn execute_module(&self, id: &str, input: &Value) -> Result<WasmExecutionResult, String> {
        self.execute_module_with_tier(id, input, ExecutionTier::Auto)
    }
    pub fn execute_module_with_tier(
        &self,
        id: &str,
        input: &Value,
        tier: ExecutionTier,
    ) -> Result<WasmExecutionResult, String> {
        if tier == ExecutionTier::WgpuJit {
            return Err("UNAVAILABLE: GPU execution is not implemented; use INTERPRETER".into());
        }
        let bytes = serde_json::to_vec(input).map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Wasm input exceeds 64 KiB".into());
        }
        let info = {
            let mut m = self.modules.lock().unwrap();
            let m = m.get_mut(id).ok_or("Unknown Wasm module")?;
            m.execution_count += 1;
            m.clone()
        };
        let started = Instant::now();
        let module = Module::new(&self.engine, &info.bytecode[..]).map_err(|e| e.to_string())?;
        let limits = StoreLimitsBuilder::new()
            .memory_size(16 * 1024 * 1024)
            .instances(1)
            .memories(1)
            .tables(1)
            .table_elements(4096)
            .trap_on_grow_failure(true)
            .build();
        let mut store = Store::new(&self.engine, limits);
        store.limiter(|s: &mut StoreLimits| s);
        store
            .set_fuel(info.instruction_limit)
            .map_err(|e| e.to_string())?;
        let instance = Linker::<StoreLimits>::new(&self.engine)
            .instantiate(&mut store, &module)
            .and_then(|p| p.start(&mut store))
            .map_err(|e| format!("Wasm instantiation trap: {e}"))?;
        let name = if instance.get_func(&store, "execute").is_some() {
            "execute"
        } else {
            "validate"
        };
        let result = if let Ok(f) = instance.get_typed_func::<(), i32>(&store, name) {
            f.call(&mut store, ())
        } else if let Ok(f) = instance.get_typed_func::<(i32, i32), i32>(&store, name) {
            let flag = i32::from(input.as_object().is_some_and(|o| !o.is_empty()));
            f.call(&mut store, (bytes.len() as i32, flag))
        } else if let Ok(f) = instance.get_typed_func::<i32, i32>(&store, name) {
            f.call(&mut store, bytes.len() as i32)
        } else {
            return Err(
                "Export execute/validate must return i32 and take zero, one or two i32 arguments"
                    .into(),
            );
        }
        .map_err(|e| format!("Wasm execution trap: {e}"))?;
        Ok(WasmExecutionResult {
            module_id: id.into(),
            exit_code: 0,
            gas_consumed: info.instruction_limit - store.get_fuel().unwrap_or(0),
            memory_used_bytes: bytes.len(),
            output_payload: json!({"engine":"wasmi","tier":"INTERPRETER","bytecodeVerified":true,"stackResult":result,"fuelLimit":info.instruction_limit,"memoryLimitBytes":16*1024*1024,"hostImportsAllowed":false}),
            duration_us: started.elapsed().as_micros() as u64,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn run(wat: &str) -> Result<WasmExecutionResult, String> {
        let vm = WasmSandbox::new();
        vm.load_wasm_binary("test", "test", wat::parse_str(wat).unwrap())?;
        vm.execute_module("test", &json!({}))
    }
    #[test]
    fn branch_and_signed_constants() {
        assert_eq!(run(r#"(module (func (export "execute") (result i32) i32.const 0 if (result i32) i32.const 1 else i32.const 2 end i32.const 3 i32.add))"#).unwrap().output_payload["stackResult"],5);
        assert_eq!(
            run(r#"(module (func (export "execute") (result i32) i32.const -1))"#)
                .unwrap()
                .output_payload["stackResult"],
            -1
        );
    }
    #[test]
    fn trap_and_fuel() {
        assert!(run(r#"(module (func (export "execute") (result i32) unreachable))"#).is_err());
        assert!(
            run(r#"(module (func (export "execute") (result i32) (loop br 0) i32.const 0))"#)
                .is_err()
        );
    }
    #[test]
    fn malformed_imports_and_memory() {
        let v = WasmSandbox::new();
        assert!(v
            .load_wasm_binary("bad", "bad", b"\0asm1234".to_vec())
            .is_err());
        assert!(run(r#"(module (import "env" "x" (func)) (func (export "execute") (result i32) i32.const 0))"#).is_err());
        assert!(
            run(r#"(module (memory 257) (func (export "execute") (result i32) i32.const 0))"#)
                .is_err()
        );
    }
    #[test]
    fn gpu_fails_closed() {
        assert!(WasmSandbox::new()
            .execute_module_with_tier("wasm-schema-validator", &json!({}), ExecutionTier::WgpuJit)
            .is_err());
    }
}
