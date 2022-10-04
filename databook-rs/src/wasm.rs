use crate::http::{build_http_url, http_headers_from_str, http_headers_to_str};
use crossbeam::channel;
use hyper::client::HttpConnector;
use hyper::{Body, Client, HeaderMap, Method, Request, Response, Uri};
use std::fmt;
use std::str;
use tokio;
use tracing::instrument;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};
use wit_bindgen_wasmtime::wasmtime::{self, Config, Engine, Instance, Linker, Module, Store}; // 0.1.25

wit_bindgen_wasmtime::import!("../wit/plugin.wit");
wit_bindgen_wasmtime::export!("../wit/runtime.wit");
use plugin::{Plugin, PluginData};
use runtime::{add_to_linker, HttpRequest, HttpResponse, Runtime};

pub struct PluginRuntime {}

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

    fn new_store(&self) -> Store<Context<PluginData, PluginData>> {
        Store::new(
            &self.engine,
            Context {
                wasi: default_wasi(),
                imports: PluginData::default(),
                exports: PluginData::default(),
                runtime: PluginRuntime {},
            },
        )
    }

    // invokes the plugin and gets the output from it
    #[instrument]
    pub fn invoke(&mut self, input: String) -> Result<String, WasmError> {
        let mut store = self.new_store();
        let (plugin, _instance) =
            Plugin::instantiate(&mut store, &self.module, &mut self.linker, |cx| {
                &mut cx.exports
            })
            .map_err(|e| WasmError::GenericError(e.to_string()))?;

        plugin
            .invoke(&mut store, &input)
            .map_err(|e| WasmError::GenericError(e.to_string()))
    }
}

impl runtime::Runtime for PluginRuntime {
    fn http(&mut self, request: HttpRequest) -> HttpResponse {
        //TODO VALIDATION

        let req = Request::builder()
            .uri(build_http_url(request.url, request.params))
            .method(request.method);
        let req = http_headers_from_str(request.headers, req);
        let req = req.body(Body::from(request.body.to_string())).unwrap(); //TODO

        //TODO ASYNC / SYNC BRIDGE
        let (tx, rx) = channel::bounded(1);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();
        handle.spawn(async move { tx.send(do_request(req).await) });
        let response = rx.recv().unwrap();

        rt.shutdown_background();
        response
    }

    fn env(&mut self, key: &str) -> String {
        //TODO if allowed get the value of key
        "".to_string()
    }
}

pub async fn do_request(request: Request<hyper::Body>) -> HttpResponse {
    let client: Client<HttpConnector, hyper::Body> = Client::new();

    tracing::info!("doing http request {:?}", request);
    let response = client.request(request).await.unwrap();
    tracing::info!("http response is {:?}", response);

    let status = response.status().as_u16();
    //TODO check status code
    let headers = http_headers_to_str(response.headers().clone()); //TODO

    let response_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_body = String::from_utf8(response_body.into_iter().collect()).expect("");

    HttpResponse {
        status,
        headers,
        response: response_body,
    }
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

        let mut runtime = PluginRuntime {};
        let response = runtime.http(req);

        assert_eq!(200, response.status)
    }
}
