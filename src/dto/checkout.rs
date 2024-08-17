use bucket_api::backend_api::CreateCheckoutRequest;
use bucket_common_types::PaymentModel;

pub struct CreateCheckoutParams {
    pub payment_model: PaymentModel,
    pub change_payment_model: bool,
}
#[derive(thiserror::Error, Debug)]
pub enum CreateCheckoutParamsParsingError {}
impl TryInto<CreateCheckoutRequest> for CreateCheckoutParams {
    type Error = CreateCheckoutParamsParsingError;

    fn try_into(self) -> Result<CreateCheckoutRequest, Self::Error> {
        Ok(CreateCheckoutRequest {
            payment_model: self.payment_model.to_string(),
            change_payment_model: self.change_payment_model,
        })
    }
}
