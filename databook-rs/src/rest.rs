use std::collections::HashMap;

pub struct InvokePluginRequest {
    pub name: String,
    pub options: HashMap<String, String>,
}

pub struct InvokePluginResponse {
    pub output: String,
}

