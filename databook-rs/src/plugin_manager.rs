use crate::wasm::WasmModule;
use log::{info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml;
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

#[derive(Debug, Deserialize, PartialEq)]
struct PluginConfig {
    name: String,
}

impl PluginConfig {
    pub fn new_from_file(path: std::path::PathBuf) -> Option<Self> {
        match fs::read_to_string(&path) {
            Ok(config) => Self::new_from_str(&config),
            Err(e) => {
                warn!("unable to read toml config file {:?}", e);
                None
            }
        }
    }
    pub fn new_from_str(config: &str) -> Option<Self> {
        match toml::from_str::<PluginConfig>(&config) {
            Ok(config) => Some(config),
            Err(e) => {
                warn!("unable to parse config file {:?}", e);
                None
            }
        }
    }
}

impl Plugin {
    pub fn new_from_folder(path: std::path::PathBuf) -> Option<Self> {
        let config = path.join("config.toml");

        if !config.is_file() {
            info!("no config file found, ignoring");
            return None;
        }

        let wasm = path.join("plugin.wasm");

        if !wasm.is_file() {
            info!("no wasm file found, ignoring");
            return None;
        }

        let config = PluginConfig::new_from_file(config);
        let wasm = match WasmModule::new(wasm.to_str().unwrap()) {
            Ok(wasm) => wasm,
            Err(_) => return None, //TODO
        };

        match config {
            Some(config) => {
                info!("valid plugin");
                Some(Self { config, wasm })
            }
            None => None,
        }
    }

    pub fn invoke(&mut self, input: String) -> Result<String, InvocationError> {
        self.wasm
            .invoke(input)
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
        return Self {
            folder,
            plugins: HashMap::new(),
        };
    }

    pub fn registry(&mut self) -> Result<(), PluginError> {
        let paths = fs::read_dir(&self.folder).map_err(|_| PluginError::InvalidFolder)?;
        for entry in paths {
            let entry = entry.map_err(|_| PluginError::InvalidFolder)?.path();
            if entry.is_dir() {
                info!("trying to install plugin {:?}", entry.display());
                // invalid plugins are silently ignored
                Plugin::new_from_folder(entry)
                    .map(|p| self.plugins.insert(p.config.name.clone(), p));
            }
        }

        Ok(())
    }

    // invokes the plugin using wasm
    pub fn invoke(&mut self, plugin_name: &str, input: String) -> Result<String, InvocationError> {
        self.plugins
            .get_mut(plugin_name)
            .map_or(Err(InvocationError::PluginDoesNotExist), |p| {
                p.invoke(input)
            })
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_create_config_from_str() {
        let config = PluginConfig::new_from_str("name = 'MyTest'");
        assert_eq!(
            Some(PluginConfig {
                name: "MyTest".into()
            }),
            config
        );
    }
}
