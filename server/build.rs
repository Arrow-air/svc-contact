//! build script to generate .rs from .proto

///generates .rs files in src directory
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "../proto";

    let proto_files = vec!["common.proto", "cargo.proto", "ready.proto"];

    let server_config = tonic_build::configure()
        .type_attribute("CargoConfirmationResponse", "#[derive(Copy)]")
        .type_attribute("ReadyRequest", "#[derive(Eq, Copy)]")
        .type_attribute("ReadyResponse", "#[derive(Eq, Copy)]");

    let client_config = server_config.clone();

    std::fs::create_dir_all("../client-grpc/src/out/")?;

    client_config
        .build_server(false)
        .out_dir("../client-grpc/src/out/")
        .compile(&proto_files, &[proto_dir])?;

    // Build the Server
    server_config
        .build_client(false)
        .compile(&proto_files, &[proto_dir])?;

    // println!("cargo:rerun-if-changed={}", proto_file);

    Ok(())
}
