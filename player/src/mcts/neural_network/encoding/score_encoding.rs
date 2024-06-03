use super::factory_encoding::FACTORY_ENCODING_SIZE;
use super::pattern_line_encoding::PLAYER_PATTERN_LINE_ENCODING_SIZE;
use super::wall_encoding::PLAYER_WALL_ENCODING_SIZE;
use crate::mcts::neural_network::layers::InputLayer;
use game::{FLOOR_LINE_PENALTY, NUM_PLAYERS};

pub const ASSUMED_MAX_SCORE: i16 = 120;
pub const PLAYER_SCORE_ENCODING_SIZE: usize =
    ASSUMED_MAX_SCORE as usize + FLOOR_LINE_PENALTY.len() + 1;

pub const OFFSET_PLAYER_SCORE: usize = FACTORY_ENCODING_SIZE
    + (PLAYER_PATTERN_LINE_ENCODING_SIZE + PLAYER_WALL_ENCODING_SIZE) * NUM_PLAYERS;

pub fn add_player_score_encoding(
    player_index: usize,
    score: i16,
    pattern_line_progress: usize,
    layer: &mut dyn InputLayer,
    is_next_round_starting_player: bool,
) {
    let pattern_line_progress = pattern_line_progress.min(FLOOR_LINE_PENALTY.len() - 1);
    let offset = OFFSET_PLAYER_SCORE + player_index * PLAYER_SCORE_ENCODING_SIZE;
    // Encode next round starting player
    if is_next_round_starting_player {
        layer.set_input(offset);
    }

    // Encode score
    let score = (score - FLOOR_LINE_PENALTY[pattern_line_progress] as i16)
        .max(0)
        .min(ASSUMED_MAX_SCORE) as usize;
    let index = offset + 1 + score;
    layer.set_input(index);

    // Encode floor line penalty
    let index = offset + 1 + ASSUMED_MAX_SCORE as usize + pattern_line_progress;
    layer.set_input(index);
}
