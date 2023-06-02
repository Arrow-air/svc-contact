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
    use crate::grpc::client::GrpcClients;
    use svc_storage_client_grpc::{ClientConnect, Id};

    struct ConfirmationInfo {
        parcel_id: String,
        itinerary_id: String,
        display_name: String,
        departure_port_name: String,
        arrival_port_name: String,
        departure_timestamp: String,
        arrival_timestamp: String
    }
    
    /// struct to implement the gRPC server functions
    #[derive(Debug, Clone)]
    pub struct CargoServerImpl {
        grpc_clients: GrpcClients,
    }

    #[tonic::async_trait]
    impl CargoRpcService for CargoServerImpl {
        /// Returns ready:true when service is available
        async fn cargo_confirmation(
            &self,
            request: Request<CargoConfirmationRequest>,
        ) -> Result<Response<CargoConfirmationResponse>, Status> {
            grpc_debug!("(grpc is_ready) entry.");
            let parcel_id = request.into_inner().parcel_id;

            //
            // use parcel_id to get parcel record
            //
            let Ok(mut client) = self.grpc_clients.storage.parcel.get_client().await else {
                grpc_error!("(grpc cargo_confirmation) error getting parcel client.");
                return Err(Status::internal("error"));
            };

            let parcel = match client.get_by_id(Id { id: parcel_id.clone() }).await {
                Ok(response) => {
                    response.into_inner()
                }
                Err(e) => {
                    grpc_error!("(grpc cargo_confirmation) error: {}", e);
                    return Err(Status::internal("error"));
                }
            };

            let Some(parcel) = parcel.data else {
                grpc_error!("(grpc cargo_confirmation) error: parcel not found.");
                return Err(Status::internal("error"));
            };

            let itinerary_id = parcel.itinerary_id;

            //
            // use itinerary_id to get itinerary record
            //
            let Ok(mut client) = self.grpc_clients.storage.itinerary.get_client().await else {
                grpc_error!("(grpc cargo_confirmation) error getting itinerary client.");
                return Err(Status::internal("error"));
            };

            let itinerary = match client.get_by_id(Id { id: itinerary_id.clone() }).await {
                Ok(response) => {
                    response.into_inner()
                }
                Err(e) => {
                    grpc_error!("(grpc cargo_confirmation) error: {}", e);
                    return Err(Status::internal("error"));
                }
            };

            let Some(itinerary) = itinerary.data else {
                grpc_error!("(grpc cargo_confirmation) error: itinerary not found.");
                return Err(Status::internal("error"));
            };

            let user_id = itinerary.user_id;

            //
            // use itinerary_id to get flight plans
            //
            let Ok(mut client) = self.grpc_clients.storage.itinerary_flight_plan_link.get_client().await else {
                grpc_error!("(grpc cargo_confirmation) error getting flight_plan client.");
                return Err(Status::internal("error"));
            };

            let flight_plans = match client.get_linked(Id { id: itinerary_id.clone() }).await {
                Ok(response) => {
                    response.into_inner()
                }
                Err(e) => {
                    grpc_error!("(grpc cargo_confirmation) error: {}", e);
                    return Err(Status::internal("error"));
                }
            };

            //
            // use itinerary record to get user_id
            //
            let Ok(mut client) = self.grpc_clients.storage.user.get_client().await else {
                grpc_error!("(grpc cargo_confirmation) error getting user client.");
                return Err(Status::internal("error"));
            };

            let user = match client.get_by_id(Id { id: user_id }).await {
                Ok(response) => {
                    response.into_inner()
                }
                Err(e) => {
                    grpc_error!("(grpc cargo_confirmation) error: {}", e);
                    return Err(Status::internal("error"));
                }
            };

            let Some(user) = user.data else {
                grpc_error!("(grpc cargo_confirmation) error: user not found.");
                return Err(Status::internal("error"));
            };

            //
            // Fill information for SendGrid
            //
            let info = ConfirmationInfo {
                parcel_id,
                itinerary_id,
                display_name: user.display_name,
            };

            grpc_info!("(grpc cargo_confirmation) confirmation info: {:?}", info);

            // TODO(R4): Send to SendGrid

            // use user_id to get user record
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
    let builder = builder.add_service(
        cargo::CargoRpcServiceServer::new(
            cargo::CargoServerImpl {
                grpc_clients: crate::grpc::client::GrpcClients::default(config)
            }
        )
    );

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
