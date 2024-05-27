use game::{MoveList, Player};
use rand::{rngs::SmallRng, SeedableRng};

pub struct GreedyPlayer {
    name: String,
    move_list: MoveList,
}

impl Default for GreedyPlayer {
    fn default() -> Self {
        let name = "Greedy Player".to_string();
        let move_list = MoveList::default();
        Self { name, move_list }
    }
}

impl Player for GreedyPlayer {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_move(&mut self, game_state: &game::GameState) -> game::Move {
        let mut game_state = game_state.clone();
        game_state.get_possible_moves(&mut self.move_list, &mut SmallRng::from_entropy());
        let mut best_move = self.move_list[0];
        let mut best_score = -1000;
        let me = usize::from(game_state.current_player);
        for move_ in &self.move_list {
            let mut game_state_clone = game_state.clone();
            game_state_clone.do_move(*move_);
            let score = game_state.scores[me];
            if score > best_score {
                best_score = score;
                best_move = *move_;
            }
        }
        best_move
    }
}
