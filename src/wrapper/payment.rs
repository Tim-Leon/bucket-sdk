use bucket_api::backend_api::{CreateCheckoutRequest, CreateCheckoutResponse};
use tonic::Status;
use crate::client::grpc::native::client::query_client::QueryClient;

pub trait ClientPaymentWrapperExt {
    async fn create_checkout_raw(&mut self, req: CreateCheckoutRequest) -> Result<CreateCheckoutResponse, tonic::Status> ;
}



impl ClientPaymentWrapperExt for QueryClient {
    async fn create_checkout_raw(&mut self, req: CreateCheckoutRequest) -> Result<CreateCheckoutResponse, tonic::Status> {
        let res =  self.create_checkout(req).await?;
        Ok(res.into_inner())
    }
}