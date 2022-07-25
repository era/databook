use std::collections::HashMap;

enum InvocationError {}

#[derive(Debug)]
struct Plugin {
    plugin_type: String,
}

#[derive(Debug)]
struct PlugManager {
    // any plugin (wasm files) in this folder will be registered
    folder: Box<std::path::Path>,

    //all plugins registered, <Type, Plugin>
    plugins: HashMap<String, Plugin>,
}

impl PlugManager {
    pub fn new(folder: Box<std::path::Path>) -> Self {
        return PlugManager {
            folder,
            plugins: HashMap::new(),
        };
    }
    // invokes the plugin using wasm
    pub fn invoke(plugin_type: String, input: String) -> Result<String, InvocationError> {
        Ok(input)
    }
}
