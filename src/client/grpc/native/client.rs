use bucket_api::backend_api::backend_api_client::BackendApiClient;
use tonic::transport::{Channel, Uri};

pub mod query_client {
    use std::str::FromStr;

    //use backend_api::backend_api_client::BackendApiClient;
    use tonic::transport::Channel;
    use tonic::transport::Uri;
    //pub mod backend_api {
    //    tonic::include_proto!("backend_api");
    //}
    use crate::client::grpc::QueryClientBuilder;
    use bucket_api::backend_api::backend_api_client::BackendApiClient;

    pub type QueryClient = BackendApiClient<Channel>; //:backend_api_server::BackendApiServer<Channel>; // BackendApiClient<Channel>;

    impl QueryClientBuilder<Channel> for QueryClient {
        async fn build(api_url: Uri) -> BackendApiClient<Channel> {
            let client = Channel::builder(api_url).connect().await.unwrap();
            BackendApiClient::new(client)
        }
        /// API_URL environment must be set to valid URL.
        async fn build_from_env() -> BackendApiClient<Channel> {
            let base_url = std::env::var("API_URL").expect("API_URL must be set");
            Self::build(Uri::from_str(base_url.as_str()).unwrap()).await
        }
    }
}
