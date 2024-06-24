use game::{wall, NUM_PLAYERS};

use crate::mcts::neural_network::layers::InputLayer;

use super::{score::ScoreEncoding, OneHotFeature};

pub const WALL_ENCODING_SIZE: usize = 2 * 1024 + 32;

pub fn calculate_upper_index(wall: u32) -> usize {
    // Row 1 and 2 (2 ** 10 = 1024)
    let mut index = wall & wall::get_row_mask(0);
    index <<= 5;
    index |= (wall & wall::get_row_mask(1)) >> 6;
    index as usize
}

pub fn calculate_middle_index(wall: u32) -> usize {
    // Row 3 and 4 (2 ** 10 = 1024)
    let mut index = (wall & wall::get_row_mask(2)) >> 12;
    index <<= 5;
    index |= (wall & wall::get_row_mask(3)) >> 18;
    index as usize
}

pub fn calculate_lower_index(wall: u32) -> usize {
    // Row 5 (2 ** 5 = 32)
    ((wall & wall::get_row_mask(4)) >> 24) as usize
}

#[derive(Clone, Default)]
pub struct WallEncoding {
    pub upper_index: [usize; NUM_PLAYERS],
    pub middle_index: [usize; NUM_PLAYERS],
    pub lower_index: [usize; NUM_PLAYERS],
}

impl OneHotFeature for WallEncoding {
    const SIZE: usize = WALL_ENCODING_SIZE;
    const PLAYER_FEATURE: bool = true;
    const MAX_ONES: usize = 3;
    const START: usize = ScoreEncoding::END;

    fn initialize(layer: &mut impl InputLayer) -> Self {
        let mut upper_index = [0; NUM_PLAYERS];
        let mut middle_index = [0; NUM_PLAYERS];
        let mut lower_index = [0; NUM_PLAYERS];
        for player_index in 0..NUM_PLAYERS {
            let empty_upper_index =
                calculate_upper_index(0) * NUM_PLAYERS + player_index + Self::START;
            layer.set_input(empty_upper_index);
            upper_index[player_index] = empty_upper_index;

            let empty_middle_index = calculate_middle_index(0) * NUM_PLAYERS
                + player_index
                + Self::START
                + 1024 * NUM_PLAYERS;
            layer.set_input(empty_middle_index);
            middle_index[player_index] = empty_middle_index;

            let empty_lower_index = calculate_lower_index(0) * NUM_PLAYERS
                + player_index
                + Self::START
                + 2048 * NUM_PLAYERS;
            layer.set_input(empty_lower_index);
            lower_index[player_index] = empty_lower_index;
        }

        Self {
            upper_index,
            middle_index,
            lower_index,
        }
    }
}

impl WallEncoding {
    pub fn set(&mut self, wall: u32, player_index: usize, layer: &mut impl InputLayer) {
        let upper_index = calculate_upper_index(wall) * NUM_PLAYERS + player_index + Self::START;
        debug_assert!(
            upper_index < Self::END,
            "Upper index out of bounds: {}",
            upper_index
        );
        if self.upper_index[player_index] != upper_index {
            layer.unset_input(self.upper_index[player_index]);
            self.upper_index[player_index] = upper_index;
            layer.set_input(upper_index);
        }

        let middle_index = calculate_middle_index(wall) * NUM_PLAYERS
            + player_index
            + Self::START
            + 1024 * NUM_PLAYERS;
        debug_assert!(
            middle_index < Self::END,
            "Middle index out of bounds: {}",
            middle_index
        );
        if self.middle_index[player_index] != middle_index {
            layer.unset_input(self.middle_index[player_index]);
            self.middle_index[player_index] = middle_index;
            layer.set_input(middle_index);
        }

        let lower_index = calculate_lower_index(wall) * NUM_PLAYERS
            + player_index
            + Self::START
            + 2048 * NUM_PLAYERS;
        debug_assert!(
            lower_index < Self::END,
            "Lower index out of bounds: {}",
            lower_index
        );
        if self.lower_index[player_index] != lower_index {
            layer.unset_input(self.lower_index[player_index]);
            self.lower_index[player_index] = lower_index;
            layer.set_input(lower_index);
        }
    }
}

#[cfg(test)]
mod tests {
    use wall::print_bitboard;

    use super::*;

    #[test]
    fn test_calculate_upper_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for row1 in 0..=31 {
            for row2 in 0..=31 {
                let wall = (row1 << 6) | row2;
                // print_bitboard(wall);
                let index = calculate_upper_index(wall);
                indices_set.insert(index);
            }
        }
        assert_eq!(indices_set.len(), 1024);
        let max = indices_set.iter().max().unwrap();
        assert_eq!(*max, 1023);
    }

    #[test]
    fn test_calculate_middle_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for row3 in 0..=31 {
            for row4 in 0..=31 {
                let wall = (row3 << 12) | (row4 << 18);
                // print_bitboard(wall);
                let index = calculate_middle_index(wall);
                indices_set.insert(index);
            }
        }
        assert_eq!(indices_set.len(), 1024);
        let max = indices_set.iter().max().unwrap();
        assert_eq!(*max, 1023);
    }

    #[test]
    fn test_calculate_lower_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for row5 in 0..=31 {
            let wall = row5 << 24;
            print_bitboard(wall);
            let index = calculate_lower_index(wall);
            indices_set.insert(index);
        }
        assert_eq!(indices_set.len(), 32);
        let max = indices_set.iter().max().unwrap();
        assert_eq!(*max, 31);
    }
}
