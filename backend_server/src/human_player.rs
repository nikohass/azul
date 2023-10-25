use game::{Move, PlayerTrait};

pub struct HumanPlayer {
    name: String,
}

impl HumanPlayer {
    pub fn new(name: String) -> Self {
        Self { name }
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
