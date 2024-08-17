use bucket_api::backend_api::backend_api_client::BackendApiClient;
use tonic::transport::Uri;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
pub mod request_ext;
pub trait QueryClientBuilder<T> {
    async fn build(api_url: Uri) -> BackendApiClient<T>;

    async fn build_from_env() -> BackendApiClient<T>;
}