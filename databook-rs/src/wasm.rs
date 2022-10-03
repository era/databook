use crossbeam::channel;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client, HeaderMap, Method, Request, Uri};
use std::collections::HashMap;
use std::fmt;
use tokio;
use tracing::instrument;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};
use wit_bindgen_wasmtime::wasmtime::{self, Config, Engine, Instance, Linker, Module, Store};

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

        let client = Client::new();
        //TODO ASYNC / SYNC BRIDGE
        let (tx, rx) = channel::bounded(1);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();
        handle.spawn(async move {
            tracing::info!("doing http request");
            tx.send(client.request(req).await.unwrap());
        });

        let response = rx.recv().unwrap();
        tracing::info!("http response is {:?}", response);
        rt.shutdown_background();

        HttpResponse {
            status: response.status().as_u16(),
            headers: "".to_string(),  //TODO
            response: "".to_string(), //response.into_body(), //TODO
        }
    }

    fn env(&mut self, key: &str) -> String {
        //TODO if allowed get the value of key
        "".to_string()
    }
}

fn build_http_url(uri: &str, params: &str) -> String {
    format!("{}?{}", uri, params)
}

fn http_headers_from_str(
    headers: &str,
    mut req: hyper::http::request::Builder,
) -> hyper::http::request::Builder {
    let splitted: Vec<&str> = headers.split('&').collect();

    for i in 0..splitted.len() {
        if let Some(header) = splitted.get(i) {
            let header: Vec<&str> = header.split('=').collect();

            match (header.get(0), header.get(1)) {
                (Some(key), Some(value)) => {
                    req = req.header(
                        key.to_string().parse::<HeaderName>().unwrap(),
                        value.to_string().parse::<HeaderValue>().unwrap(),
                    )
                }
                _ => continue,
            };
        }
    }
    return req;
}
