// use super::layers::InputLayer;
// use game::{
//     wall::{get_col_mask, get_row_mask},
//     Factories, GameState, Move, MoveGenerationResult, MoveList, TileColor, CENTER_FACTORY_INDEX,
//     NUM_FACTORIES, NUM_PLAYERS, NUM_TILE_COLORS, WALL_COLOR_MASKS,
// };
// use rand::{rngs::SmallRng, SeedableRng as _};
// use std::collections::{HashMap, HashSet};

// // Assume never more than 10 tiles of a color in the center factory. TODO: Add all possible combinations just like its done for the other factories??
// pub const CENTER_FACTORY_ENCODING_SIZE: usize = NUM_TILE_COLORS * 15;
// pub const NUM_NON_CENTER_FACTORIES: usize = NUM_FACTORIES - 1;
// pub const NON_CENTER_FACTORY_ENCODING_SIZE: usize =
//     NUM_NON_CENTER_FACTORIES * NUM_POSSIBLE_FACTORY_PERMUTATIONS; // there are 71 possible combinations of tiles in a factory and NUM_NON_CENTER_FACTORIES opportunities for duplicates
// pub const FACTORY_ENCODING_SIZE: usize =
//     CENTER_FACTORY_ENCODING_SIZE + NON_CENTER_FACTORY_ENCODING_SIZE;
// pub const SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE: usize =
//     (1 + 2 + 3 + 4 + 5) * NUM_TILE_COLORS + 5;
// pub const SINGLE_PLAYER_WALL_ENCODING_SIZE: usize = 2_usize.pow(5) * 5 * 3; //2**3 permutations for each row, col and diagonal
// pub const MAX_SCORE: i16 = 110;
// pub const SCORE_ENCODING_SIZE: usize = MAX_SCORE as usize * NUM_PLAYERS;

// pub const TOTAL_ENCODING_SIZE: usize = FACTORY_ENCODING_SIZE
//     + SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE * NUM_PLAYERS
//     + SINGLE_PLAYER_WALL_ENCODING_SIZE * NUM_PLAYERS
//     + SCORE_ENCODING_SIZE * NUM_PLAYERS;

// const PATTERN_LINE_OFFSETS: [usize; 5] = [0, 6, 17, 33, 54];

// pub fn encode_factories(
//     factories: &Factories,
//     layer: &mut dyn InputLayer,
// ) -> [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS] {
//     // This function writes the encoding of the factories to the input of the layer of the first layer of the neural network
//     // It must only be called if the factories have been refilled. If the factories have not been emptied, the encoding will be incorrect and the neural network will be confused.
//     let mut indices = [0; NUM_NON_CENTER_FACTORIES];
//     for (factory_index, factory) in factories.iter().take(NUM_NON_CENTER_FACTORIES).enumerate() {
//         indices[factory_index] = hash_factory(factory);
//     }

//     let mut count = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
//     let mut factory_count: [usize; 71] = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];

//     // Count occurrences of each index
//     for &index in indices.iter() {
//         count[index] += 1;
//         factory_count[index] += 1;
//     }

//     // Call the set_input method with index and count
//     for index in indices.iter() {
//         let index = *index;
//         if count[index] > 0 {
//             layer.set_input(index * NUM_TILE_COLORS + count[index] - 1);
//             count[index] = 0;
//         }
//     }

//     // Encode the center factory. It is special since it can hold any number of tiles of a color, but NUM_NON_CENTER_FACTORIES * 3 tiles max.
//     // Current approach: Just encode how many tiles of each type there are ignoring the relationship between the tile colors.
//     for (color, &count) in factories[NUM_FACTORIES - 1].iter().enumerate() {
//         layer.set_input(NON_CENTER_FACTORY_ENCODING_SIZE + color * 15 + count as usize);
//     }
//     // TODO: In 3 and 4 player games where the number of factories is higher, we risk writing outside the bounds of the part of the array dedicated to the center factory encoding.

//     factory_count
// }

// pub fn encode_pattern_lines(
//     pattern_line_occupancy: &[u8; 5],
//     pattern_line_colors: &[Option<TileColor>; 5],
//     player_index: usize,
//     layer: &mut dyn InputLayer,
// ) {
//     for (pattern_line_index, pattern_line_color) in pattern_line_colors.iter().enumerate() {
//         if let Some(color) = pattern_line_color.map(usize::from) {
//             let num_tiles = pattern_line_occupancy[pattern_line_index] as usize;
//             let index = if num_tiles == 0 {
//                 PATTERN_LINE_OFFSETS[pattern_line_index]
//             } else {
//                 PATTERN_LINE_OFFSETS[pattern_line_index]
//                     + 1
//                     + (num_tiles - 1) * NUM_TILE_COLORS
//                     + color
//             };
//             layer.set_input(
//                 FACTORY_ENCODING_SIZE
//                     + index
//                     + player_index * SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE,
//             );
//         }
//     }
// }

// pub fn encode_wall(wall: u32, player_index: usize, layer: &mut dyn InputLayer) {
//     // Encode a players wall. The wall is encoded in 3 parts: rows, columns and diagonals.
//     // Each uses bitwise operations to extract the relevant 5 bits from the wall.
//     // The bits are then used as an index to set the input of the layer.
//     let mut base_index = FACTORY_ENCODING_SIZE
//         + SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE * NUM_PLAYERS
//         + player_index * SINGLE_PLAYER_WALL_ENCODING_SIZE;

//     // Encode rows
//     for row in 0..5 {
//         let index = (wall & get_row_mask(row)) >> (row * 6);
//         let index = base_index + 0b11111 * row + index as usize;
//         layer.set_input(index);
//     }

//     base_index += 0b11111 * 5;
//     // Encode columns
//     for col in 0..5 {
//         let mut index = 0;
//         let masked = wall & get_col_mask(col);
//         for row in 0..5 {
//             index |= (masked >> (row * 6)) & 0b11111;
//         }
//         let index = base_index + index as usize + 0b11111 * col;
//         layer.set_input(index);
//     }

//     base_index += 0b11111 * 5;
//     // Encode diagonals
//     for (diagonal, mask) in WALL_COLOR_MASKS.iter().enumerate() {
//         let mut index = 0;
//         let masked = wall & mask;
//         for row in 0..5 {
//             index |= (masked >> (row * 6)) & 0b11111;
//         }
//         let index = base_index + index as usize + diagonal * 0b11111;
//         layer.set_input(index);
//     }
// }

// pub fn encode_player_score(score: i16, player_index: usize, layer: &mut dyn InputLayer) {
//     let index = score.max(0).min(MAX_SCORE) as usize;
//     layer.set_input(
//         FACTORY_ENCODING_SIZE
//             + SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE * NUM_PLAYERS
//             + SINGLE_PLAYER_WALL_ENCODING_SIZE * NUM_PLAYERS
//             + player_index * SCORE_ENCODING_SIZE
//             + index,
//     );
// }

// pub fn encode_game_state(
//     game_state: &GameState,
//     layer: &mut dyn InputLayer,
// ) -> [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS] {
//     let factory_count = encode_factories(&game_state.factories, layer);

//     let mut current_player = game_state.current_player;
//     let pattern_line_occupancy = game_state.pattern_lines_occupancy;
//     let pattern_line_colors = game_state.pattern_lines_colors;
//     let walls = game_state.walls;
//     let scores = game_state.scores;
//     for encoding_index in 0..NUM_PLAYERS {
//         let player_index = usize::from(current_player);
//         encode_pattern_lines(
//             &pattern_line_occupancy[player_index],
//             &pattern_line_colors[player_index],
//             encoding_index,
//             layer,
//         );

//         encode_wall(walls[player_index], encoding_index, layer);
//         encode_player_score(scores[player_index], encoding_index, layer);

//         current_player = current_player.next();
//     }

//     factory_count
// }

// pub fn decode_move(
//     game_state: &mut GameState,
//     encoding: &[f32],
//     move_to_index: &HashMap<(usize, u8, u8), usize>,
// ) -> Move {
//     let mut best_move = None;
//     let mut best_score = f32::NEG_INFINITY;

//     let mut move_list = MoveList::default();
//     game_state.get_possible_moves(&mut move_list, &mut SmallRng::from_entropy());

//     for move_ in move_list.into_iter() {
//         let move_index = if move_.factory_index < NUM_NON_CENTER_FACTORIES as u8 {
//             let factory = game_state.factories[move_.factory_index as usize];
//             let factory_index = hash_factory(&factory);
//             let key = (
//                 factory_index,
//                 u8::from(move_.color),
//                 move_.pattern_line_index,
//             );
//             move_to_index[&key]
//         } else {
//             let factory_index = INDEX_TO_FACTORY.len();
//             let key = (
//                 factory_index,
//                 u8::from(move_.color),
//                 move_.pattern_line_index,
//             );
//             move_to_index[&key]
//         };

//         let score = encoding[move_index];

//         if score > best_score {
//             best_score = score;
//             best_move = Some(move_);
//         }
//     }

//     *best_move.unwrap()
// }

// pub fn build_move_lookup() -> HashMap<(usize, u8, u8), usize> {
//     let mut move_index_set = HashSet::new();
//     for (factory_index, factory) in INDEX_TO_FACTORY.iter().enumerate() {
//         for tile_color in 0..NUM_TILE_COLORS {
//             if factory[tile_color] > 0 {
//                 for pattern_line_index in 0..6 {
//                     let key = (factory_index, tile_color as u8, pattern_line_index as u8);
//                     move_index_set.insert(key);
//                 }
//             }
//         }
//     }

//     for tile_color in 0..NUM_TILE_COLORS {
//         for pattern_line_index in 0..6 {
//             let key = (INDEX_TO_FACTORY.len(), tile_color as u8, pattern_line_index);
//             move_index_set.insert(key);
//         }
//     }

//     let mut move_to_index = HashMap::new();
//     let mut index = 0;
//     for key in move_index_set {
//         move_to_index.insert(key, index);
//         index += 1;
//     }

//     move_to_index
// }

// pub struct Accumulator<L: InputLayer> {
//     layer: L,
//     factory_count: [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
//     center_factory: Option<[u8; NUM_TILE_COLORS]>,
//     total_factory_count: usize,
// }

// impl<L: InputLayer> Accumulator<L> {
//     pub fn new(layer: L) -> Self {
//         Self {
//             layer,
//             factory_count: [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
//             center_factory: None,
//             total_factory_count: 0,
//         }
//     }

//     pub fn reset(&mut self) {
//         self.layer.reset();
//     }

//     pub fn set_state(&mut self, game_state: &GameState) {
//         self.reset();
//         self.factory_count = encode_game_state(game_state, &mut self.layer);
//     }

//     pub fn do_move(&mut self, game_state: &mut GameState, mov: Move) {
//         let current_player = usize::from(game_state.current_player);
//         let factory_index = mov.factory_index as usize;
//         let color = mov.color as usize;
//         let factory = game_state.factories[factory_index];
//         let pattern_line_index = mov.pattern_line_index as usize;
//         println!("Do move: {}", mov);

//         // Put the remaining tiles in the center factory
//         if factory_index == CENTER_FACTORY_INDEX {
//             // If we took tiles from the center, we only remove the color we took
//             game_state.factories[factory_index][color] = 0;
//             self.layer.unset_input(
//                 NON_CENTER_FACTORY_ENCODING_SIZE + color * 15 + factory[color] as usize,
//             );
//             println!(
//                 "The move takes from the center. Unset color {} count {}",
//                 color, factory[color]
//             );
//             if !game_state.tile_taken_from_center {
//                 // If we are the first player to take tiles from the center in this round, we become the starting player for the next round and advance the floor line
//                 game_state.next_round_starting_player = game_state.current_player;
//                 game_state.floor_line_progress[current_player] += 1;
//                 game_state.tile_taken_from_center = true;
//             }
//         } else {
//             // For each tile color in the factory, put the tiles into the center if they are not the color we took
//             let factory_hash = hash_factory(&factory);
//             println!(
//                 "Unset factory hash {} index {} count {}",
//                 factory_hash,
//                 factory_hash * NUM_TILE_COLORS + self.factory_count[factory_hash] - 1,
//                 self.factory_count[factory_hash] - 1
//             );
//             self.layer
//                 .unset_input(factory_hash * NUM_TILE_COLORS + self.factory_count[factory_hash] - 1);
//             self.layer.set_input(
//                 (NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1) * NUM_TILE_COLORS
//                     + self.factory_count[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1],
//             );
//             println!(
//                 "Set factory hash {} index {} count {}",
//                 NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1,
//                 (NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1) * NUM_TILE_COLORS
//                     + self.factory_count[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1]
//                     - 1,
//                 self.factory_count[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1]
//             );
//             self.factory_count[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1] += 1;
//             self.factory_count[factory_hash] -= 1;
//             for (color_index, count) in factory.iter().enumerate() {
//                 if *count == 0 {
//                     continue;
//                 }
//                 if color_index != color {
//                     let previous_count = game_state.factories[CENTER_FACTORY_INDEX][color_index];
//                     self.layer.unset_input(
//                         NON_CENTER_FACTORY_ENCODING_SIZE
//                             + color_index * 15
//                             + previous_count as usize,
//                     );
//                     println!(
//                         "The move takes from a non-center factory. Unset color {} count {} index {}",
//                         color_index, previous_count, NON_CENTER_FACTORY_ENCODING_SIZE
//                         + color_index * 15
//                         + previous_count as usize
//                     );

//                     game_state.factories[CENTER_FACTORY_INDEX][color_index] += count;

//                     let new_count = game_state.factories[CENTER_FACTORY_INDEX][color_index];
//                     self.layer.set_input(
//                         NON_CENTER_FACTORY_ENCODING_SIZE + color_index * 15 + new_count as usize,
//                     );
//                     println!(
//                         "The move takes from a non-center factory. Set color {} and count {} index {}",
//                         color_index, new_count, NON_CENTER_FACTORY_ENCODING_SIZE + color_index * 15 + new_count as usize
//                     );
//                 }
//             }

//             // If we took tiles from a factory, we empty it completely
//             game_state.factories[factory_index] = [0; NUM_TILE_COLORS];
//         }

//         // Step 2. Place the tiles in the pattern lines or discard them
//         if pattern_line_index != 5 {
//             let previous_color = game_state.pattern_lines_colors[current_player]
//                 [pattern_line_index]
//                 .map(usize::from)
//                 .unwrap_or(5);
//             let num_tiles =
//                 game_state.pattern_lines_occupancy[current_player][pattern_line_index] as usize;
//             let index = if num_tiles == 0 {
//                 PATTERN_LINE_OFFSETS[pattern_line_index]
//             } else {
//                 PATTERN_LINE_OFFSETS[pattern_line_index]
//                     + 1
//                     + (num_tiles - 1) * NUM_TILE_COLORS
//                     + previous_color
//             };
//             println!(
//                 "Placing the tiles in the pattern line. Unset {}",
//                 FACTORY_ENCODING_SIZE
//                     + index
//                     + current_player * SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE
//             );
//             self.layer.unset_input(
//                 FACTORY_ENCODING_SIZE
//                     + index
//                     + current_player * SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE,
//             );

//             game_state.pattern_lines_colors[current_player][pattern_line_index] = Some(mov.color);
//             game_state.pattern_lines_occupancy[current_player][pattern_line_index] += mov.places;

//             let num_tiles =
//                 game_state.pattern_lines_occupancy[current_player][pattern_line_index] as usize;
//             let index = if num_tiles == 0 {
//                 PATTERN_LINE_OFFSETS[pattern_line_index]
//             } else {
//                 PATTERN_LINE_OFFSETS[pattern_line_index]
//                     + 1
//                     + (num_tiles - 1) * NUM_TILE_COLORS
//                     + color
//             };
//             println!(
//                 "Placing the tiles in the pattern line. Set {}",
//                 FACTORY_ENCODING_SIZE
//                     + index
//                     + current_player * SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE
//             );
//             self.layer.set_input(
//                 FACTORY_ENCODING_SIZE
//                     + index
//                     + current_player * SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE,
//             );
//         }

//         // Advance the floor line if the move discards tiles
//         game_state.floor_line_progress[current_player] += mov.discards;
//         game_state.out_of_bag[color] += mov.discards; // Discarded patterns are added to the out_of_bag. They will be put bag into the bag at the end of the round

//         // Advance the player
//         game_state.current_player = game_state.current_player.next();

//         #[cfg(debug_assertions)]
//         game_state.check_integrity().unwrap();
//     }

//     pub fn output(&self) -> &[f32] {
//         self.layer.output()
//     }
// }

// #[cfg(test)]
// mod test {
//     use std::collections::HashSet;

//     use game::{Bag, MoveList};
//     use rand::{rngs::SmallRng, Rng as _, SeedableRng};

//     use super::*;

//     pub struct MockLayer {
//         pub input: Vec<usize>,
//         pub input_float: Vec<f32>,
//     }

//     impl InputLayer for MockLayer {
//         fn set_input(&mut self, index: usize) {
//             self.input.push(index);
//             self.input_float = self.input.iter().map(|&x| x as f32).collect();
//         }

//         fn unset_input(&mut self, index: usize) {
//             let length_before = self.input.len();
//             self.input.retain(|&x| x != index);
//             let length_after = self.input.len();
//             assert_eq!(length_before - 1, length_after);
//             self.input_float = self.input.iter().map(|&x| x as f32).collect();
//         }

//         fn reset(&mut self) {
//             self.input.clear();
//         }

//         fn output(&self) -> &[f32] {
//             &self.input_float
//         }
//     }

//     fn decode_factories(input: &[usize], panic_on_other: bool) -> Factories {
//         let mut factories = Factories::empty();
//         let mut current_index = 0;
//         for index in input.iter() {
//             let factory_index = index / NUM_TILE_COLORS;
//             let count = index % NUM_TILE_COLORS;
//             if panic_on_other && factory_index >= FACTORY_ENCODING_SIZE {
//                 continue;
//             }
//             if *index < NON_CENTER_FACTORY_ENCODING_SIZE {
//                 println!("Factory index: {}, count: {}", factory_index, count + 1);
//                 for _ in 0..(count + 1) {
//                     factories[current_index] = INDEX_TO_FACTORY[factory_index];
//                     current_index += 1;
//                 }
//             }
//         }
//         factories
//     }

//     #[test]
//     fn test_factory_encoding() {
//         let mut rng = SmallRng::from_seed([0; 32]);
//         for _ in 0..1000 {
//             let mut factories = Factories::empty();
//             factories.refill_by_drawing_from_bag(
//                 &mut Bag::from([20; 5]),
//                 &mut Bag::from([0; 5]),
//                 &mut rng,
//             );

//             while rng.gen_bool(0.5) {
//                 let index = rng.gen_range(0..NUM_FACTORIES);
//                 factories[index] = [0; NUM_TILE_COLORS];
//             }

//             let mut layer = MockLayer {
//                 input: Vec::new(),
//                 input_float: Vec::new(),
//             };
//             encode_factories(&factories, &mut layer);

//             let mut unique_factories = HashSet::new();
//             for factory in factories.iter().take(NUM_NON_CENTER_FACTORIES) {
//                 unique_factories.insert(hash_factory(factory));
//             }

//             let num_encoded_factories = layer.input.len();
//             assert_eq!(
//                 num_encoded_factories,
//                 unique_factories.len() + NUM_TILE_COLORS
//             );

//             println!("Input indices: {:?}", layer.input);
//             let decoded = decode_factories(&layer.input, true);

//             let factory_sort = |factory: &[u8; NUM_TILE_COLORS]| {
//                 if factory.iter().all(|&x| x == 0) {
//                     return 1000;
//                 }
//                 hash_factory(factory)
//             };

//             let mut sorted_factories = factories.to_vec();
//             sorted_factories.sort_by_key(factory_sort);
//             let mut sorted_decoded = decoded.to_vec();
//             sorted_decoded.sort_by_key(factory_sort);
//             println!("{:?}", sorted_factories);
//             println!("{:?}", sorted_decoded);
//             assert_eq!(sorted_factories, sorted_decoded);
//         }
//     }

//     fn decode_pattern_lines(
//         indices: Vec<usize>,
//         player_index: usize,
//     ) -> ([u8; 5], [Option<TileColor>; 5]) {
//         let indices = indices
//             .iter()
//             .map(|&x| {
//                 x - FACTORY_ENCODING_SIZE - player_index * SINGLE_PLAYER_PATTERN_LINE_ENCODING_SIZE
//             })
//             .collect::<Vec<_>>();
//         let mut pattern_line_occupancy = [0; 5];
//         let mut pattern_line_colors = [None; 5];

//         println!("Decoded: Indices: {:?}", indices);

//         for index in indices {
//             for (pattern_line_index, offset) in [0, 6, 17, 33, 54].iter().enumerate() {
//                 let next_offset = if pattern_line_index + 1 < 5 {
//                     PATTERN_LINE_OFFSETS[pattern_line_index + 1]
//                 } else {
//                     80
//                 };

//                 if *offset <= index && index < next_offset {
//                     let relative_index = index - offset;

//                     if relative_index == 0 {
//                         pattern_line_occupancy[pattern_line_index] = 0;
//                         pattern_line_colors[pattern_line_index] = None;
//                     } else {
//                         let num_tiles = (relative_index - 1) / NUM_TILE_COLORS + 1;
//                         let color_index = (relative_index - 1) % NUM_TILE_COLORS;
//                         pattern_line_occupancy[pattern_line_index] = num_tiles as u8;
//                         pattern_line_colors[pattern_line_index] =
//                             Some(TileColor::from(color_index as u8));
//                     }
//                 }
//             }
//         }

//         println!(
//             "Decoded: Pattern line occupancy: {:?}",
//             pattern_line_occupancy
//         );
//         println!("Decoded: Pattern line colors: {:?}", pattern_line_colors);
//         (pattern_line_occupancy, pattern_line_colors)
//     }

//     #[test]
//     fn test_pattern_line_encoding() {
//         let mut rng = SmallRng::from_seed([0; 32]);
//         let mut move_list = MoveList::default();
//         for _ in 0..100 {
//             let mut game_state = GameState::new(&mut rng);
//             for _ in 0..12 {
//                 game_state.get_possible_moves(&mut move_list, &mut rng);
//                 let random_move = rng.gen_range(0..move_list.len());
//                 game_state.do_move(move_list[random_move]);
//             }
//             println!("{}", game_state);

//             let pattern_line_occupancy = game_state.pattern_lines_occupancy;
//             let pattern_line_colors = game_state.pattern_lines_colors;
//             println!("Pattern line occupancy: {:?}", pattern_line_occupancy[0]);
//             println!("Pattern line colors: {:?}", pattern_line_colors[0]);
//             let mut mock_layer = MockLayer {
//                 input: Vec::new(),
//                 input_float: Vec::new(),
//             };
//             encode_pattern_lines(
//                 &pattern_line_occupancy[0],
//                 &pattern_line_colors[0],
//                 0,
//                 &mut mock_layer,
//             );
//             let (decoded_occupancy, decoded_colors) = decode_pattern_lines(mock_layer.input, 0);

//             assert_eq!(pattern_line_occupancy[0], decoded_occupancy);
//             assert_eq!(pattern_line_colors[0], decoded_colors);
//         }
//     }

//     #[test]
//     fn test_accumulator() {
//         let mut rng = SmallRng::from_seed([1; 32]);
//         let mut game_state = GameState::new(&mut rng);
//         let mut move_list = MoveList::default();
//         println!("{}", game_state);
//         let mut accumulator = Accumulator::new(MockLayer {
//             input: Vec::new(),
//             input_float: Vec::new(),
//         });
//         let mut test_accumulator = Accumulator::new(MockLayer {
//             input: Vec::new(),
//             input_float: Vec::new(),
//         });
//         accumulator.set_state(&game_state);
//         test_accumulator.set_state(&game_state);

//         assert_eq!(accumulator.output(), test_accumulator.output());
//         loop {
//             let result = game_state.get_possible_moves(&mut move_list, &mut rng);
//             if result == MoveGenerationResult::GameOver {
//                 break;
//             }

//             if result == MoveGenerationResult::RoundOver {
//                 accumulator.set_state(&game_state);
//             }
//             let mut game_state_clone = game_state.clone();
//             let random_move = move_list[rng.gen_range(0..move_list.len())];
//             accumulator.do_move(&mut game_state, random_move);
//             game_state_clone.do_move(random_move);
//             assert_eq!(game_state.to_fen(), game_state_clone.to_fen());
//             test_accumulator.set_state(&game_state_clone);
//             println!("{}", game_state);

//             let mut out1 = accumulator
//                 .output()
//                 .iter()
//                 .map(|&x| x as usize)
//                 .collect::<Vec<_>>();
//             out1.sort();
//             let mut out2 = test_accumulator
//                 .output()
//                 .iter()
//                 .map(|&x| x as usize)
//                 .collect::<Vec<_>>();
//             out2.sort();

//             let mut one = decode_factories(&out1, false);
//             let mut two = decode_factories(&out2, false);
//             one.sort_by_key(hash_factory);
//             two.sort_by_key(hash_factory);
//             println!("After update: {:?}", one);
//             println!("Should be   : {:?}", two);
//             assert_eq!(out1, out2);
//         }
//     }
// }
