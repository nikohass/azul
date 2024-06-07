use super::{
    encoding_v2::{Accumulator, ENCODING_SIZE},
    layers::{EfficentlyUpdatableDenseLayer, Layer},
};
use game::GameState;
use ndarray::{Array1, Array2};

pub const INPUT_SIZE: usize = ENCODING_SIZE + 8 - ENCODING_SIZE % 8; // Round up to nearest multiple of 8
pub const ACCUMULATOR_OUTPUT_SIZE: usize = 1080;
pub struct Model {
    input_layer: Accumulator<EfficentlyUpdatableDenseLayer<ACCUMULATOR_OUTPUT_SIZE>>,
    // hidden_layer: DenseLayer,
    // output_layer: DenseLayer,

    // input_activation_buffer: [f32; LAYER_DIMENSIONS[0][1]],
    // hidden_layer_buffer: [f32; LAYER_DIMENSIONS[1][1]],
    // hidden_activation_buffer: [f32; LAYER_DIMENSIONS[1][1]],
    // output_layer_buffer: [f32; LAYER_DIMENSIONS[2][1]],
}

impl Default for Model {
    fn default() -> Self {
        let input_layer = EfficentlyUpdatableDenseLayer::new(INPUT_SIZE);
        // let hidden_layer = DenseLayer::new(ACCUMULATOR_OUTPUT_SIZE, LAYER_DIMENSIONS[1][1]);
        // let output_layer = DenseLayer::new(LAYER_DIMENSIONS[2][0], LAYER_DIMENSIONS[2][1]);

        Model {
            input_layer: Accumulator::new(input_layer),
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
        // apply_relu(self.input_layer.output(), &mut self.input_activation_buffer);
        // self.hidden_layer
        //     .forward(&self.input_activation_buffer, &mut self.hidden_layer_buffer);
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
        self.input_layer.output()
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.input_layer
            .set_game_state(game_state, usize::from(game_state.current_player));
    }

    // pub fn set_input(&mut self, index: usize) {
    //     self.input_layer.get_mut_layer().set_input(index);
    // }

    pub fn load_from_file(&mut self, file_path: &str) {
        // let weights_biases =
        //     load_weights_biases(file_path).expect("Failed to load weights and biases");
        let layers = load_model(file_path).expect("Failed to load weights and biases");

        // let input_layer_weights: Vec<Vec<f32>> = serde_json::from_value(
        //     weights
        //         .get(0)
        //         .expect("Failed to load input layer weights")
        //         .clone(),
        // )
        // .expect("Failed to parse input layer weights");
        let weights_array = layers[0].weights.clone();
        // let weights_array = Array2::from_shape_vec(
        //     (input_layer_weights.len(), input_layer_weights[0].len()),
        //     input_layer_weights.into_iter().flatten().collect(),
        // )
        // .expect("Failed to convert input layer weights to Array2");
        self.input_layer.mut_layer().set_weights(weights_array);

        // Set input layer biases
        // let input_layer_biases: Vec<f32> = serde_json::from_value(
        //     weights
        //         .get(0)
        //         .expect("Failed to load input layer biases")
        //         .clone(),
        // )
        // .expect("Failed to parse input layer biases");
        // let biases_array = Array1::from(input_layer_biases);
        let biases_array = layers[0].biases.clone();
        self.input_layer.mut_layer().set_biases(&biases_array);

        // // Set hidden layer weights
        // let hidden_layer_weights: Vec<Vec<f32>> = serde_json::from_value(
        //     weights_biases
        //         .get("layers.2.weight")
        //         .expect("Failed to load hidden layer weights")
        //         .clone(),
        // )
        // .expect("Failed to parse hidden layer weights");
        // let weights_array = Array2::from_shape_vec(
        //     (hidden_layer_weights.len(), hidden_layer_weights[0].len()),
        //     hidden_layer_weights.into_iter().flatten().collect(),
        // )
        // .expect("Failed to convert hidden layer weights to Array2");
        // self.hidden_layer.set_weights(weights_array);

        // // Set hidden layer biases
        // let hidden_layer_biases: Vec<f32> = serde_json::from_value(
        //     weights_biases
        //         .get("layers.2.bias")
        //         .expect("Failed to load hidden layer biases")
        //         .clone(),
        // )
        // .expect("Failed to parse hidden layer biases");
        // let biases_array = Array1::from(hidden_layer_biases);
        // self.hidden_layer.set_biases(&biases_array);

        // // Set output layer weights
        // let output_layer_weights: Vec<Vec<f32>> = serde_json::from_value(
        //     weights_biases
        //         .get("layers.4.weight")
        //         .expect("Failed to load output layer weights")
        //         .clone(),
        // )
        // .expect("Failed to parse output layer weights");
        // let weights_array = Array2::from_shape_vec(
        //     (output_layer_weights.len(), output_layer_weights[0].len()),
        //     output_layer_weights.into_iter().flatten().collect(),
        // )
        // .expect("Failed to convert output layer weights to Array2");
        // self.output_layer.set_weights(weights_array);

        // // Set output layer biases
        // let output_layer_biases: Vec<f32> = serde_json::from_value(
        //     weights_biases
        //         .get("layers.4.bias")
        //         .expect("Failed to load output layer biases")
        //         .clone(),
        // )
        // .expect("Failed to parse output layer biases");
        // let biases_array = Array1::from(output_layer_biases);
        // self.output_layer.set_biases(&biases_array);

        // self.reset();
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
