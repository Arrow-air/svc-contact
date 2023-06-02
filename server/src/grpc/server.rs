//! gRPC server implementation

/// Common types used by the gRPC server
pub mod common {
    tonic::include_proto!("common");
}

///module generated from proto/svc-contact-grpc.proto
pub mod grpc_server {
    #![allow(unused_qualifications, missing_docs)]
    tonic::include_proto!("ready");

    #[cfg(feature = "cargo")]
    tonic::include_proto!("cargo");
}

use grpc_server::ready_rpc_service_server::{ReadyRpcService, ReadyRpcServiceServer};
use grpc_server::{ReadyRequest, ReadyResponse};

use crate::config::Config;
use crate::shutdown_signal;

use std::fmt::Debug;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[cfg(feature = "cargo")]
mod cargo {
    pub use super::grpc_server::cargo_rpc_service_server::{
        CargoRpcService, CargoRpcServiceServer,
    };
    use super::grpc_server::{CargoConfirmationRequest, CargoConfirmationResponse};
    use tonic::{Request, Response, Status};

    /// struct to implement the gRPC server functions
    #[derive(Debug, Default, Copy, Clone)]
    pub struct CargoServerImpl {}

    #[tonic::async_trait]
    impl CargoRpcService for CargoServerImpl {
        /// Returns ready:true when service is available
        async fn cargo_confirmation(
            &self,
            _request: Request<CargoConfirmationRequest>,
        ) -> Result<Response<CargoConfirmationResponse>, Status> {
            grpc_debug!("(grpc is_ready) entry.");

            //
            // TODO(R3) Contact svc-storage and get user_id from itinerary_id
            //
            let response = CargoConfirmationResponse { success: true };
            Ok(Response::new(response))
        }
    }
}

/// struct to implement the gRPC server functions
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadyServerImpl {}

#[tonic::async_trait]
impl ReadyRpcService for ReadyServerImpl {
    /// Returns ready:true when service is available
    async fn is_ready(
        &self,
        _request: Request<ReadyRequest>,
    ) -> Result<Response<ReadyResponse>, Status> {
        grpc_debug!("(grpc is_ready) entry.");
        let response = ReadyResponse { ready: true };
        Ok(Response::new(response))
    }
}

/// Starts the grpc servers for this microservice using the provided configuration
///
/// # Example:
/// ```
/// use svc_contact::grpc::server::grpc_server;
/// use svc_contact::config::Config;
/// async fn example() -> Result<(), tokio::task::JoinError> {
///     let config = Config::default();
///     tokio::spawn(grpc_server(config)).await
/// }
/// ```
#[cfg(not(tarpaulin_include))]
pub async fn grpc_server(config: Config) {
    grpc_debug!("(grpc_server) entry.");

    // GRPC Server
    let grpc_port = config.docker_port_grpc;
    let full_grpc_addr: SocketAddr = match format!("[::]:{}", grpc_port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            grpc_error!("Failed to parse gRPC address: {}", e);
            return;
        }
    };

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    let ready_impl = ReadyServerImpl::default();
    health_reporter
        .set_serving::<ReadyRpcServiceServer<ReadyServerImpl>>()
        .await;

    //start server
    grpc_info!("Starting GRPC servers on: {}.", full_grpc_addr);
    let builder = Server::builder()
        .add_service(health_service)
        .add_service(ReadyRpcServiceServer::new(ready_impl));

    #[cfg(feature = "cargo")]
    let builder = builder.add_service(cargo::CargoRpcServiceServer::new(
        cargo::CargoServerImpl::default(),
    ));

    match builder
        .serve_with_shutdown(full_grpc_addr, shutdown_signal("grpc"))
        .await
    {
        Ok(_) => grpc_info!("gRPC server running at: {}.", full_grpc_addr),
        Err(e) => {
            grpc_error!("could not start gRPC server: {}", e);
        }
    };
}
