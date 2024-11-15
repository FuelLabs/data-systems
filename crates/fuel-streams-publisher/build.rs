fn main() {
    let protos = &["proto/fuel_streamer.proto"];

    // server
    let prost_config_server = prost_build::Config::default();
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .build_transport(true)
        .file_descriptor_set_path("src/grpc/fuel_streamer.bin")
        .out_dir("src/grpc")
        .compile_protos_with_config(prost_config_server, protos, &[] as &[&str])
        .expect("failed to compile server protos");

    // client
    let prost_config_client = prost_build::Config::default();
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .build_transport(true)
        .file_descriptor_set_path("src/grpc/fuel_streamer.bin")
        .out_dir("src/grpc")
        .compile_protos_with_config(prost_config_client, protos, &[] as &[&str])
        .expect("failed to compile client protos");
}
