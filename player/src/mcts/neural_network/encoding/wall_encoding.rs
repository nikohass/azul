use game::{
    wall::{get_col_mask, get_row_mask, WALL_COLOR_MASKS},
    NUM_PLAYERS,
};

use crate::mcts::neural_network::encoding::factory_encoding::FACTORY_ENCODING_SIZE;

use super::{super::layers::InputLayer, pattern_line_encoding::PLAYER_PATTERN_LINE_ENCODING_SIZE};

pub const PLAYER_WALL_ENCODING_SIZE: usize = 2_usize.pow(5) * 5 * 3;

pub fn add_wall_encoding(wall: u32, player_index: usize, layer: &mut dyn InputLayer) {
    // Encode a players wall. The wall is encoded in 3 parts: rows, columns and diagonals.
    // Each uses bitwise operations to extract the relevant 5 bits from the wall.
    // The bits are then used as an index to set the input of the layer.
    let mut base_index = FACTORY_ENCODING_SIZE
        + PLAYER_PATTERN_LINE_ENCODING_SIZE * NUM_PLAYERS
        + player_index * PLAYER_WALL_ENCODING_SIZE;

    // Encode rows
    for row in 0..5 {
        let index = (wall & get_row_mask(row)) >> (row * 6);
        let index = base_index + 0b11111 * row + index as usize;
        layer.set_input(index);
    }

    base_index += 0b11111 * 5;
    // Encode columns
    for col in 0..5 {
        let mut index = 0;
        let masked = wall & get_col_mask(col);
        for row in 0..5 {
            index |= (masked >> (row * 6)) & 0b11111;
        }
        let index = base_index + index as usize + 0b11111 * col;
        layer.set_input(index);
    }

    base_index += 0b11111 * 5;
    // Encode diagonals
    for (diagonal, mask) in WALL_COLOR_MASKS.iter().enumerate() {
        let mut index = 0;
        let masked = wall & mask;
        for row in 0..5 {
            index |= (masked >> (row * 6)) & 0b11111;
        }
        let index = base_index + index as usize + diagonal * 0b11111;
        layer.set_input(index);
    }
}

// The wall only changes after round evaluation. Because many things change ther we just reset the accumulator
