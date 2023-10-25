use game::{Move, PlayerTrait};

use crate::websocket_api::WebSocketConnection;

pub struct HumanPlayer {
    name: String,
    websocket: WebSocketConnection,
}

impl HumanPlayer {
    pub fn new(name: String, websocket: WebSocketConnection) -> Self {
        Self { name, websocket }
    }
}

impl PlayerTrait for HumanPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_move(&self, game_state: game::GameState) -> Move {
        let possible_moves = game_state.get_possible_moves();
        todo!();
    }
}
