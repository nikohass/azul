use game::{wall::WALL_COLOR_MASKS, *};
use rand::{rngs::SmallRng, Rng};

use super::value::Value;

pub fn playout(game_state: &mut GameState, rng: &mut SmallRng) -> Value {
    loop {
        match get_random_move(game_state, rng) {
            None => {
                return Value::from_game_scores(game_state.get_scores());
            }
            Some(move_) => game_state.do_move(move_),
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
    let pattern_lines = game_state.get_pattern_lines_occupancy()[current_player];
    let pattern_line_colors = game_state.get_pattern_lines_colors()[current_player];
    let wall_occupancy = game_state.get_wall_ocupancy()[current_player];
    let factories = game_state.get_factories();

    let mut possible_takes: Vec<(usize, TileColor, u8)> = Vec::new();
    for (factory_index, factory_content) in factories.iter().enumerate() {
        for (tile_color, number_of_tiles) in factory_content.iter().enumerate() {
            if *number_of_tiles == 0 {
                continue;
            }

            let color = TileColor::from(tile_color);
            possible_takes.push((factory_index, color, *number_of_tiles));
        }
    }

    let (factory_index, color, mut number_of_tiles) =
        possible_takes[rng.gen_range(0..possible_takes.len())];

    let mut remaining_space: [u8; 6] = [1, 2, 3, 4, 5, 255];
    for (pattern_line_index, number_of_tiles) in pattern_lines.iter().enumerate() {
        remaining_space[pattern_line_index] -= *number_of_tiles;

        if let Some(existing_color) = pattern_line_colors[pattern_line_index] {
            if color != existing_color {
                remaining_space[pattern_line_index] = 0;
            }
        } else {
            let wall_mask = WALL_COLOR_MASKS[color as usize];
            let row_mask: u32 = wall::get_row_mask(pattern_line_index);
            if wall_occupancy & row_mask & wall_mask > 0 {
                remaining_space[pattern_line_index] = 0;
            }
        }
    }
    let total_remaining_space: u8 = remaining_space.iter().take(5).sum();
    let mut pattern = [0; 6];
    if total_remaining_space <= number_of_tiles {
        // We must discard tiles here
        pattern = remaining_space;
        pattern[5] = number_of_tiles - total_remaining_space;
    } else {
        // Select a pattern line to fill
        while number_of_tiles > 0 {
            let pattern_line_index = rng.gen_range(0..5);
            if remaining_space[pattern_line_index] == 0 {
                continue;
            }
            let tiles_to_place = u8::min(number_of_tiles, remaining_space[pattern_line_index]);
            pattern[pattern_line_index] += tiles_to_place;
            number_of_tiles -= tiles_to_place;
            remaining_space[pattern_line_index] -= tiles_to_place;
        }
    }

    Some(Move {
        take_from_factory_index: factory_index as u8,
        color,
        pattern,
    })
}
