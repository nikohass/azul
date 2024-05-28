use game::{
    GameState, CENTER_FACTORY_INDEX, NUM_NON_CENTER_FACTORIES, NUM_PLAYERS,
    NUM_POSSIBLE_FACTORY_PERMUTATIONS,
};

use super::{
    factory_encoding::{add_center_factory_encoding, add_non_center_factory_encoding},
    layers::InputLayer,
    pattern_line_encoding::add_pattern_line_encoding,
    wall_encoding::add_wall_encoding,
};

pub struct Accumulator<L: InputLayer> {
    layer: L,
    multi_factory_counter: [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
}

impl<L: InputLayer> Accumulator<L> {
    pub fn new(layer: L) -> Self {
        Self {
            layer,
            multi_factory_counter: [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
        }
    }

    pub fn reset(&mut self) {
        self.layer.reset();
        self.multi_factory_counter = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.reset();

        // Encode factories
        for factory in game_state.factories.iter().take(NUM_NON_CENTER_FACTORIES) {
            add_non_center_factory_encoding(
                factory,
                &mut self.multi_factory_counter,
                &mut self.layer,
            );
        }

        // Encode center factory
        let center_factory = &game_state.factories[CENTER_FACTORY_INDEX];
        for (tile_color, num_tiles) in center_factory.iter().enumerate() {
            add_center_factory_encoding(*num_tiles as usize, tile_color, &mut self.layer);
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
                let num_tiles =
                    game_state.pattern_lines_occupancy[player_index][pattern_line_index];
                add_pattern_line_encoding(
                    pattern_line_index,
                    num_tiles as usize,
                    *pattern_line_color,
                    player_index,
                    &mut self.layer,
                )
            }

            // Encode wall
            add_wall_encoding(
                game_state.walls[player_index],
                player_index,
                &mut self.layer,
            );

            // TODO: Encode floor line and player scores
        }
    }

    // pub fn do_move(
    //     &mut self,
    //     game_state: &mut GameState,
    //     mov: Move,
    //     move_generation_result: MoveGenerationResult,
    // ) {
    //     match move_generation_result {
    //         MoveGenerationResult::GameOver => {
    //             self.reset();
    //         }
    //         MoveGenerationResult::RoundOver => {
    //             self.set_game_state(game_state);
    //         }
    //         _ => {}
    //     }

    //     let current_player = usize::from(game_state.current_player);
    //     let factory_index = mov.factory_index;
    //     let color = mov.color as usize;
    //     let factory = game_state.factories[factory_index];
    //     let pattern_line_index = mov.pattern_line_index as usize;

    //     if factory_index == CENTER_FACTORY_INDEX {
    //         // If we took tiles from the center, we only remove the color we took
    //         game_state.factories[factory_index][color] = 0;

    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::SmallRng, SeedableRng};

    pub struct MockLayer {
        pub input: Vec<usize>,
    }

    impl InputLayer for MockLayer {
        fn set_input(&mut self, index: usize) {
            println!("Setting input: {}", index);
            self.input.push(index);
        }

        fn unset_input(&mut self, index: usize) {
            println!("Unsetting input: {}", index);
            let length_before = self.input.len();
            self.input.retain(|&x| x != index);
            let length_after = self.input.len();
            assert_eq!(length_before as i64 - 1, length_after as i64);
        }

        fn reset(&mut self) {
            self.input.clear();
        }

        fn output(&self) -> &[f32] {
            &[]
        }
    }

    #[test]
    fn test_accumulator() {
        let mut rng = SmallRng::from_seed([0; 32]);
        let game_state = GameState::new(&mut rng);

        let mut accumulator = Accumulator::new(MockLayer { input: vec![] });
        println!("{}", game_state);
        accumulator.set_game_state(&game_state);
        // TODO:
    }
}
