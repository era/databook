use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvokePluginRequest {
    pub name: String,
    pub options: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InvokePluginResponse {
    pub output: Option<String>,
    pub error: Option<String>,
}
