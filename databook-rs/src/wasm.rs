use crate::http::{build_http_url, http_headers_from_str, http_headers_to_str};
use crate::plugin_config::PluginConfig;
use crossbeam::channel;
use hyper::client::HttpConnector;
use hyper::{Body, Client, HeaderMap, Method, Request, Response, Uri};
use std::fmt;
use std::str;
use tokio;
use tracing::instrument;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};
use wit_bindgen_host_wasmtime_rust::wasmtime::{
    self, Config, Engine, Instance, Linker, Module, Store,
}; // 0.1.25

wit_bindgen_host_wasmtime_rust::import!("../wit/plugin.wit");
wit_bindgen_host_wasmtime_rust::export!("../wit/runtime.wit");
use plugin::{Plugin, PluginData};
use runtime::{add_to_linker, Error, HttpRequest, HttpResponse, Runtime};

const HTTP_REQUEST_FAILED: u16 = 100;
const HTTP_INVALID_BODY: u16 = 101;

pub struct PluginRuntime {
    config: PluginConfig,
}

struct Context<I, E> {
    wasi: wasmtime_wasi::WasiCtx,
    imports: I,
    exports: E,
    runtime: PluginRuntime,
}
type PluginStore = Store<Context<PluginData, PluginData>>;

#[derive(Debug)]
pub enum WasmError {
    GenericError(String),
}

pub struct WasmModule {
    module: Module,
    linker: Linker<Context<PluginData, PluginData>>,
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

        wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context<PluginData, PluginData>| {
            &mut cx.wasi
        })
        .map_err(|e| WasmError::GenericError(e.to_string()))?;

        add_to_linker(&mut linker, |cx| &mut cx.runtime)
            .map_err(|e| WasmError::GenericError(e.to_string()))?;
        Ok(Self {
            module,
            linker,
            engine,
        })
    }

    fn new_store(&self, config: PluginConfig) -> Store<Context<PluginData, PluginData>> {
        Store::new(
            &self.engine,
            Context {
                wasi: default_wasi(),
                imports: PluginData::default(),
                exports: PluginData::default(),
                runtime: PluginRuntime { config },
            },
        )
    }

    // invokes the plugin and gets the output from it
    #[instrument]
    pub fn invoke<'a>(&mut self, input: String, config: PluginConfig) -> Result<String, WasmError> {
        let mut store = self.new_store(config);
        let (plugin, _instance) =
            Plugin::instantiate(&mut store, &self.module, &mut self.linker, |cx| {
                &mut cx.exports
            })
            .map_err(|e| {
                tracing::error!("error while instantiating plugin {:?}", e);
                WasmError::GenericError(e.to_string())
            })?;

        plugin
            .invoke(&mut store, &input)
            .map_err(|e| WasmError::GenericError(e.to_string()))
    }
}

impl runtime::Runtime for PluginRuntime {
    fn http(&mut self, request: HttpRequest) -> Result<HttpResponse, Error> {
        //TODO VALIDATION

        let req = Request::builder()
            .uri(build_http_url(request.url, request.params))
            .method(request.method);
        let req = http_headers_from_str(request.headers, req);

        let req = req
            .body(Body::from(request.body.to_string()))
            .map_err(|e| Error {
                code: HTTP_INVALID_BODY,
                message: e.to_string(),
            })?;

        //TODO ASYNC / SYNC BRIDGE
        let (tx, rx) = channel::bounded(1);
        let rt = tokio::runtime::Runtime::new().expect("Could not start a new Tokio runtime");
        let handle = rt.handle();
        handle.spawn(async move { tx.send(do_request(req).await) });
        let response = rx.recv().unwrap();

        rt.shutdown_background();
        response
    }

    fn env(&mut self, key: &str) -> Result<String, Error> {
        //TODO if allowed get the value of key
        Ok("".to_string())
    }
}

pub async fn do_request(request: Request<hyper::Body>) -> Result<HttpResponse, Error> {
    let client: Client<HttpConnector, hyper::Body> = Client::new();

    tracing::info!("doing http request {:?}", request);

    let response = client.request(request).await.map_err(|e| Error {
        code: HTTP_REQUEST_FAILED,
        message: e.to_string(),
    })?;

    tracing::info!("http response is {:?}", response);

    let status = response.status().as_u16();
    //TODO check status code
    let headers = http_headers_to_str(response.headers().clone()); //TODO

    let response_body = hyper::body::to_bytes(response.into_body())
        .await
        .map_err(|e| Error {
            code: HTTP_REQUEST_FAILED,
            message: e.to_string(),
        })?;

    let response_body = String::from_utf8(response_body.into_iter().collect())
        .expect("could not collect response body to convert to string");

    Ok(HttpResponse {
        status,
        headers,
        response: response_body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
            url: &mock_server.uri(),
            params: "test=a",
            body: "{}",
            headers: "bc=1&ac=2",
        };

        let mut runtime = PluginRuntime {
            config: PluginConfig {
                name: "TestPlugin".to_string(),
                allowed_env_vars: None,
                allowed_domains: Some(vec![mock_server.uri().clone()]),
            },
        };

        let response = match runtime.http(req) {
            Ok(response) => response,
            _ => panic!("http request failed"),
        };

        assert_eq!(200, response.status)
    }
}
