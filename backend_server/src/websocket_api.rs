use crate::shared_state::SharedState;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};

type WsTcpStream = WebSocketStream<tokio::net::TcpStream>;
type WsSink = futures::stream::SplitSink<WsTcpStream, Message>;

pub async fn handle_websocket_connection(stream: tokio::net::TcpStream, addr: SocketAddr) {
    log::info!("New WebSocket connection: {}", addr);
    if let Err(err) = handle_websocket(stream, addr).await {
        log::error!("WebSocket connection error: {:?}", err);
    }
}

async fn handle_websocket(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (write, mut read) = ws_stream.split();
    let write = SharedState::new(write);

    while let Some(result) = read.next().await {
        match result {
            Ok(message) => handle_message(&message, &write, addr).await?,
            Err(err) => log::error!("Error reading message: {:?}", err),
        }
    }
    Ok(())
}

async fn handle_message(
    message: &Message,
    write: &SharedState<WsSink>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::from_str::<serde_json::Value>(&message.to_string())?;
    let event = json["event"]
        .as_str()
        .ok_or_else(|| Box::<dyn std::error::Error + Send + Sync>::from("Missing event field"))?;
    log::info!("Received {} from {}", event, addr);
    match event {
        "ping" => handle_ping(write, addr).await?,
        _ => log::error!("Unknown event: {}", event),
    }

    Ok(())
}

async fn handle_ping(
    write: &SharedState<WsSink>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pong = serde_json::json!({
        "event": "pong",
    });
    write
        .lock()
        .await
        .send(Message::Text(pong.to_string()))
        .await?;
    log::info!("Sent pong to {}", addr);
    Ok(())
}

pub async fn run_websocket_api() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:3001".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    log::info!("WebSocket server running on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_websocket_connection(stream, addr));
    }
    Ok(())
}
