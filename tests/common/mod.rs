use std::str::FromStr;

use bucket_sdk::api::BucketApi;

pub fn setup() -> BucketApi {
    let endpoint = url::Url::from_str(std::env::var("API_URL").unwrap().as_str()).unwrap();
    let token = std::env::var("TOKEN").unwrap();
    
    BucketApi::new(&endpoint, &token)
}
