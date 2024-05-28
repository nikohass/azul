use futures_util::{stream::SplitSink, SinkExt as _, StreamExt as _};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::api::ApiRequest;
use crate::buffer::ReplayEntry;

pub struct ReplayBufferClient {
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl ReplayBufferClient {
    // Connect to the WebSocket server
    pub async fn connect(url: &str) -> Self {
        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect to WebSocket");
        let (write, _) = ws_stream.split();

        Self { write }
    }

    // Send an AddEntries request
    pub async fn add_entries(&mut self, nonce: u64, entries: Vec<ReplayEntry>) {
        let request = ApiRequest::AddEntries(nonce, entries);
        self.send_request(request).await;
    }

    // Send a SampleEntries request
    pub async fn sample_entries(&mut self, nonce: u64, count: usize) {
        let request = ApiRequest::SampleEntries(nonce, count);
        self.send_request(request).await;
    }

    // Send a SetBufferSize request
    pub async fn set_buffer_size(&mut self, nonce: u64, size: usize) {
        let request = ApiRequest::SetBufferSize(nonce, size);
        self.send_request(request).await;
    }

    // Send a request to the server
    async fn send_request(&mut self, request: ApiRequest) {
        let request_bytes = serde_json::to_vec(&request).expect("Failed to serialize request");
        self.write
            .send(Message::Binary(request_bytes))
            .await
            .expect("Failed to send request");
    }
}
