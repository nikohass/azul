use game::{GameState, PlayerTrait};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameManagerState {
    NotStarted,
    WaitingForPlayerTurn(usize),
    GameOver,
}

pub struct GameManager {
    id: String,
    game_state: GameState,
    players: Vec<Box<dyn PlayerTrait>>,
    state: GameManagerState,
}

impl GameManager {
    pub fn with_players(players: Vec<Box<dyn PlayerTrait>>) -> Self {
        let mut game_state = GameState::default();
        Self {
            id: Uuid::new_v4().to_string(),
            game_state,
            players,
            state: GameManagerState::NotStarted,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn get_players(&self) -> &[Box<dyn PlayerTrait>] {
        &self.players
    }

    pub fn get_state(&self) -> GameManagerState {
        self.state
    }

    pub fn reset(&mut self) {
        self.game_state = GameState::default();
        self.state = GameManagerState::NotStarted;
    }

    pub fn run_game(&mut self) {
        let game_state = &mut self.game_state;

        let mut round = 0;
        log::info!("Starting game {}", self.id);
        loop {
            game_state.fill_factories(); // Fill the factories before every round
            game_state.check_integrity(); // Check the integrity of the game state. If it is not valid, panic and crash the tokio task

            let mut turn = 0;
            loop {
                let possible_moves = game_state.get_possible_moves();
                if possible_moves.is_empty() {
                    // If there are no legal moves we end the game
                    break;
                }

                // Get the move from the current player
                let current_player = game_state.get_current_player();
                let current_player = usize::from(current_player);
                log::info!(
                    "Round {}, turn {}, player {}",
                    round,
                    turn,
                    self.players[current_player].name()
                );
                let move_ = self.players[current_player].get_move(game_state.clone());

                // Validate the move
                if !possible_moves.contains(&move_) {
                    // If the move is not legal, panic and crash the tokio task
                    panic!(
                        "Player {} made an illegal move: {:?}",
                        current_player, move_
                    );
                }

                // Apply the move to the game state
                game_state.do_move(move_);

                // Check integrity of the game state after the move
                game_state.check_integrity();

                turn += 1;
            }
            // At the end of the round, evaluate it by counting the points and moving the first player marker
            let is_game_over = game_state.evaluate_round(); // If a player has ended the game, this will return true
            if is_game_over {
                self.state = GameManagerState::GameOver;
                break;
            }
            round += 1;
        }
    }
}
