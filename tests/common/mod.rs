use std::str::FromStr;

use bucket_sdk::api::BucketApi;

pub fn setup() -> BucketApi {
    let endpoint = url::Url::from_str(std::env::var("BUCKETDRIVE_ENDPOINT").unwrap().as_str()).unwrap();
    let token = std::env::var("BUCKETDRIVE_TOKEN").unwrap();
    let api_client = BucketApi::new(&endpoint, &token);
    api_client
}
