use crate::factories::{self, CENTER_FACTORY_INDEX, NUM_FACTORIES};
use crate::move_::Move;
use crate::move_list::MoveList;
use crate::player::Player;
use crate::tile_color::{TileColor, NUM_TILE_COLORS};
use crate::wall::{self, WALL_COLOR_MASKS};
use rand::SeedableRng;
use std::fmt::Write;

pub const NUM_PLAYERS: usize = 2;
pub const FLOOR_LINE_PENALTY: [u8; 8] = [0, 1, 2, 4, 6, 8, 11, 14];

fn find_tile_combinations(
    tiles_left: u8,
    current_pattern: &mut [u8; 6],
    remaining_space: &mut [u8; 6],
    results: &mut Vec<[u8; 6]>,
    start_index: usize, // Add a parameter to keep track of the start index
) {
    if tiles_left == 0 {
        results.push(*current_pattern);
        return;
    }

    for pattern_line_index in start_index..6 {
        if remaining_space[pattern_line_index] > 0 {
            remaining_space[pattern_line_index] -= 1;
            current_pattern[pattern_line_index] += 1;
            find_tile_combinations(
                tiles_left - 1,
                current_pattern,
                remaining_space,
                results,
                pattern_line_index,
            ); // pass pattern_line_index to enforce order
            remaining_space[pattern_line_index] += 1;
            current_pattern[pattern_line_index] -= 1;
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    bag: [u8; NUM_TILE_COLORS], // For each color, how many tiles are left in the bag
    out_of_bag: [u8; NUM_TILE_COLORS],
    factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES], // For each factory, how many tiles of each color are in it (including the center)
    rng: rand::rngs::SmallRng,

    scores: [i16; NUM_PLAYERS], // For each player, how many points they have
    floor_line_progress: [u8; NUM_PLAYERS], // For each player, how many tiles they have in their penalty

    walls: [[u32; NUM_TILE_COLORS]; NUM_PLAYERS], // For each player, and each color, the locations of the tiles on their wall
    wall_occupancy: [u32; NUM_PLAYERS],           // For each player, the occupancy of their wall

    pattern_lines_occupancy: [[u8; 5]; NUM_PLAYERS], // For each player, the occupancy of their pattern lines
    pattern_lines_colors: [[Option<TileColor>; 5]; NUM_PLAYERS], // For each player, the color of their pattern lines. If the pattern line is empty, the color is 255

    current_player: Player,

    next_round_starting_player: Option<Player>,
}

impl GameState {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
            ..Default::default()
        }
    }

    pub fn with_rng(rng: rand::rngs::SmallRng) -> Self {
        Self {
            rng,
            ..Default::default()
        }
    }

    pub fn get_current_player(&self) -> Player {
        self.current_player
    }

    pub fn get_next_round_starting_player(&self) -> Option<Player> {
        self.next_round_starting_player
    }

    pub fn get_scores(&self) -> [i16; NUM_PLAYERS] {
        self.scores
    }

    pub fn get_bag(&self) -> [u8; NUM_TILE_COLORS] {
        self.bag
    }

    pub fn get_factories(&self) -> &[[u8; NUM_TILE_COLORS]; NUM_FACTORIES] {
        &self.factories
    }

    pub fn get_floor_line_progress(&self) -> [u8; NUM_PLAYERS] {
        self.floor_line_progress
    }

    pub fn get_walls(&self) -> &[[u32; NUM_TILE_COLORS]; NUM_PLAYERS] {
        &self.walls
    }

    pub fn get_pattern_lines_occupancy(&self) -> &[[u8; 5]; NUM_PLAYERS] {
        &self.pattern_lines_occupancy
    }

    pub fn get_pattern_lines_colors(&self) -> &[[Option<TileColor>; 5]; NUM_PLAYERS] {
        &self.pattern_lines_colors
    }

    pub fn serialize_string(&self) -> String {
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

        let mut factories = [0b0_u64; NUM_FACTORIES];
        for (factory_index, factory) in self.factories.iter().enumerate().take(CENTER_FACTORY_INDEX)
        {
            let mut factory_binary: u64 = 0b0_u64;
            for (color_index, number_of_tiles) in factory.iter().enumerate() {
                let number_of_tiles = *number_of_tiles as u64;
                factory_binary |= number_of_tiles << (color_index * 3);
            }
            factories[factory_index] = factory_binary;
        }
        factories[CENTER_FACTORY_INDEX] = 0b0_u64;
        for color_index in 0..NUM_TILE_COLORS {
            let number_of_tiles = self.factories[CENTER_FACTORY_INDEX][color_index] as u64;
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

        let mut walls = [0b0_u32; NUM_PLAYERS];
        for (wall_index, wall) in self.walls.iter().enumerate() {
            for bitboard in wall.iter() {
                walls[wall_index] |= bitboard;
            }
        }
        let walls_string = walls
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

        let next_round_starting_player = match self.next_round_starting_player {
            None => 255,
            Some(player) => usize::from(player),
        };

        format!(
            "{}_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}",
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
        )
    }

    pub fn deserialize_string(string: &str) -> Result<Self, String> {
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
        let current_player = Player::new(current_player);

        let next_round_starting_player = entries.get(2).ok_or("No next round starting player")?;
        let next_round_starting_player = next_round_starting_player
            .parse::<u8>()
            .map_err(|_| "Invalid next round starting player")?;
        let next_round_starting_player = if next_round_starting_player == 255 {
            None
        } else {
            Some(Player::new(next_round_starting_player))
        };

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
        let mut factories = [[0; NUM_TILE_COLORS]; NUM_FACTORIES];
        for (factory_index, factory_string) in factories_strings
            .split('-')
            .enumerate()
            .take(CENTER_FACTORY_INDEX)
        {
            let factory_binary = factory_string
                .parse::<u64>()
                .map_err(|_| "Invalid factory")?;
            for color_index in 0..NUM_TILE_COLORS {
                factories[factory_index][color_index] =
                    ((factory_binary >> (color_index * 3)) & 0b111) as u8;
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
        let mut walls = [[0; NUM_TILE_COLORS]; NUM_PLAYERS];

        for (wall_index, wall_string) in walls_strings.split('-').enumerate() {
            let wall_binary = wall_string.parse::<u32>().map_err(|_| "Invalid wall")?;
            for (color_index, color_mask) in WALL_COLOR_MASKS.iter().enumerate() {
                walls[wall_index][color_index] = wall_binary & color_mask;
            }
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

        let mut wall_occupancy = [0u32; NUM_PLAYERS];
        for (player_index, wall) in walls.iter().enumerate() {
            for bitboard in wall.iter() {
                wall_occupancy[player_index] |= bitboard;
            }
        }

        println!("Bag: {:?}", bag);
        println!("Out of bag: {:?}", out_of_bag);

        Ok(Self {
            current_player,
            bag,
            out_of_bag,
            factories,
            scores,
            floor_line_progress,
            walls,
            wall_occupancy,
            pattern_lines_occupancy,
            pattern_lines_colors,
            next_round_starting_player,

            ..Default::default()
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
                let pattern_line_color =
                    self.pattern_lines_colors[player_index][pattern_line_index].unwrap(); // Must be Some because the pattern line is full
                let color_mask = wall::WALL_COLOR_MASKS[pattern_line_color as usize];
                let new_tile = row_mask & color_mask;
                let new_tile_pos = new_tile.trailing_zeros() as u8;
                let score_for_tile =
                    wall::get_placed_tile_score(self.wall_occupancy[player_index], new_tile_pos);
                score += score_for_tile as i16;
                // println!("Evaluate round: Player {} score: {} for tile at position {} in pattern line {}", player_index, score_for_tile, new_tile_pos, pattern_line_index);

                // Add the tile to the wall
                self.walls[player_index][pattern_line_color as usize] |= new_tile;
                self.wall_occupancy[player_index] |= new_tile;

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

            // Reset
            self.floor_line_progress[player_index] = 0;

            // Check if the row is complete
            let complete_row_exists =
                wall::check_complete_row_exists(self.wall_occupancy[player_index]);
            if complete_row_exists {
                is_game_over = true;
            }
        }

        self.current_player = self.next_round_starting_player.unwrap(); // Must be Some because the round is over
        self.next_round_starting_player = None;

        if is_game_over {
            self.evaluate_end_of_game();
        }

        is_game_over
    }

    fn evaluate_end_of_game(&mut self) {
        for (player, wall_occupancy) in self.wall_occupancy.iter().enumerate() {
            let complete_rows = wall::count_complete_rows(*wall_occupancy);
            let complete_colums = wall::count_complete_columns(*wall_occupancy);
            let complete_colors = wall::count_full_colors(*wall_occupancy);
            let score =
                complete_rows as i16 * 2 + complete_colums as i16 * 7 + complete_colors as i16 * 10;
            self.scores[player] += score;
        }
    }

    pub fn do_move(&mut self, mov: Move) {
        // Step 1: Put the remaining tiles in the center
        let current_player: usize = self.current_player.into();

        let take_from_factory_index = mov.take_from_factory_index as usize;
        let color = mov.color as usize;
        let factory_content: [u8; 5] = self.factories[take_from_factory_index];
        if take_from_factory_index == CENTER_FACTORY_INDEX {
            // If we took tiles from the center, we only remove the color we took
            self.factories[take_from_factory_index][color] = 0;
            // If we are the first player to take tiles from the center in this round, we become the starting player for the next round
            if self.next_round_starting_player.is_none() {
                self.next_round_starting_player = Some(self.current_player);
                // Floor line progress + 1
                self.floor_line_progress[current_player] += 1;
            }
        } else {
            // Only put the tiles in the center if we are not taking from the center

            for (color_index, factory_content) in factory_content.iter().enumerate() {
                // For each tile color in the factory, put the tiles into the center if they are not the color we took
                if color_index != color {
                    self.factories[CENTER_FACTORY_INDEX][color_index] += factory_content;
                }
            }

            // If we took tiles from a factory, we empty it completely
            self.factories[take_from_factory_index] = [0; NUM_TILE_COLORS];
        }

        // Step 2. Place the tiles in the pattern lines or discard them
        for pattern_line_index in 0..5 {
            // Update / Check color of pattern line
            if mov.pattern[pattern_line_index] > 0 {
                #[cfg(debug_assertions)]
                {
                    let pattern_line_color_match = {
                        if let Some(pattern_line_color) =
                            self.pattern_lines_colors[current_player][pattern_line_index]
                        {
                            pattern_line_color == mov.color
                        } else {
                            true
                        }
                    };
                    if !pattern_line_color_match {
                        println!("{}", self);
                        println!("{}", mov);
                        println!(
                            "{:?}",
                            self.pattern_lines_colors[current_player][pattern_line_index]
                        );
                        println!("{}", mov.color);
                    }
                    debug_assert!(
                        pattern_line_color_match,
                        "Pattern line color does not match"
                    )
                }
                // Set the color of the pattern line
                self.pattern_lines_colors[current_player][pattern_line_index] = Some(mov.color);
            }
            // Add the new tile to the pattern line of the player
            self.pattern_lines_occupancy[current_player][pattern_line_index] +=
                mov.pattern[pattern_line_index];
        }
        // Advance the floor line if the move discards tiles
        self.floor_line_progress[current_player] += mov.pattern[5];
        self.out_of_bag[color] += mov.pattern[5]; // Discarded patterns are added to the out_of_bag. They will be put bag into the bag at the end of the round

        // Advance the player
        self.current_player = self.current_player.next();

        #[cfg(debug_assertions)]
        self.check_integrity();
    }

    pub fn get_possible_moves(&mut self, move_list: &mut MoveList) -> (bool, bool) {
        // -> (is_game_over, did_fill_factories)
        move_list.clear();
        let current_player: usize = self.current_player.into();
        let player_pattern_lines: [u8; 5] = self.pattern_lines_occupancy[current_player];
        let player_pattern_line_colors: [Option<TileColor>; 5] =
            self.pattern_lines_colors[current_player];

        for (factory_index, factory) in self.factories.iter().enumerate() {
            for (color, number) in factory.iter().enumerate() {
                if *number == 0 {
                    continue;
                }

                let mut remaining_space: [u8; 6] = [1, 2, 3, 4, 5, 255];
                for (pattern_line_index, number_of_tiles) in player_pattern_lines.iter().enumerate()
                {
                    remaining_space[pattern_line_index] -= *number_of_tiles;
                    if let Some(existing_color) = player_pattern_line_colors[pattern_line_index] {
                        if color != usize::from(existing_color) {
                            remaining_space[pattern_line_index] = 0;
                        }
                    } else {
                        // Make sure the wall has space for the tiles
                        let wall_mask = WALL_COLOR_MASKS[color];
                        let wall_occupancy = self.wall_occupancy[current_player];
                        let row = wall::get_row_mask(pattern_line_index);
                        if wall_occupancy & row & wall_mask > 0 {
                            remaining_space[pattern_line_index] = 0;
                        }
                    }
                }

                let mut possible_patterns = Vec::new();
                find_tile_combinations(
                    *number,
                    &mut [0, 0, 0, 0, 0, 0],
                    &mut remaining_space,
                    &mut possible_patterns,
                    0,
                );

                for pattern in possible_patterns {
                    move_list.push(Move {
                        take_from_factory_index: factory_index as u8,
                        color: TileColor::from(color as u8),
                        pattern,
                    });
                }
            }
        }

        if move_list.is_empty() {
            let is_game_over = self.evaluate_round();
            if !is_game_over {
                self.fill_factories();
                self.get_possible_moves(move_list);
            }
            (is_game_over, true)
        } else {
            (false, false)
        }
    }

    pub fn fill_factories(&mut self) {
        factories::fill_factories(
            &mut self.factories,
            &mut self.bag,
            &mut self.out_of_bag,
            &mut self.rng,
        );
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
            if tile_count > 4 {
                // In the rare case that you run out of tiles again while there are none left in the lid, start the new round as usual even though not all Factory displays are properly filled.
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

        // Make sure occupancy is correct
        for player in 0..NUM_PLAYERS {
            let mut calculated_occupancy: u32 = 0b0;
            for color in 0..NUM_TILE_COLORS {
                let bitboard = self.walls[player][color];
                calculated_occupancy |= bitboard;
            }
            if calculated_occupancy != self.wall_occupancy[player] {
                is_valid = false;
                println!(
                    "Player {} has wrong occupancy. Expected: {:b} Actual: {:b}",
                    player, calculated_occupancy, self.wall_occupancy[player]
                );
            }
        }

        // Check pattern line color assignment / occupancy match
        for player in 0..NUM_PLAYERS {
            for pattern_line in 0..5 {
                let color = self.pattern_lines_colors[player][pattern_line];
                match color {
                    None => {
                        if self.pattern_lines_occupancy[player][pattern_line] != 0 {
                            println!("There are tiles in pattern line {} of player {} but they don't have a color", pattern_line, player);
                            is_valid = false;
                        }
                    }
                    Some(_color) => {
                        if self.pattern_lines_occupancy[player][pattern_line] == 0 {
                            println!("There are no tiles in pattern line {} of player {} but the line has a color {:?}", pattern_line, player, color);
                            is_valid = false;
                        }
                    }
                }
            }
        }

        // Count the entire number of tiles in the game
        let mut tile_count = self.bag;
        // println!("Tile count bag:             {:?}", tile_count);
        for factory in self.factories.iter() {
            for (color, number) in factory.iter().enumerate() {
                tile_count[color] += number;
            }
        }
        // println!("Tile count + factories:     {:?}", tile_count);

        for (color, num) in self.out_of_bag.iter().enumerate() {
            tile_count[color] += num;
        }
        // println!("Tile count + out of bag:    {:?}", tile_count);

        for player in 0..NUM_PLAYERS {
            for pattern_line_index in 0..5 {
                let color = self.pattern_lines_colors[player][pattern_line_index];
                /*println!(
                    "pattern line index {} color: {:?}",
                    pattern_line_index, color
                );*/
                if let Some(color) = color {
                    let color = color as usize;
                    /*println!(
                        "self.pattern_lines_occupancy[player][color]: {:?}",
                        self.pattern_lines_occupancy[player][pattern_line_index]
                    );*/
                    tile_count[color] += self.pattern_lines_occupancy[player][pattern_line_index];
                }
            }
        }
        // println!("Tile count + pattern lines: {:?}", tile_count);

        for player in 0..NUM_PLAYERS {
            for (color, wall) in self.walls[player].iter().enumerate() {
                tile_count[color] += wall.count_ones() as u8;
            }
        }
        // println!("Tile count + wall:          {:?}", tile_count);

        // Make sure the total number of tiles is 20 for each
        //let sum_floor_line_progress: u8 = self.floor_line_progress.iter().sum();
        let sum_tiles_in_game: u8 = tile_count.iter().sum();
        //let sum_tiles_in_game = sum_tiles_in_game + sum_floor_line_progress;
        if sum_tiles_in_game != 100 {
            println!(
                "The sum of the tiles does not add up to 100. {:?} = {}",
                tile_count,
                tile_count.iter().sum::<u8>(),
                //sum_floor_line_progress
            );
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

        if !is_valid {
            println!("{}", self);
            panic!("Game state is invalid");
        }
    }

    // pub fn reconstruct_move_sequence(&mut self, past_state: &GameState, move_list: &mut MoveList) {
    //     // Given two game states, reconstruct the move sequence that led from the past state to the current state

    // }
}

impl Default for GameState {
    fn default() -> Self {
        let mut ret = Self {
            bag: [20, 20, 20, 20, 20],
            out_of_bag: [0; NUM_TILE_COLORS],
            factories: [[0; NUM_TILE_COLORS]; NUM_FACTORIES],
            scores: [0; NUM_PLAYERS],
            floor_line_progress: [0; NUM_PLAYERS],
            walls: [[0; NUM_TILE_COLORS]; NUM_PLAYERS],
            wall_occupancy: [0; NUM_PLAYERS],
            current_player: Player::new(0),
            pattern_lines_occupancy: [[0; 5]; NUM_PLAYERS],
            pattern_lines_colors: [[None; 5]; NUM_PLAYERS],
            rng: rand::rngs::SmallRng::from_entropy(),
            next_round_starting_player: None,
        };
        ret.fill_factories();
        ret
    }
}

fn bag_to_string(game_state: &GameState) -> String {
    let mut string = String::from("BAG       ");
    for (color, number_of_tiles_left) in game_state.bag.iter().enumerate() {
        write!(
            string,
            "{} {}\t",
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
        let tile_count: usize = factory.iter().sum::<u8>() as usize;

        if factory_index == CENTER_FACTORY_INDEX {
            string.push_str("Center-");
        } else {
            write!(string, "{}-", factory_index + 1).unwrap();
        }

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
                for (color, wall_color) in WALL_COLOR_MASKS.iter().enumerate() {
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
    for pattern_index in 0..5 {
        let pattern_color: Option<TileColor> = pattern_colors[pattern_index];
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
        }
        string.push_str(&color.1);
        string.push('\n');
    }

    string
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::with_capacity(1024);
        string.push_str(&bag_to_string(self));
        string.push_str(&factories_to_string(self));

        // Player header
        for player_index in 0..NUM_PLAYERS {
            if usize::from(self.current_player) == player_index {
                // set text color to black and background to white
                string.push_str("\x1b[30m\x1b[47m");
            }
            string.push_str(&format!(
                "PLAYER {} {:16}\x1b[0m |  ",
                player_index, self.scores[player_index]
            ));
        }

        string.push('\n');

        // Compute max lines
        let mut max_lines = 0;
        for player_index in 0..NUM_PLAYERS {
            let pattern_string = player_pattern_board_to_string(self, player_index);
            let wall_string = player_wall_to_string(self, player_index);
            max_lines = max_lines.max(pattern_string.lines().count());
            max_lines = max_lines.max(wall_string.lines().count());
        }

        // Player pattern boards next to walls
        for line in 0..max_lines {
            for player_index in 0..NUM_PLAYERS {
                let pattern_string = player_pattern_board_to_string(self, player_index);
                let wall_string = player_wall_to_string(self, player_index);

                let pattern_line = pattern_string.lines().nth(line).unwrap_or("");
                let wall_line = wall_string.lines().nth(line).unwrap_or("");

                string.push_str(pattern_line);
                string.push_str(" -> "); // separator between pattern and wall
                string.push_str(wall_line);
                string.push_str("|  ");
            }
            string.push('\n');
        }

        // Player floor lines
        for player_index in 0..NUM_PLAYERS {
            string.push_str(&format!(
                "Floor line: {:2}            |  ",
                self.floor_line_progress[player_index]
            ));
        }
        string.push('\n');
        string.push_str(self.serialize_string().as_str());
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
            let mut game_state = GameState::default();
            loop {
                game_state.fill_factories();

                game_state.check_integrity();

                loop {
                    game_state.get_possible_moves(&mut move_list);
                    if move_list.is_empty() {
                        break;
                    }
                    let move_ = move_list[rng.gen_range(0..move_list.len())];

                    game_state.do_move(move_);
                    game_state.check_integrity();

                    let string = game_state.serialize_string();
                    let reconstructed_game_state =
                        GameState::deserialize_string(string.as_str()).unwrap();
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
                        game_state.floor_line_progress,
                        reconstructed_game_state.floor_line_progress,
                        "Floor line progress"
                    );
                    assert_eq!(game_state.walls, reconstructed_game_state.walls, "Walls");
                    assert_eq!(
                        game_state.wall_occupancy, reconstructed_game_state.wall_occupancy,
                        "Wall occupancy"
                    );
                    assert_eq!(
                        game_state.pattern_lines_occupancy,
                        reconstructed_game_state.pattern_lines_occupancy,
                        "Pattern lines occupancy"
                    );
                    assert_eq!(
                        game_state.pattern_lines_colors,
                        reconstructed_game_state.pattern_lines_colors,
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
                let is_game_over = game_state.evaluate_round();
                if is_game_over {
                    println!("The game ended after round evaluation");
                    break;
                }
            }
        }
    }
}
