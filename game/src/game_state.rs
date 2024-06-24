use crate::factories::{Factories, CENTER_FACTORY_INDEX, NUM_FACTORIES};
use crate::move_::Move;
use crate::move_list::MoveList;
use crate::player::PlayerMarker;
use crate::tile_color::{TileColor, NUM_TILE_COLORS};
use crate::wall::{self, WALL_COLOR_MASKS};
use crate::{GameError, NUM_PLAYERS};
use rand::rngs::SmallRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const FLOOR_LINE_PENALTY: [u8; 8] = [0, 1, 2, 4, 6, 8, 11, 14];

pub type Bag = [u8; NUM_TILE_COLORS];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveGenerationResult {
    GameOver,
    RoundOver,
    Continue,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub bag: Bag, // For each color, how many tiles are left in the bag
    pub out_of_bag: Bag,
    pub factories: Factories, // For each factory, how many tiles of each color are in it (including the center)

    pub scores: [i16; NUM_PLAYERS], // For each player, how many points they have
    pub floor_line_progress: [u8; NUM_PLAYERS], // For each player, how many tiles they have in their penalty

    pub walls: [u32; NUM_PLAYERS], // For each player, the occupancy of their wall

    pub pattern_lines_occupancy: [[u8; 5]; NUM_PLAYERS], // For each player, the occupancy of their pattern lines
    pub pattern_lines_colors: [[Option<TileColor>; 5]; NUM_PLAYERS], // For each player, the color of their pattern lines. If the pattern line is empty, the color is 255

    pub current_player: PlayerMarker,
    pub next_round_starting_player: PlayerMarker,
    pub tile_taken_from_center: bool,
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = self.to_fen();
        string.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        GameState::from_fen(&string).map_err(serde::de::Error::custom)
    }
}

impl GameState {
    pub fn to_fen(&self) -> String {
        let number_of_players = NUM_PLAYERS as u8;
        let bag = (self.bag[0] as usize)
            | (self.bag[1] as usize) << 8
            | (self.bag[2] as usize) << 16
            | (self.bag[3] as usize) << 24
            | (self.bag[4] as usize) << 32;

        let out_of_bag = (self.out_of_bag[0] as usize)
            | (self.out_of_bag[1] as usize) << 8
            | (self.out_of_bag[2] as usize) << 16
            | (self.out_of_bag[3] as usize) << 24
            | (self.out_of_bag[4] as usize) << 32;

        // 3 bits per color per factory
        // 3 bits * 5 colors * max 10 factories = 150 bit
        // The center might be larger

        let mut factories = [0b0_u128; NUM_FACTORIES];
        for (factory_index, factory) in self.factories.iter().enumerate().take(CENTER_FACTORY_INDEX)
        {
            let mut factory_binary = 0b0_u128;
            for (color_index, number_of_tiles) in factory.iter().enumerate() {
                let number_of_tiles = *number_of_tiles as u128;
                factory_binary |= number_of_tiles << (color_index * 4);
            }
            factories[factory_index] = factory_binary;
        }
        factories[CENTER_FACTORY_INDEX] = 0b0_u128;
        for color_index in 0..NUM_TILE_COLORS {
            let number_of_tiles = self.factories[CENTER_FACTORY_INDEX][color_index] as u128;
            factories[CENTER_FACTORY_INDEX] |= number_of_tiles << (color_index * 8);
        }

        let factories_string = factories
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("-");

        let mut scores = 0b0_u64;
        for (player_index, score) in self.scores.iter().enumerate() {
            let score = (*score + 1000) as u64;
            scores |= score << (player_index * 16);
        }

        let mut floor_line_progress = 0b0_u64;
        for (player_index, progress) in self.floor_line_progress.iter().enumerate() {
            let progress = *progress as u64;
            floor_line_progress |= progress << (player_index * 8);
        }

        let walls_string = self
            .walls
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("-");

        // 4 * 5 * 8
        let mut pattern_line_occupancy = [0b0_u64; NUM_PLAYERS];
        for (player_index, pattern_lines) in self.pattern_lines_occupancy.iter().enumerate() {
            pattern_line_occupancy[player_index] |= pattern_lines[0] as u64;
            pattern_line_occupancy[player_index] |= (pattern_lines[1] as u64) << 8;
            pattern_line_occupancy[player_index] |= (pattern_lines[2] as u64) << 16;
            pattern_line_occupancy[player_index] |= (pattern_lines[3] as u64) << 24;
            pattern_line_occupancy[player_index] |= (pattern_lines[4] as u64) << 32;
        }
        let pattern_line_string = pattern_line_occupancy
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("-");

        let mut pattern_line_colors = [0b0_u64; NUM_PLAYERS];
        for (player_index, pattern_lines) in self.pattern_lines_colors.iter().enumerate() {
            for (line_index, color) in pattern_lines.iter().enumerate() {
                if let Some(color) = color {
                    pattern_line_colors[player_index] |= (*color as u64) << (line_index * 8);
                } else {
                    pattern_line_colors[player_index] |= 255 << (line_index * 8);
                }
            }
        }
        let pattern_line_colors_string = pattern_line_colors
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("-");

        let next_round_starting_player = usize::from(self.next_round_starting_player);

        format!(
            "{}_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}",
            number_of_players,
            usize::from(self.current_player),
            next_round_starting_player,
            bag,
            out_of_bag,
            factories_string,
            scores,
            floor_line_progress,
            walls_string,
            pattern_line_string,
            pattern_line_colors_string,
            self.tile_taken_from_center as u8
        )
    }

    pub fn from_fen(string: &str) -> Result<Self, String> {
        let entries: Vec<&str> = string.split('_').collect();

        let number_of_players = entries.first().ok_or("No number of players")?;
        let number_of_players = number_of_players
            .parse::<u8>()
            .map_err(|_| "Invalid number of players")?;
        if number_of_players != NUM_PLAYERS as u8 {
            return Err(format!(
                "Number of players in string ({}) does not match number of players in game ({})",
                number_of_players, NUM_PLAYERS
            ));
        }

        let current_player = entries.get(1).ok_or("No current player")?;
        let current_player = current_player
            .parse::<u8>()
            .map_err(|_| "Invalid current player")?;
        let current_player = PlayerMarker::new(current_player);

        let next_round_starting_player = entries.get(2).ok_or("No next round starting player")?;
        let next_round_starting_player = next_round_starting_player
            .parse::<u8>()
            .map_err(|_| "Invalid next round starting player")?;
        let next_round_starting_player = PlayerMarker::new(next_round_starting_player);

        let bag_binary = entries.get(3).ok_or("No bag")?;
        let bag_binary = bag_binary.parse::<usize>().map_err(|_| "Invalid bag")?;
        let mut bag = [0; NUM_TILE_COLORS];
        bag[0] = (bag_binary & 0xFF) as u8;
        bag[1] = ((bag_binary >> 8) & 0xFF) as u8;
        bag[2] = ((bag_binary >> 16) & 0xFF) as u8;
        bag[3] = ((bag_binary >> 24) & 0xFF) as u8;
        bag[4] = ((bag_binary >> 32) & 0xFF) as u8;

        let out_of_bag_binary = entries.get(4).ok_or("No out of bag")?;
        let out_of_bag_binary = out_of_bag_binary
            .parse::<usize>()
            .map_err(|_| "Invalid out of bag")?;
        let mut out_of_bag = [0; NUM_TILE_COLORS];
        out_of_bag[0] = (out_of_bag_binary & 0xFF) as u8;
        out_of_bag[1] = ((out_of_bag_binary >> 8) & 0xFF) as u8;
        out_of_bag[2] = ((out_of_bag_binary >> 16) & 0xFF) as u8;
        out_of_bag[3] = ((out_of_bag_binary >> 24) & 0xFF) as u8;
        out_of_bag[4] = ((out_of_bag_binary >> 32) & 0xFF) as u8;

        let factories_strings = entries.get(5).ok_or("No factories")?;
        let mut factories = Factories::empty();
        for (factory_index, factory_string) in factories_strings
            .split('-')
            .enumerate()
            .take(CENTER_FACTORY_INDEX)
        {
            let factory_binary = factory_string
                .parse::<u128>()
                .map_err(|_| "Invalid factory")?;
            for color_index in 0..NUM_TILE_COLORS {
                factories[factory_index][color_index] =
                    ((factory_binary >> (color_index * 4)) & 0b1111) as u8;
            }
        }
        for color_index in 0..NUM_TILE_COLORS {
            let factory_string = factories_strings
                .split('-')
                .nth(CENTER_FACTORY_INDEX)
                .ok_or("No center factory")?;
            let factory_binary = factory_string
                .parse::<u64>()
                .map_err(|_| "Invalid factory")?;

            factories[CENTER_FACTORY_INDEX][color_index] =
                ((factory_binary >> (color_index * 8)) & 0xFF) as u8;
        }

        let scores_binary = entries.get(6).ok_or("No scores")?;
        let scores_binary = scores_binary
            .parse::<usize>()
            .map_err(|_| "Invalid scores")?;
        let mut scores = [0; NUM_PLAYERS];

        for (player_index, player_score) in scores.iter_mut().enumerate() {
            *player_score = ((scores_binary >> (player_index * 16)) & 0xFFFF) as i16 - 1000;
        }

        let floor_line_progress_binary = entries.get(7).ok_or("No floor line progress")?;
        let floor_line_progress_binary = floor_line_progress_binary
            .parse::<usize>()
            .map_err(|_| "Invalid floor line progress")?;

        let mut floor_line_progress = [0; NUM_PLAYERS];
        for (player_index, progress) in floor_line_progress.iter_mut().enumerate() {
            *progress = ((floor_line_progress_binary >> (player_index * 8)) & 0xFF) as u8;
        }

        let walls_strings = entries.get(8).ok_or("No walls")?;
        let mut walls = [0_u32; NUM_PLAYERS];
        for (wall_index, wall_string) in walls_strings.split('-').enumerate() {
            let wall_binary = wall_string.parse::<u32>().map_err(|_| "Invalid wall")?;
            walls[wall_index] = wall_binary;
        }

        let pattern_lines_strings = entries.get(9).ok_or("No pattern lines")?;
        let mut pattern_lines_occupancy = [[0; 5]; NUM_PLAYERS];
        for (player_index, pattern_string) in pattern_lines_strings.split('-').enumerate() {
            let pattern_binary = pattern_string
                .parse::<u64>()
                .map_err(|_| "Invalid pattern")?;
            for line_index in 0..5 {
                pattern_lines_occupancy[player_index][line_index] =
                    ((pattern_binary >> (line_index * 8)) & 0xFF) as u8;
            }
        }

        let pattern_lines_colors_strings = entries.get(10).ok_or("No pattern lines colors")?;
        let mut pattern_lines_colors = [[Option::None; 5]; NUM_PLAYERS];
        for (player_index, player_string) in pattern_lines_colors_strings.split('-').enumerate() {
            let player_binary = player_string.parse::<u64>().map_err(|_| "Invalid player")?;
            for line_index in 0..5 {
                let color = ((player_binary >> (line_index * 8)) & 0xFF) as u8;
                if color == 255 {
                    pattern_lines_colors[player_index][line_index] = None;
                } else {
                    pattern_lines_colors[player_index][line_index] = Some(TileColor::from(color));
                }
            }
        }

        let tile_taken_from_center = entries.get(11).ok_or("No tile taken from center")?;
        let tile_taken_from_center = tile_taken_from_center
            .parse::<u8>()
            .map_err(|_| "Invalid tile taken from center")?;
        let tile_taken_from_center = tile_taken_from_center == 1;

        Ok(Self {
            current_player,
            bag,
            out_of_bag,
            factories,
            scores,
            floor_line_progress,
            walls,
            pattern_lines_occupancy,
            pattern_lines_colors,
            next_round_starting_player,
            tile_taken_from_center,
        })
    }

    pub fn evaluate_round(&mut self) -> bool {
        let mut is_game_over = false;
        for player_index in 0..NUM_PLAYERS {
            let mut score: i16 = 0;
            // Find completed pattern lines
            for (pattern_line_index, no_tiles_in_pattern_line) in self.pattern_lines_occupancy
                [player_index]
                .iter_mut()
                .enumerate()
            {
                if *no_tiles_in_pattern_line as usize != pattern_line_index + 1 {
                    continue;
                }
                let row_mask = wall::get_row_mask(pattern_line_index);

                #[cfg(debug_assertions)]
                if self.pattern_lines_colors[player_index][pattern_line_index].is_none() {
                    // If the pattern line is empty, we can't place a tile in it
                    println!("{}", self);
                    println!(
                        "Player {} pattern line {} is empty",
                        player_index, pattern_line_index
                    );
                    panic!(
                        "Player {} pattern line {} is empty",
                        player_index, pattern_line_index
                    )
                }

                let pattern_line_color =
                    self.pattern_lines_colors[player_index][pattern_line_index].unwrap(); // Must be Some because the pattern line is full
                let color_mask = wall::WALL_COLOR_MASKS[pattern_line_color as usize];
                let new_tile = row_mask & color_mask;
                let new_tile_pos = new_tile.trailing_zeros() as u8;
                let score_for_tile =
                    wall::get_placed_tile_score(self.walls[player_index], new_tile_pos);
                score += score_for_tile as i16;

                // Add the tile to the wall
                self.walls[player_index] |= new_tile;

                // Remove the tile from the pattern line
                self.out_of_bag[pattern_line_color as usize] += *no_tiles_in_pattern_line - 1; // -1 because one is placed on the board
                *no_tiles_in_pattern_line = 0;

                // Remove the color from the pattern line
                self.pattern_lines_colors[player_index][pattern_line_index] = None;
            }

            // Penalize the player for the tiles in the floor line
            let floor_line_progress = self.floor_line_progress[player_index]
                .min(FLOOR_LINE_PENALTY.len() as u8 - 1)
                as usize;
            let penalty = FLOOR_LINE_PENALTY[floor_line_progress];
            score -= penalty as i16;

            self.scores[player_index] += score;

            // Make sure no player falls below 0 points
            self.scores[player_index] = self.scores[player_index].max(0);

            // Reset
            self.floor_line_progress[player_index] = 0;

            // Check if the row is complete
            let complete_row_exists = wall::check_complete_row_exists(self.walls[player_index]);
            if complete_row_exists {
                is_game_over = true;
            }
        }

        self.current_player = self.next_round_starting_player; // Must be Some because the round is over
        self.tile_taken_from_center = false;

        if is_game_over {
            self.evaluate_end_of_game();
        }

        is_game_over
    }

    fn evaluate_end_of_game(&mut self) {
        let mut players_complete_rows = [0; NUM_PLAYERS];
        let mut players_with_highest_score = [false; NUM_PLAYERS];
        let mut highest_score = i16::MIN;
        let mut num_players_with_highest_score = 0;

        for (player, wall_occupancy) in self.walls.iter().enumerate() {
            let complete_rows = wall::count_complete_rows(*wall_occupancy);
            players_complete_rows[player] = complete_rows;
            let complete_colums = wall::count_complete_columns(*wall_occupancy);
            let complete_colors = wall::count_full_colors(*wall_occupancy);
            let score =
                complete_rows as i16 * 2 + complete_colums as i16 * 7 + complete_colors as i16 * 10;
            self.scores[player] += score;

            #[allow(clippy::comparison_chain)]
            if self.scores[player] > highest_score {
                highest_score = self.scores[player];
                players_with_highest_score = [false; NUM_PLAYERS];
                players_with_highest_score[player] = true;
                num_players_with_highest_score = 1;
            } else if self.scores[player] == highest_score {
                players_with_highest_score[player] = true;
                num_players_with_highest_score += 1;
            }
        }

        // In case of a tie, the player with the most complete rows wins
        if num_players_with_highest_score > 1 {
            let mut max_complete_rows = 0;
            let mut player_with_most_rows = usize::MAX;
            let mut tie_for_most_rows = false;

            for player in 0..NUM_PLAYERS {
                if players_with_highest_score[player] {
                    #[allow(clippy::comparison_chain)]
                    if players_complete_rows[player] > max_complete_rows {
                        max_complete_rows = players_complete_rows[player];
                        player_with_most_rows = player;
                        tie_for_most_rows = false;
                    } else if players_complete_rows[player] == max_complete_rows {
                        tie_for_most_rows = true;
                    }
                }
            }

            // Add +1 to the score of the player with the most complete rows if there's no tie for rows
            if !tie_for_most_rows && player_with_most_rows != usize::MAX {
                // Since this function does not return a game result we add +1 to the winning player so that they actually win
                // It would be better if we could distinguish between a tie and a win without having to add +1 to the score
                self.scores[player_with_most_rows] += 1;
            }
        }
    }

    pub fn do_move(&mut self, mov: Move) {
        let current_player: usize = self.current_player.into();
        let factory_index = mov.factory_index as usize;
        let color = mov.color as usize;
        let factory: [u8; 5] = self.factories[factory_index];
        let pattern_line_index = mov.pattern_line_index as usize;

        // Step 1: Put the remaining tiles in the center
        if factory_index == CENTER_FACTORY_INDEX {
            // If we took tiles from the center, we only remove the color we took
            self.factories[factory_index][color] = 0;
            if !self.tile_taken_from_center {
                // If we are the first player to take tiles from the center in this round, we become the starting player for the next round and advance the floor line
                self.next_round_starting_player = self.current_player;
                self.floor_line_progress[current_player] += 1;
                self.tile_taken_from_center = true;
            }
        } else {
            // For each tile color in the factory, put the tiles into the center if they are not the color we took
            for (color_index, factory) in factory.iter().enumerate() {
                if color_index != color {
                    self.factories[CENTER_FACTORY_INDEX][color_index] += factory;
                }
            }

            // If we took tiles from a factory, we empty it completely
            self.factories[factory_index] = [0; NUM_TILE_COLORS];
        }

        // Step 2. Place the tiles in the pattern lines or discard them
        if pattern_line_index != 5 {
            self.pattern_lines_colors[current_player][pattern_line_index] = Some(mov.color);
            self.pattern_lines_occupancy[current_player][pattern_line_index] += mov.places;
        }

        // Advance the floor line if the move discards tiles
        self.floor_line_progress[current_player] += mov.discards;
        self.out_of_bag[color] += mov.discards; // Discarded patterns are added to the out_of_bag. They will be put bag into the bag at the end of the round

        // Advance the player
        self.current_player = self.current_player.next();

        #[cfg(debug_assertions)]
        self.check_integrity().unwrap();
    }

    pub fn get_possible_moves(
        &mut self,
        move_list: &mut MoveList,
        rng: &mut SmallRng,
    ) -> MoveGenerationResult {
        move_list.clear(); // Clear any remaining moves from the previous round

        let is_round_over = self.factories.is_empty();
        if is_round_over {
            let is_game_over = self.evaluate_round();

            #[cfg(debug_assertions)]
            self.check_integrity().unwrap();

            if is_game_over {
                return MoveGenerationResult::GameOver;
            }

            self.fill_factories(rng);
            #[cfg(debug_assertions)]
            self.check_integrity().unwrap();
        }

        let current_player: usize = self.current_player.into();

        // Get the pattern lines of the current player, the moves will be placed in the pattern lines
        let player_pattern_lines: [u8; 5] = self.pattern_lines_occupancy[current_player];
        let player_pattern_line_colors: [Option<TileColor>; 5] =
            self.pattern_lines_colors[current_player];

        // We will use this to keep track of the remaining space in our pattern lines
        let mut remaining_space: [u8; 5] = [1, 2, 3, 4, 5]; // 255 is the floor line
        for (pattern_line_index, number_of_tiles) in player_pattern_lines.iter().enumerate() {
            // Subtract the number of tiles already in the pattern line from the total space
            remaining_space[pattern_line_index] -= *number_of_tiles;
        }

        let wall = self.walls[current_player];
        // Iterate over all factory and all color combinations
        for (factory_index, factory) in self.factories.iter().enumerate() {
            for (color, number) in factory.iter().enumerate() {
                // If there are no tiles of this color in the factory, skip it
                if *number == 0 {
                    continue;
                }

                move_list.push(Move {
                    factory_index: factory_index as u8,
                    color: TileColor::from(color as u8),
                    pattern_line_index: 5,
                    discards: *number,
                    places: 0,
                });

                for (pattern_line_index, pattern_line_space) in remaining_space.iter().enumerate() {
                    let pattern_line_space = *pattern_line_space;
                    let can_place = (*number).min(pattern_line_space);
                    if can_place == 0 {
                        continue;
                    }
                    let cannot_place = *number - can_place;

                    // If the pattern line has a different color, skip it.
                    if let Some(existing_color) = player_pattern_line_colors[pattern_line_index] {
                        if color != usize::from(existing_color) {
                            continue;
                        }
                    }

                    // If the wall at this position is already full, skip this color
                    let wall_mask = WALL_COLOR_MASKS[color];
                    let row_mask = wall::get_row_mask(pattern_line_index);
                    if wall & row_mask & wall_mask == 0 {
                        move_list.push(Move {
                            factory_index: factory_index as u8,
                            color: TileColor::from(color as u8),
                            pattern_line_index: pattern_line_index as u8,
                            discards: cannot_place,
                            places: can_place,
                        });
                    }
                }
            }
        }

        if is_round_over {
            MoveGenerationResult::RoundOver
        } else {
            MoveGenerationResult::Continue
        }
    }

    pub fn fill_factories(&mut self, rng: &mut SmallRng) {
        self.factories
            .refill_by_drawing_from_bag(&mut self.bag, &mut self.out_of_bag, rng);
    }

    pub fn check_integrity(&self) -> Result<(), GameError> {
        let mut is_valid = true;
        let mut error_description = String::new();

        // Make sure the bag has less or equal than 20 tiles of each color
        for color in self.bag.iter() {
            if *color > 20 {
                is_valid = false;
                error_description.push_str("Bag has more than 20 tiles of a color\n");
            }
        }

        // Make sure the factories have 0 or 4 tiles (Except the center)
        for factory in self.factories.iter().take(CENTER_FACTORY_INDEX) {
            let mut tile_count = 0;
            for color in factory.iter() {
                tile_count += color;
            }
            if tile_count > 4 {
                // In the rare case that you run out of tiles again while there are none left in the lid, start the new round as usual even though not all Factory displays are properly filled.
                is_valid = false;
                error_description.push_str(&format!("Factory has {} tiles\n", tile_count));
            }
        }

        // Check pattern line color assignment / occupancy match
        for player in 0..NUM_PLAYERS {
            for pattern_line in 0..5 {
                let color = self.pattern_lines_colors[player][pattern_line];
                match color {
                    None => {
                        if self.pattern_lines_occupancy[player][pattern_line] != 0 {
                            error_description.push_str(&format!(
                                "There are tiles in pattern line {} of player {} but they don't have a color\n",
                                pattern_line, player
                            ));
                            is_valid = false;
                        }
                    }
                    Some(_color) => {
                        if self.pattern_lines_occupancy[player][pattern_line] == 0 {
                            error_description.push_str(&format!(
                                "There are no tiles in pattern line {} of player {} but the line has a color {:?}\n",
                                pattern_line, player, color
                            ));
                            is_valid = false;
                        }
                    }
                }
            }
        }

        // Count the entire number of tiles in the game
        let mut tile_count = self.bag;
        for factory in self.factories.iter() {
            for (color, number) in factory.iter().enumerate() {
                tile_count[color] += number;
            }
        }

        for (color, num) in self.out_of_bag.iter().enumerate() {
            tile_count[color] += num;
        }

        for player in 0..NUM_PLAYERS {
            for pattern_line_index in 0..5 {
                let color = self.pattern_lines_colors[player][pattern_line_index];
                if let Some(color) = color {
                    let color = color as usize;
                    tile_count[color] += self.pattern_lines_occupancy[player][pattern_line_index];
                }
            }
        }

        for player in 0..NUM_PLAYERS {
            for (color_index, wall) in WALL_COLOR_MASKS.iter().enumerate() {
                let color = wall & self.walls[player];
                tile_count[color_index] += color.count_ones() as u8;
            }
        }

        // Make sure the total number of tiles is 20 for each
        let sum_tiles_in_game: u8 = tile_count.iter().sum();
        if sum_tiles_in_game != 100 {
            error_description.push_str(&format!(
                "The sum of the tiles does not add up to 100. {:?} = {}\n",
                tile_count,
                tile_count.iter().sum::<u8>()
            ));
            error_description.push_str(&format!("{}\n", self));

            is_valid = false;
        }
        for (color, number) in tile_count.iter().enumerate() {
            if *number > 20 {
                is_valid = false;
                println!(
                    "There are {} tiles of color {} in the game",
                    number,
                    TileColor::from(color as u8)
                );
            }
        }

        match is_valid {
            true => Ok(()),
            false => Err(GameError::InvalidGameState(error_description)),
        }
    }

    pub fn new(rng: &mut SmallRng) -> Self {
        let mut ret = Self {
            bag: [20, 20, 20, 20, 20],
            out_of_bag: [0; NUM_TILE_COLORS],
            factories: Factories::empty(),
            scores: [0; NUM_PLAYERS],
            floor_line_progress: [0; NUM_PLAYERS],
            walls: [0; NUM_PLAYERS],
            current_player: PlayerMarker::new(0),
            pattern_lines_occupancy: [[0; 5]; NUM_PLAYERS],
            pattern_lines_colors: [[None; 5]; NUM_PLAYERS],
            next_round_starting_player: PlayerMarker::new(0),
            tile_taken_from_center: false,
        };
        ret.fill_factories(rng);
        ret
    }

    pub fn empty() -> Self {
        Self {
            bag: [20; NUM_TILE_COLORS],
            out_of_bag: [0; NUM_TILE_COLORS],
            factories: Factories::empty(),
            scores: [0; NUM_PLAYERS],
            floor_line_progress: [0; NUM_PLAYERS],
            walls: [0; NUM_PLAYERS],
            current_player: PlayerMarker::new(0),
            pattern_lines_occupancy: [[0; 5]; NUM_PLAYERS],
            pattern_lines_colors: [[None; 5]; NUM_PLAYERS],
            next_round_starting_player: PlayerMarker::new(0),
            tile_taken_from_center: false,
        }
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = crate::formatting::display_gamestate(self, None);
        string.push_str(self.to_fen().as_str());
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};

    #[test]
    fn test_serialize_deserialize() {
        let mut move_list = MoveList::default();
        for _ in 0..20 {
            let mut rng: rand::rngs::SmallRng = SeedableRng::seed_from_u64(0);
            let mut game_state = GameState::new(&mut rng);
            loop {
                game_state.check_integrity().unwrap();

                let result = game_state.get_possible_moves(&mut move_list, &mut rng);
                if result == MoveGenerationResult::GameOver {
                    break;
                }

                let move_ = move_list[rng.gen_range(0..move_list.len())];

                game_state.do_move(move_);
                println!("Did move: {}", move_);
                println!("{}", game_state);
                game_state.check_integrity().unwrap();

                let string = game_state.to_fen();
                let reconstructed_game_state = GameState::from_fen(string.as_str()).unwrap();
                assert_eq!(game_state.bag, reconstructed_game_state.bag, "Bag");
                assert_eq!(
                    game_state.out_of_bag, reconstructed_game_state.out_of_bag,
                    "Out of bag"
                );
                assert_eq!(
                    game_state.factories, reconstructed_game_state.factories,
                    "Factories"
                );
                assert_eq!(game_state.scores, reconstructed_game_state.scores, "Scores");
                assert_eq!(
                    game_state.floor_line_progress, reconstructed_game_state.floor_line_progress,
                    "Floor line progress"
                );
                assert_eq!(
                    game_state.walls, reconstructed_game_state.walls,
                    "Wall occupancy"
                );
                assert_eq!(
                    game_state.pattern_lines_occupancy,
                    reconstructed_game_state.pattern_lines_occupancy,
                    "Pattern lines occupancy"
                );
                assert_eq!(
                    game_state.pattern_lines_colors, reconstructed_game_state.pattern_lines_colors,
                    "Pattern lines colors"
                );
                assert_eq!(
                    game_state.current_player, reconstructed_game_state.current_player,
                    "Current player"
                );
                assert_eq!(
                    game_state.next_round_starting_player,
                    reconstructed_game_state.next_round_starting_player,
                    "Next round starting player"
                );
            }
        }
    }
}
