use rand::Rng;

use crate::{move_::Move, GameState};

use super::NUM_PLAYERS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Player(u8);

impl Player {
    pub fn new(id: u8) -> Self {
        Self(id)
    }

    pub fn next(&self) -> Self {
        Self((self.0 + 1) % (NUM_PLAYERS as u8))
    }
}

impl From<Player> for u8 {
    fn from(val: Player) -> Self {
        val.0
    }
}

impl From<Player> for usize {
    fn from(val: Player) -> Self {
        val.0 as usize
    }
}

pub trait PlayerTrait: Send + Sync {
    fn name(&self) -> &str;
    fn get_move(&self, game_state: GameState) -> Move;
}

// TODO: Move players to a separate crate
pub struct RandomPlayer {
    name: String,
}

impl RandomPlayer {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl PlayerTrait for RandomPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_move(&self, game_state: GameState) -> Move {
        let mut rng = rand::thread_rng();
        let possible_moves = game_state.get_possible_moves();
        let index = rng.gen_range(0..possible_moves.len());
        possible_moves[index]
    }
}
