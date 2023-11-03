use game::{MoveList, PlayerTrait};

pub struct GreedyPlayer {
    name: String,
    move_list: MoveList,
}

impl GreedyPlayer {
    pub fn new(name: String) -> Self {
        let move_list = MoveList::default();
        Self { name, move_list }
    }
}

#[async_trait::async_trait]
impl PlayerTrait for GreedyPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_move(&mut self, mut game_state: game::GameState) -> game::Move {
        game_state.get_possible_moves(&mut self.move_list);
        let mut best_move = self.move_list[0];
        let mut best_score = -1000;
        let me = usize::from(game_state.get_current_player());
        for move_ in &self.move_list {
            let mut game_state_clone = game_state.clone();
            game_state_clone.do_move(*move_);
            let score = game_state.get_scores()[me];
            if score > best_score {
                best_score = score;
                best_move = *move_;
            }
        }
        best_move
    }
}
