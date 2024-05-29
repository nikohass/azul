use super::value::Value;
use game::*;
use rand::{rngs::SmallRng, Rng};

#[rustfmt::skip]
const PERMUTATIONS: [[u8; 5]; 120] = [[0, 1, 2, 3, 4],[0, 1, 2, 4, 3],[0, 1, 3, 2, 4],[0, 1, 3, 4, 2],[0, 1, 4, 2, 3],[0, 1, 4, 3, 2],[0, 2, 1, 3, 4],[0, 2, 1, 4, 3],[0, 2, 3, 1, 4],[0, 2, 3, 4, 1],[0, 2, 4, 1, 3],[0, 2, 4, 3, 1],[0, 3, 1, 2, 4],[0, 3, 1, 4, 2],[0, 3, 2, 1, 4],[0, 3, 2, 4, 1],[0, 3, 4, 1, 2],[0, 3, 4, 2, 1],[0, 4, 1, 2, 3],[0, 4, 1, 3, 2],[0, 4, 2, 1, 3],[0, 4, 2, 3, 1],[0, 4, 3, 1, 2],[0, 4, 3, 2, 1],[1, 0, 2, 3, 4],[1, 0, 2, 4, 3],[1, 0, 3, 2, 4],[1, 0, 3, 4, 2],[1, 0, 4, 2, 3],[1, 0, 4, 3, 2],[1, 2, 0, 3, 4],[1, 2, 0, 4, 3],[1, 2, 3, 0, 4],[1, 2, 3, 4, 0],[1, 2, 4, 0, 3],[1, 2, 4, 3, 0],[1, 3, 0, 2, 4],[1, 3, 0, 4, 2],[1, 3, 2, 0, 4],[1, 3, 2, 4, 0],[1, 3, 4, 0, 2],[1, 3, 4, 2, 0],[1, 4, 0, 2, 3],[1, 4, 0, 3, 2],[1, 4, 2, 0, 3],[1, 4, 2, 3, 0],[1, 4, 3, 0, 2],[1, 4, 3, 2, 0],[2, 0, 1, 3, 4],[2, 0, 1, 4, 3],[2, 0, 3, 1, 4],[2, 0, 3, 4, 1],[2, 0, 4, 1, 3],[2, 0, 4, 3, 1],[2, 1, 0, 3, 4],[2, 1, 0, 4, 3],[2, 1, 3, 0, 4],[2, 1, 3, 4, 0],[2, 1, 4, 0, 3],[2, 1, 4, 3, 0],[2, 3, 0, 1, 4],[2, 3, 0, 4, 1],[2, 3, 1, 0, 4],[2, 3, 1, 4, 0],[2, 3, 4, 0, 1],[2, 3, 4, 1, 0],[2, 4, 0, 1, 3],[2, 4, 0, 3, 1],[2, 4, 1, 0, 3],[2, 4, 1, 3, 0],[2, 4, 3, 0, 1],[2, 4, 3, 1, 0],[3, 0, 1, 2, 4],[3, 0, 1, 4, 2],[3, 0, 2, 1, 4],[3, 0, 2, 4, 1],[3, 0, 4, 1, 2],[3, 0, 4, 2, 1],[3, 1, 0, 2, 4],[3, 1, 0, 4, 2],[3, 1, 2, 0, 4],[3, 1, 2, 4, 0],[3, 1, 4, 0, 2],[3, 1, 4, 2, 0],[3, 2, 0, 1, 4],[3, 2, 0, 4, 1],[3, 2, 1, 0, 4],[3, 2, 1, 4, 0],[3, 2, 4, 0, 1],[3, 2, 4, 1, 0],[3, 4, 0, 1, 2],[3, 4, 0, 2, 1],[3, 4, 1, 0, 2],[3, 4, 1, 2, 0],[3, 4, 2, 0, 1],[3, 4, 2, 1, 0],[4, 0, 1, 2, 3],[4, 0, 1, 3, 2],[4, 0, 2, 1, 3],[4, 0, 2, 3, 1],[4, 0, 3, 1, 2],[4, 0, 3, 2, 1],[4, 1, 0, 2, 3],[4, 1, 0, 3, 2],[4, 1, 2, 0, 3],[4, 1, 2, 3, 0],[4, 1, 3, 0, 2],[4, 1, 3, 2, 0],[4, 2, 0, 1, 3],[4, 2, 0, 3, 1],[4, 2, 1, 0, 3],[4, 2, 1, 3, 0],[4, 2, 3, 0, 1],[4, 2, 3, 1, 0],[4, 3, 0, 1, 2],[4, 3, 0, 2, 1],[4, 3, 1, 0, 2],[4, 3, 1, 2, 0],[4, 3, 2, 0, 1],[4, 3, 2, 1, 0],];

pub fn playout(mut game_state: GameState, rng: &mut SmallRng) -> (Value, u16) {
    #[cfg(debug_assertions)]
    game_state
        .check_integrity()
        .expect("Game state integrity check failed before playout");

    // There are situations where every single player is only able to discard tiles
    // In this case, the game is in a infinite loop and we should break out of it
    let mut playout_depth = 0;
    for _ in 0..90 {
        match get_random_move(&mut game_state, rng) {
            Some(move_) => {
                game_state.do_move(move_);
            }
            None => break,
        }
        playout_depth += 1;
    }

    (Value::from_game_scores(game_state.scores), playout_depth)
}

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
