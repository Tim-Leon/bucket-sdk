fn main() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["bucket-api/proto/backend_api.proto"],
            &["bucket-api/proto"],
        )
        .unwrap();
}
