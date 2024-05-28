use crate::buffer::ReplayEntry;
use crate::BUFFER;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::Message;

#[derive(Deserialize, Serialize)]
pub enum ApiRequest {
    AddEntries(u64, Vec<ReplayEntry>),
    SampleEntries(u64, usize),
    SetBufferSize(u64, usize),
}

#[derive(Serialize, Deserialize)]
pub enum ApiResponse {
    Success(u64),
    Entries(u64, Vec<ReplayEntry>),
}

pub async fn handle_connection(raw_stream: TcpStream) {
    let ws_stream = accept_async(raw_stream)
        .await
        .expect("Failed to accept WebSocket");
    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(_)) => {
                log::error!("Received text message. Only binary messages are supported");
            }
            Ok(Message::Binary(bin)) => {
                handle_message(&mut write, bin).await;
            }
            Ok(Message::Ping(data)) => {
                write
                    .send(Message::Pong(data))
                    .await
                    .expect("Failed to send Pong");
            }
            Ok(Message::Close(reason)) => {
                write
                    .send(Message::Close(reason))
                    .await
                    .expect("Failed to close WebSocket");
                break;
            }
            Err(_) => {
                break;
            }
            _ => {}
        }
    }
}

async fn handle_message(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    message: Vec<u8>,
) {
    let request: ApiRequest =
        serde_json::from_slice(&message).expect("Failed to deserialize request");

    let response = handle_request(request).await;

    let response_bytes = serde_json::to_vec(&response).expect("Failed to serialize response");
    write
        .send(Message::Binary(response_bytes))
        .await
        .expect("Failed to send response");
}

async fn handle_request(request: ApiRequest) -> ApiResponse {
    let mut buffer = BUFFER.lock().await;
    match request {
        ApiRequest::AddEntries(nonce, entries) => {
            buffer.add_entries(entries);
            ApiResponse::Success(nonce)
        }
        ApiRequest::SampleEntries(nonce, n) => {
            let entries = buffer.sample_n_entries(n, &mut rand::thread_rng());
            ApiResponse::Entries(nonce, entries)
        }
        ApiRequest::SetBufferSize(nonce, size) => {
            buffer.set_max_size(size);
            ApiResponse::Success(nonce)
        }
    }
}
