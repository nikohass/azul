use super::*;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use std::fmt::Write;

pub const NUM_PLAYERS: usize = 3;
const PLAYERS_TO_FACTORIES: [usize; 3] = [6, 8, 10];
pub const NUM_FACTORIES: usize = PLAYERS_TO_FACTORIES[NUM_PLAYERS - 2];
const LEFTOVER_PENALTY: [u8; 7] = [1, 2, 4, 6, 8, 11, 14];

pub const PATTERN_MASKS: [u16; 5] = [
    0b1,
    0b11_0,
    0b111_00_0,
    0b1111_000_00_0,
    0b11111_0000_000_00_0,
];

fn is_pattern_empty(pattern_bitboard: u16, pattern_index: usize) -> bool {
    pattern_bitboard & PATTERN_MASKS[pattern_index] == 0
}

fn is_pattern_full(pattern_bitboard: u16, pattern_index: usize) -> bool {
    pattern_bitboard & PATTERN_MASKS[pattern_index] == PATTERN_MASKS[pattern_index]
}

pub struct GameState {
    pub bag: [u8; NUM_TILE_COLORS], // For each color, how many tiles are left in the bag
    pub factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES], // For each factory, how many tiles of each color are in it (including the center)
    pub scores: [u16; NUM_PLAYERS], // For each player, how many points they have
    pub floor_line_progress: [u8; NUM_PLAYERS], // For each player, how many tiles they have in their penalty
    pub walls: [[u32; NUM_TILE_COLORS]; NUM_PLAYERS], // For each player, and each color, the locations of the tiles on their wall
    pub wall_occupancy: [u32; NUM_PLAYERS], // For each player, the occupancy of their wall
    // pattern lines
    pub current_player: Player,
    pub pattern_lines_occupancy: [u16; NUM_PLAYERS], // For each player, the occupancy of their pattern lines
    pub pattern_lines_colors: [u8; NUM_PLAYERS], // For each player, the color of their pattern lines. If the pattern line is empty, the color is 255
}

impl GameState {
    pub fn do_move(move_: Move) {
        match move_ {
            Move::TakeFactory(factory_index, tile_color) => {

            }
            _ => {}
        }
    }

    pub fn fill_factories(&mut self) {
        let mut rng = SmallRng::from_entropy();
        // Make sure the bag is empty
        //assert!(self.bag.iter().sum::<u8>() == 0);
        // Make sure the center is empty
        for color in self.factories.last_mut().unwrap().iter_mut() {
            assert!(*color == 0);
        }
        // Fill the bag
        self.bag = [20, 20, 20, 20, 20];
    
        for factory in self.factories.iter_mut().take(NUM_FACTORIES - 1) {
            for color in factory.iter_mut() {
                // Make sure the factory is empty
                assert!(*color == 0);
            }
            // Fill the factory
            let mut tiles_left = 4;
            while tiles_left > 0 {
                let tile_color = rng.gen_range(0..NUM_TILE_COLORS);
                if self.bag[tile_color] > 0 {
                    self.bag[tile_color] -= 1;
                    factory[tile_color] += 1;
                    tiles_left -= 1;
                }
            }
        }
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::with_capacity(1024);  // Assuming enough capacity

        string.push_str("BAG       ");
        for (color, number_of_tiles_left) in self.bag.iter().enumerate() {
            use std::fmt::Write as _; // Importing the trait for write!
            write!(string, "{}: {} ", TileColor::from(color), number_of_tiles_left).unwrap();
        }
        string.push('\n');

        string.push_str("FACTORIES ");
        for (factory_index, factory) in self.factories.iter().enumerate() {
            if factory_index == NUM_FACTORIES - 1 {
                string.push_str("Center: ");
            } else {
                write!(string, "{}: ", factory_index).unwrap();
            }

            let tile_count: usize = factory.iter().sum::<u8>() as usize;
            for (color, number_of_tiles) in factory.iter().enumerate() {
                string.push_str(&TileColor::from(color).to_string().repeat(*number_of_tiles as usize));
            }
            
            if factory_index != NUM_FACTORIES - 1 {
                string.push_str(&".".repeat(4 - tile_count));
            }
            string.push(' ');
        }
        string.push('\n');

        string.push_str("SCORES    ");
        string.push_str(&self.scores[..NUM_PLAYERS].iter().map(ToString::to_string).collect::<Vec<String>>().join("/"));
        string.push('\n');

        for i in 0..NUM_PLAYERS {
            write!(string, "PLAYER {}  \n", i).unwrap();
            for y in 0..5 {
                for x in 0..5 {
                    let bit: u32 = 1 << (y * 6 + x);
                    let mut found = false;
                    for color in 0..NUM_TILE_COLORS {
                        if self.walls[i][color] & bit > 0 {
                            string.push_str(&TileColor::from(color).to_string());
                            found = true;
                            break;
                        }
                    }

                    if !found && self.wall_occupancy[i] & bit == 0 {
                        for color in 0..NUM_TILE_COLORS {
                            if WALL_COLOR[color] & bit > 0 {
                                let (start, end) = TileColor::from(color).get_color_string();
                                write!(string, "{}.{}", start, end).unwrap();
                                break;
                            }
                        }
                    }
                    string.push(' ');
                }
                string.push('\n');
            }
            string.push_str("PATTERN   ");
            // TODO:
            string.push('\n');
            string.push_str("FLOOR     ");
            for _ in 0..self.floor_line_progress[i] {
                string.push_str("X");
            }
            string.push('\n');
        }

        write!(f, "{}", string)
    }
}


impl Default for GameState {
    fn default() -> Self {
        Self {
            bag: [20, 20, 20, 20, 20],
            factories: [[0; NUM_TILE_COLORS]; NUM_FACTORIES],
            scores: [0; NUM_PLAYERS],
            floor_line_progress: [0; NUM_PLAYERS],
            walls: [[0; NUM_TILE_COLORS]; NUM_PLAYERS],
            wall_occupancy: [0; NUM_PLAYERS],
            current_player: Player::new(0),
            pattern_lines_occupancy: [0; NUM_PLAYERS],
            pattern_lines_colors: [255; NUM_PLAYERS],
        }
    }
}


const VALID_FIELDS: u32 = 0b00_0_11111_0_11111_0_11111_0_11111_0_11111;
const WALL_COLOR: [u32; NUM_TILE_COLORS] = [
    0b00_0_10000_0_01000_0_00100_0_00010_0_00001, // BLUE
    0b00_0_00001_0_10000_0_01000_0_00100_0_00010, // Yellow
    0b00_0_00010_0_00001_0_10000_0_01000_0_00100, // RED
    0b00_0_00100_0_00010_0_00001_0_10000_0_01000, // Black
    0b00_0_01000_0_00100_0_00010_0_00001_0_10000, // White
];


fn get_placed_tile_score(occupancy: u32, new_tile_pos: u8) -> u32 {
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

/*
0  1  2  3  4  (5)
6  7  8  9  10 (11)
12 13 14 15 16 (17)
18 19 20 21 22 (23)
24 25 26 27 28 (29)
*/

pub fn print_bitboard(bitboard: u32) {
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
