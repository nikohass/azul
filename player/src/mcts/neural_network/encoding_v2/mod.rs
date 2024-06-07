use super::layers::InputLayer;
use game::{GameState, TileColor};
use pattern_lines::{LOWER_PATTERN_LINES_SIZE, UPPER_PATTERN_LINES_SIZE};

pub mod pattern_lines;
pub mod wall;

const WALL_OFFSET: usize = LOWER_PATTERN_LINES_SIZE + UPPER_PATTERN_LINES_SIZE;
const WALL_SIZE: usize = 2 * 1024 + 32;

pub const ENCODING_SIZE: usize = WALL_OFFSET + WALL_SIZE;

pub const INPUT_SIZE: usize = ENCODING_SIZE + 8 - (ENCODING_SIZE % 8);

pub struct Accumulator<L: InputLayer> {
    upper_pattern_lines: usize,
    lower_pattern_lines: usize,
    wall_indices: [usize; 3],
    layer: L,
}

impl<L: InputLayer> Accumulator<L> {
    pub fn new(mut layer: L) -> Self {
        layer.reset();
        Self {
            upper_pattern_lines: 0, // 0 Means all empty
            lower_pattern_lines: 0, // 0 Means all empty
            wall_indices: [WALL_OFFSET, WALL_OFFSET + 1024, WALL_OFFSET + 2048],
            layer,
        }
    }

    pub fn update_upper_pattern_lines(
        &mut self,
        pattern_lines: [u8; 5],
        colors: [Option<TileColor>; 5],
        update: bool,
    ) {
        let index = pattern_lines::calculate_upper_index(pattern_lines, colors);
        if update {
            self.layer.unset_input(self.upper_pattern_lines);
        }
        self.layer.set_input(index);
        self.upper_pattern_lines = index;
    }

    pub fn update_lower_pattern_lines(
        &mut self,
        pattern_lines: [u8; 5],
        colors: [Option<TileColor>; 5],
        update: bool,
    ) {
        let index =
            pattern_lines::calculate_lower_index(pattern_lines, colors) + UPPER_PATTERN_LINES_SIZE;
        if update {
            self.layer.unset_input(self.lower_pattern_lines);
        }
        self.layer.set_input(index);
        self.lower_pattern_lines = index;
    }

    pub fn update_wall(&mut self, wall: u32, update: bool) {
        let upper_index = wall::calculate_upper_index(wall) + WALL_OFFSET;
        let middle_index = wall::calculate_middle_index(wall) + WALL_OFFSET + 1024;
        let lower_index = wall::calculate_lower_index(wall) + WALL_OFFSET + 2048;

        if update {
            self.layer.unset_input(self.wall_indices[0]);
            self.layer.unset_input(self.wall_indices[1]);
            self.layer.unset_input(self.wall_indices[2]);
        }

        self.layer.set_input(upper_index);
        self.layer.set_input(middle_index);
        self.layer.set_input(lower_index);

        self.wall_indices = [upper_index, middle_index, lower_index];
    }

    pub fn set_game_state(&mut self, game_state: &GameState, player: usize) {
        self.layer.reset();
        self.update_upper_pattern_lines(
            game_state.pattern_lines_occupancy[player],
            game_state.pattern_lines_colors[player],
            false,
        );
        self.update_lower_pattern_lines(
            game_state.pattern_lines_occupancy[player],
            game_state.pattern_lines_colors[player],
            false,
        );
        self.update_wall(game_state.walls[player], false);
    }

    pub fn output(&self) -> &[f32] {
        self.layer.output()
    }

    pub fn layer(&self) -> &L {
        &self.layer
    }

    pub fn mut_layer(&mut self) -> &mut L {
        &mut self.layer
    }
}
