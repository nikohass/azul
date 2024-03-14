use game::{GameState, Move, MoveList, Player};
use rand::{Rng, SeedableRng};

pub struct RandomPlayer {
    name: String,
    move_list: MoveList,
    rng: rand::rngs::SmallRng,
}

impl RandomPlayer {
    pub fn new(name: String) -> Self {
        let move_list = MoveList::default();
        let rng = rand::rngs::SmallRng::from_entropy();
        Self {
            name,
            move_list,
            rng,
        }
    }
}

#[async_trait::async_trait]
impl Player for RandomPlayer {
    fn get_name(&self) -> &str {
        &self.name
    }

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        let mut game_state = game_state.clone();
        game_state.get_possible_moves(&mut self.move_list, &mut self.rng);
        self.move_list[self.rng.gen_range(0..self.move_list.len())]
    }
}

impl Default for RandomPlayer {
    fn default() -> Self {
        Self::new("RandomPlayer".to_string())
    }
}
