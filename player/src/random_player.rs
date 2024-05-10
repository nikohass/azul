use game::{GameState, Move, MoveList, Player};
use rand::{Rng, SeedableRng};

pub struct RandomPlayer {
    name: String,
    move_list: MoveList,
    rng: rand::rngs::SmallRng,
}

impl Player for RandomPlayer {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_move(&mut self, game_state: &GameState) -> Move {
        let mut game_state = game_state.clone();
        game_state.get_possible_moves(&mut self.move_list, &mut self.rng);
        self.move_list[self.rng.gen_range(0..self.move_list.len())]
    }
}

impl Default for RandomPlayer {
    fn default() -> Self {
        let move_list = MoveList::default();
        let rng = rand::rngs::SmallRng::from_entropy();
        Self {
            move_list,
            name: "Random Player".to_string(),
            rng,
        }
    }
}
