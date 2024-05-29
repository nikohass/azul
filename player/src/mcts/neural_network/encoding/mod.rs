use factory_encoding::FACTORY_ENCODING_SIZE;
use game::NUM_PLAYERS;
use pattern_line_encoding::PLAYER_PATTERN_LINE_ENCODING_SIZE;
use score_encoding::PLAYER_SCORE_ENCODING_SIZE;
use wall_encoding::PLAYER_WALL_ENCODING_SIZE;

pub mod factory_encoding;
pub mod pattern_line_encoding;
pub mod score_encoding;
pub mod wall_encoding;

pub const TOTAL_ENCODING_SIZE: usize = FACTORY_ENCODING_SIZE + (
    PLAYER_PATTERN_LINE_ENCODING_SIZE + PLAYER_WALL_ENCODING_SIZE + PLAYER_SCORE_ENCODING_SIZE
) * NUM_PLAYERS;