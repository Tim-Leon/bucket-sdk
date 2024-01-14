use bucket_client_core::api::BucketApi;

pub fn setup() -> BucketApi {
    let endpoint = std::env::var("BUCKETDRIVE_ENDPOINT").unwrap();
    let api_client = BucketApi::new(endpoint);
    api_client
}
