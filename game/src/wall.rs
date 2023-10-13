use crate::NUM_TILE_COLORS;

// A wall is a 5x5 grid of tiles. Each tile can be one of 5 colors.
/*
0  1  2  3  4  (5)
6  7  8  9  10 (11)
12 13 14 15 16 (17)
18 19 20 21 22 (23)
24 25 26 27 28 (29)
*/

// Bitboard of all possible tile locations
pub const VALID_WALL_TILES: u32 = 0b00_0_11111_0_11111_0_11111_0_11111_0_11111;
// Bitboards of the background color of the wall
pub const WALL_COLOR: [u32; NUM_TILE_COLORS] = [
    0b00_0_10000_0_01000_0_00100_0_00010_0_00001, // BLUE
    0b00_0_00001_0_10000_0_01000_0_00100_0_00010, // Yellow
    0b00_0_00010_0_00001_0_10000_0_01000_0_00100, // RED
    0b00_0_00100_0_00010_0_00001_0_10000_0_01000, // Black
    0b00_0_01000_0_00100_0_00010_0_00001_0_10000, // White
];

// Given a occupancy bitboard and a position of a new tile, calculate the number of points that tile would score
pub fn get_placed_tile_score(occupancy: u32, new_tile_pos: u8) -> u32 {
    count_column_neighbors(occupancy, new_tile_pos) + count_row_neighbors(occupancy, new_tile_pos)
}

fn count_row_neighbors(mut occupancy: u32, new_tile_pos: u8) -> u32 {
    let new_tile: u32 = 1 << new_tile_pos; // Create a bitboard with the new tile on it
    occupancy |= new_tile; // Add the new tile to the occupancy
    let mut neighbors = 0b0; // Create empty bitboard to store neighbors and tile (the tile also counts as a point)
                             // For each bit in the row, add it to the neighbors bitboard
    let mut bit = new_tile;
    while bit & occupancy > 0 {
        neighbors |= bit;
        bit <<= 1;
    }
    bit = new_tile;
    while bit & occupancy > 0 {
        neighbors |= bit;
        bit >>= 1;
    }
    // Return the number of neighbors (including the tile itself)
    neighbors.count_ones()
}

fn count_column_neighbors(mut occupancy: u32, new_tile_pos: u8) -> u32 {
    let new_tile: u32 = 1 << new_tile_pos; // Create a bitboard with the new tile on it
    occupancy |= new_tile; // Add the new tile to the occupancy
    let mut neighbors = 0b0; // Create empty bitboard to store neighbors and tile (the tile also counts as a point)
                             // For each bit in the column, add it to the neighbors bitboard
    let mut bit = new_tile;
    while bit & occupancy > 0 {
        neighbors |= bit;
        bit <<= 6;
    }
    bit = new_tile;
    while bit & occupancy > 0 {
        neighbors |= bit;
        bit >>= 6;
    }
    // Return the number of neighbors (including the tile itself)
    neighbors.count_ones()
}

pub fn print_32_bit_bitboard(bitboard: u32) {
    let mut string = String::new();
    for y in 0..5 {
        for x in 0..6 {
            let bit: u32 = 1 << (y * 6 + x);
            let is_one = bit & bitboard > 0;

            // Color the last column
            if x == 5 {
                string.push_str("\u{001b}[31m");
            }

            // Print 1 or 0
            if is_one {
                string.push_str("1 ");
            } else {
                string.push_str("0 ");
            }

            // Reset the color
            if x == 5 {
                string.push_str("\u{001b}[0m");
            }
        }
        string.push_str("\n");
    }
    println!("{}", string);
}

#[inline(always)]
pub fn check_complete_row_exists(mut occupancy: u32) -> bool {
    occupancy &= occupancy >> 1;
    occupancy &= occupancy >> 2;
    occupancy &= occupancy >> 1;
    occupancy > 0
}

#[inline(always)]
pub fn count_complete_rows(mut occupancy: u32) -> u32 {
    occupancy &= occupancy >> 1;
    occupancy &= occupancy >> 2;
    occupancy &= occupancy >> 1;
    occupancy.count_ones()
}

#[inline(always)]
pub fn count_complete_columns(mut occupancy: u32) -> u32 {
    occupancy &= occupancy >> 6;
    occupancy &= occupancy >> 12;
    occupancy &= occupancy >> 6;
    occupancy.count_ones()
}
