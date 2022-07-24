use databook::databook_server::{Databook, DatabookServer};
use databook::{GetRequest, GetResponse};

use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::instrument;

pub mod databook {
    tonic::include_proto!("databook");
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
    // Setups tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    // Setups GRPC server
    let addr = "[::1]:50051".parse()?;
    Server::builder()
        .add_service(DatabookServer::new(DatabookGrpc::default()))
        .serve(addr)
        .await?;
    Ok(())
}
