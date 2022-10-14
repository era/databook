use crate::plugin_config::PluginConfig;
use crate::wasm::WasmModule;

use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub enum InvocationError {
    PluginDoesNotExist,
    GenericError,
}
#[derive(Debug)]
pub enum PluginError {
    InvalidFolder,
}

#[derive(Debug)]
struct Plugin {
    config: PluginConfig,
    wasm: WasmModule,
}

impl Plugin {
    // new_from_folder returns a Plugin if a valid config.toml and plugin.wasm
    // can be found on the folder. If it cannot it will return None.
    pub fn new_from_folder(path: std::path::PathBuf) -> Option<Self> {
        let config_file = path.join("config.toml");

        if !config_file.is_file() {
            tracing::info!("no config file found, ignoring");
            return None;
        }

        let wasm_path = path.join("plugin.wasm");

        if !wasm_path.is_file() {
            tracing::info!("no wasm file found, ignoring");
            return None;
        }

        let config = PluginConfig::new_from_file(config_file);

        // loads the wasm module from the wasm_path
        let wasm = match WasmModule::new(wasm_path.to_str().unwrap()) {
            Ok(wasm) => wasm,
            Err(_) => return None,
        };

        match config {
            Some(config) => {
                tracing::info!("valid plugin");
                Some(Self { config, wasm })
            }
            None => None,
        }
    }

    // instantiate the wasm module and calls (exported) invoke function
    // passing the input to it
    pub fn invoke(&self, input: HashMap<String, String>) -> Result<String, InvocationError> {
        self.wasm
            .invoke(input, self.config.clone())
            .map_err(|_| InvocationError::GenericError)
    }
}

#[derive(Debug)]
pub struct PluginManager {
    // any plugin (wasm files) in this folder will be registered
    folder: std::path::PathBuf,

    //all plugins registered, <Name, Plugin>
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    pub fn new(folder: std::path::PathBuf) -> Self {
        Self {
            folder,
            plugins: HashMap::new(),
        }
    }

    pub fn registry(&mut self) -> Result<(), PluginError> {
        let paths = fs::read_dir(&self.folder).map_err(|_| PluginError::InvalidFolder)?;
        for entry in paths {
            let entry = entry.map_err(|_| PluginError::InvalidFolder)?.path();
            if entry.is_dir() {
                tracing::info!("trying to install plugin {:?}", entry.display());
                // invalid plugins are silently ignored
                Plugin::new_from_folder(entry)
                    .map(|p| self.plugins.insert(p.config.name.clone(), p));
            }
        }

        Ok(())
    }

    // invokes the plugin using wasm
    pub fn invoke(
        &self,
        plugin_name: &str,
        input: HashMap<String, String>,
    ) -> Result<String, InvocationError> {
        self.plugins
            .get(plugin_name)
            .map_or(Err(InvocationError::PluginDoesNotExist), |p| {
                p.invoke(input)
            })
    }
}
