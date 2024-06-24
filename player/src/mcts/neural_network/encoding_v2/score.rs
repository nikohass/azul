use game::{FLOOR_LINE_PENALTY, NUM_PLAYERS};

use crate::mcts::neural_network::layers::InputLayer;

use super::{pattern_lines::LowerPatternLinesEncoding, OneHotFeature};

const FLOOR_LINE_ENCODING_SIZE: usize = FLOOR_LINE_PENALTY.len();
const MAX_SCORE: i16 = 120;
pub const SCORE_ENCODING_SIZE: usize = MAX_SCORE as usize + FLOOR_LINE_ENCODING_SIZE + 1;

fn calculate_floor_line_index(progress: u8) -> usize {
    (progress as usize).min(FLOOR_LINE_PENALTY.len() - 1)
}

pub fn calculate_score_index(score: i16, floor_line_progress: u8) -> (usize, usize) {
    // Encode the score and floor line progress
    // The floor line progress is encoded as a one-hot vector with one value for each possible floor line progress
    // The score is encoded as a one-hot vector with one value for each possible score. The floor line penalty is subtracted from the score before encoding
    let floor_line_progress_index = calculate_floor_line_index(floor_line_progress);
    let floor_line_penalty = FLOOR_LINE_PENALTY[floor_line_progress_index];
    let adjusted_score = score - (floor_line_penalty as i16);
    let score_index = adjusted_score as usize;
    (
        score_index.min(MAX_SCORE as usize) + FLOOR_LINE_ENCODING_SIZE,
        floor_line_progress_index,
    )
}

pub struct ScoreEncoding {
    pub score_index: [usize; NUM_PLAYERS],
    pub floor_line_index: [usize; NUM_PLAYERS],
}

impl OneHotFeature for ScoreEncoding {
    const SIZE: usize = SCORE_ENCODING_SIZE;
    const PLAYER_FEATURE: bool = true;
    const MAX_ONES: usize = 2;
    const START: usize = LowerPatternLinesEncoding::END;

    fn initialize(layer: &mut impl InputLayer) -> Self {
        let mut score_index_ = [0; NUM_PLAYERS];
        let mut floor_line_index_ = [0; NUM_PLAYERS];
        for player_index in 0..NUM_PLAYERS {
            let (score_index, floor_line_index) = calculate_score_index(0, 0);
            let score_index = score_index * NUM_PLAYERS + player_index + Self::START;
            let floor_line_index = floor_line_index * NUM_PLAYERS + player_index + Self::START;
            layer.set_input(score_index);
            layer.set_input(floor_line_index);
            score_index_[player_index] = score_index;
            floor_line_index_[player_index] = floor_line_index;
        }
        Self {
            score_index: score_index_,
            floor_line_index: floor_line_index_,
        }
    }
}

impl ScoreEncoding {
    pub fn set(
        &mut self,
        score: i16,
        floor_line_progress: u8,
        player_index: usize,
        layer: &mut impl InputLayer,
    ) {
        let (score_index, floor_line_index) = calculate_score_index(score, floor_line_progress);
        let score_index = score_index * NUM_PLAYERS + player_index + Self::START;
        debug_assert!(
            score_index < Self::END,
            "Score index out of bounds: {}, end: {}",
            score_index,
            Self::END
        );
        let floor_line_index = floor_line_index * NUM_PLAYERS + player_index + Self::START;
        debug_assert!(
            floor_line_index < Self::END,
            "Floor line index out of bounds: {}, end: {}",
            floor_line_index,
            Self::END
        );
        if self.score_index[player_index] != score_index {
            layer.unset_input(self.score_index[player_index]);
            self.score_index[player_index] = score_index;
            layer.set_input(score_index);
        }

        if self.floor_line_index[player_index] != floor_line_index {
            layer.unset_input(self.floor_line_index[player_index]);
            self.floor_line_index[player_index] = floor_line_index;
            layer.set_input(floor_line_index);
        }
    }
}
