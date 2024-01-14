/*
WIll load the BUCKETDRIVE_ENDPOINT from .env file.
Then run the following tests against it. This is used to both validate that this client is implemented correctly but also that the server response is correct.
*/

use bucket_client_core::api::BucketApi;
use bucket_client_core::{
    api::CreateBucketParams, controller::account::authentication, query_client::QueryClient,
};
use common::setup;
mod common;

#[tokio::test]
async fn check_signup() {
    let mut query_client = QueryClient::build_from_env();
    let email = "".to_string();
    let username = "".to_string();
    let password = "".to_string();
    let captcha = "".to_string();
    authentication::register(&mut query_client, email, username, password, captcha)
        .await
        .unwrap();
}

#[tokio::test]
async fn check_login() {
    let mut query_client = QueryClient::build_from_env();
    let email = "".to_string();
    let password = "".to_string();
    authentication::login(&mut query_client, email, password, None)
        .await
        .unwrap();
}
#[tokio::test]
async fn check_create_bucket() {
    let apii = setup();
    apii.create_bucket(CreateBucketParams {
        target_user_id: todo!(),
        target_bucket_id: todo!(),
        target_directory: todo!(),
        source_files: todo!(),
        encryption: todo!(),
        total_size_in_bytes: todo!(),
        hashed_password: todo!(),
    })
}
#[tokio::test]
async fn check_delete_bucket() {
    let api = setup();
    api.delete_bucket();
}

#[tokio::test]
async fn check_upload_to_bucket() {
    let api = setup();
    api.upload_to_bucket();
}
#[tokio::test]
async fn check_download_from_bucket() {}
#[tokio::test]
async fn check_zero_knowledge_encryption() {}
#[tokio::test]
async fn check_compression() {}
#[tokio::test]
async fn check_share_bucket() {}
