use crate::Bag;
use crate::{tile_color::NUM_TILE_COLORS, NUM_PLAYERS};
use rand::rngs::SmallRng;
use rand::Rng;

// Map the number of players to the number of factories (+1 for the center factory at the last index)
const PLAYERS_TO_FACTORIES: [usize; 3] = [6, 8, 10];
pub const NUM_FACTORIES: usize = PLAYERS_TO_FACTORIES[NUM_PLAYERS - 2];
pub const CENTER_FACTORY_INDEX: usize = NUM_FACTORIES - 1;

pub type Factory = [u8; NUM_TILE_COLORS];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Factories {
    factories: [Factory; NUM_FACTORIES],
}

impl std::ops::Deref for Factories {
    type Target = [Factory; NUM_FACTORIES];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.factories
    }
}

impl std::ops::DerefMut for Factories {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.factories
    }
}

impl std::ops::Index<usize> for Factories {
    type Output = Factory;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.factories[index]
    }
}

impl std::ops::IndexMut<usize> for Factories {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.factories[index]
    }
}

impl From<[Factory; NUM_FACTORIES]> for Factories {
    #[inline]
    fn from(factories: [Factory; NUM_FACTORIES]) -> Self {
        Factories { factories }
    }
}

impl Factories {
    #[inline]
    pub fn empty() -> Self {
        Factories {
            factories: [[0; NUM_TILE_COLORS]; NUM_FACTORIES],
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter()
            .all(|factory| factory.iter().all(|&tile_count| tile_count == 0))
    }

    #[inline]
    pub fn refill_by_drawing_from_bag(
        &mut self,
        bag: &mut Bag,
        out_of_bag: &mut Bag,
        rng: &mut SmallRng,
    ) {
        // Make sure the center is empty
        #[cfg(debug_assertions)]
        {
            for color in self.factories.last_mut().unwrap().iter_mut() {
                debug_assert!(*color == 0);
            }
        }

        let mut tiles_left_in_bag = bag.iter().sum::<u8>();
        for factory in self.factories.iter_mut().take(CENTER_FACTORY_INDEX) {
            #[cfg(debug_assertions)]
            {
                for color in factory.iter_mut() {
                    debug_assert!(*color == 0);
                }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use rand::SeedableRng;

    #[test]
    fn test_empty_factories() {
        let factories = Factories::empty();

        for factory in factories.iter() {
            for &tile_count in factory.iter() {
                assert_eq!(tile_count, 0);
            }
        }
    }

    #[test]
    fn test_factories_refill_from_bag() {
        // Make sure the number of tiles placed in the factories is correct

        // Setup factories and bag
        let mut factories: Factories = Factories::empty();
        let mut bag = [20, 20, 20, 20, 20];
        let mut out_of_bag: Bag = [0, 0, 0, 0, 0];
        let mut rng = SmallRng::seed_from_u64(42); // Seed RNG for reproducibility

        // Run the function
        factories.refill_by_drawing_from_bag(&mut bag, &mut out_of_bag, &mut rng);

        // Check that the factories are filled correctly
        let remaining_tiles_in_bag = bag.iter().sum::<u8>();
        let expected_num_tiles_in_factories = 4 * (NUM_FACTORIES as u8 - 1); // -1 because the center factory is not filled

        assert_eq!(
            remaining_tiles_in_bag,
            100 - expected_num_tiles_in_factories
        );
    }

    #[test]
    fn test_index_access() {
        let mut factories = Factories::empty();
        // Modify a factory
        factories[0][1] = 5;

        assert_eq!(factories[0][1], 5);
    }

    #[test]
    fn test_bag_refill_after_emptying() {
        // Make sure the function refills the bag from the out_of_bag when it runs out of tiles

        // Setup factories and bag
        let mut factories: Factories = Factories::empty();
        let mut bag = [1, 1, 1, 1, 1]; // not enough tiles to fill all factories
        let mut out_of_bag: Bag = [19, 19, 19, 19, 19];
        let mut rng = SmallRng::seed_from_u64(42); // Seed RNG for reproducibility

        // Run the function
        factories.refill_by_drawing_from_bag(&mut bag, &mut out_of_bag, &mut rng);

        // Check that the factories are filled correctly
        assert_eq!(out_of_bag, [0, 0, 0, 0, 0]); // There are no tiles left in the out_of_bag because they were all moved to the bag
    }

    #[test]
    fn test_bag_refill_not_enough_tiles() {
        // Make sure the factory filling works even if there are not enough tiles in the bag and out_of_bag

        // Setup factories and bag
        let mut factories: Factories = Factories::empty();
        let mut bag = [0, 0, 0, 0, 0]; // no tiles in the bag
        let mut out_of_bag: Bag = [0, 0, 0, 0, 10]; // only 10 white tiles in the out_of_bag
        let mut rng = SmallRng::seed_from_u64(42); // Seed RNG for reproducibility

        // Run the function
        factories.refill_by_drawing_from_bag(&mut bag, &mut out_of_bag, &mut rng);

        // Check that the factories are filled correctly
        assert_eq!(out_of_bag, [0, 0, 0, 0, 0]); // There are no tiles left in the out_of_bag because they were all moved to the bag
        assert_eq!(bag, [0, 0, 0, 0, 0]); // The bag is refilled with the tiles from the out_of_bag, but the tiles are distributed to the factories
        let num_tiles_in_factories = factories.iter().flatten().sum::<u8>();
        assert_eq!(num_tiles_in_factories, 10); // All 10 white tiles are placed in the factories
    }
}
