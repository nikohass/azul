use super::NUM_PLAYERS;
use crate::{move_::Move, GameState};

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

#[async_trait::async_trait]
pub trait Player: Send + Sync {
    fn get_name(&self) -> &str;
    async fn get_move(&mut self, game_state: &GameState) -> Move;

    // Optional methods for settings and state updates that not all players need
    async fn notify_move(&mut self, _new_game_state: &GameState, _move_: Move) {}
    async fn set_time(&mut self, _time: u64) {}
    async fn set_pondering(&mut self, _pondering: bool) {}
    async fn reset(&mut self) {}
}
