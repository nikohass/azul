use game::{TileColor, NUM_PLAYERS, NUM_TILE_COLORS};

use crate::mcts::neural_network::layers::InputLayer;

use super::{factories::NonCenterFactoryEncoding, OneHotFeature};

const UPPER_PATTERN_LINE_STATES: [usize; 3] = [6, 11, 16];
const LOWER_PATTERN_LINE_STATES: [usize; 2] = [21, 26];

pub const UPPER_PATTERN_LINES_ENCODING_SIZE: usize = 1056;
pub const LOWER_PATTERN_LINES_ENCODING_SIZE: usize = 546;

fn base_index(num_tiles: u8, color: Option<TileColor>) -> usize {
    if num_tiles == 0 {
        0
    } else {
        1 + ((num_tiles as usize - 1) * NUM_TILE_COLORS) + usize::from(color.unwrap())
        // Safe to unwrap because the color is only None if num_tiles is 0
    }
}

pub fn calculate_index(
    pattern_lines: [u8; 5],
    colors: [Option<TileColor>; 5],
    start: usize,
    count: usize,
    states: &[usize],
) -> usize {
    let mut index = 0;
    let mut multiplier = 1;

    for i in 0..count {
        let num_tiles = pattern_lines[start + i];
        let color = colors[start + i];
        let idx = base_index(num_tiles, color);
        index += idx * multiplier;
        multiplier *= states[i];
    }

    index
}

pub fn calculate_upper_index(pattern_lines: [u8; 5], colors: [Option<TileColor>; 5]) -> usize {
    calculate_index(pattern_lines, colors, 0, 3, &UPPER_PATTERN_LINE_STATES)
}

pub fn calculate_lower_index(pattern_lines: [u8; 5], colors: [Option<TileColor>; 5]) -> usize {
    calculate_index(pattern_lines, colors, 3, 2, &LOWER_PATTERN_LINE_STATES)
}

#[derive(Clone)]
pub struct UpperPatternLinesEncoding {
    pub index: [usize; NUM_PLAYERS],
}

impl OneHotFeature for UpperPatternLinesEncoding {
    const SIZE: usize = UPPER_PATTERN_LINES_ENCODING_SIZE;
    const PLAYER_FEATURE: bool = true;
    const MAX_ONES: usize = 1;
    const START: usize = NonCenterFactoryEncoding::END;

    fn initialize(layer: &mut impl InputLayer) -> Self {
        let mut index = [0; NUM_PLAYERS];
        for player_index in 0..NUM_PLAYERS {
            layer.set_input(Self::START + player_index);
            index[player_index] = Self::START + player_index;
        }
        Self {
            index
        }
    }
}

impl UpperPatternLinesEncoding {
    pub fn set(
        &mut self,
        pattern_lines: [u8; 5],
        colors: [Option<TileColor>; 5],
        player_index: usize,
        layer: &mut impl InputLayer,
    ) {
        let index = calculate_upper_index(pattern_lines, colors) * NUM_PLAYERS + player_index + Self::START;
        if self.index[player_index] != index {
            layer.unset_input(self.index[player_index]);
            self.index[player_index] = index;
            layer.set_input(index);
        }
    }
}

#[derive(Clone)]
pub struct LowerPatternLinesEncoding {
    pub index: [usize; NUM_PLAYERS],
}

impl OneHotFeature for LowerPatternLinesEncoding {
    const SIZE: usize = LOWER_PATTERN_LINES_ENCODING_SIZE;
    const PLAYER_FEATURE: bool = true;
    const MAX_ONES: usize = 1;
    const START: usize = UpperPatternLinesEncoding::END;

    fn initialize(layer: &mut impl InputLayer) -> Self {
        let mut index = [0; NUM_PLAYERS];
        for player_index in 0..NUM_PLAYERS {
            layer.set_input(Self::START + player_index);
            index[player_index] = Self::START + player_index;
        }
        Self {
            index
        }
    }
}

impl LowerPatternLinesEncoding {
    pub fn set(
        &mut self,
        pattern_lines: [u8; 5],
        colors: [Option<TileColor>; 5],
        player_index: usize,
        layer: &mut impl InputLayer,
    ) {
        let index = calculate_lower_index(pattern_lines, colors) * NUM_PLAYERS + player_index + Self::START;
        if self.index[player_index] != index {
            layer.unset_input(self.index[player_index]);
            self.index[player_index] = index;
            layer.set_input(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_upper_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for tiles1 in 0..=1 {
            for color1 in 0..NUM_TILE_COLORS {
                for tiles2 in 0..=2 {
                    for color2 in 0..NUM_TILE_COLORS {
                        for tiles3 in 0..=3 {
                            for color3 in 0..NUM_TILE_COLORS {
                                let pattern_lines =
                                    [tiles1 as u8, tiles2 as u8, tiles3 as u8, 0, 0];
                                let colors = [
                                    if tiles1 == 0 {
                                        None
                                    } else {
                                        Some(TileColor::from(color1))
                                    },
                                    if tiles2 == 0 {
                                        None
                                    } else {
                                        Some(TileColor::from(color2))
                                    },
                                    if tiles3 == 0 {
                                        None
                                    } else {
                                        Some(TileColor::from(color3))
                                    },
                                    None,
                                    None,
                                ];
                                let index = calculate_upper_index(pattern_lines, colors);
                                indices_set.insert(index);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(indices_set.len(), 1056);
    }

    #[test]
    fn test_calculate_lower_index_all_combinations() {
        let start_time = std::time::Instant::now();
        let mut indices_set = std::collections::HashSet::new();
        for tiles4 in 0..=4 {
            for color4 in 0..NUM_TILE_COLORS {
                for tiles5 in 0..=5 {
                    for color5 in 0..NUM_TILE_COLORS {
                        let pattern_lines = [0, 0, 0, tiles4 as u8, tiles5 as u8];
                        let colors = [
                            None,
                            None,
                            None,
                            if tiles4 == 0 {
                                None
                            } else {
                                Some(TileColor::from(color4))
                            },
                            if tiles5 == 0 {
                                None
                            } else {
                                Some(TileColor::from(color5))
                            },
                        ];
                        let index = calculate_lower_index(pattern_lines, colors);
                        indices_set.insert(index);
                    }
                }
            }
        }
        assert_eq!(indices_set.len(), 546);
        println!("Time taken: {:?}", start_time.elapsed());
    }
}
