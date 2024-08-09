use std::convert::Infallible;
use std::sync::Arc;
use bucket_api::webhook_event::WebhookEvents;
use bucket_common_types::WebhookSignatureScheme;
use futures::SinkExt;
use tokio_tungstenite_wasm::WebSocketStream;
use crate::api::ApiToken;

pub trait WebhookEventHandler  {
    fn handle_webhook_event(&self, event: &WebhookEvents) -> Result<(), Infallible>;
}

pub trait WebhookConnector {
    async fn connect<S: AsRef<str>>(url: S, api_token: ApiToken, webhook_signature_scheme: WebhookSignatureScheme, event_handler : impl WebhookEventHandler) -> Result<Self, Infallible>;

    async fn send<E>(&mut self, item:E) -> Result<(), Infallible>;
}


pub struct WebhookClient  {
    pub web_socket_stream: WebSocketStream,
    pub event_handler: Arc<dyn WebhookEventHandler>,
}


impl WebhookConnector for WebhookClient {
    async fn connect<S: AsRef<str>>(url: S, api_token: ApiToken, webhook_signature_scheme: WebhookSignatureScheme, event_handler: impl WebhookEventHandler) -> Result<Self, Infallible> {
        let ws = tokio_tungstenite_wasm::connect(url).await.unwrap();
        Ok(Self {
            web_socket_stream: ws,
            event_handler,
        })
    }

    async fn send<E>(&mut self, item: E) -> Result<(), Infallible> {
        todo!()
    }
}


impl WebhookClient {
    async fn send<E>(&mut self, item: E) -> Result<(), Infallible> {
        self.web_socket_stream.send(item).await.unwrap();
        Ok(())
    }
}

