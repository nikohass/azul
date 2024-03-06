use game::*;
use rand::{rngs::SmallRng, Rng};

use super::value::Value;

const MOVE_GENERATION_RETRIES: usize = 5;

pub fn playout(game_state: &mut GameState, rng: &mut SmallRng, move_list: &mut MoveList) -> Value {
    loop {
        match get_random_move(game_state, rng, move_list) {
            None => {
                return Value::from_game_scores(game_state.get_scores());
            }
            Some(move_) => game_state.do_move(move_),
        }
    }
}

#[inline]
fn get_not_empty_factories(factories: &[[u8; 5]; NUM_FACTORIES]) -> Vec<u8> {
    factories
        .iter()
        .enumerate()
        .filter(|(_, factory)| factory.iter().any(|&tile| tile != 0))
        .map(|(i, _)| i as u8)
        .collect()
}

#[inline]
fn get_fallback_move(
    game_state: &mut GameState,
    rng: &mut SmallRng,
    move_list: &mut MoveList,
) -> Option<Move> {
    let (is_game_over, _) = game_state.get_possible_moves(move_list, rng);
    if is_game_over {
        None
    } else {
        Some(move_list[rng.gen_range(0..move_list.len())])
    }
}

#[inline]
fn get_player_wall_data(
    game_state: &GameState,
    player_index: usize,
) -> ([u8; 5], [Option<game::TileColor>; 5], u32) {
    (
        game_state.get_pattern_lines_occupancy()[player_index],
        game_state.get_pattern_lines_colors()[player_index],
        game_state.get_wall_ocupancy()[player_index],
    )
}

fn attempt_move_creation(
    rng: &mut SmallRng,
    factories: &[[u8; 5]; NUM_FACTORIES],
    not_empty_factories: &[u8],
    pattern_lines: &[u8; 5],
    pattern_line_colors: &[Option<game::TileColor>; 5],
    wall_occupancy: u32,
) -> Option<Move> {
    // Select a random factory and color to take
    let factory_index = not_empty_factories[rng.gen_range(0..not_empty_factories.len())] as usize;
    let mut tile_color = rng.gen_range(0..5);
    while factories[factory_index][tile_color] == 0 {
        tile_color = rng.gen_range(0..5);
    }
    let number_of_tiles_to_distribute = factories[factory_index][tile_color];
    // Calculate the remaining space in our pattern lines for each color
    let mut remaining_space: [u8; 6] = [1, 2, 3, 4, 5, 255]; // 255 is a placeholder for the floor line
    let mut total_remaining_space = 0;
    for (pattern_line_index, number_of_tiles) in pattern_lines.iter().enumerate() {
        remaining_space[pattern_line_index] -= *number_of_tiles; // We subtract the number of tiles already in the pattern line from the total space
                                                                 // If there are tiles of a different color in the patternline already, we can't put any more tiles in it, so we set the remaining space to 0
        if let Some(existing_color) = pattern_line_colors[pattern_line_index] {
            if tile_color != usize::from(existing_color) {
                remaining_space[pattern_line_index] = 0;
            }
        } else {
            // If the pattern line did not have a color yet, we need to check whether we are allowed to place this color here
            // It is not possible to place a tile in a pattern line if the corresponding row in the wall is already full
            let wall_mask = game::wall::WALL_COLOR_MASKS[tile_color];
            let row_mask = game::wall::get_row_mask(pattern_line_index);
            if wall_occupancy & row_mask & wall_mask > 0 {
                remaining_space[pattern_line_index] = 0;
            }
        }
        if remaining_space[pattern_line_index] == number_of_tiles_to_distribute {
            // Heuristic: If we find a pattern line that we can fill completely, we will do that
            let mut pattern = [0; 6];
            pattern[pattern_line_index] = number_of_tiles_to_distribute;
            return Some(Move {
                take_from_factory_index: factory_index as u8,
                color: game::TileColor::from(tile_color),
                pattern,
            });
        }
        total_remaining_space += remaining_space[pattern_line_index];
    }
    if total_remaining_space < number_of_tiles_to_distribute {
        // We do not want to be in this situation, because it means that we will have to put tiles on the floor line
        return None; // Skip this iteration and try again
    }

    // TODO: Distribute the tiles over the pattern lines in a smart way

    None
}

#[inline]
pub fn get_random_move(
    game_state: &mut GameState,
    rng: &mut SmallRng,
    move_list: &mut MoveList,
) -> Option<Move> {
    // Returns a random move from the given game state or None if there are no possible moves
    // We use the game state to generate one possible random move.
    // In case we fail MOVE_GENERATION_RETRIES times, we fall back to the entire move generation and pick a random move from there.
    // By doing this kind of move generation we will alter the probabilities of the moves
    // we should make sure that the probabilities are altered in a way that is beneficial to us
    // e.g. we should pick moves that are more likely to be good for us, to make the playout more realistic

    let factories: &[[u8; 5]; NUM_FACTORIES] = game_state.get_factories();
    let not_empty_factories = get_not_empty_factories(factories);
    if not_empty_factories.is_empty() {
        // There are no non-empty factories, we can't generate a move. Fall back to default move generation
        return get_fallback_move(game_state, rng, move_list);
    }

    // There are non-empty factories, we can proceed to pick a random move
    let current_player: usize = usize::from(game_state.get_current_player());
    let (pattern_lines, pattern_line_colors, wall_occupancy) =
        get_player_wall_data(game_state, current_player);

    for _ in 0..MOVE_GENERATION_RETRIES {
        if let Some(move_) = attempt_move_creation(
            rng,
            factories,
            &not_empty_factories,
            &pattern_lines,
            &pattern_line_colors,
            wall_occupancy,
        ) {
            return Some(move_);
        }
    }

    // Fallback to default move generation
    let (is_game_over, _) = game_state.get_possible_moves(move_list, rng);
    if is_game_over {
        None
    } else {
        Some(move_list[rng.gen_range(0..move_list.len())])
    }
}
