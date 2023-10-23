use crate::factories::CENTER_FACTORY_INDEX;

use super::*;
use std::fmt::Write;

pub const NUM_PLAYERS: usize = 2;
const LEFTOVER_PENALTY: [u8; 7] = [1, 2, 4, 6, 8, 11, 14];

pub struct GameState {
    pub bag: [u8; NUM_TILE_COLORS], // For each color, how many tiles are left in the bag
    pub factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES], // For each factory, how many tiles of each color are in it (including the center)

    pub scores: [u16; NUM_PLAYERS], // For each player, how many points they have
    pub floor_line_progress: [u8; NUM_PLAYERS], // For each player, how many tiles they have in their penalty

    pub walls: [[u32; NUM_TILE_COLORS]; NUM_PLAYERS], // For each player, and each color, the locations of the tiles on their wall
    pub wall_occupancy: [u32; NUM_PLAYERS], // For each player, the occupancy of their wall

    pub pattern_lines_occupancy: [[u8; 5]; NUM_PLAYERS], // For each player, the occupancy of their pattern lines
    pub pattern_lines_colors: [[Option<TileColor>; 5]; NUM_PLAYERS], // For each player, the color of their pattern lines. If the pattern line is empty, the color is 255

    pub current_player: Player,
}

impl GameState {
    pub fn do_move(&mut self, mov: Move) {
        // Step 1. Take the tiles from the factory or center

        let take_from_factory_index = mov.take_from_factory_index as usize;
        let color = mov.color as usize;
        let factory_content: [u8; 5] = self.factories[take_from_factory_index];
        //let took = factory_content[color]; // Get the number of tiles of the color that were taken
        //let took = mov.number_of_tiles_taken;
        //debug_assert!(took == mov.pattern.count_ones() as u8);
        // Now we have the number of tiles that we took from the factory. We need to put the rest into the center
        for color_index in 0..NUM_TILE_COLORS {
            if color_index == color {
                continue;
            }
            if take_from_factory_index != CENTER_FACTORY_INDEX {
                self.factories[CENTER_FACTORY_INDEX][color_index] += factory_content[color_index];
            }
        }
        // Empty the factory
        if take_from_factory_index != CENTER_FACTORY_INDEX {
            self.factories[take_from_factory_index] = [0; NUM_TILE_COLORS];
        } else {
            // only empty the color we took from the center
            self.factories[take_from_factory_index][color] = 0;
        }

        /*debug_assert!(
            took as u32 == mov.pattern.count_ones(),
            "Took {} tiles but pattern has {} tiles",
            took,
            mov.pattern.count_ones()
        );*/

        // Step 2. Place the tiles in the pattern lines. For that we need to know which player is playing
        let current_player_index: usize = self.current_player.into();

        // Place the tiles in the pattern lines
        // debug_assert!(
        //     self.pattern_lines_occupancy[current_player_index] & mov.pattern == 0,
        //     "Pattern lines already (partially) occupied"
        // );
        // for (pattern_index, pattern_mask) in PATTERN_MASKS.iter().enumerate() {
        //     if !pattern_mask & mov.pattern > 0 {
        //         continue;
        //     }
        //     debug_assert!(self.pattern_lines_colors[current_player_index][pattern_index].is_none(),);
        //     self.pattern_lines_colors[current_player_index][pattern_index] = Some(mov.color);
        // }
        //self.pattern_lines_occupancy[current_player_index] |= mov.pattern;
        for i in 0..5 {
            if mov.pattern[i] > 0 {
                debug_assert!({
                    if let Some(pattern_line_color) =
                        self.pattern_lines_colors[current_player_index][i]
                    {
                        pattern_line_color == mov.color
                    } else {
                        true
                    }
                });
                self.pattern_lines_colors[current_player_index][i] = Some(mov.color);
            }
            self.pattern_lines_occupancy[current_player_index][i] += mov.pattern[i];
        }

        // Advance the player
        self.current_player = self.current_player.next();
    }

    // pub fn get_possible_moves(&self) -> Vec<Move> {
    //     let mut moves: Vec<Move> = Vec::with_capacity(100);
    //     let current_player: usize = self.current_player.into();
    //     // Iterate over every factory
    //     for (factory_index, factory) in self.factories.iter().enumerate() {
    //         // Itterate over every color in the factory
    //         for (color, number) in factory.iter().enumerate() {
    //             if *number == 0 {
    //                 // If we can't take any tiles of this color, skip it
    //                 continue;
    //             }
    //             for pattern_line_index in 0..5 {
    //                 // Get the current color of the pattern line
    //                 let pattern_line_color =
    //                     self.pattern_lines_colors[current_player][pattern_line_index];

    //                 // If the pattern line has a asigned color and it's not the same as the color we are taking, skip it, we can't place the tiles there
    //                 if let Some(pattern_line_color) = pattern_line_color {
    //                     if pattern_line_color != TileColor::from(color as u8) {
    //                         continue;
    //                     }
    //                 }

    //                 // If the pattern is full, skip it
    //                 if self.pattern_lines_occupancy[current_player][pattern_line_index]
    //                     == (pattern_line_index as u8) + 1
    //                 {
    //                     continue;
    //                 }

    //                 // Set the pattern line to the color we are taking. TODO: All combinations should be appended to the list and the sum fo the pattern array must be equal to number
    //                 let mut pattern: [u8; 5] = [0; 5];
    //                 pattern[pattern_line_index] = 1;
    //                 moves.push(Move {
    //                     take_from_factory_index: factory_index as u8,
    //                     color: TileColor::from(color as u8),
    //                     number_of_tiles_taken: *number,
    //                     pattern,
    //                 });
    //             }
    //         }
    //     }

    //     moves
    // }

    pub fn get_possible_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(100);
        let current_player: usize = self.current_player.into();

        for (factory_index, factory) in self.factories.iter().enumerate() {
            for (color, number) in factory.iter().enumerate() {
                if *number == 0 {
                    continue;
                }

                let mut possible_patterns = Vec::new();
                self.find_tile_combinations(
                    *number as usize,
                    color as u8,
                    current_player,
                    &mut [0u8; 6], // 5 for pattern lines 6th for discard
                    0,
                    &mut possible_patterns,
                );

                for pattern in possible_patterns {
                    moves.push(Move {
                        take_from_factory_index: factory_index as u8,
                        color: TileColor::from(color as u8),
                        pattern,
                    });
                }
            }
        }

        moves
    }

    fn find_tile_combinations(
        &self,
        tiles_left: usize,
        color: u8,
        current_player: usize,
        current_pattern: &mut [u8; 6],
        pattern_line_start: usize,
        results: &mut Vec<[u8; 6]>,
    ) {
        if tiles_left == 0 {
            results.push(*current_pattern);
            return;
        }

        // Handle discarding tiles
        if pattern_line_start == 5 {
            current_pattern[5] += tiles_left as u8;
            results.push(*current_pattern);
            current_pattern[5] -= tiles_left as u8;
            return;
        }

        for pattern_line_index in pattern_line_start..5 {
            let pattern_line_color = self.pattern_lines_colors[current_player][pattern_line_index];

            if let Some(pattern_line_color) = pattern_line_color {
                if pattern_line_color != TileColor::from(color) {
                    continue;
                }
            }

            let max_for_pattern = pattern_line_index + 1;
            let tiles_already_in_pattern =
                self.pattern_lines_occupancy[current_player][pattern_line_index] as usize;
            let space_in_pattern = max_for_pattern - tiles_already_in_pattern;

            if space_in_pattern > 0 {
                let tiles_to_place = space_in_pattern.min(tiles_left);
                current_pattern[pattern_line_index] += tiles_to_place as u8;
                self.find_tile_combinations(
                    tiles_left - tiles_to_place,
                    color,
                    current_player,
                    current_pattern,
                    pattern_line_index + 1,
                    results,
                );
                current_pattern[pattern_line_index] -= tiles_to_place as u8;
            }
        }

        // If you reach here, and there are still tiles left, discard them
        if tiles_left > 0 {
            current_pattern[5] += tiles_left as u8;
            results.push(*current_pattern);
            current_pattern[5] -= tiles_left as u8;
        }
    }

    pub fn fill_factories(&mut self) {
        factories::fill_factories(&mut self.factories, &mut self.bag);
    }

    pub fn check_integrity(&self) {
        let mut is_valid = true;

        // Make sure the bag has less or equal than 20 tiles of each color
        for color in self.bag.iter() {
            if *color > 20 {
                is_valid = false;
                println!("Bag has more than 20 tiles of a color");
            }
        }

        // Make sure the factories have 0 or 4 tiles (Except the center)
        for factory in self.factories.iter().take(CENTER_FACTORY_INDEX) {
            let mut tile_count = 0;
            for color in factory.iter() {
                tile_count += color;
            }
            if tile_count != 0 && tile_count != 4 {
                is_valid = false;
                println!("Factory has {} tiles", tile_count);
            }
        }

        // Make sure there are no tiles of different colors on the same position on the wall
        for player in 0..NUM_PLAYERS {
            let mut occupancy: u32 = 0b0;
            for color in 0..NUM_TILE_COLORS {
                let bitboard = self.walls[player][color];
                if bitboard & occupancy > 0 {
                    is_valid = false;
                    println!(
                        "Player {} has tiles of different colors on the same position on the wall",
                        player
                    );
                }
                if self.wall_occupancy[player] & bitboard != bitboard {
                    is_valid = false;
                    println!(
                        "Player {} has tiles on the wall that are not in the occupancy",
                        player
                    );
                }
                occupancy |= bitboard;
            }
        }
        if !is_valid {
            println!("{}", self);
            panic!("Game state is invalid");
        }
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
            pattern_lines_occupancy: [[0; 5]; NUM_PLAYERS],
            pattern_lines_colors: [[None; 5]; NUM_PLAYERS],
        }
    }
}

fn bag_to_string(game_state: &GameState) -> String {
    let mut string = String::from("BAG       ");
    for (color, number_of_tiles_left) in game_state.bag.iter().enumerate() {
        write!(
            string,
            "{}: {} ",
            TileColor::from(color),
            number_of_tiles_left
        )
        .unwrap();
    }
    string.push('\n');
    string
}

fn factories_to_string(game_state: &GameState) -> String {
    let mut string = String::from("FACTORIES ");
    for (factory_index, factory) in game_state.factories.iter().enumerate() {
        if factory_index == CENTER_FACTORY_INDEX {
            string.push_str("Center: ");
        } else {
            write!(string, "{}: ", factory_index + 1).unwrap();
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
    string
}

fn player_wall_to_string(game_state: &GameState, player_index: usize) -> String {
    let mut string = String::new();

    //writeln!(string, "PLAYER {}  ", player_index).unwrap();

    for y in 0..5 {
        for x in 0..5 {
            let bit: u32 = 1 << (y * 6 + x);
            let mut found = false;

            for color in 0..NUM_TILE_COLORS {
                if game_state.walls[player_index][color] & bit > 0 {
                    string.push_str(&TileColor::from(color).to_string());
                    found = true;
                    break;
                }
            }

            if !found && game_state.wall_occupancy[player_index] & bit == 0 {
                for (color, wall_color) in WALL_COLOR.iter().enumerate() {
                    if wall_color & bit > 0 {
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

    string
}

fn player_pattern_board_to_string(game_state: &GameState, player_index: usize) -> String {
    let mut string = String::new();

    let pattern_line_occupancy = game_state.pattern_lines_occupancy[player_index];
    let pattern_colors = game_state.pattern_lines_colors[player_index];
    for (pattern_index, pattern_mask) in PATTERN_MASKS.iter().enumerate() {
        let pattern_color = pattern_colors[pattern_index];
        let color = if let Some(pattern_color) = pattern_color {
            pattern_color.get_color_string()
        } else {
            ("".to_string(), "".to_string())
        };
        write!(string, "{}{} ", color.0, pattern_index + 1).unwrap();
        let leading_whitespace = (4 - pattern_index) * 2;
        for _ in 0..leading_whitespace {
            string.push(' ');
        }
        //let pattern_line = pattern_line_occupancy & pattern_mask;
        //let count = pattern_line.count_ones() as usize;
        let count = pattern_line_occupancy[pattern_index] as usize;
        let missing = pattern_index + 1 - count;
        for _ in 0..missing {
            string.push_str(". ");
        }
        for _ in 0..count {
            if let Some(c) = pattern_color {
                string.push_str(&format!("{} ", c));
            } else {
                string.push_str("X ");
            }
            //string.push(' ')
        }
        string.push_str(&color.1);
        string.push('\n');
    }

    string
}

fn merge_pattern_and_wall(pattern: &str, wall: &str) -> String {
    let delimiter = " ->  "; // 3 spaces delimiter, adjust as needed
    let mut result = String::new();

    let mut pattern_lines = pattern.lines();
    let mut wall_lines = wall.lines();

    loop {
        let pattern_line = pattern_lines.next();
        let wall_line = wall_lines.next();

        if pattern_line.is_none() && wall_line.is_none() {
            break;
        }

        if let Some(pl) = pattern_line {
            result.push_str(pl);
        }

        result.push_str(delimiter);

        if let Some(wl) = wall_line {
            result.push_str(wl);
        }

        result.push('\n');
    }

    result
}

// Using the merge function in your fmt or wherever you are aggregating the

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::with_capacity(1024); // Assuming enough capacity
        string.push_str(&bag_to_string(self));
        string.push_str(&factories_to_string(self));
        for player_index in 0..NUM_PLAYERS {
            let score = self.scores[player_index];
            string.push_str(&format!("PLAYER {} (score: {})\n", player_index, score));
            let pattern_string = player_pattern_board_to_string(self, player_index);
            let wall_string = player_wall_to_string(self, player_index);
            string.push_str(&merge_pattern_and_wall(&pattern_string, &wall_string));

            string.push_str(
                format!("Floor line: {}/14", self.floor_line_progress[player_index]).as_str(),
            );

            string.push('\n');
            /*for _ in 0..self.floor_line_progress[player_index] {
                string.push_str("X");
            }*/
        }

        write!(f, "{}", string)
    }
}
