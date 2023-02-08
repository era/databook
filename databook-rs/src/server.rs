#![feature(proc_macro_hygiene, decl_macro)]
use clap::Parser;
use databook::databook_server::{Databook, DatabookServer};
use databook::{GetRequest, GetResponse};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::RwLock;
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};
use tracing::instrument;

#[macro_use]
extern crate rocket;
use rocket::request::Form;
use rocket_contrib::json::Json;
use tokio::spawn;

mod plugin_config;
mod plugin_manager;
mod plugin_runtime;
mod rest;
mod wasm;

pub mod databook {
    tonic::include_proto!("databook");
}

static PLUGINS: OnceCell<RwLock<plugin_manager::PluginManager>> = OnceCell::new();

// CLI arguments to start the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Path of the plugin folder so we know from where to load it
    #[clap(short, long, value_parser, default_value_t = String::from("./plugins"))]
    plugin_folder: String,
    #[clap(short, long, value_parser, default_value_t = String::from("[::1]:50051"))]
    address_to_listen: String,
}

#[derive(Debug)]
pub struct DatabookGrpc {}

impl DatabookGrpc {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl Databook for DatabookGrpc {
    #[instrument]
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        tracing::info!("received get request");
        let response = tokio::task::spawn_blocking(|| {
            let request = request.into_inner();
            match PLUGINS.get() {
                Some(p) => p
                    .write() //TODO
                    .map_err(|e| {
                        tracing::error!("Could not get lock for plugins object {:?}", e);
                        Status::new(Code::Internal, "Internal Error")
                    })?
                    .invoke(&request.name, request.options)
                    .map_err(|e| {
                        tracing::error!("error while calling wasm plugin {:?}", e);
                        Status::new(Code::Internal, "Internal Error")
                    }),
                None => Err(Status::new(Code::Internal, "No plugins setup")),
            }
        })
        .await
        .expect("could not join back the tokio task thread");

        match response {
            Ok(response) => Ok(Response::new(GetResponse { output: response })),
            Err(e) => Err(e),
        }
    }
}

impl Default for DatabookGrpc {
    fn default() -> Self {
        Self::new()
    }
}

#[instrument]
#[post("/invoke", data = "<request>")]
fn rest_invoke(request: Json<rest::InvokePluginRequest>) -> Json<rest::InvokePluginResponse> {
    tracing::info!("received get request");
    let response = {
        let request = request.into_inner();
        match PLUGINS.get() {
            Some(p) => p
                .write() //TODO
                .unwrap() //TODO
                .invoke(&request.name, request.options)
                .map_err(|e| {
                    tracing::error!("error while calling wasm plugin {:?}", e);
                    format!("error while invoking plugin {:?}", e)
                }),
            None => Err("No plugins setup".to_string()),
        }
    };

    match response {
        Ok(response) => Json(rest::InvokePluginResponse {
            output: Some(response),
            error: None,
        }),
        Err(e) => Json(rest::InvokePluginResponse {
            output: None,
            error: Some(e.to_string()),
        }),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // Setups tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let mut plugin_manager = plugin_manager::PluginManager::new(PathBuf::from(args.plugin_folder));
    plugin_manager
        .registry()
        .expect("could not register plugins");

    PLUGINS
        .set(RwLock::new(plugin_manager))
        .expect("should always add plugin manager to once_cell");

    let rt = tokio::runtime::Runtime::new().unwrap();

    let rest_join = rt.spawn(async {
        rocket::ignite().mount("/", routes![rest_invoke]).launch();
    });

    let grpc_join = rt.spawn(async move {
        // Setups GRPC server
        let addr = args.address_to_listen.parse().unwrap();
        let grpc = DatabookGrpc::new();
        Server::builder()
            .add_service(DatabookServer::new(grpc))
            .serve(addr)
            .await
            .unwrap();
    });

    //TODO should block on both the rest and the grpc together
    // if any finish, we should stop the service
    // because something went wrong
    rt.block_on(rest_join).unwrap();
    rt.block_on(grpc_join).unwrap();
    Ok(())
}
