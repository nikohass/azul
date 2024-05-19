use super::NUM_PLAYERS;
use crate::{move_::Move, GameState, TimeControl};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerMarker(u8);

impl PlayerMarker {
    #[inline]
    pub fn new(id: u8) -> Self {
        Self(id)
    }

    #[inline]
    pub fn next(&self) -> Self {
        Self((self.0 + 1) % (NUM_PLAYERS as u8))
    }
}

impl From<PlayerMarker> for u8 {
    fn from(val: PlayerMarker) -> Self {
        val.0
    }
}

impl From<PlayerMarker> for usize {
    fn from(val: PlayerMarker) -> Self {
        val.0 as usize
    }
}

pub trait Player: Send + Sync {
    fn get_name(&self) -> &str;
    fn set_name(&mut self, _name: &str) {}
    fn get_move(&mut self, game_state: &GameState) -> Move;

    // Optional methods for settings and state updates that not all players need
    fn notify_move(&mut self, _new_game_state: &GameState, _move_: Move) {}
    fn notify_factories_refilled(&mut self, _game_state: &GameState) {}
    fn notify_game_over(&mut self, _game_state: &GameState) {}
    fn set_time(&mut self, _time: TimeControl) {}
    fn set_pondering(&mut self, _pondering: bool) {}
    fn set_thread_count(&mut self, _thread_count: u32) {}
    fn reset(&mut self) {}
}
