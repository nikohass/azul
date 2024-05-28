use game::{
    hash_factory, NUM_NON_CENTER_FACTORIES, NUM_POSSIBLE_FACTORY_PERMUTATIONS, NUM_TILE_COLORS,
};

use super::layers::InputLayer;

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
    // InputModification {
    //     remove_index: if old_count != 0 {
    //         Some(factory_hash * NUM_TILE_COLORS + old_count - 1)
    //     } else {
    //         None
    //     },
    //     add_index: Some(factory_hash * NUM_TILE_COLORS + new_count - 1),
    // }
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
    // InputModification {
    //     remove_index: Some(factory_hash * NUM_TILE_COLORS + old_count - 1),
    //     // add_index: factory_hash * NUM_TILE_COLORS + new_count - 1,
    //     add_index: if new_count != 0 {
    //         Some(factory_hash * NUM_TILE_COLORS + new_count - 1)
    //     } else {
    //         None
    //     },
    // }
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

// #[cfg(test)]
// mod tests {
//     use crate::mcts::neural_network::hashing::unhash_factory;

//     use super::*;

//     #[test]
//     fn test_add_duplicate_non_center_factory() {
//         let mut multi_factory_counter = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
//         let factory = unhash_factory(0);
//         let modification = add_non_center_factory(&factory, &mut multi_factory_counter);

//         assert_eq!(modification.remove_index, None);
//         assert_eq!(modification.add_index, Some(0));

//         let modification = add_non_center_factory(&factory, &mut multi_factory_counter);
//         assert_eq!(modification.remove_index, Some(0));
//         assert_eq!(modification.add_index, Some(1));
//     }

//     #[test]
//     fn test_add_factory_bounds() {
//         println!("Encoding size: {}", NON_CENTER_FACTORY_ENCODING_SIZE);
//         let mut multi_factory_counter = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
//         for _ in 0..NUM_NON_CENTER_FACTORIES {
//             for factory_hash in 0..NUM_POSSIBLE_FACTORY_PERMUTATIONS {
//                 let factory = unhash_factory(factory_hash);
//                 let modification = add_non_center_factory(&factory, &mut multi_factory_counter);
//                 if let Some(remove_index) = modification.remove_index {
//                     assert!(
//                         remove_index < NON_CENTER_FACTORY_ENCODING_SIZE,
//                         "remove_index: {}, factory_hash: {}",
//                         remove_index,
//                         factory_hash
//                     );
//                 }

//                 if let Some(add_index) = modification.add_index {
//                     assert!(
//                         add_index < NON_CENTER_FACTORY_ENCODING_SIZE,
//                         "add_index: {}, factory_hash: {}",
//                         add_index,
//                         factory_hash
//                     );
//                 }
//             }
//         }
//         assert_eq!(
//             multi_factory_counter.iter().sum::<usize>(),
//             NUM_NON_CENTER_FACTORIES * NUM_POSSIBLE_FACTORY_PERMUTATIONS
//         );
//     }

//     #[test]
//     fn test_add_remove() {
//         let mut multi_factory_counter = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
//         let factory = unhash_factory(0);

//         let add_modification = add_non_center_factory(&factory, &mut multi_factory_counter);
//         let remove_modification = remove_non_center_factory(&factory, &mut multi_factory_counter);

//         println!("add_modification: {:#?}", add_modification);
//         println!("remove_modification: {:#?}", remove_modification);

//         if let Some(remove_index) = add_modification.remove_index {
//             assert_eq!(remove_index, remove_modification.add_index.unwrap());
//         }
//         if let Some(add_index) = add_modification.add_index {
//             assert_eq!(add_index, remove_modification.remove_index.unwrap());
//         }

//         assert_eq!(
//             multi_factory_counter,
//             [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS]
//         );
//     }

//     #[test]
//     fn test_dead_input_neurons() {
//         let mut add_visited = [false; NON_CENTER_FACTORY_ENCODING_SIZE];
//         let mut remove_visited = [false; NON_CENTER_FACTORY_ENCODING_SIZE];
//         let mut multi_factory_counter = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
//         for factory_hash in 0..NUM_POSSIBLE_FACTORY_PERMUTATIONS {
//             let factory = unhash_factory(factory_hash);

//             for _ in 0..NUM_NON_CENTER_FACTORIES {
//                 let modification = add_non_center_factory(&factory, &mut multi_factory_counter);
//                 add_visited[modification.add_index.unwrap()] = true;
//             }

//             for _ in 0..NUM_NON_CENTER_FACTORIES {
//                 let modification = remove_non_center_factory(&factory, &mut multi_factory_counter);
//                 remove_visited[modification.remove_index.unwrap()] = true;
//             }
//         }

//         for i in 0..NON_CENTER_FACTORY_ENCODING_SIZE {
//             assert!(add_visited[i]);
//             assert!(remove_visited[i]);
//         }
//     }
// }
