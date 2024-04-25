use super::value::Value;
use game::{wall::get_placed_tile_score, *};
use rand::{rngs::SmallRng, Rng, SeedableRng as _};

#[rustfmt::skip]
const PERMUTATIONS: [[u8; 5]; 120] = [[0, 1, 2, 3, 4],[0, 1, 2, 4, 3],[0, 1, 3, 2, 4],[0, 1, 3, 4, 2],[0, 1, 4, 2, 3],[0, 1, 4, 3, 2],[0, 2, 1, 3, 4],[0, 2, 1, 4, 3],[0, 2, 3, 1, 4],[0, 2, 3, 4, 1],[0, 2, 4, 1, 3],[0, 2, 4, 3, 1],[0, 3, 1, 2, 4],[0, 3, 1, 4, 2],[0, 3, 2, 1, 4],[0, 3, 2, 4, 1],[0, 3, 4, 1, 2],[0, 3, 4, 2, 1],[0, 4, 1, 2, 3],[0, 4, 1, 3, 2],[0, 4, 2, 1, 3],[0, 4, 2, 3, 1],[0, 4, 3, 1, 2],[0, 4, 3, 2, 1],[1, 0, 2, 3, 4],[1, 0, 2, 4, 3],[1, 0, 3, 2, 4],[1, 0, 3, 4, 2],[1, 0, 4, 2, 3],[1, 0, 4, 3, 2],[1, 2, 0, 3, 4],[1, 2, 0, 4, 3],[1, 2, 3, 0, 4],[1, 2, 3, 4, 0],[1, 2, 4, 0, 3],[1, 2, 4, 3, 0],[1, 3, 0, 2, 4],[1, 3, 0, 4, 2],[1, 3, 2, 0, 4],[1, 3, 2, 4, 0],[1, 3, 4, 0, 2],[1, 3, 4, 2, 0],[1, 4, 0, 2, 3],[1, 4, 0, 3, 2],[1, 4, 2, 0, 3],[1, 4, 2, 3, 0],[1, 4, 3, 0, 2],[1, 4, 3, 2, 0],[2, 0, 1, 3, 4],[2, 0, 1, 4, 3],[2, 0, 3, 1, 4],[2, 0, 3, 4, 1],[2, 0, 4, 1, 3],[2, 0, 4, 3, 1],[2, 1, 0, 3, 4],[2, 1, 0, 4, 3],[2, 1, 3, 0, 4],[2, 1, 3, 4, 0],[2, 1, 4, 0, 3],[2, 1, 4, 3, 0],[2, 3, 0, 1, 4],[2, 3, 0, 4, 1],[2, 3, 1, 0, 4],[2, 3, 1, 4, 0],[2, 3, 4, 0, 1],[2, 3, 4, 1, 0],[2, 4, 0, 1, 3],[2, 4, 0, 3, 1],[2, 4, 1, 0, 3],[2, 4, 1, 3, 0],[2, 4, 3, 0, 1],[2, 4, 3, 1, 0],[3, 0, 1, 2, 4],[3, 0, 1, 4, 2],[3, 0, 2, 1, 4],[3, 0, 2, 4, 1],[3, 0, 4, 1, 2],[3, 0, 4, 2, 1],[3, 1, 0, 2, 4],[3, 1, 0, 4, 2],[3, 1, 2, 0, 4],[3, 1, 2, 4, 0],[3, 1, 4, 0, 2],[3, 1, 4, 2, 0],[3, 2, 0, 1, 4],[3, 2, 0, 4, 1],[3, 2, 1, 0, 4],[3, 2, 1, 4, 0],[3, 2, 4, 0, 1],[3, 2, 4, 1, 0],[3, 4, 0, 1, 2],[3, 4, 0, 2, 1],[3, 4, 1, 0, 2],[3, 4, 1, 2, 0],[3, 4, 2, 0, 1],[3, 4, 2, 1, 0],[4, 0, 1, 2, 3],[4, 0, 1, 3, 2],[4, 0, 2, 1, 3],[4, 0, 2, 3, 1],[4, 0, 3, 1, 2],[4, 0, 3, 2, 1],[4, 1, 0, 2, 3],[4, 1, 0, 3, 2],[4, 1, 2, 0, 3],[4, 1, 2, 3, 0],[4, 1, 3, 0, 2],[4, 1, 3, 2, 0],[4, 2, 0, 1, 3],[4, 2, 0, 3, 1],[4, 2, 1, 0, 3],[4, 2, 1, 3, 0],[4, 2, 3, 0, 1],[4, 2, 3, 1, 0],[4, 3, 0, 1, 2],[4, 3, 0, 2, 1],[4, 3, 1, 0, 2],[4, 3, 1, 2, 0],[4, 3, 2, 0, 1],[4, 3, 2, 1, 0],];

pub fn playout(mut game_state: GameState, rng: &mut SmallRng) -> Value {
    game_state
        .check_integrity()
        .expect("Game state integrity check failed before playout");
    let mut move_count = 0;
    loop {
        match get_random_move(&mut game_state, rng) {
            None => {
                return Value::from_game_scores(game_state.get_scores());
            }
            Some(move_) => {
                game_state.do_move(move_);
            }
        }
        // There are situations where every single player is only able to discard tiles
        // In this case, the game is in a infinite loop and we should break out of it
        move_count += 1;
        if move_count > 90 {
            // Max realistic game lenght is 85 moves
            return Value::from_game_scores(game_state.get_scores());
        }
    }
}

pub fn get_random_move(game_state: &mut GameState, rng: &mut SmallRng) -> Option<Move> {
    let is_round_over = game_state.get_factories().is_empty();

    if is_round_over {
        let is_game_over = game_state.evaluate_round();

        if is_game_over {
            return None;
        }

        game_state.fill_factories(rng);
    }

    let current_player: usize = game_state.get_current_player().into();
    let pattern_line_colors = game_state.get_pattern_lines_colors()[current_player];
    let pattern_lines_occupancy = game_state.get_pattern_lines_occupancy()[current_player];
    let wall_occupancy = game_state.get_wall_ocupancy()[current_player];
    let factories = game_state.get_factories();

    // Calculate the wall after the end of this round by placing all full pattern lines on the wall already
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

    // Calculate the score we gain by placing a tile in each field of the wall and how many tiles we are missing in the pattern lines
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

    // println!("Tile Scores:\n");
    // for color in 0..NUM_TILE_COLORS {
    //     let color = TileColor::from(color);
    //     print!("   {}", color);
    // }
    // println!();
    // for line in wall_field_importance.iter() {
    //     print!(" ");
    //     for field in line.iter() {
    //         print!("{:3} ", field);
    //     }
    //     println!();
    // }

    // println!("Missing tiles\n");
    // for (color, line) in missing_tiles.iter().enumerate() {
    //     let color = TileColor::from(color as u8);
    //     print!("{}", color);
    //     for field in line.iter() {
    //         print!("{:3} ", field);
    //     }
    //     println!();
    // }
    // println!("Possible takes: {:?}", possible_takes);

    let mut best_score = f32::NEG_INFINITY;
    let mut best_move = None;
    let floor_line_progress = game_state.get_floor_line_progress()[current_player]
        .min(FLOOR_LINE_PENALTY.len() as u8 - 1) as usize;
    let floor_line_penalty = FLOOR_LINE_PENALTY[floor_line_progress];

    for (factory_index, factory_content) in factories.iter().enumerate() {
        for (tile_color, number_of_tiles) in factory_content.iter().enumerate() {
            let mut number_of_tiles = *number_of_tiles;
            if number_of_tiles == 0 {
                continue;
            }

            let color = TileColor::from(tile_color);

            let mut tiles_to_discard = 0;
            let mut score: f32 = 0.0;
            let maximum_tiles = missing_tiles[tile_color].iter().sum();

            if number_of_tiles > maximum_tiles {
                tiles_to_discard = number_of_tiles - maximum_tiles;
                number_of_tiles = maximum_tiles;
            }

            let mut pattern: [u8; 6] = [0; 6];
            if maximum_tiles == 0 {
                tiles_to_discard += number_of_tiles;
            } else if number_of_tiles == maximum_tiles {
                pattern = missing_tiles[tile_color];
                score += wall_field_score[tile_color].iter().sum::<u8>() as f32;
            } else {
                let order = PERMUTATIONS[rng.gen_range(0..PERMUTATIONS.len())];
                for pattern_line_index in order.iter() {
                    let pattern_line_index = *pattern_line_index as usize;
                    let missing_tiles = missing_tiles[tile_color][pattern_line_index];
                    if missing_tiles == 0 {
                        continue;
                    }
                    pattern[pattern_line_index] = u8::min(number_of_tiles, missing_tiles);
                    if pattern[pattern_line_index] == missing_tiles {
                        score += wall_field_score[pattern_line_index][tile_color] as f32;
                    }
                    number_of_tiles -= pattern[pattern_line_index];
                    if number_of_tiles == 0 {
                        break;
                    }
                }
                tiles_to_discard += number_of_tiles;
            }
            pattern[5] = tiles_to_discard;

            let new_floor_line_progress = floor_line_progress + tiles_to_discard as usize;
            let new_floor_line_penalty =
                FLOOR_LINE_PENALTY[new_floor_line_progress.min(FLOOR_LINE_PENALTY.len() - 1)];
            let difference = new_floor_line_penalty - floor_line_penalty;
            score -= difference as f32;

            if score > best_score && (rng.gen_bool(0.25) || best_move.is_none()) {
                best_score = score;
                best_move = Some((factory_index, color, pattern));
            }
        }
    }

    if let Some((factory_index, color, pattern)) = best_move {
        let move_ = Move {
            take_from_factory_index: factory_index as u8,
            color,
            pattern,
        };
        Some(move_)
    } else {
        None
    }
}

pub struct HeuristicMoveGenerationPlayer {
    rng: SmallRng,
}

impl Default for HeuristicMoveGenerationPlayer {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
        }
    }
}

#[async_trait::async_trait]
impl Player for HeuristicMoveGenerationPlayer {
    async fn get_move(&mut self, game_state: &GameState) -> Move {
        let mut game_state = game_state.clone();
        get_random_move(&mut game_state, &mut self.rng).unwrap()
    }

    fn get_name(&self) -> &str {
        "HeuristicMoveGenerationPlayer"
    }
}
