use super::{node::Node, value::Value};
use game::*;
use rand::{rngs::SmallRng, Rng};
use wall::get_placed_tile_score;

pub trait PlayoutPolicy: Default + Send + Sync {
    fn playout(&mut self, game_state: &mut GameState, rng: &mut SmallRng) -> (Value, u16);
}

#[derive(Default)]
pub struct RandomPlayoutPolicy;

impl PlayoutPolicy for RandomPlayoutPolicy {
    fn playout(&mut self, game_state: &mut GameState, rng: &mut SmallRng) -> (Value, u16) {
        #[cfg(debug_assertions)]
        game_state
            .check_integrity()
            .expect("Game state integrity check failed before playout");

        // There are situations where every single player is only able to discard tiles
        // In this case, the game is in a infinite loop and we should break out of it
        let mut playout_depth = 0;
        for _ in 0..90 {
            match get_random_move(game_state, rng) {
                Some(move_) => {
                    game_state.do_move(move_);
                }
                None => break,
            }
            playout_depth += 1;
        }

        (Value::from_game_scores(game_state.scores), playout_depth)
    }
}

#[rustfmt::skip]
const PERMUTATIONS: [[u8; 5]; 120] = [[0, 1, 2, 3, 4],[0, 1, 2, 4, 3],[0, 1, 3, 2, 4],[0, 1, 3, 4, 2],[0, 1, 4, 2, 3],[0, 1, 4, 3, 2],[0, 2, 1, 3, 4],[0, 2, 1, 4, 3],[0, 2, 3, 1, 4],[0, 2, 3, 4, 1],[0, 2, 4, 1, 3],[0, 2, 4, 3, 1],[0, 3, 1, 2, 4],[0, 3, 1, 4, 2],[0, 3, 2, 1, 4],[0, 3, 2, 4, 1],[0, 3, 4, 1, 2],[0, 3, 4, 2, 1],[0, 4, 1, 2, 3],[0, 4, 1, 3, 2],[0, 4, 2, 1, 3],[0, 4, 2, 3, 1],[0, 4, 3, 1, 2],[0, 4, 3, 2, 1],[1, 0, 2, 3, 4],[1, 0, 2, 4, 3],[1, 0, 3, 2, 4],[1, 0, 3, 4, 2],[1, 0, 4, 2, 3],[1, 0, 4, 3, 2],[1, 2, 0, 3, 4],[1, 2, 0, 4, 3],[1, 2, 3, 0, 4],[1, 2, 3, 4, 0],[1, 2, 4, 0, 3],[1, 2, 4, 3, 0],[1, 3, 0, 2, 4],[1, 3, 0, 4, 2],[1, 3, 2, 0, 4],[1, 3, 2, 4, 0],[1, 3, 4, 0, 2],[1, 3, 4, 2, 0],[1, 4, 0, 2, 3],[1, 4, 0, 3, 2],[1, 4, 2, 0, 3],[1, 4, 2, 3, 0],[1, 4, 3, 0, 2],[1, 4, 3, 2, 0],[2, 0, 1, 3, 4],[2, 0, 1, 4, 3],[2, 0, 3, 1, 4],[2, 0, 3, 4, 1],[2, 0, 4, 1, 3],[2, 0, 4, 3, 1],[2, 1, 0, 3, 4],[2, 1, 0, 4, 3],[2, 1, 3, 0, 4],[2, 1, 3, 4, 0],[2, 1, 4, 0, 3],[2, 1, 4, 3, 0],[2, 3, 0, 1, 4],[2, 3, 0, 4, 1],[2, 3, 1, 0, 4],[2, 3, 1, 4, 0],[2, 3, 4, 0, 1],[2, 3, 4, 1, 0],[2, 4, 0, 1, 3],[2, 4, 0, 3, 1],[2, 4, 1, 0, 3],[2, 4, 1, 3, 0],[2, 4, 3, 0, 1],[2, 4, 3, 1, 0],[3, 0, 1, 2, 4],[3, 0, 1, 4, 2],[3, 0, 2, 1, 4],[3, 0, 2, 4, 1],[3, 0, 4, 1, 2],[3, 0, 4, 2, 1],[3, 1, 0, 2, 4],[3, 1, 0, 4, 2],[3, 1, 2, 0, 4],[3, 1, 2, 4, 0],[3, 1, 4, 0, 2],[3, 1, 4, 2, 0],[3, 2, 0, 1, 4],[3, 2, 0, 4, 1],[3, 2, 1, 0, 4],[3, 2, 1, 4, 0],[3, 2, 4, 0, 1],[3, 2, 4, 1, 0],[3, 4, 0, 1, 2],[3, 4, 0, 2, 1],[3, 4, 1, 0, 2],[3, 4, 1, 2, 0],[3, 4, 2, 0, 1],[3, 4, 2, 1, 0],[4, 0, 1, 2, 3],[4, 0, 1, 3, 2],[4, 0, 2, 1, 3],[4, 0, 2, 3, 1],[4, 0, 3, 1, 2],[4, 0, 3, 2, 1],[4, 1, 0, 2, 3],[4, 1, 0, 3, 2],[4, 1, 2, 0, 3],[4, 1, 2, 3, 0],[4, 1, 3, 0, 2],[4, 1, 3, 2, 0],[4, 2, 0, 1, 3],[4, 2, 0, 3, 1],[4, 2, 1, 0, 3],[4, 2, 1, 3, 0],[4, 2, 3, 0, 1],[4, 2, 3, 1, 0],[4, 3, 0, 1, 2],[4, 3, 0, 2, 1],[4, 3, 1, 0, 2],[4, 3, 1, 2, 0],[4, 3, 2, 0, 1],[4, 3, 2, 1, 0],];

pub fn get_random_move(game_state: &mut GameState, rng: &mut SmallRng) -> Option<Move> {
    let is_round_over = game_state.factories.is_empty();

    if is_round_over {
        let is_game_over = game_state.evaluate_round();

        if is_game_over {
            return None;
        }

        game_state.fill_factories(rng);
    }

    let factories: &Factories = &game_state.factories;
    let current_player: usize = game_state.current_player.into();
    let pattern_line_colors = game_state.pattern_lines_colors[current_player];
    let pattern_lines_occupancy = game_state.pattern_lines_occupancy[current_player];
    let wall_occupancy = game_state.walls[current_player];

    let mut random_factory_index;
    loop {
        random_factory_index = rng.gen_range(0..NUM_FACTORIES);
        if factories[random_factory_index].iter().any(|&tile| tile > 0) {
            break;
        }
    }

    let mut random_tile_color;
    let mut tile_number;
    loop {
        random_tile_color = rng.gen_range(0..5);
        tile_number = factories[random_factory_index][random_tile_color];
        if tile_number > 0 {
            break;
        }
    }

    let try_ordering = PERMUTATIONS[rng.gen_range(0..PERMUTATIONS.len())];
    let color_mask = WALL_COLOR_MASKS[random_tile_color];

    for pattern_line_index in try_ordering.iter() {
        if let Some(pattern_line_color) = pattern_line_colors[*pattern_line_index as usize] {
            if pattern_line_color != TileColor::from(random_tile_color) {
                continue;
            }
        }

        let row_mask = wall::get_row_mask(*pattern_line_index as usize);
        let tile = row_mask & color_mask;
        let already_occupied = wall_occupancy & tile > 0;
        if already_occupied {
            continue;
        }

        let pattern_line_space =
            1 + pattern_line_index - pattern_lines_occupancy[*pattern_line_index as usize];
        if pattern_line_space == 0 {
            continue;
        }

        let can_place = tile_number.min(pattern_line_space);
        let cannot_place = tile_number - can_place;

        return Some(Move {
            factory_index: random_factory_index as u8,
            color: TileColor::from(random_tile_color as u8),
            pattern_line_index: *pattern_line_index,
            discards: cannot_place,
            places: can_place,
        });
    }

    Some(Move {
        factory_index: random_factory_index as u8,
        color: TileColor::from(random_tile_color as u8),
        pattern_line_index: 5, // Discard
        discards: tile_number,
        places: 0,
    })
}

pub struct MetaPlayoutPolicy {
    iterations: usize,
    move_list: MoveList,
}

impl PlayoutPolicy for MetaPlayoutPolicy {
    // Use small MCTS to playout within big MCTS (experimental not actually effective)
    fn playout(&mut self, game_state: &mut GameState, rng: &mut SmallRng) -> (Value, u16) {
        let mut playout_depth = 0;
        for _ in 0..90 {
            let mut node = Node::new_deterministic(Move::DUMMY);
            if game_state.get_possible_moves(&mut self.move_list, rng)
                == MoveGenerationResult::GameOver
            {
                break;
            }
            node.expand(game_state, &mut self.move_list, rng);
            for _ in 0..self.iterations {
                node.iteration(
                    &mut game_state.clone(),
                    &mut self.move_list,
                    rng,
                    &mut RandomPlayoutPolicy,
                );
            }
            let player_index = usize::from(game_state.current_player);
            let best_move = node.best_move(player_index);
            if let Some(best_move) = best_move {
                game_state.do_move(best_move);
            } else {
                break;
            }
            playout_depth += 1;
        }

        (Value::from_game_scores(game_state.scores), playout_depth)
    }
}

impl Default for MetaPlayoutPolicy {
    fn default() -> Self {
        Self {
            iterations: 50_000,
            move_list: MoveList::new(),
        }
    }
}

pub struct HeuristicPlayoutPolicy {
    pub params: HeuristicPlayoutParams,
}

impl Default for HeuristicPlayoutPolicy {
    fn default() -> Self {
        Self {
            params: DEFAULT_PARAMS,
        }
    }
}

impl PlayoutPolicy for HeuristicPlayoutPolicy {
    fn playout(&mut self, game_state: &mut GameState, rng: &mut SmallRng) -> (Value, u16) {
        let mut playout_depth = 0;
        for _ in 0..90 {
            let move_ = get_heuristic_move(game_state, rng, &self.params);
            if let Some(move_) = move_ {
                game_state.do_move(move_);
            } else {
                break;
            }
            playout_depth += 1;
        }

        (Value::from_game_scores(game_state.scores), playout_depth)
    }
}

pub fn get_heuristic_move(
    game_state: &mut GameState,
    rng: &mut SmallRng,
    params: &HeuristicPlayoutParams,
) -> Option<Move> {
    let mut best_move: Option<Move> = None;
    let mut best_move_score = f32::NEG_INFINITY;

    let is_round_over = game_state.factories.is_empty();
    if is_round_over {
        let is_game_over = game_state.evaluate_round();
        if is_game_over {
            return None;
        }
        game_state.fill_factories(rng);
    }

    let current_player = usize::from(game_state.current_player);

    // Get the pattern lines of the current player, the moves will be placed in the pattern lines
    let player_pattern_line_colors: [Option<TileColor>; 5] =
        game_state.pattern_lines_colors[current_player];

    const P: f64 = 0.8;

    let wall = game_state.walls[current_player];
    let heuristic_state = HeuristicGameState::from_game_state(game_state, current_player);

    // Iterate over all factory and all color combinations
    for (factory_index, factory) in game_state.factories.iter().enumerate() {
        for (color, number) in factory.iter().enumerate() {
            // If there are no tiles of this color in the factory, skip it
            if *number == 0 {
                continue;
            }

            let places = 0;
            let discards = *number;
            let value = evaluate_single_move(params, &heuristic_state, color, discards, places, 5);
            if value > best_move_score && (best_move.is_none() || rng.gen_bool(P)) {
                best_move_score = value;
                best_move = Some(Move {
                    factory_index: factory_index as u8,
                    color: TileColor::from(color as u8),
                    pattern_line_index: 5,
                    discards,
                    places,
                });
            }

            for (pattern_line_index, pattern_line_space) in
                heuristic_state.remaining_space.iter().enumerate()
            {
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
                    let discards = cannot_place;
                    let places = can_place;
                    let value = evaluate_single_move(
                        params,
                        &heuristic_state,
                        color,
                        discards,
                        places,
                        pattern_line_index,
                    );
                    if value > best_move_score && (best_move.is_none() || rng.gen_bool(P)) {
                        best_move_score = value;
                        best_move = Some(Move {
                            factory_index: factory_index as u8,
                            color: TileColor::from(color as u8),
                            pattern_line_index: pattern_line_index as u8,
                            discards,
                            places,
                        });
                    }
                }
            }
        }
    }

    best_move
}

pub trait Flatten {
    fn flatten(&self) -> Vec<f32>;
    fn inflate(array: &[f32], start: usize) -> (Self, usize)
    where
        Self: Sized;
}

impl<const N: usize, const M: usize> Flatten for [[f32; N]; M] {
    fn flatten(&self) -> Vec<f32> {
        self.iter()
            .flat_map(|&arr| arr.iter().cloned().collect::<Vec<_>>())
            .collect()
    }

    fn inflate(array: &[f32], start: usize) -> (Self, usize) {
        let mut result = [[0.0; N]; M];
        let mut index = start;
        for row in result.iter_mut() {
            for elem in row.iter_mut() {
                *elem = array[index];
                index += 1;
            }
        }
        (result, index)
    }
}

impl Flatten for HeuristicPlayoutParams {
    fn flatten(&self) -> Vec<f32> {
        let mut result = Vec::new();
        result.extend(self.discard_penalty.flatten());
        result.extend(self.place_bonus.flatten());
        result.extend(self.fills_entire_pattern_line_bonus.flatten());
        result.extend(self.score_bonus.flatten());
        result
    }

    fn inflate(array: &[f32], start: usize) -> (Self, usize) {
        let (discard_penalty, index) = <[[f32; 8]; 8]>::inflate(array, start);
        let (place_bonus, index) =
            <[[f32; NUM_PATTERN_LINES + 1]; NUM_TILE_COLORS]>::inflate(array, index);
        let (fills_entire_pattern_line_bonus, index) =
            <[[f32; NUM_PATTERN_LINES]; NUM_TILE_COLORS]>::inflate(array, index);
        let (score_bonus, index) = <[[f32; 6]; 20]>::inflate(array, index);

        (
            HeuristicPlayoutParams {
                discard_penalty,
                place_bonus,
                fills_entire_pattern_line_bonus,
                score_bonus,
            },
            index,
        )
    }
}

#[derive(Clone)]
pub struct HeuristicPlayoutParams {
    pub discard_penalty: [[f32; 8]; 8],
    pub place_bonus: [[f32; NUM_PATTERN_LINES + 1]; NUM_TILE_COLORS],
    pub fills_entire_pattern_line_bonus: [[f32; NUM_PATTERN_LINES]; NUM_TILE_COLORS],
    pub score_bonus: [[f32; 6]; 20], // 0..=5 missing tiles (6), 0..=19 score (20)
}

pub const DEFAULT_PARAMS: HeuristicPlayoutParams = HeuristicPlayoutParams {
    discard_penalty: [
        [-0., -1., -1., -2., -2., -2., -3., -3.],
        [-1., -1., -2., -2., -2., -3., -3., -3.],
        [-1., -2., -2., -2., -3., -3., -3., -3.],
        [-2., -2., -2., -3., -3., -3., -3., -3.],
        [-2., -2., -3., -3., -3., -3., -3., -3.],
        [-2., -3., -3., -3., -3., -3., -3., -3.],
        [-3., -3., -3., -3., -3., -3., -3., -3.],
        [-3., -3., -3., -3., -3., -3., -3., -3.],
    ],
    // place_bonus: [[0.1; NUM_PATTERN_LINES + 1]; NUM_TILE_COLORS],
    place_bonus: [
        [0.0, 0.5, 0.5, 0.5, 0.0, 0.0],
        [0.5, 0.5, 0.5, 0.0, 0.0, 0.0],
        [0.5, 0.5, 0.0, 0.0, 0.5, 0.0],
        [0.5, 0.0, 0.0, 0.5, 0.5, 0.0],
        [0.0, 0.0, 0.5, 0.5, 0.5, 0.0],
    ],
    fills_entire_pattern_line_bonus: [[3.0, 6.0, 4.0, 0.8, 3.0]; NUM_TILE_COLORS],
    score_bonus: [
        [0., 0., 0., 0., 0., 0.],
        [1., 0.8, 0.6, 0.4, 0.2, 0.],
        [2., 1.6, 1.2, 0.8, 0.4, 0.],
        [3., 2.4, 1.8, 1.2, 0.6, 0.],
        [4., 3.2, 2.4, 1.6, 0.8, 0.],
        [5., 4., 3., 2., 1., 0.],
        [6., 4.8, 3.6, 2.4, 1.2, 0.],
        [7., 5.6, 4.2, 2.8, 1.4, 0.],
        [8., 6.4, 4.8, 3.2, 1.6, 0.],
        [9., 7.2, 5.4, 3.6, 1.8, 0.],
        [10., 8., 6., 4., 2., 0.],
        [11., 8.8, 6.6, 4.4, 2.2, 0.],
        [12., 9.6, 7.2, 4.8, 2.4, 0.],
        [13., 10.4, 7.8, 5.2, 2.6, 0.],
        [14., 11.2, 8.4, 5.6, 2.8, 0.],
        [15., 12., 9., 6., 3., 0.],
        [16., 12.8, 9.6, 6.4, 3.2, 0.],
        [17., 13.6, 10.2, 6.8, 3.4, 0.],
        [18., 14.4, 10.8, 7.2, 3.6, 0.],
        [19., 15.2, 11.4, 7.6, 3.8, 0.],
    ],
};

pub struct HeuristicGameState {
    pub remaining_space: [u8; 5],
    pub floor_line_progress: usize,
    pub wall_field_score: [[u8; 5]; 5],
    pub missing_tiles: [[u8; 6]; 5],
}

impl HeuristicGameState {
    pub fn from_game_state(game_state: &GameState, player_index: usize) -> Self {
        let player_pattern_lines: [u8; 5] = game_state.pattern_lines_occupancy[player_index];
        let player_pattern_line_colors: [Option<TileColor>; 5] =
            game_state.pattern_lines_colors[player_index];

        let wall = game_state.walls[player_index];
        let wall_after_round =
            calculate_wall_after_round(wall, &player_pattern_lines, &player_pattern_line_colors);
        let (wall_field_score, missing_tiles) = calculate_hypothetical_points_gained(
            wall,
            wall_after_round,
            &player_pattern_line_colors,
            &player_pattern_lines,
        );

        // We will use this to keep track of the remaining space in our pattern lines
        let mut remaining_space: [u8; 5] = [1, 2, 3, 4, 5]; // 255 is the floor line
        for (pattern_line_index, number_of_tiles) in player_pattern_lines.iter().enumerate() {
            // Subtract the number of tiles already in the pattern line from the total space
            remaining_space[pattern_line_index] -= *number_of_tiles;
        }

        Self {
            remaining_space,
            floor_line_progress: (game_state.floor_line_progress[player_index] as usize)
                .min(FLOOR_LINE_PENALTY.len() - 1),
            wall_field_score,
            missing_tiles,
        }
    }
}

pub fn evaluate_single_move(
    params: &HeuristicPlayoutParams,
    state: &HeuristicGameState,
    color: usize,
    discards: u8,
    places: u8,
    pattern_line_index: usize,
) -> f32 {
    let discard_penalty =
        params.discard_penalty[(discards as usize).min(7)][state.floor_line_progress];
    let place_bonus = params.place_bonus[color][pattern_line_index] * places as f32;

    let mut fills_entire_pattern_line_bonus = 0.;
    let mut score_bonus = 0.;
    if pattern_line_index < 5 {
        if state.remaining_space[pattern_line_index] <= places {
            fills_entire_pattern_line_bonus =
                params.fills_entire_pattern_line_bonus[color][pattern_line_index]
        }

        let missing_tiles = state.missing_tiles[color][pattern_line_index]; // 0..=5
        let hypothetical_score_on_completion =
            state.wall_field_score[pattern_line_index][color].min(19) as usize;
        score_bonus = params.score_bonus[hypothetical_score_on_completion][missing_tiles as usize];
    }

    discard_penalty + place_bonus + fills_entire_pattern_line_bonus + score_bonus
}

const SIZE: usize = 239;

pub fn to_input_vec(
    state: &HeuristicGameState,
    color: usize,
    discards: u8,
    places: u8,
    pattern_line_index: usize,
) -> [f32; SIZE] {
    let mut x = [0.0; SIZE];
    let mut index = 0;

    // 1. Discard penalty features
    let discard_index = (discards as usize).min(7);
    x[index + discard_index * 8 + state.floor_line_progress] = 1.0;
    index += 8 * 8;

    // 2. Place bonus features
    x[index + color * (NUM_PATTERN_LINES + 1) + pattern_line_index] = places as f32;
    index += NUM_TILE_COLORS * (NUM_PATTERN_LINES + 1);

    // 3. Fills entire pattern line bonus features
    if pattern_line_index < 5 && state.remaining_space[pattern_line_index] <= places {
        x[index + color * NUM_PATTERN_LINES + pattern_line_index] = 1.0;
    }
    index += NUM_TILE_COLORS * NUM_PATTERN_LINES;

    // 4. Score bonus features
    if pattern_line_index < 5 {
        let missing_tiles = state.missing_tiles[color][pattern_line_index] as usize;
        let hypothetical_score_on_completion =
            state.wall_field_score[pattern_line_index][color].min(19) as usize;
        x[index + hypothetical_score_on_completion * 6 + missing_tiles] = 1.0;
    }

    x
}

pub fn apply_parameters(params: &HeuristicPlayoutParams, x: &[f32; SIZE]) -> f32 {
    let flat = params.flatten();
    let mut sum = 0.;
    for (i, value) in flat.iter().enumerate() {
        sum += value * x[i];
    }

    sum
}

pub fn calculate_wall_after_round(
    wall_occupancy: u32,
    pattern_lines_occupancy: &[u8; 5],
    pattern_line_colors: &[Option<TileColor>; 5],
) -> u32 {
    // Calculate the wall after the end of this round by placing all full pattern lines on the wall
    let mut wall_after_round = wall_occupancy;
    for (pattern_line_index, no_tiles_in_pattern_line) in pattern_lines_occupancy.iter().enumerate()
    {
        if *no_tiles_in_pattern_line as usize != pattern_line_index + 1 {
            continue;
        }
        let color = pattern_line_colors[pattern_line_index].unwrap();
        let color_mask = wall::WALL_COLOR_MASKS[color as usize];
        let row_mask = wall::get_row_mask(pattern_line_index);
        let new_tile = row_mask & color_mask;
        wall_after_round |= new_tile;
    }

    wall_after_round
}

pub fn calculate_hypothetical_points_gained(
    wall_occupancy: u32,
    wall_after_round: u32,
    pattern_line_colors: &[Option<TileColor>; 5],
    pattern_lines_occupancy: &[u8; 5],
) -> ([[u8; 5]; 5], [[u8; 6]; 5]) {
    let mut wall_field_score: [[u8; 5]; 5] = [[0; 5]; 5];
    let mut missing_tiles: [[u8; 6]; 5] = [[0; 6]; 5];

    let current_complete_rows = wall::count_complete_rows(wall_after_round);
    let current_complete_columns = wall::count_complete_columns(wall_after_round);
    let current_full_colors = wall::count_full_colors(wall_after_round);

    for (color, color_mask) in wall::WALL_COLOR_MASKS.iter().enumerate() {
        for row in 0..5 {
            let row_mask = wall::get_row_mask(row);
            let tile = row_mask & color_mask;
            let tile_pos = tile.trailing_zeros();
            let already_occupied = wall_after_round & tile > 0;

            if let Some(line_color) = pattern_line_colors[row] {
                if line_color != TileColor::from(color) || already_occupied {
                    wall_field_score[row][color] = 0;
                    missing_tiles[color][row] = 0;
                } else {
                    missing_tiles[color][row] = row as u8 + 1 - pattern_lines_occupancy[row];
                }
            } else {
                missing_tiles[color][row] = row as u8 + 1;
            }
            if already_occupied {
                wall_field_score[row][color] = 0;
                missing_tiles[color][row] = 0;
                continue;
            }

            let score = get_placed_tile_score(wall_occupancy, tile_pos as u8);
            let wall_with_tile = wall_after_round | tile;
            let new_complete_rows = wall::count_complete_rows(wall_with_tile);
            let row_score = new_complete_rows - current_complete_rows;
            let new_complete_columns = wall::count_complete_columns(wall_with_tile);
            let col_score = new_complete_columns - current_complete_columns;
            let new_full_colors = wall::count_full_colors(wall_with_tile);
            let color_score = new_full_colors - current_full_colors;

            let final_score = score + row_score * 2 + col_score * 7 + color_score * 10;
            wall_field_score[row][color] = final_score as u8;
        }
    }

    (wall_field_score, missing_tiles)
}
