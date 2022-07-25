use std::collections::HashMap;

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
}
