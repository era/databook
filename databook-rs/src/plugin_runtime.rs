use crate::plugin_config::PluginConfig;
use std::collections::HashMap;
use std::{env, fmt};
use url::{Host, Url};
use wasmtime::{Engine, Module, Store, Config};
use wasmtime::component::Linker;
use wasmtime::component::Component;
use tracing::instrument;
//wit_bindgen_host_wasmtime_rust::export!("../wit/runtime.wit");
wasmtime::component::bindgen!({world: "databook"});
use host::{
    Error, HttpHeader, HttpRequest, HttpResponse, LogLevel
};

const HTTP_REQUEST_FAILED: u16 = 100;

impl PartialEq for HttpHeader {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}
impl Eq for HttpHeader {}

pub struct PluginRuntime {
    pub config: PluginConfig,
    pub input: HashMap<String, String>,
    pub wasi: wasmtime_wasi::WasiCtx,
}

fn default_wasi() -> wasmtime_wasi::WasiCtx {
    wasmtime_wasi::sync::WasiCtxBuilder::new()
        .inherit_stdio()
        .build()
}

impl host::Host for PluginRuntime {
    fn http(&mut self, request: HttpRequest) -> Result<Result<HttpResponse, Error>, anyhow::Error> {
        if !self.is_domain_allowed(&request.url) {
            return Ok(Err(Error {
                code: 0,
                message: format!(
                    "URL {:?} is not allowed, please add it to the allowed_domains",
                    request.url
                ),
            }));
        }

        let client = reqwest::blocking::Client::new();
        let uri = build_http_url(&request.url, &request.params);

        let req = match request.method.to_uppercase().as_str() {
            "GET" => client.get(uri),
            "POST" => client.post(uri),
            "PUT" => client.put(uri),
            "DELETE" => client.delete(uri),
            _ => {
                return Ok(Err(Error {
                    code: 0,
                    message: "Invalid HTTP METHOD".into(),
                }))
            }
        };

        let req = req.body(request.body.to_string());

        let req = http_headers_from_runtime(&request.headers, req);

        let response = req.send().map_err(|e| Error {
            code: HTTP_REQUEST_FAILED,
            message: e.to_string(),
        }).unwrap(); //TODO

        let headers = http_headers_to_runtime(response.headers());

        Ok(Ok(HttpResponse {
            status: response.status().as_u16(),
            response: response.text().map_err(|e| Error {
                code: 0,
                message: format!("Could not parse http response as text {:?}", e),
            })?,
            headers,
        }))
    }

    fn env(&mut self, key: String) -> Result<Result<String, Error>, anyhow::Error> {
        let key = &key;
        if self.is_env_var_allowed(key) {
            Ok(env::var(key).map_err(|e| Error {
                code: 0,
                message: e.to_string(),
            }))
        } else {
            Ok(Err(Error {
                code: 0,
                message: format!(
                    "Key {:?} is not readable for plugin {:?}",
                    key, self.config.name
                ),
            }))
        }
    }

    fn get(&mut self, key: String) -> Result<Option<String>, anyhow::Error> {
        Ok(self.input.get(&key).cloned()) //TODO
    }

    fn log(&mut self, level: LogLevel, message: String) -> std::result::Result<(), anyhow::Error> {
        match level {
            LogLevel::Error => tracing::error!("{}", message),
            LogLevel::Debug => tracing::debug!("{}", message),
            LogLevel::Info => tracing::info!("{}", message),
            LogLevel::Warn => tracing::warn!("{}", message),
            LogLevel::Trace => tracing::trace!("{}", message),
        };
        Ok(())
    }
}

impl PluginRuntime {
    fn is_domain_allowed(&self, domain: &str) -> bool {
        if let Some(ref allowed_domain) = self.config.allowed_domains {
            match Url::parse(domain) {
                Ok(url) => {
                    if let Some(host) = url.host() {
                        allowed_domain
                            .iter()
                            .any(|i| Host::parse(i).unwrap() == host)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }
    fn is_env_var_allowed(&self, value: &str) -> bool {
        if let Some(ref allowed_vars) = self.config.allowed_env_vars {
            allowed_vars.iter().any(|i| i == value)
        } else {
            false
        }
    }
}

fn build_http_url(uri: &str, params: &str) -> String {
    format!("{}?{}", uri, params)
}

fn http_headers_from_runtime(
    headers: &Vec<HttpHeader>,
    mut req: reqwest::blocking::RequestBuilder,
) -> reqwest::blocking::RequestBuilder {
    for header in headers {
        req = req.header(header.key.clone(), header.value.clone())
    }
    req
}

fn http_headers_to_runtime(header_map: &reqwest::header::HeaderMap) -> Vec<HttpHeader> {
    let mut runtime_headers = Vec::<HttpHeader>::new();
    for (key, value) in header_map {
        let runtime_header = HttpHeader {
            key: key.as_str().into(),
            value: value.to_str().unwrap().into(),
        };
        runtime_headers.push(runtime_header);
    }
    runtime_headers
}

#[derive(Debug)]
pub enum WasmError {
    GenericError(String),
}

pub struct WasmModule {
    module: Component,
    linker: Linker<PluginRuntime>,
    engine: Engine,
}

impl fmt::Debug for WasmModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WasmModule").finish()
    }
}

impl WasmModule {
    pub fn new(path: &str) -> Result<Self, WasmError> {
        // An engine stores and configures global compilation settings like
        // optimization level, enabled wasm features, etc.
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).unwrap();//TODO


        // We start off by creating a `Module` which represents a compiled form
        // of our input wasm module. In this case it'll be JIT-compiled after
        // we parse the text format.
        //could use from_binary as well
        let module = Component::from_file(&engine, path)
            .map_err(|e| {
                tracing::error!("{:?}", e);
                WasmError::GenericError(format!("{} {}",e.to_string(), path))
            })?;

        let mut linker = Linker::new(&engine);

        // let mut linker2 = wasmtime::Linker::new(&engine);
        // wasmtime_wasi::add_to_linker(&mut linker2, |cx: &mut PluginRuntime| &mut cx.wasi)
        //     .map_err(|e| WasmError::GenericError(e.to_string()))?;

        PluginSystem::add_to_linker(&mut linker, |state: &mut PluginRuntime| state)
            .map_err(|e| WasmError::GenericError(e.to_string()))?;

        Ok(Self {
            module,
            linker,
            engine,
        })
    }

    fn new_store(&self, config: PluginConfig, input: HashMap<String, String>) -> Store<PluginRuntime> {
        Store::new(
            &self.engine,
    PluginRuntime { config, input, wasi: default_wasi() }
        )
    }

    // invokes the plugin and gets the output from it
    #[instrument]
    pub fn invoke<'a>(
        &mut self,
        input: HashMap<String, String>,
        config: PluginConfig,
    ) -> Result<String, WasmError> {
        let mut store = self.new_store(config, input);
        let (plugin, _instance) =
            PluginSystem::instantiate(&mut store, &self.module, &mut self.linker)
                .map_err(|e| {
                    tracing::error!("error while instantiating plugin {:?}", e);
                    WasmError::GenericError(e.to_string())
                })?;

        plugin.call_invoke(&mut store)
            .map_err(|e| WasmError::GenericError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::host::Host;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_runtime_http() {
        let mock_server = tokio_test::block_on(MockServer::start());
        tokio_test::block_on(
            Mock::given(method("GET"))
                .and(path("/"))
                .respond_with(ResponseTemplate::new(200))
                // Mounting the mock on the mock server - it's now effective!
                .mount(&mock_server),
        );

        let req = HttpRequest {
            method: "get".into(),
            url: mock_server.uri(),
            params: "test=a".to_string(),
            body: "{}".to_string(),
            headers: [
                HttpHeader {
                    key: "bc".to_string(),
                    value: "1".to_string(),
                },
                HttpHeader {
                    key: "ac".to_string(),
                    value: "2".to_string(),
                },
            ]
            .to_vec(),
        };

        let mut runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: None,
                allowed_domains: Some(vec!["127.0.0.1".to_string()]),
            },
            input: HashMap::new(),
            wasi: default_wasi()
        };

        let response = match runtime.http(req).unwrap() {
            Ok(response) => response,
            Err(e) => panic!("http request failed: {:?}", e),
        };

        assert_eq!(200, response.status)
    }

    #[test]
    fn test_is_domain_allowed() {
        let runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: None,
                allowed_domains: Some(vec!["google.com".to_string()]),
            },
            input: HashMap::new(),
            wasi: default_wasi()
        };

        assert!(runtime.is_domain_allowed("https://google.com/something"));
        assert!(!runtime.is_domain_allowed("https://bing.com"));
    }

    #[test]
    fn test_is_allowed_env_var() {
        let runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: Some(vec!["TEST".to_string()]),
                allowed_domains: None,
            },
            input: HashMap::new(),
            wasi: default_wasi()
        };

        assert!(!runtime.is_env_var_allowed("TEST1"));

        assert!(runtime.is_env_var_allowed("TEST"));
    }

    #[test]
    fn test_get_input() {
        let mut runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: Some(vec!["TEST".to_string()]),
                allowed_domains: None,
            },
            input: HashMap::from([("my".to_string(), "test".to_string())]),
            wasi: default_wasi()
        };
        assert_eq!(Some("test".to_string()), runtime.get("my".to_string()).unwrap());
    }

    #[test]
    fn test_read_env_var() {
        let mut runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: Some(vec!["TEST".to_string()]),
                allowed_domains: None,
            },
            input: HashMap::new(),
            wasi: default_wasi()
        };
        env::set_var("TEST".to_string(), "VAL".to_string());

        assert_eq!("VAL".to_string(), runtime.env("TEST".to_string()).unwrap().unwrap());
    }

    #[test]
    fn test_build_http_url() {
        let url = build_http_url("http://www.elias.sh/", "ab=1&aa=2");
        assert_eq!(url, "http://www.elias.sh/?ab=1&aa=2");
    }

    #[test]
    fn test_http_headers_from_runtime() {
        let client = reqwest::blocking::Client::new().post("https://google.com");
        let mut headers = Vec::<HttpHeader>::new();
        headers.push(HttpHeader {
            key: "content".to_string(),
            value: "x".to_string(),
        });
        headers.push(HttpHeader {
            key: "something".to_string(),
            value: "y".to_string(),
        });

        let mut header_map = reqwest::header::HeaderMap::new();
        header_map.insert("content", "x".parse().unwrap());
        header_map.insert("something", "y".parse().unwrap());

        assert_eq!(
            &header_map,
            http_headers_from_runtime(&headers, client)
                .build()
                .unwrap()
                .headers()
        );
    }

    #[test]
    fn test_http_headers_to_runtime() {
        let mut header_map = reqwest::header::HeaderMap::new();
        header_map.insert("content", "x".parse().unwrap());
        header_map.insert("something", "y".parse().unwrap());

        assert_eq!(
            [
                HttpHeader {
                    key: "content".to_string(),
                    value: "x".to_string()
                },
                HttpHeader {
                    key: "something".to_string(),
                    value: "y".to_string()
                }
            ]
            .to_vec(),
            http_headers_to_runtime(&header_map)
        )
    }

    #[test]
    fn test_log_levels() {
        use log::Level;
        use logtest::Logger;

        let mut logger = Logger::start();

        let mut runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: Some(vec!["TEST".to_string()]),
                allowed_domains: None,
            },
            input: HashMap::new(),
            wasi: default_wasi(),
        };
        let my_message = "my";

        let levels = HashMap::from([
            (Level::Info, LogLevel::Info),
            (Level::Debug, LogLevel::Debug),
            (Level::Trace, LogLevel::Trace),
            (Level::Error, LogLevel::Error),
            (Level::Warn, LogLevel::Warn),
        ]);

        for (level, runtime_level) in levels {
            runtime.log(runtime_level, my_message.to_string());
            let message = logger.pop().unwrap();
            assert_eq!(message.level(), level);
            assert_eq!(my_message, message.args());
        }
    }
}
