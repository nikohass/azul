use game::{
    hash_factory, NUM_NON_CENTER_FACTORIES, NUM_POSSIBLE_FACTORY_PERMUTATIONS, NUM_TILE_COLORS,
};

use super::super::layers::InputLayer;

pub const MAX_TILES_OF_COLOR_IN_CENTER: usize = 15;
pub const NON_CENTER_FACTORY_ENCODING_SIZE: usize =
    NUM_NON_CENTER_FACTORIES * NUM_POSSIBLE_FACTORY_PERMUTATIONS; // there are 71 possible combinations of tiles in a factory and NUM_NON_CENTER_FACTORIES opportunities for duplicates
pub const CENTER_FACTORY_ENCODING_SIZE: usize = NUM_TILE_COLORS * MAX_TILES_OF_COLOR_IN_CENTER;
pub const FACTORY_ENCODING_SIZE: usize =
    NON_CENTER_FACTORY_ENCODING_SIZE + CENTER_FACTORY_ENCODING_SIZE;

pub fn add_non_center_factory_encoding(
    factory: &[u8; NUM_TILE_COLORS],
    multi_factory_counter: &mut [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
    layer: &mut dyn InputLayer,
) {
    let factory_hash = hash_factory(factory); // Index of the factory

    let old_count = multi_factory_counter[factory_hash]; // Number of factories with the same hash
    let new_count = old_count + 1; // Increase since we are adding a new factory
    multi_factory_counter[factory_hash] = new_count;
    if old_count != 0 {
        layer.unset_input(factory_hash * NUM_TILE_COLORS + old_count - 1);
    }
    layer.set_input(factory_hash * NUM_TILE_COLORS + new_count - 1);
}

pub fn remove_non_center_factory_encoding(
    factory: &[u8; NUM_TILE_COLORS],
    multi_factory_counter: &mut [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
    layer: &mut dyn InputLayer,
) {
    let factory_hash = hash_factory(factory); // Index of the factory

    let old_count = multi_factory_counter[factory_hash]; // Number of factories with the same hash
    let new_count = old_count - 1; // Decrease since we are removing a factory
    multi_factory_counter[factory_hash] = new_count;
    layer.set_input(factory_hash * NUM_TILE_COLORS + old_count - 1);
    if new_count != 0 {
        layer.unset_input(factory_hash * NUM_TILE_COLORS + new_count - 1);
    }
}

pub fn get_center_factory_index(num_tiles: usize, tile_color: usize) -> usize {
    NON_CENTER_FACTORY_ENCODING_SIZE + tile_color * MAX_TILES_OF_COLOR_IN_CENTER + num_tiles
}

pub fn add_center_factory_encoding(
    num_tiles: usize,
    tile_color: usize,
    layer: &mut dyn InputLayer,
) {
    let index = get_center_factory_index(num_tiles, tile_color);
    layer.set_input(index);
}

pub fn remove_center_factory_encoding(
    num_tiles: usize,
    tile_color: usize,
    layer: &mut dyn InputLayer,
) {
    let index = get_center_factory_index(num_tiles, tile_color);
    layer.unset_input(index);
}
