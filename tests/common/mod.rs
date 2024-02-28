use std::str::FromStr;

use bucket_sdk::api::BucketClient;

pub fn setup() -> BucketClient {
    let endpoint = url::Url::from_str(std::env::var("API_URL").unwrap().as_str()).unwrap();
    let token = std::env::var("TOKEN").unwrap();

    BucketClient::new(&endpoint, &token)
}
