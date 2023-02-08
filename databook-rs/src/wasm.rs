use crate::plugin_config::PluginConfig;
use crate::plugin_runtime::runtime::add_to_linker;
use crate::plugin_runtime::PluginRuntime;
use std::collections::HashMap;
use std::fmt;
use std::str;

use tracing::instrument;

use wasmtime::{Engine, Linker, Module, Store}; // 0.1.25

//wit_bindgen_host_wasmtime_rust::import!("../wit/plugin.wit");
use plugin::{Plugin, PluginData};

struct Context {
    wasi: wasmtime_wasi::WasiCtx,
    exports: PluginData,
    runtime: PluginRuntime,
}

#[derive(Debug)]
pub enum WasmError {
    GenericError(String),
}

pub struct WasmModule {
    module: Module,
    linker: Linker<Context>,
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

        wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context| &mut cx.wasi)
            .map_err(|e| WasmError::GenericError(e.to_string()))?;

        add_to_linker(&mut linker, |cx| &mut cx.runtime)
            .map_err(|e| WasmError::GenericError(e.to_string()))?;

        Ok(Self {
            module,
            linker,
            engine,
        })
    }

    fn new_store(&self, config: PluginConfig, input: HashMap<String, String>) -> Store<Context> {
        Store::new(
            &self.engine,
            Context {
                wasi: default_wasi(),
                exports: PluginData::default(),
                runtime: PluginRuntime { config, input },
            },
        )
    }

    // invokes the plugin and gets the output from it
    #[instrument]
    pub fn invoke<'a>(
        &self,
        input: HashMap<String, String>,
        config: PluginConfig,
    ) -> Result<String, WasmError> {
        let mut store = self.new_store(config, input);
        let (plugin, _instance) =
            Plugin::instantiate(&mut store, &self.module, &mut self.linker.clone(), |cx| {
                &mut cx.exports
            })
            .map_err(|e| {
                tracing::error!("error while instantiating plugin {:?}", e);
                WasmError::GenericError(e.to_string())
            })?;

        plugin
            .invoke(&mut store)
            .map_err(|e| WasmError::GenericError(e.to_string()))
    }
}
