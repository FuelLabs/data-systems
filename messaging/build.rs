fn main() {
    let protos = &["proto/fuel_types.proto"];

    let prost_config_server = prost_build::Config::default();
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .out_dir("src")
        .compile_protos_with_config(prost_config_server, protos, &[] as &[&str])
        .expect("failed to compile protos");
}
