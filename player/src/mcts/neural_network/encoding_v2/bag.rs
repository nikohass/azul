use game::NUM_TILE_COLORS;

use crate::mcts::neural_network::layers::InputLayer;

use super::{wall::WallEncoding, OneHotFeature};

const MAX_NUM_TILES_PER_COLOR: usize = 20;

pub struct BagEncoding {
    bag: [usize; NUM_TILE_COLORS],
}

impl OneHotFeature for BagEncoding {
    const SIZE: usize = MAX_NUM_TILES_PER_COLOR * NUM_TILE_COLORS;
    const PLAYER_FEATURE: bool = false;
    const MAX_ONES: usize = NUM_TILE_COLORS;
    const START: usize = WallEncoding::END;

    fn initialize(layer: &mut impl InputLayer) -> Self {
        for i in 0..NUM_TILE_COLORS {
            let set = i + Self::START;
            layer.set_input(set);
        }
        Self {
            bag: [0; NUM_TILE_COLORS],
        }
    }
}

impl BagEncoding {
    pub fn set_for_color(&mut self, color: usize, count: usize, layer: &mut impl InputLayer) {
        let current_count = self.bag[color];
        if current_count != count {
            let set = count * NUM_TILE_COLORS + color + Self::START;
            let unset = current_count * NUM_TILE_COLORS + color + Self::START;
            self.bag[color] = count;
            layer.set_input(set);
            layer.unset_input(unset);
        }
    }

    pub fn set_bag(&mut self, bag: &[u8; NUM_TILE_COLORS], layer: &mut impl InputLayer) {
        for (color, count) in bag.iter().enumerate() {
            self.set_for_color(color, *count as usize, layer);
        }
    }
}
