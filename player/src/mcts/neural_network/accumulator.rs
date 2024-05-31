use game::{GameState, NUM_POSSIBLE_FACTORY_PERMUTATIONS};

use super::{encoding::encode_game_state, layers::InputLayer};

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

        encode_game_state(game_state, &mut self.layer, &mut self.multi_factory_counter);
    }

    pub fn get_mut_layer(&mut self) -> &mut L {
        &mut self.layer
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

    pub fn output(&self) -> &[f32] {
        self.layer.output()
    }
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
