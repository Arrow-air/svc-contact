//! gRPC client implementation

use std::env;

#[allow(unused_qualifications, missing_docs)]
use svc_contact_client_grpc::clients::{
    ready_rpc_service_client::ReadyRpcServiceClient, ReadyRequest,
};

#[cfg(feature = "cargo")]
use svc_contact_client_grpc::clients::{
    cargo_rpc_service_client::CargoRpcServiceClient, CargoConfirmationRequest,
};

/// Provide endpoint url to use
pub fn get_grpc_endpoint() -> String {
    //parse socket address from env variable or take default value
    let address = match env::var("SERVER_HOSTNAME") {
        Ok(val) => val,
        Err(_) => "localhost".to_string(), // default value
    };

    let port = match env::var("SERVER_PORT_GRPC") {
        Ok(val) => val,
        Err(_) => "50051".to_string(), // default value
    };

    format!("http://{}:{}", address, port)
}

/// Example of using the ready service
async fn ready_example(grpc_endpoint: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ReadyRpcServiceClient::connect(grpc_endpoint).await?;
    let request = tonic::Request::new(ReadyRequest {});
    let response = client.is_ready(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}

/// Example of using the cargo service
#[cfg(feature = "cargo")]
async fn cargo_example(grpc_endpoint: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CargoRpcServiceClient::connect(grpc_endpoint).await?;
    let request = tonic::Request::new(CargoConfirmationRequest {
        package_id: uuid::Uuid::new_v4().to_string(),
    });

    let response = client.cargo_confirmation(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}

/// Example svc-template-client-grpc
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_endpoint = get_grpc_endpoint();

    println!(
        "NOTE: Ensure the server is running on {} or this example will fail.",
        grpc_endpoint
    );

    ready_example(grpc_endpoint.clone()).await?;

    #[cfg(feature = "cargo")]
    cargo_example(grpc_endpoint.clone()).await?;

    Ok(())
}
