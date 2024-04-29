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
    fn set_name(&mut self, _name: &str) {}
    async fn get_move(&mut self, game_state: &GameState) -> Move;

    // Optional methods for settings and state updates that not all players need
    async fn notify_move(&mut self, _new_game_state: &GameState, _move_: Move) {}
    async fn set_time(&mut self, _time: u64) {}
    async fn set_pondering(&mut self, _pondering: bool) {}
    async fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use rand::{rngs::SmallRng, SeedableRng as _};

    use crate::MoveList;

    use super::*;

    pub struct MockPlayer {
        name: String,
    }

    #[async_trait::async_trait]
    impl Player for MockPlayer {
        fn get_name(&self) -> &str {
            &self.name
        }

        async fn get_move(&mut self, game_state: &GameState) -> Move {
            let mut game_state = game_state.clone();
            let mut move_list = MoveList::default();
            let mut rng = SmallRng::seed_from_u64(0);
            game_state.get_possible_moves(&mut move_list, &mut rng);
            move_list[0]
        }
    }

    #[test]
    fn test_player_marker() {
        let marker = PlayerMarker::new(0);
        assert_eq!(marker.next(), PlayerMarker::new(1));
    }

    #[tokio::test]
    async fn test_mock_player() {
        let mut player = MockPlayer {
            name: "MockPlayer".to_string(),
        };
        let mut rng = SmallRng::seed_from_u64(0);
        let game_state = GameState::new(&mut rng);
        let _move: Move = player.get_move(&game_state).await;

        assert_eq!(player.get_name(), "MockPlayer");
    }
}
