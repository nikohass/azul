use super::{
    encoding_v2::{Accumulator, ENCODING_SIZE},
    layers::{apply_relu, DenseLayer, EfficentlyUpdatableDenseLayer, Layer},
};
use game::GameState;
use ndarray::{Array1, Array2};

pub const INPUT_SIZE: usize = ENCODING_SIZE + 8 - ENCODING_SIZE % 8; // Round up to nearest multiple of 8
pub const ACCUMULATOR_OUTPUT_SIZE: usize = 256;

pub struct Model {
    input_layer: Accumulator<EfficentlyUpdatableDenseLayer<ACCUMULATOR_OUTPUT_SIZE>>,
    // hidden_layer: DenseLayer,
    // output_layer: DenseLayer,
    input_activation_buffer: [f32; ACCUMULATOR_OUTPUT_SIZE],
    // hidden_layer_buffer: [f32; LAYER_DIMENSIONS[1][1]],
    // hidden_activation_buffer: [f32; LAYER_DIMENSIONS[1][1]],
    // output_layer_buffer: [f32; LAYER_DIMENSIONS[2][1]],
    l2: DenseLayer,
    l2_buffer: [f32; 1080],
}

impl Default for Model {
    fn default() -> Self {
        let input_layer = EfficentlyUpdatableDenseLayer::new(INPUT_SIZE);
        // let hidden_layer = DenseLayer::new(ACCUMULATOR_OUTPUT_SIZE, LAYER_DIMENSIONS[1][1]);
        // let output_layer = DenseLayer::new(LAYER_DIMENSIONS[2][0], LAYER_DIMENSIONS[2][1]);

        Model {
            input_layer: Accumulator::new(input_layer),
            l2: DenseLayer::new(ACCUMULATOR_OUTPUT_SIZE, 1080),
            input_activation_buffer: [0.0; ACCUMULATOR_OUTPUT_SIZE],
            l2_buffer: [0.0; 1080],
            // hidden_layer,
            // output_layer,
            // input_activation_buffer: [0.0; LAYER_DIMENSIONS[0][1]],
            // hidden_layer_buffer: [0.0; LAYER_DIMENSIONS[1][1]],
            // hidden_activation_buffer: [0.0; LAYER_DIMENSIONS[1][1]],
            // output_layer_buffer: [0.0; LAYER_DIMENSIONS[2][1]],
        }
    }
}

impl Model {
    pub fn forward(&mut self) -> &[f32] {
        apply_relu(self.input_layer.output(), &mut self.input_activation_buffer);
        self.l2
            .forward(&self.input_activation_buffer, &mut self.l2_buffer);
        &self.l2_buffer
        // apply_relu(
        //     &self.hidden_layer_buffer,
        //     &mut self.hidden_activation_buffer,
        // );
        // self.output_layer.forward(
        //     &self.hidden_activation_buffer,
        //     &mut self.output_layer_buffer,
        // );
        // // sigmoid(self.output_layer_buffer[0])
        // self.output_layer_buffer[0]
        // self.input_layer.output()
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.input_layer
            .set_game_state(game_state, usize::from(game_state.current_player));
    }

    pub fn load_from_file(&mut self, file_path: &str) {
        let layers = load_model(file_path).expect("Failed to load weights and biases");

        let weights_array = layers[0].weights.clone();
        self.input_layer.mut_layer().set_weights(weights_array);
        let biases_array = layers[0].biases.clone();
        self.input_layer.mut_layer().set_biases(&biases_array);

        let weights_array = layers[1].weights.clone();
        self.l2.set_weights(weights_array);
        let biases_array = layers[1].biases.clone();
        self.l2.set_biases(&biases_array);
    }

    // TODO: Update
}

use serde::{Deserialize, Serialize};
// use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct WeightsBiases {
    pub weights: Array2<f32>,
    pub biases: Array1<f32>,
}

pub fn load_model(file_path: &str) -> Result<Vec<WeightsBiases>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let weights_biases: Vec<WeightsBiases> = bincode::deserialize_from(reader)?;
    Ok(weights_biases)
}

pub fn store_model(
    file_path: &str,
    layers: Vec<WeightsBiases>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(file_path)?;
    bincode::serialize_into(file, &layers)?;
    Ok(())
}
