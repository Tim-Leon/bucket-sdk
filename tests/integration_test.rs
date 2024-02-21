/*
WIll load the BUCKETDRIVE_ENDPOINT from .env file.
Then run the following tests against it. This is used to both validate that this client is implemented correctly but also that the server response is correct.
*/

// https://docs.hcaptcha.com/#integration-testing-test-keys
struct hcaptcha_test_key_set {
    pub site_key: String,
    pub secret_key: String,
    pub response_token: String,
}

impl Default for hcaptcha_test_key_set {
    fn default() -> Self {
        Self {
            site_key: "10000000-ffff-ffff-ffff-000000000001".to_string(),
            secret_key: "0x0000000000000000000000000000000000000000".to_string(),
            response_token: "10000000-aaaa-bbbb-cccc-000000000001".to_string(),
        }
    }
}

mod common;

#[cfg(test)]
mod tests {

    use bucket_sdk::{controller::account::authentication, query_client::QueryClient};

    #[tokio::test]
    async fn check_signup() {
        let mut query_client = QueryClient::build_from_env();
        let email = "email@domain.com".to_string();
        let username = "awesomeusername".to_string();
        let password = "awesomepassword".to_string();
        let captcha = "".to_string();
        authentication::register(&mut query_client, email.as_str(), username.as_str(), password.as_str(), captcha.as_str())
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
    // #[tokio::test]
    // async fn check_create_bucket() {
    //     let api = setup();
    //     let var_name = CreateBucketParams {
    //         target_user_id: todo!(),
    //         name: todo!(),
    //         visibility: todo!(),
    //         encryption: todo!(),
    //         password: todo!(),
    //         target_directory: todo!(),
    //         source_files: todo!(),
    //         description: todo!(),
    //         storage_class: todo!(),
    //         expire_at: todo!(),
    //         expected_capacity: todo!(),
    //         is_nsfw: todo!(),
    //         is_searchable: todo!(),
    //         is_sharable: todo!(),
    //         is_bucket_cloneable: todo!(),
    //         is_prepaid: todo!(),
    //         bucket_compression: todo!(),
    //         tags: todo!(),
    //         total_size_in_bytes: todo!(),
    //     };
    //     api.create_bucket(var_name).await.unwrap();
    // }

    // #[tokio::test]
    // async fn check_upload_to_bucket() {}
    // #[tokio::test]
    // async fn check_download_from_bucket() {}
    // #[tokio::test]
    // async fn check_zero_knowledge_encryption() {}
    // #[tokio::test]
    // async fn check_compression() {}
    // #[tokio::test]
    // async fn check_share_bucket() {}
    // #[tokio::test]
    // async fn check_delete_bucket() {
    //     let api = setup();
    //     let param = DeleteBucketParams {
    //         bucket_guid: BucketGuid {
    //             user_id: todo!(),
    //             bucket_id: todo!(),
    //         },
    //     };
    // }
}
