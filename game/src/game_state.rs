use super::*;

pub const NUM_PLAYERS: usize = 3;
const PLAYERS_TO_FACTORIES: [usize; 3] = [6, 8, 10];
pub const NUM_FACTORIES: usize = PLAYERS_TO_FACTORIES[NUM_PLAYERS - 2];
const LEFTOVER_PENALTY: [u8; 7] = [1, 2, 4, 6, 8, 11, 14];

pub struct GameState {
    bag: [u8; NUM_TILE_COLORS], // For each color, how many tiles are left in the bag
    factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES], // For each factory, how many tiles of each color are in it (including the center)
    scores: [u16; NUM_PLAYERS], // For each player, how many points they have
    floor_line_progress: [u8; NUM_PLAYERS], // For each player, how many tiles they have in their penalty
    walls: [[u32; NUM_TILE_COLORS]; NUM_PLAYERS], // For each player, and each color, the locations of the tiles on their wall
    wall_occupancy: [u32; NUM_PLAYERS], // For each player, the occupancy of their wall
}

impl GameState {
    //pub fn do_move()
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
        }
    }
}

pub struct Bitboard(u32);


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

const VALID_FIELDS: u32 = 0b00_0_11111_0_11111_0_11111_0_11111_0_11111;
const WALL_COLOR: [u32; NUM_TILE_COLORS] = [
    0b00_0_10000_0_01000_0_00100_0_00010_0_00001, // BLUE
    0b00_0_00001_0_10000_0_01000_0_00100_0_00010, // Yellow
    0b00_0_00010_0_00001_0_10000_0_01000_0_00100, // RED
    0b00_0_00100_0_00010_0_00001_0_10000_0_01000, // Black
    0b00_0_01000_0_00100_0_00010_0_00001_0_10000, // White
];
