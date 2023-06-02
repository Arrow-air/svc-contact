//! gRPC client helpers implementation

use svc_storage_client_grpc::Clients;

/// Struct to hold all gRPC client connections
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct GrpcClients {
    pub storage: Clients
}

impl GrpcClients {
    /// Create new GrpcClients with defaults
    pub fn default(config: crate::config::Config) -> Self {
        let storage_clients = Clients::new(config.storage_host_grpc, config.storage_port_grpc);

        GrpcClients {
            storage: storage_clients,
        }
    }
}
