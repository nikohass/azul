use crate::{game_manager::Match, human_player::HumanPlayer, shared_state::SharedState};
use futures::{SinkExt, StreamExt};
use game::PlayerTrait;
use player::random_player::RandomPlayer;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};

type WsTcpStream = WebSocketStream<tokio::net::TcpStream>;
type WsSink = futures::stream::SplitSink<WsTcpStream, Message>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Ping,
    NewGame,
    Error,
    StartGame,
    GameStateUpdate,
    GameOver,
    MoveRequest,
    MoveResponse,
}

impl EventType {
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "ping" => Some(Self::Ping),
            "new_game" => Some(Self::NewGame),
            "error" => Some(Self::Error),
            "start_game" => Some(Self::StartGame),
            "game_state_update" => Some(Self::GameStateUpdate),
            "game_over" => Some(Self::GameOver),
            "move_request" => Some(Self::MoveRequest),
            "move_response" => Some(Self::MoveResponse),
            _ => None,
        }
    }
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        match self {
            Self::Ping => "ping",
            Self::NewGame => "new_game",
            Self::Error => "error",
            Self::StartGame => "start_game",
            Self::GameStateUpdate => "game_state_update",
            Self::GameOver => "game_over",
            Self::MoveRequest => "move_request",
            Self::MoveResponse => "move_response",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
    pub event_type: EventType,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct WebSocketConnection {
    write: SharedState<WsSink>,
    addr: SocketAddr,
    read_broadcast: tokio::sync::broadcast::Sender<WebSocketMessage>,
    // pending_responses: SharedState<HashMap<String, tokio::sync::mpsc::Sender<WebSocketMessage>>>,
}

impl WebSocketConnection {
    pub async fn start(stream: tokio::net::TcpStream, addr: SocketAddr) -> Result<(), String> {
        log::info!("New WebSocket connection: {}", addr);
        let ws_stream = accept_async(stream).await;
        let (write, read) = match ws_stream {
            Ok(ws_stream) => ws_stream.split(),
            Err(err) => {
                log::error!("WebSocket connection error: {:?}", err);
                return Err(format!("WebSocket connection error: {:?}", err));
            }
        };

        let broadcast: (
            tokio::sync::broadcast::Sender<WebSocketMessage>,
            tokio::sync::broadcast::Receiver<WebSocketMessage>,
        ) = tokio::sync::broadcast::channel::<WebSocketMessage>(16);

        let write = SharedState::new(write);

        let connection = Self {
            write,
            addr,
            read_broadcast: broadcast.0,
            // pending_responses: SharedState::new(HashMap::new()),
        };

        connection.spawn_receiver_task(read).await;

        connection.run().await
    }

    pub fn get_address(&self) -> SocketAddr {
        self.addr
    }

    async fn spawn_receiver_task(&self, mut read: futures::stream::SplitStream<WsTcpStream>) {
        // Spawn a thread that will send all received messages to all broadcast receivers

        let broadcast_sender: tokio::sync::broadcast::Sender<WebSocketMessage> =
            self.read_broadcast.clone();

        tokio::task::spawn(async move {
            while let Some(result) = read.next().await {
                match result {
                    Ok(data) => {
                        let json = serde_json::from_str::<serde_json::Value>(&data.to_string());
                        if let Err(err) = json {
                            log::error!("Error parsing message: {:?}", err);
                            continue;
                        }
                        let json = json.unwrap();
                        let data = json["data"].clone();
                        let event = match json["event"].as_str() {
                            Some(string) => match EventType::from_str(string) {
                                Some(event) => event,
                                None => {
                                    log::error!("Unknown event: {}", string);
                                    continue;
                                }
                            },
                            None => {
                                log::error!("Missing event field");
                                continue;
                            }
                        };
                        let message = WebSocketMessage {
                            event_type: event,
                            data,
                        };
                        let _ = broadcast_sender.send(message);
                        // if let Err(err) = send_result {
                        //     log::error!("Error sending message: {:?}", err);
                        //     log::info!("Stopping receiver task");
                        //     break;
                        // }
                    }
                    Err(err) => log::error!("Error reading message: {:?}", err),
                }
            }
        });
    }

    fn get_read_broadcast(&self) -> tokio::sync::broadcast::Receiver<WebSocketMessage> {
        self.read_broadcast.subscribe()
    }

    pub fn send_message(&self, message: WebSocketMessage) {
        let json = serde_json::json!({
            "event": message.event_type.to_string(),
            "data": message.data,
        });
        let json = json.to_string();
        let message = Message::Text(json);
        let write = self.write.clone();
        tokio::spawn(async move {
            let mut write = write.lock().await;
            if let Err(err) = write.send(message).await {
                log::error!("Error sending message: {:?}", err);
            }
            println!("Sent message");
        });
    }

    async fn run(self) -> Result<(), String> {
        let mut receiver: tokio::sync::broadcast::Receiver<WebSocketMessage> =
            self.get_read_broadcast();
        while let Ok(message) = receiver.recv().await {
            log::info!("Received message: {:?}", message);
            let response = match message.event_type {
                EventType::NewGame => self.handle_new_game_msg(message).await,
                EventType::Ping => {
                    log::debug!("Received ping event");
                    Ok(WebSocketMessage {
                        event_type: EventType::Ping,
                        data: serde_json::json!({}),
                    })
                }
                EventType::StartGame => self.handle_start_game_msg(message).await,
                EventType::Error => {
                    log::error!("Client sent error event");
                    continue;
                }
                // EventType::MoveResponse => {
                //     log::info!("Client sent a move.");
                //     let request_id = message.data["request_id"]
                //         .as_str()
                //         .ok_or("Missing request_id field")?;
                //     println!("Accessing pending_responses");
                //     let sender = self.pending_responses.lock().await.remove(request_id);
                //     println!("Accessing pending_responses done");
                //     if let Some(sender) = sender {
                //         let result: Result<
                //             (),
                //             tokio::sync::mpsc::error::SendError<WebSocketMessage>,
                //         > = sender.send(message).await;
                //         if let Err(err) = result {
                //             log::error!("Error sending move response: {}", err);
                //         }
                //     } else {
                //         log::error!(
                //             "Received move response for unknown request id: {}",
                //             request_id
                //         );
                //     }
                //     continue;
                // }
                EventType::GameStateUpdate | EventType::GameOver | EventType::MoveRequest => {
                    log::error!(
                        "Client sent {} event, this event is only sent by the server",
                        message.event_type.to_string()
                    );
                    continue;
                }
                EventType::MoveResponse => {
                    continue;
                }
            };
            match response {
                Ok(response) => self.send_message(response),
                Err(err) => {
                    log::error!("Error handling message: {}", err);
                    self.send_message(WebSocketMessage {
                        event_type: EventType::Error,
                        data: serde_json::json!({
                            "error": err,
                        }),
                    });
                }
            }
        }
        log::info!("WebSocket connection closed: {}", self.addr);
        Ok(())
    }

    async fn handle_new_game_msg(
        &self,
        message: WebSocketMessage,
    ) -> Result<WebSocketMessage, String> {
        let data = message.data;
        println!("{:#?}", data);

        let mut players: Vec<Box<dyn PlayerTrait>> = Vec::new();
        for player_json in data["players"].as_array().ok_or("Missing players field")? {
            let name = player_json["name"].as_str().ok_or("Missing name field")?;
            let player_type = player_json["type"].as_str().ok_or("Missing type field")?;
            let player: Box<dyn PlayerTrait> = match player_type {
                "human" => Box::new(HumanPlayer::new(name.to_string(), self.clone())),
                "random" => Box::new(RandomPlayer::new(name.to_string())),
                "greedy" => Box::new(player::greedy_player::GreedyPlayer::new(name.to_string())),
                "mcts" => Box::<player::mcts::node::MonteCarloTreeSearch>::default(),
                _ => return Err(format!("Unknown player type: {}", player_type)),
            };
            players.push(player);
        }
        let game_manager_shared = Match::new_with_players(players).await;
        let game_manager = game_manager_shared.lock().await;
        let player_names = game_manager.get_player_names().await;
        let game_id = game_manager.get_id().to_string();

        let response = WebSocketMessage {
            event_type: EventType::NewGame,
            data: serde_json::json!(
                {
                    "id": game_id,
                    "players": player_names,
                }
            ),
        };
        log::info!("Created new game: {}", game_id);
        Ok(response)
    }

    async fn handle_start_game_msg(
        &self,
        message: WebSocketMessage,
    ) -> Result<WebSocketMessage, String> {
        let id = message.data["id"].as_str().ok_or("Missing id field")?;
        let start_msg = WebSocketMessage {
            event_type: EventType::StartGame,
            data: serde_json::json!({ "id": id }),
        };
        self.send_message(start_msg);
        let game_manager_shared = Match::get_game(id)
            .await
            .ok_or(format!("Game with id {} not found", id))?;
        let mut game_manager = game_manager_shared.lock().await;
        log::info!("Starting game {}", id);
        game_manager.start_match(self.clone()).await;
        let response = WebSocketMessage {
            event_type: EventType::GameOver,
            data: serde_json::json!({ "id": id }),
        };
        Ok(response)
    }

    pub async fn send_and_recv_move(
        &self,
        move_request: WebSocketMessage,
        request_id: &str,
    ) -> tokio::sync::mpsc::Receiver<WebSocketMessage> {
        log::info!("Sending move requests (id: {})", request_id);
        let mut read_broadcast = self.get_read_broadcast();
        let my_request_id = request_id.to_string();
        let (sender, receiver) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            while let Ok(message) = read_broadcast.recv().await {
                log::info!("Received message: {:?}", message);
                if let EventType::MoveResponse = message.event_type {
                    log::info!("Client sent a move.");
                    if let Some(actual_request_id) = message.data["request_id"].as_str() {
                        if my_request_id == actual_request_id {
                            log::info!("Received move response for request id: {}", my_request_id);
                            if let Err(e) = sender.send(message).await {
                                log::error!("Failed to send move response: {:?}", e);
                            }
                            break;
                        }
                    } else {
                        log::error!("Missing request_id field");
                    }
                }
            }
        });

        self.send_message(move_request);
        log::info!("Waiting for move response (id: {})", request_id);
        receiver
    }
}

// pub async fn handle_websocket_connection(stream: tokio::net::TcpStream, addr: SocketAddr) {
//     log::info!("New WebSocket connection: {}", addr);
//     if let Err(err) = handle_websocket(stream, addr).await {
//         log::error!("WebSocket connection error: {:?}", err);
//     }
// }

// async fn handle_message(
//     message: &Message,
//     write: &SharedState<WsSink>,
//     addr: SocketAddr,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let json = serde_json::from_str::<serde_json::Value>(&message.to_string())?;
//     let event = json["event"]
//         .as_str()
//         .ok_or_else(|| Box::<dyn std::error::Error + Send + Sync>::from("Missing event field"))?;
//     log::info!("Received {} from {}", event, addr);
//     match event {
//         "ping" => handle_ping(write, addr).await?,
//         "new_game" => handle_new_game(write, addr, json).await?,
//         _ => log::error!("Unknown event: {}", event),
//     }

//     Ok(())
// }

// async fn handle_ping(
//     write: &SharedState<WsSink>,
//     addr: SocketAddr,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let pong = serde_json::json!({
//         "event": "pong",
//     });
//     write
//         .lock()
//         .await
//         .send(Message::Text(pong.to_string()))
//         .await?;
//     log::info!("Sent pong to {}", addr);
//     Ok(())
// }

// async fn handle_new_game(
//     write: &SharedState<WsSink>,
//     addr: SocketAddr,
//     json: serde_json::Value,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let pretty = serde_json::to_string_pretty(&json)?;
//     println!("{}", pretty);

//     let id = new_game.get_id();
//     let response = serde_json::json!({
//         "event": "game_created",
//         "id": id,
//         "players": players,
//     });
//     write
//         .lock()
//         .await
//         .send(Message::Text(response.to_string()))
//         .await?;
//     log::info!("Sent game_created to {}", addr);
//     Ok(())
// }

pub async fn run_websocket_api() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:3001".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    log::info!("WebSocket server running on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(WebSocketConnection::start(stream, addr));
    }
    Ok(())
}
