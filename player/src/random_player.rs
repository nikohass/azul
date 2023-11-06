use game::{GameState, Move, MoveList, PlayerTrait};
use rand::{Rng, SeedableRng};

pub struct RandomPlayer {
    name: String,
    move_list: MoveList,
    rng: rand::rngs::StdRng,
}

impl RandomPlayer {
    pub fn new(name: String) -> Self {
        let move_list = MoveList::default();
        let rng = rand::rngs::StdRng::from_entropy();
        Self {
            name,
            move_list,
            rng,
        }
    }
}

#[async_trait::async_trait]
impl PlayerTrait for RandomPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_move(&mut self, mut game_state: GameState) -> Move {
        game_state.get_possible_moves(&mut self.move_list);
        self.move_list[self.rng.gen_range(0..self.move_list.len())]
    }
}
