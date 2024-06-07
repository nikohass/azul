mod factories;
mod formatting;
mod game_state;
pub mod match_;
mod move_;
mod move_list;
mod player;
mod tile_color;
mod time_control;
pub mod wall;

pub use factories::{
    hash_factory, unhash_factory, Factories, Factory, CENTER_FACTORY_INDEX, INDEX_TO_FACTORY,
    NUM_FACTORIES, NUM_NON_CENTER_FACTORIES, NUM_POSSIBLE_FACTORY_PERMUTATIONS,
};
pub use formatting::display_gamestate;
pub use game_state::Bag;
pub use game_state::GameState;
pub use game_state::{MoveGenerationResult, FLOOR_LINE_PENALTY};
pub use move_::Move;
pub use move_list::MoveList;
pub use player::{Player, PlayerMarker};
pub use tile_color::TileColor;
pub use tile_color::NUM_TILE_COLORS;
pub use time_control::TimeControl;
pub use wall::{field_at, WALL_COLOR_MASKS};
pub const NUM_PATTERN_LINES: usize = 5;

#[derive(Debug, PartialEq)]
pub enum GameError {
    IllegalMove,
    PlayerCountMismatch,
    InvalidGameState(String),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::IllegalMove => write!(f, "Illegal move"),
            GameError::PlayerCountMismatch => write!(f, "Player count mismatch"),
            GameError::InvalidGameState(s) => write!(f, "Invalid game state: {}", s),
        }
    }
}

#[cfg(feature = "three_players")]
pub const NUM_PLAYERS: usize = 3;

#[cfg(feature = "four_players")]
pub const NUM_PLAYERS: usize = 4;

#[cfg(not(any(feature = "three_players", feature = "four_players")))]
pub const NUM_PLAYERS: usize = 2;
