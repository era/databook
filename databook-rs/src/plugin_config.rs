use serde::Deserialize;
use std::fs;
use toml;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub allowed_env_vars: Option<Vec<String>>,
}

impl PluginConfig {
    pub fn new_from_file(path: std::path::PathBuf) -> Option<Self> {
        match fs::read_to_string(&path) {
            Ok(config) => Self::new_from_str(&config),
            Err(e) => {
                tracing::warn!("unable to read toml config file {:?}", e);
                None
            }
        }
    }
    pub fn new_from_str(config: &str) -> Option<Self> {
        match toml::from_str::<PluginConfig>(&config) {
            Ok(config) => Some(config),
            Err(e) => {
                tracing::warn!("unable to parse config file {:?}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_create_config_from_str() {
        let config = PluginConfig::new_from_str("name = 'MyTest'\nallowed_env_vars=['A']");
        assert_eq!(
            Some(PluginConfig {
                name: "MyTest".into(),
                allowed_env_vars: Some(vec!["A".to_string()]),
            }),
            config
        );
    }
}
