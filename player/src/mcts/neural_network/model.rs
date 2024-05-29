use game::GameState;
use rand::{rngs::SmallRng, SeedableRng};

use super::{
    accumulator::Accumulator, encoding::TOTAL_ENCODING_SIZE, layers::{DenseLayer, EfficentlyUpdatableDenseLayer}
};

pub const INPUT_SIZE: usize = TOTAL_ENCODING_SIZE + 8 - TOTAL_ENCODING_SIZE % 8; // Round up to nearest multiple of 8 

pub const LAYER_DIMENSIONS: [[usize; 2]; 3] = [[INPUT_SIZE, 512], [512, 256], [256, 8]];

const ACCUMULATOR_INPUT_SIZE: usize = LAYER_DIMENSIONS[0][0];
const ACCUMULATOR_OUTPUT_SIZE: usize = LAYER_DIMENSIONS[0][1];

pub struct Model {
    input_layer: Accumulator<EfficentlyUpdatableDenseLayer<ACCUMULATOR_OUTPUT_SIZE>>,
    hidden_layer: DenseLayer,
    output_layer: DenseLayer,

    hidden_buffer: [f32; LAYER_DIMENSIONS[1][1]],
    output_buffer: [f32; LAYER_DIMENSIONS[2][1]],
}

impl Default for Model {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        let mut input_layer = EfficentlyUpdatableDenseLayer::new(ACCUMULATOR_INPUT_SIZE);
        let mut hidden_layer = DenseLayer::new(ACCUMULATOR_OUTPUT_SIZE, LAYER_DIMENSIONS[1][1]);
        let mut output_layer = DenseLayer::new(LAYER_DIMENSIONS[2][0], LAYER_DIMENSIONS[2][1]);
        input_layer.initialize_random(&mut rng);
        hidden_layer.initialize_random(&mut rng);
        output_layer.initialize_random(&mut rng);
        Model {
            input_layer: Accumulator::new(input_layer),
            hidden_layer,
            output_layer,
            hidden_buffer: [0.0; LAYER_DIMENSIONS[1][1]],
            output_buffer: [0.0; LAYER_DIMENSIONS[2][1]],
        }
    }
}

impl Model {
    pub fn forward(&mut self) -> &[f32] {
        self.hidden_layer
            .forward(self.input_layer.output(), &mut self.hidden_buffer);
        self.output_layer
            .forward(&self.hidden_buffer, &mut self.output_buffer);
        &self.output_buffer
    }

    pub fn reset(&mut self) {
        self.input_layer.reset();
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.input_layer.set_game_state(game_state);
    }

    // TODO: Update
}
