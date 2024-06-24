use game::{
    hash_factory, Factories, NUM_NON_CENTER_FACTORIES, NUM_POSSIBLE_FACTORY_PERMUTATIONS,
    NUM_TILE_COLORS,
};

use crate::mcts::neural_network::layers::InputLayer;

use super::OneHotFeature;

pub const NON_CENTER_FACTORY_ENCODING_SIZE: usize =
    NUM_POSSIBLE_FACTORY_PERMUTATIONS * NUM_NON_CENTER_FACTORIES; // TODO: Add center

pub const MAX_TILES_OF_COLOR_IN_CENTER: usize = 10; // Ignore everything above 10
pub const CENTER_FACTORY_ENCODING_SIZE: usize = NUM_TILE_COLORS * MAX_TILES_OF_COLOR_IN_CENTER;

pub const FACTORY_ENCODING_SIZE: usize =
    NON_CENTER_FACTORY_ENCODING_SIZE + CENTER_FACTORY_ENCODING_SIZE;

pub fn set_center_factory_index(
    num_tiles: usize,
    tile_color: usize,
    center_factory_counts: &mut [usize; NUM_TILE_COLORS],
) -> Option<(usize, usize)> {
    // Returns index to set and to unset

    let current_count = center_factory_counts[tile_color];
    if current_count != num_tiles {
        let set = num_tiles * NUM_TILE_COLORS + tile_color;
        let unset = current_count * NUM_TILE_COLORS + tile_color;
        center_factory_counts[tile_color] = num_tiles;
        Some((set, unset))
    } else {
        None
    }
}

#[derive(Clone)]
pub struct CenterFactoryEncoding {
    center_factory_counts: [usize; NUM_TILE_COLORS],
}

impl OneHotFeature for CenterFactoryEncoding {
    const SIZE: usize = CENTER_FACTORY_ENCODING_SIZE;
    const PLAYER_FEATURE: bool = false;
    const MAX_ONES: usize = 5;
    const START: usize = 0;

    fn initialize(layer: &mut impl InputLayer) -> Self {
        for i in 0..NUM_TILE_COLORS {
            let set = i + Self::START;
            layer.set_input(set);
        }
        Self {
            center_factory_counts: [0; NUM_TILE_COLORS],
        }
    }
}

impl CenterFactoryEncoding {
    pub fn set_for_color(
        &mut self,
        num_tiles: usize,
        tile_color: usize,
        layer: &mut impl InputLayer,
    ) {
        if let Some((set, unset)) =
            set_center_factory_index(num_tiles, tile_color, &mut self.center_factory_counts)
        {
            layer.set_input(set);
            layer.unset_input(unset);
        }
    }

    pub fn set(&mut self, center_factory: &[u8; NUM_TILE_COLORS], layer: &mut impl InputLayer) {
        for (tile_color, num_tiles) in center_factory.iter().enumerate() {
            self.set_for_color(*num_tiles as usize, tile_color, layer);
        }
    }
}

#[inline(always)]
pub fn get_factory_index(factory: &[u8; NUM_TILE_COLORS]) -> usize {
    hash_factory(factory)
}

pub fn add_non_center_factory_index(
    factory: &[u8; NUM_TILE_COLORS],
    factory_counts: &mut [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
) -> usize {
    let factory_index = get_factory_index(factory);
    let count = factory_counts[factory_index];
    factory_counts[factory_index] += 1;
    count * NUM_POSSIBLE_FACTORY_PERMUTATIONS + factory_index
}

pub fn remove_non_center_factory_index(
    factory: &[u8; NUM_TILE_COLORS],
    factory_counts: &mut [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
) -> usize {
    let factory_index = get_factory_index(factory);
    let count = factory_counts[factory_index] - 1;
    factory_counts[factory_index] = count;
    count * NUM_POSSIBLE_FACTORY_PERMUTATIONS + factory_index
}

#[derive(Clone)]
pub struct NonCenterFactoryEncoding {
    factory_counts: [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
}

impl OneHotFeature for NonCenterFactoryEncoding {
    const SIZE: usize = NON_CENTER_FACTORY_ENCODING_SIZE;
    const PLAYER_FEATURE: bool = false;
    const MAX_ONES: usize = NUM_NON_CENTER_FACTORIES;
    const START: usize = CenterFactoryEncoding::END;

    fn initialize(_: &mut impl InputLayer) -> Self {
        Self {
            factory_counts: [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
        }
    }
}

impl NonCenterFactoryEncoding {
    pub fn set_factories(&mut self, factories: &Factories, layer: &mut impl InputLayer) {
        if self.factory_counts[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1] != NUM_NON_CENTER_FACTORIES {
            // Not all are empty, we need to reset
            for factory_index in 0..NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1 {
                let count = self.factory_counts[factory_index];
                for _ in 0..count {
                    let count = self.factory_counts[factory_index] - 1;
                    self.factory_counts[factory_index] = count;
                    let index = count * NUM_POSSIBLE_FACTORY_PERMUTATIONS + factory_index;
                    layer.unset_input(index + Self::START);
                    self.factory_counts[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1] += 1;
                }
                self.factory_counts[factory_index] = 0;
            }
        }

        self.factory_counts[NUM_POSSIBLE_FACTORY_PERMUTATIONS - 1] = 0;

        for factory in factories.iter() {
            let index = add_non_center_factory_index(factory, &mut self.factory_counts);
            layer.set_input(index + Self::START);
        }
    }

    pub fn remove_factory(&mut self, factory: &[u8; NUM_TILE_COLORS], layer: &mut impl InputLayer) {
        let index_to_remove = remove_non_center_factory_index(factory, &mut self.factory_counts);
        let index_to_add =
            add_non_center_factory_index(&[0; NUM_TILE_COLORS], &mut self.factory_counts);
        layer.unset_input(index_to_remove + Self::START);
        layer.set_input(index_to_add + Self::START);
    }
}

#[cfg(test)]
mod tests {
    use game::NUM_FACTORIES;

    use crate::mcts::neural_network::encoding_v2::tests::MockLayer;

    use super::*;

    #[test]
    fn test_get_factory_index() {
        let mut factory_counts = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
        let factory = [0; NUM_TILE_COLORS];

        let index_1 = add_non_center_factory_index(&factory, &mut factory_counts);
        let index_2 = add_non_center_factory_index(&factory, &mut factory_counts);
        let undo_1 = remove_non_center_factory_index(&factory, &mut factory_counts);
        let undo_2 = remove_non_center_factory_index(&factory, &mut factory_counts);
        assert_eq!(index_1, undo_2);
        assert_eq!(index_2, undo_1);

        println!("index_1: {}", index_1);
        println!("index_2: {}", index_2);
        println!("undo_1: {}", undo_1);
        println!("undo_2: {}", undo_2);
    }

    #[test]
    fn test() {
        let mut layer = MockLayer::default();
        let mut encoding = NonCenterFactoryEncoding::initialize(&mut layer);

        // Empty [425, 496, 567, 638, 709, 780]
        let factories: [[u8; 5]; 6] = [[1, 1, 1, 1, 0]; NUM_FACTORIES];
        encoding.set_factories(&factories.into(), &mut layer);
        let input_before = layer.input().to_vec();
        encoding.set_factories(&factories.into(), &mut layer);
        let input_after = layer.input().to_vec();
        println!("{:?}", input_before);
        println!("{:?}", input_after);
        assert_eq!(input_before, input_after);

        for i in 0..NUM_FACTORIES {
            encoding.remove_factory(&factories[i], &mut layer);
        }

        let input = layer.input();
        println!("{:?}", input);
        println!("Now everything is empty. Set again");
        encoding.set_factories(&factories.into(), &mut layer);
    }
}
