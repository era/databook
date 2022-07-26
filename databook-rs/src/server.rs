use clap::Parser;
use databook::databook_server::{Databook, DatabookServer};
use std::path::PathBuf;

use databook::{GetRequest, GetResponse};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};
use tracing::instrument;

mod plugin_manager;
mod wasm;

pub mod databook {
    tonic::include_proto!("databook");
}

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
pub struct DatabookGrpc {
    plugin_manager: plugin_manager::PluginManager,
}

impl DatabookGrpc {
    pub fn new(plugin_folder: String) -> Self {
        let plugin_manager = plugin_manager::PluginManager::new(PathBuf::from(plugin_folder));
        Self { plugin_manager }
    }
}

#[tonic::async_trait]
impl Databook for DatabookGrpc {
    #[instrument]
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        tracing::info!("Starting get work");
        let response = self
            .plugin_manager
            .invoke(&request.into_inner().name, "sample_input".to_string())
            .map_err(|e| Status::new(Code::Internal, "Internal Error"))?; //TODO

        let reply = GetResponse { output: response };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    // Setups tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    // Setups GRPC server
    let addr = args.address_to_listen.parse()?;
    let grpc = DatabookGrpc::new(args.plugin_folder);
    Server::builder()
        .add_service(DatabookServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
