use std::fmt;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

wit_bindgen_wasmtime::import!("../wit/plugin.wit");
use plugin::{Plugin, PluginData};

struct Context<I, E> {
    wasi: wasmtime_wasi::WasiCtx,
    imports: I,
    exports: E,
}
type PluginStore = Store<Context<PluginData, PluginData>>;

#[derive(Debug)]
pub enum WasmError {
    GenericError(String),
}

pub struct WasmModule {
    module: Module,
    linker: Linker<Context<PluginData, PluginData>>,
    engine: Engine,
}
impl fmt::Debug for WasmModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WasmModule").finish()
    }
}

fn default_wasi() -> wasmtime_wasi::WasiCtx {
    wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .build()
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

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<PluginData, PluginData>| {
            &mut cx.wasi
        })
        .map_err(|e| WasmError::GenericError(e.to_string()))?;

        Ok(Self {
            module,
            linker,
            engine,
        })
    }

    fn new_store(&self) -> Store<Context<PluginData, PluginData>> {
        Store::new(
            &self.engine,
            Context {
                wasi: default_wasi(),
                imports: PluginData::default(),
                exports: PluginData::default(),
            },
        )
    }

    // invokes the plugin and gets the output from it
    pub fn invoke(&mut self, input: String) -> Result<String, WasmError> {
        let mut store = self.new_store();
        let (plugin, _instance) =
            Plugin::instantiate(&mut store, &self.module, &mut self.linker, |cx| {
                &mut cx.exports
            })
            .map_err(|e| WasmError::GenericError(e.to_string()))?;

        plugin
            .invoke(&mut store, &input)
            .map_err(|e| WasmError::GenericError(e.to_string()))
    }
}
