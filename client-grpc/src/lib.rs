//! Exposes svc-contact Client Functions

/// Common types used by the gRPC clients
pub mod common {
    include!("out/common.rs");
}

/// The various gRPC clients for using the contact service
#[allow(unused_qualifications, missing_docs)]
pub mod clients {
    #[cfg(feature = "cargo")]
    include!("out/cargo.rs");
    include!("out/ready.rs");
}
