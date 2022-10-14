use databook::databook_client::DatabookClient;
use databook::GetRequest;
use std::collections::HashMap;

pub mod databook {
    tonic::include_proto!("databook");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DatabookClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetRequest {
        name: "hello_world".into(),
        options: HashMap::new(),
    });

    let response = client.get(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
