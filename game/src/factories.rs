use crate::{tile_color::NUM_TILE_COLORS, NUM_PLAYERS};
use rand::rngs::SmallRng;
use rand::Rng;

// Map the number of players to the number of factories (+1 for the center factory at the last index)
const PLAYERS_TO_FACTORIES: [usize; 3] = [6, 8, 10];
pub const NUM_FACTORIES: usize = PLAYERS_TO_FACTORIES[NUM_PLAYERS - 2];
pub const CENTER_FACTORY_INDEX: usize = NUM_FACTORIES - 1;

#[inline]
pub fn fill_factories(
    factories: &mut [[u8; NUM_TILE_COLORS]; NUM_FACTORIES],
    bag: &mut [u8; NUM_TILE_COLORS],
    out_of_bag: &mut [u8; NUM_TILE_COLORS],
    rng: &mut SmallRng,
) {
    // Make sure the center is empty
    #[cfg(debug_assertions)]
    for color in factories.last_mut().unwrap().iter_mut() {
        debug_assert!(*color == 0);
    }
    let mut tiles_left_in_bag = bag.iter().sum::<u8>();

    for factory in factories.iter_mut().take(CENTER_FACTORY_INDEX) {
        #[cfg(debug_assertions)]
        for color in factory.iter_mut() {
            debug_assert!(*color == 0);
        }

        // Fill the factory with 4 tiles
        let mut remaining_tiles_to_fill = 4;
        while remaining_tiles_to_fill > 0 {
            let tile_color = rng.gen_range(0..NUM_TILE_COLORS);
            if bag[tile_color] > 0 {
                bag[tile_color] -= 1;
                factory[tile_color] += 1;
                remaining_tiles_to_fill -= 1;
                tiles_left_in_bag -= 1;
            }
            if tiles_left_in_bag == 0 {
                // Refill the bag
                bag.copy_from_slice(out_of_bag);
                tiles_left_in_bag = bag.iter().sum::<u8>();
                out_of_bag.fill(0);

                // In the rare case that you run out of tiles again while there are none left in the lid, start the new round as usual even though not all Factory displays are properly filled.
                if tiles_left_in_bag == 0 {
                    break;
                }
            }
        }
    }
}
