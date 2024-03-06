use crate::tile_color::NUM_TILE_COLORS;
/*
    A wall is a 5x5 grid of tiles. Each tile can be one of 5 colors.
    0  1  2  3  4  (5)
    6  7  8  9  10 (11)
    12 13 14 15 16 (17)
    18 19 20 21 22 (23)
    24 25 26 27 28 (29)
*/

// Bitboard of all possible tile locations
#[allow(clippy::unusual_byte_groupings)]
pub const VALID_WALL_TILES: u32 = 0b00_0_11111_0_11111_0_11111_0_11111_0_11111;

// Bitboards of the background color of the wall
#[allow(clippy::unusual_byte_groupings)]
pub const WALL_COLOR_MASKS: [u32; NUM_TILE_COLORS] = [
    0b00_0_10000_0_01000_0_00100_0_00010_0_00001, // BLUE
    0b00_0_00001_0_10000_0_01000_0_00100_0_00010, // Yellow
    0b00_0_00010_0_00001_0_10000_0_01000_0_00100, // RED
    0b00_0_00100_0_00010_0_00001_0_10000_0_01000, // Black
    0b00_0_01000_0_00100_0_00010_0_00001_0_10000, // White
];

pub const ROW_MASK: u32 = 0b11111;

#[inline]
pub fn field_at(row: usize, col: usize) -> u32 {
    1 << (row * 6 + col)
}

#[inline]
pub fn get_row_mask(row_index: usize) -> u32 {
    ROW_MASK << (row_index * 6)
}

// Given a occupancy bitboard and a position of a new tile, calculate the number of points that tile would score
#[inline]
pub fn get_placed_tile_score(occupancy: u32, new_tile_pos: u8) -> u32 {
    let col = count_column_neighbors(occupancy, new_tile_pos) - 1;
    let row = count_row_neighbors(occupancy, new_tile_pos) - 1;
    if col > 0 && row > 0 {
        col + row + 2 // We count the tile itself as a point in both directions
    } else {
        col + row + 1 // We count the tile itself as a point in one direction
    }
}

const ROW_NEIGHBORS_LOOKUP: [[u8; 32]; 5] = [
    [
        1, 1, 2, 2, 1, 1, 3, 3, 1, 1, 2, 2, 1, 1, 4, 4, 1, 1, 2, 2, 1, 1, 3, 3, 1, 1, 2, 2, 1, 1,
        5, 0,
    ],
    [
        1, 2, 1, 2, 2, 3, 2, 3, 1, 2, 1, 2, 3, 4, 3, 4, 1, 2, 1, 2, 2, 3, 2, 3, 1, 2, 1, 2, 4, 5,
        4, 0,
    ],
    [
        1, 1, 2, 3, 1, 1, 2, 3, 2, 2, 3, 4, 2, 2, 3, 4, 1, 1, 2, 3, 1, 1, 2, 3, 3, 3, 4, 5, 3, 3,
        4, 0,
    ],
    [
        1, 1, 1, 1, 2, 2, 3, 4, 1, 1, 1, 1, 2, 2, 3, 4, 2, 2, 2, 2, 3, 3, 4, 5, 2, 2, 2, 2, 3, 3,
        4, 0,
    ],
    [
        1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 4, 5, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3,
        4, 0,
    ],
];

#[inline]
fn count_row_neighbors(occupancy: u32, new_tile_pos: u8) -> u32 {
    let row_index: u8 = new_tile_pos / 6;
    let new_tile_pos = new_tile_pos % 6;
    let lookup_key: u32 = occupancy >> (row_index * 6) & ROW_MASK;
    ROW_NEIGHBORS_LOOKUP[new_tile_pos as usize][lookup_key as usize] as u32
}

#[inline]
fn count_column_neighbors(mut occupancy: u32, new_tile_pos: u8) -> u32 {
    // Create a bitboard with the new tile on it
    let new_tile: u32 = 1 << new_tile_pos;
    // Add the new tile to the occupancy
    occupancy |= new_tile;
    // Create empty bitboard to store neighbors and tile (the tile also counts as a point)
    let mut neighbors = 0b0;
    // For each bit in the column, add it to the neighbors bitboard
    let mut bit = new_tile;
    while bit & occupancy > 0 {
        neighbors |= bit;
        bit <<= 6;
    }
    // The same for the row
    bit = new_tile;
    while bit & occupancy > 0 {
        neighbors |= bit;
        bit >>= 6;
    }
    // Return the number of neighbors (including the tile itself)
    neighbors.count_ones()
}

#[inline]
pub fn check_complete_row_exists(mut occupancy: u32) -> bool {
    occupancy &= occupancy >> 1;
    occupancy &= occupancy >> 2;
    occupancy &= occupancy >> 1;
    occupancy > 0
}

#[inline]
pub fn count_complete_rows(mut occupancy: u32) -> u32 {
    occupancy &= occupancy >> 1;
    occupancy &= occupancy >> 2;
    occupancy &= occupancy >> 1;
    occupancy.count_ones()
}

#[inline]
pub fn count_complete_columns(mut occupancy: u32) -> u32 {
    occupancy &= occupancy >> 6;
    occupancy &= occupancy >> 12;
    occupancy &= occupancy >> 6;
    occupancy.count_ones()
}

pub fn count_full_colors(occupancy: u32) -> u32 {
    let mut num_full_colors = 0;
    for color_mask in WALL_COLOR_MASKS.iter() {
        if occupancy & color_mask == *color_mask {
            num_full_colors += 1;
        }
    }
    num_full_colors
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;

    fn count_row_neighbors_check(mut occupancy: u32, new_tile_pos: u8) -> u32 {
        // Create a bitboard with the new tile on it
        let new_tile: u32 = 1 << new_tile_pos;
        // Add the new tile to the occupancy
        occupancy |= new_tile;
        // Create empty bitboard to store neighbors and tile (the tile also counts as a point)
        let mut neighbors = 0b0;
        // For each bit in the row, add it to the neighbors bitboard
        let mut bit = new_tile;
        while bit & occupancy > 0 {
            neighbors |= bit;
            bit <<= 1;
        }
        // The same for the column
        bit = new_tile;
        while bit & occupancy > 0 {
            neighbors |= bit;
            bit >>= 1;
        }
        // Return the number of neighbors (including the tile itself)
        neighbors.count_ones()
    }

    fn display_bitboard(bitboard: u32) {
        for row in 0..5 {
            for col in 0..5 {
                // Calculate the position of the bit to check
                let bit_position = row * 6 + (4 - col);
                // Check if the bit is set in the bitboard
                let is_set = bitboard & (1 << bit_position) != 0;
                // Print an 'X' if the bit is set; otherwise, print a '.'
                print!("{}", if is_set { '1' } else { '.' });
            }
            // Newline after each row
            println!();
        }
    }

    #[test]
    fn test_count_row_neighbors_quick() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let occupancy: u32 = rng.gen();
            let occupancy = occupancy & VALID_WALL_TILES;

            let mut new_tile_pos;
            loop {
                new_tile_pos = rng.gen_range(0..30);
                if 1 << new_tile_pos & occupancy == 0 && 1 << new_tile_pos & VALID_WALL_TILES > 0 {
                    break;
                }
            }
            println!("Occupancy:");
            display_bitboard(occupancy);
            println!("New tile:");
            display_bitboard(1 << new_tile_pos);
            let expected = count_row_neighbors_check(occupancy, new_tile_pos);
            let actual = count_row_neighbors(occupancy, new_tile_pos);
            println!("Expected: {}", expected);
            println!("Actual: {}", actual);
            assert_eq!(expected, actual);
        }
    }
}
