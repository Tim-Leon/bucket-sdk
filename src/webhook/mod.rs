use bucket_api::webhook_event::WebhookEvents;
use bucket_common_types::WebhookSignatureScheme;
use futures::SinkExt;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_tungstenite_wasm::{Message, WebSocketStream};
use url::Url;
use crate::token::ApiToken;

pub trait WebhookEventHandler: Sized {
    fn handle_webhook_event(&self, event: &WebhookEvents) -> Result<(), Infallible>;
}

pub trait WebhookConnector<WH: WebhookEventHandler>: Sized {
    async fn connect(
        url: Url,
        api_token: ApiToken,
        webhook_signature_scheme: WebhookSignatureScheme,
        event_handler: WH,
    ) -> Result<Self, Infallible>;

}

pub struct WebhookClient<WH: WebhookEventHandler> {
    pub web_socket_stream: WebSocketStream,
    pub event_handler: WH,
}

impl<WH: WebhookEventHandler> WebhookConnector<WH> for WebhookClient<WH> {
    async fn connect<>(
        url: Url,
        api_token: ApiToken,
        webhook_signature_scheme: WebhookSignatureScheme,
        event_handler: WH,
    ) -> Result<Self, Infallible> {
        let ws = tokio_tungstenite_wasm::connect(url).await.unwrap();
        Ok(Self {
            web_socket_stream: ws,
            event_handler,
        })
    }


}

impl<WH: WebhookEventHandler> WebhookClient<WH> {
    async fn send(&mut self, item: Message) -> Result<(), Infallible> {
        self.web_socket_stream.send(item).await.unwrap();
        Ok(())
    }
}
