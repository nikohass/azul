use crate::{
    game_manager::game_state_to_json,
    websocket_api::{EventType, WebSocketConnection, WebSocketMessage},
};
use game::{GameState, Move, MoveList, PlayerTrait};

pub struct HumanPlayer {
    name: String,
    websocket: WebSocketConnection,
    move_list: MoveList,
}

impl HumanPlayer {
    pub fn new(name: String, websocket: WebSocketConnection) -> Self {
        let move_list = MoveList::default();
        Self {
            name,
            websocket,
            move_list,
        }
    }
}

#[async_trait::async_trait]
impl PlayerTrait for HumanPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_move(&mut self, mut game_state: GameState) -> Move {
        game_state.get_possible_moves(&mut self.move_list);

        for _ in 0..10 {
            let request_id = uuid::Uuid::new_v4().to_string();
            let json = format_move_request_json(&game_state, &self.move_list, &request_id);
            let move_request_msg = WebSocketMessage {
                event_type: EventType::MoveRequest,
                data: json,
            };

            log::info!("Sending move request");
            let mut receiver = self
                .websocket
                .send_and_recv_move(move_request_msg, &request_id)
                .await;

            let response = receiver.recv().await.unwrap();

            log::info!("Received move response");
            let move_index = response.data["move_index"].as_u64();
            match move_index {
                Some(move_index) => {
                    log::info!("Received move index: {}", move_index);
                    if move_index >= self.move_list.len() as u64 {
                        log::info!("Received invalid move index: {:?}", move_index);
                        continue;
                    }
                    return self.move_list[move_index as usize];
                }
                None => {
                    log::info!("Received invalid move index: {:?}", move_index);
                }
            }
        }
        panic!("Failed to get valid move");
    }
}

fn format_move_request_json(
    game_state: &GameState,
    move_list: &MoveList,
    request_id: &str,
) -> serde_json::Value {
    let game_state_json = game_state_to_json(game_state);
    let mut moves = Vec::new();
    for move_ in move_list {
        let mut move_json = serde_json::Map::new();
        move_json.insert(
            "take_from_factory_index".to_string(),
            move_.take_from_factory_index.into(),
        );
        let color = char::from(move_.color).to_string();
        move_json.insert("color".to_string(), color.into());
        let pattern_json = serde_json::Value::from(move_.pattern.to_vec());
        move_json.insert("pattern".to_string(), pattern_json);
        moves.push(serde_json::Value::from(move_json));
    }
    let move_list_json = serde_json::Value::from(moves);
    let mut json = serde_json::Map::new();
    json.insert("game_state".to_string(), game_state_json);
    json.insert("move_list".to_string(), move_list_json);
    json.insert("request_id".to_string(), request_id.into());
    serde_json::Value::from(json)
}
