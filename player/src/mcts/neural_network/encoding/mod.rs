use self::{
    factory_encoding::{add_center_factory_encoding, add_non_center_factory_encoding},
    pattern_line_encoding::add_pattern_line_encoding,
    wall_encoding::add_wall_encoding,
};
use super::layers::InputLayer;
use factory_encoding::FACTORY_ENCODING_SIZE;
use game::{
    GameState, CENTER_FACTORY_INDEX, INDEX_TO_FACTORY, NUM_NON_CENTER_FACTORIES, NUM_PLAYERS,
    NUM_POSSIBLE_FACTORY_PERMUTATIONS, NUM_TILE_COLORS,
};
use pattern_line_encoding::PLAYER_PATTERN_LINE_ENCODING_SIZE;
use score_encoding::{add_player_score_encoding, PLAYER_SCORE_ENCODING_SIZE};
use std::collections::{HashMap, HashSet};
use wall_encoding::PLAYER_WALL_ENCODING_SIZE;

pub mod factory_encoding;
pub mod pattern_line_encoding;
pub mod score_encoding;
pub mod wall_encoding;

pub const TOTAL_ENCODING_SIZE: usize = FACTORY_ENCODING_SIZE
    + (PLAYER_PATTERN_LINE_ENCODING_SIZE + PLAYER_WALL_ENCODING_SIZE + PLAYER_SCORE_ENCODING_SIZE)
        * NUM_PLAYERS;

pub fn encode_game_state(
    game_state: &GameState,
    layer: &mut dyn InputLayer,
    multi_factory_counter: &mut [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
) {
    // Encode factories
    for factory in game_state.factories.iter().take(NUM_NON_CENTER_FACTORIES) {
        add_non_center_factory_encoding(factory, multi_factory_counter, layer);
    }

    // Encode center factory
    let center_factory = &game_state.factories[CENTER_FACTORY_INDEX];
    for (tile_color, num_tiles) in center_factory.iter().enumerate() {
        add_center_factory_encoding(*num_tiles as usize, tile_color, layer);
    }

    // TODO: Encode the number of tiles in and out of bag

    // Encode players
    for player_index in 0..NUM_PLAYERS {
        // Encode pattern lines
        for (pattern_line_index, pattern_line_color) in game_state.pattern_lines_colors
            [player_index]
            .iter()
            .enumerate()
        {
            let num_tiles = game_state.pattern_lines_occupancy[player_index][pattern_line_index];
            add_pattern_line_encoding(
                pattern_line_index,
                num_tiles as usize,
                *pattern_line_color,
                player_index,
                layer,
            )
        }

        // Encode wall
        add_wall_encoding(game_state.walls[player_index], player_index, layer);

        // TODO: Encode floor line and player scores
        let is_next_round_starting_player = usize::from(game_state.next_round_starting_player)
            == player_index
            && game_state.tile_taken_from_center;
        add_player_score_encoding(
            player_index,
            game_state.scores[player_index],
            game_state.floor_line_progress[player_index] as usize,
            layer,
            is_next_round_starting_player,
        )
    }
}

pub fn build_move_lookup() -> HashMap<(usize, u8, u8), usize> {
    let mut move_index_set = HashSet::new();
    for (factory_index, factory) in INDEX_TO_FACTORY.iter().enumerate() {
        for (tile_color, &tile_count) in factory.iter().enumerate() {
            if tile_count > 0 {
                for pattern_line_index in 0..6 {
                    let key = (factory_index, tile_color as u8, pattern_line_index as u8);
                    move_index_set.insert(key);
                }
            }
        }
    }

    for tile_color in 0..NUM_TILE_COLORS {
        for pattern_line_index in 0..6 {
            let key = (INDEX_TO_FACTORY.len(), tile_color as u8, pattern_line_index);
            move_index_set.insert(key);
        }
    }

    let mut move_to_index = HashMap::new();
    for (index, key) in move_index_set.into_iter().enumerate() {
        move_to_index.insert(key, index);
    }

    move_to_index
}
