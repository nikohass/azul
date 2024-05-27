use super::{factory_encoding::FACTORY_ENCODING_SIZE, layers::InputLayer};
use game::{TileColor, NUM_PLAYERS, NUM_TILE_COLORS};

pub const PLAYER_PATTERN_LINE_ENCODING_SIZE: usize = 81;
const PATTERN_LINE_OFFSETS: [usize; 6] = [0, 1, 7, 18, 34, 55];
pub const PATTENR_LINE_ENCODING_SIZE: usize = PLAYER_PATTERN_LINE_ENCODING_SIZE * NUM_PLAYERS;

pub fn get_pattern_line_index(
    pattern_line_index: usize,
    num_tiles: usize,
    color: Option<TileColor>,
    player_index: usize,
) -> usize {
    let index = if num_tiles == 0 {
        PATTERN_LINE_OFFSETS[pattern_line_index]
    } else {
        PATTERN_LINE_OFFSETS[pattern_line_index]
            + 1
            + (num_tiles - 1) * NUM_TILE_COLORS
            + color.map(usize::from).unwrap()
    };

    index + FACTORY_ENCODING_SIZE + player_index * PLAYER_PATTERN_LINE_ENCODING_SIZE
}

pub fn add_pattern_line_encoding(
    pattern_line_index: usize,
    num_tiles: usize,
    color: Option<TileColor>,
    player_index: usize,
    layer: &mut dyn InputLayer,
) {
    let index = get_pattern_line_index(pattern_line_index, num_tiles, color, player_index);
    layer.set_input(index);
}

pub fn remove_pattern_line_encoding(
    pattern_line_index: usize,
    num_tiles: usize,
    color: Option<TileColor>,
    player_index: usize,
    layer: &mut dyn InputLayer,
) {
    let index = get_pattern_line_index(pattern_line_index, num_tiles, color, player_index);
    layer.unset_input(index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_line() {
        let mut encoding = [0; PLAYER_PATTERN_LINE_ENCODING_SIZE];

        for pattern_line_index in 0..6 {
            for num_tiles in 0..pattern_line_index + 1 {
                if num_tiles > 0 {
                    for color in 0..NUM_TILE_COLORS {
                        let result = get_pattern_line_index(
                            pattern_line_index,
                            num_tiles,
                            Some(TileColor::from(color)),
                            0,
                        ) - FACTORY_ENCODING_SIZE;
                        encoding[result] += 1;
                    }
                } else {
                    let result = get_pattern_line_index(pattern_line_index, 0, None, 0)
                        - FACTORY_ENCODING_SIZE;
                    encoding[result] += 1;
                }
            }
        }

        println!("encoding: {:?}", encoding);
        assert_eq!(encoding, [1; PLAYER_PATTERN_LINE_ENCODING_SIZE],);
    }
}
