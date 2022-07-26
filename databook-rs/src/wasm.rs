use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

#[derive(Debug)]
pub enum WasmError {
    GenericError(String),
}

#[derive(Debug)]
pub struct WasmModule {
    pub instance: Instance,
}

impl WasmModule {
    pub fn new(path: &str) -> Result<Self, WasmError> {
        // An engine stores and configures global compilation settings like
        // optimization level, enabled wasm features, etc.
        let engine = Engine::default();

        // We start off by creating a `Module` which represents a compiled form
        // of our input wasm module. In this case it'll be JIT-compiled after
        // we parse the text format.
        //could use from_binary as well
        let module =
            Module::from_file(&engine, path).map_err(|e| WasmError::GenericError(e.to_string()))?;

        // A `Store` is what will own instances, functions, globals, etc. All wasm
        // items are stored within a `Store`, and it's what we'll always be using to
        // interact with the wasm world. Custom data can be stored in stores but for
        // now we just use `()`.
        let mut store = Store::new(&engine, ());

        let mut linker = Linker::new(&engine);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| WasmError::GenericError(e.to_string()))?;

        //memory setup
        let memory_type = MemoryType::new(1, None);
        let memory = Memory::new(store, memory_type);
        Ok(Self { instance })
    }

    pub fn invoke(&self, input: String) -> Result<String, WasmError> {
        Ok(input)
    }
}
