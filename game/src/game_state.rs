use super::*;
use std::fmt::Write;

pub const NUM_PLAYERS: usize = 3;
const LEFTOVER_PENALTY: [u8; 7] = [1, 2, 4, 6, 8, 11, 14];

pub struct GameState {
    pub bag: [u8; NUM_TILE_COLORS], // For each color, how many tiles are left in the bag
    pub factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES], // For each factory, how many tiles of each color are in it (including the center)

    pub scores: [u16; NUM_PLAYERS], // For each player, how many points they have
    pub floor_line_progress: [u8; NUM_PLAYERS], // For each player, how many tiles they have in their penalty

    pub walls: [[u32; NUM_TILE_COLORS]; NUM_PLAYERS], // For each player, and each color, the locations of the tiles on their wall
    pub wall_occupancy: [u32; NUM_PLAYERS], // For each player, the occupancy of their wall

    pub pattern_lines_occupancy: [u16; NUM_PLAYERS], // For each player, the occupancy of their pattern lines
    pub pattern_lines_colors: [u8; NUM_PLAYERS], // For each player, the color of their pattern lines. If the pattern line is empty, the color is 255

    pub current_player: Player,
}

impl GameState {
    pub fn do_move(move_: Move) {}

    pub fn fill_factories(&mut self) {
        factories::fill_factories(&mut self.factories, &mut self.bag);
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

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::with_capacity(1024); // Assuming enough capacity

        string.push_str("BAG       ");
        for (color, number_of_tiles_left) in self.bag.iter().enumerate() {
            use std::fmt::Write as _; // Importing the trait for write!
            write!(
                string,
                "{}: {} ",
                TileColor::from(color),
                number_of_tiles_left
            )
            .unwrap();
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
                string.push_str(
                    &TileColor::from(color)
                        .to_string()
                        .repeat(*number_of_tiles as usize),
                );
            }

            if factory_index != NUM_FACTORIES - 1 {
                string.push_str(&".".repeat(4 - tile_count));
            }
            string.push(' ');
        }
        string.push('\n');

        string.push_str("SCORES    ");
        string.push_str(
            &self.scores[..NUM_PLAYERS]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("/"),
        );
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
