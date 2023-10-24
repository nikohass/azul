use crate::{NUM_PLAYERS, NUM_TILE_COLORS};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

// Map the number of players to the number of factories (+1 for the center factory at the last index)
const PLAYERS_TO_FACTORIES: [usize; 3] = [6, 8, 10];
pub const NUM_FACTORIES: usize = PLAYERS_TO_FACTORIES[NUM_PLAYERS - 2];
pub const CENTER_FACTORY_INDEX: usize = NUM_FACTORIES - 1;

pub fn fill_factories(
    factories: &mut [[u8; NUM_TILE_COLORS]; NUM_FACTORIES],
    bag: &mut [u8; NUM_TILE_COLORS],
    out_of_bag: &mut [u8; NUM_TILE_COLORS],
) {
    // println!("Filling factories");
    let mut rng = SmallRng::from_entropy();
    //assert!(self.bag.iter().sum::<u8>() == 0);
    // Make sure the center is empty
    #[cfg(debug_assertions)]
    for color in factories.last_mut().unwrap().iter_mut() {
        debug_assert!(*color == 0);
    }
    // Fill the bag
    //bag.copy_from_slice(&[20, 20, 20, 20, 20]);
    let mut tiles_left_in_bag = bag.iter().sum::<u8>();

    for factory in factories.iter_mut().take(CENTER_FACTORY_INDEX) {
        // Make sure the factory is empty
        #[cfg(debug_assertions)]
        for color in factory.iter_mut() {
            debug_assert!(*color == 0);
        }
        // Fill the factory
        let mut tiles_left = 4;
        while tiles_left > 0 {
            let tile_color = rng.gen_range(0..NUM_TILE_COLORS);
            if bag[tile_color] > 0 {
                bag[tile_color] -= 1;
                factory[tile_color] += 1;
                tiles_left -= 1;
                tiles_left_in_bag -= 1;
            }
            if tiles_left_in_bag == 0 {
                // Refill the bag
                // println!("Refilling bag");
                // println!("Bag: {:?}", bag);
                bag.copy_from_slice(out_of_bag);
                tiles_left_in_bag = bag.iter().sum::<u8>();
                // println!("Bag: {:?}", bag);
                out_of_bag.fill(0);
                // println!("Out of bag: {:?}", out_of_bag);
            }
        }
    }
}
