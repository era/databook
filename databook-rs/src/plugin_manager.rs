use std::collections::HashMap;

enum InvocationError {}
// Plugins are the combination of plugin.wasm and config.toml
// the config has the following keys:
// plugin_name (e.g. prometheus)
//  any other key will be available inside options hashmap
#[derive(Debug)]
struct Plugin {
    name: String,
    options: HashMap<String, String>,
}

#[derive(Debug)]
struct PluginManager {
    // any plugin (wasm files) in this folder will be registered
    folder: Box<std::path::Path>,

    //all plugins registered, <Name, Plugin>
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    pub fn new(folder: Box<std::path::Path>) -> Self {
        return Self {
            folder,
            plugins: HashMap::new(),
        };
    }
    // invokes the plugin using wasm
    pub fn invoke(plugin_name: String, input: String) -> Result<String, InvocationError> {
        Ok(input)
    }
}
