pub mod query_client {
    use std::str::FromStr;

    use backend_api::backend_api_client::BackendApiClient;
    use tonic::transport::Channel;
    use tonic::transport::Uri;

    pub mod backend_api {
        tonic::include_proto!("backend_api");
    }

    pub type QueryClient = BackendApiClient<Channel>;

    impl QueryClient {
        pub async fn build(api_url: &str) -> BackendApiClient<Channel> {
            let url = Uri::from_str(api_url).unwrap();
            let client = tonic::transport::Channel::builder(url).connect().await.unwrap();
            BackendApiClient::new(client)
        }

        /*
         * API_URL environment must be set to valid URL.
         */
        pub async fn build_from_env() -> BackendApiClient<Channel> {
            let base_url = std::env::var("API_URL").expect("API_URL must be set");
            Self::build(&base_url).await
        }
    }
}
