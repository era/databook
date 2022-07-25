use clap::Parser;
use databook::databook_server::{Databook, DatabookServer};

use databook::{GetRequest, GetResponse};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::instrument;

mod plugin_manager;

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

#[derive(Debug, Default)]
pub struct DatabookGrpc {}

#[tonic::async_trait]
impl Databook for DatabookGrpc {
    #[instrument]
    async fn get(&self, _request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        tracing::info!("Starting get work");
        let reply = GetResponse {};
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
    Server::builder()
        .add_service(DatabookServer::new(DatabookGrpc::default()))
        .serve(addr)
        .await?;
    Ok(())
}
