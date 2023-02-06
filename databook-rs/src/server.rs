use clap::Parser;
use databook::databook_server::{Databook, DatabookServer};
use databook::{GetRequest, GetResponse};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::RwLock;
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};
use tracing::instrument;

#[macro_use] extern crate rocket;

mod plugin_config;
mod plugin_manager;
mod plugin_runtime;
mod wasm;
mod rest;

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
                    .read()
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


//TODO create rocket service


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    // Setups GRPC server
    let addr = args.address_to_listen.parse()?;
    let grpc = DatabookGrpc::new();
    Server::builder()
        .add_service(DatabookServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
