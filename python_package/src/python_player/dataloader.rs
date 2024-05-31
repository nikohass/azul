use game::{
    hash_factory, MoveList, CENTER_FACTORY_INDEX, INDEX_TO_FACTORY,
    NUM_POSSIBLE_FACTORY_PERMUTATIONS,
};
use ndarray::{s, Array2};
use numpy::{PyArray2, ToPyArray};
use player::mcts::neural_network::encoding::{
    build_move_lookup, encode_game_state as encode_game_state_original,
};
use player::mcts::neural_network::layers::InputLayer;
use player::mcts::neural_network::model::INPUT_SIZE;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use rand::rngs::SmallRng;
use rand::SeedableRng as _;
use replay_buffer::ReplayBufferClient;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::python_game::game_state::GameState;
use crate::python_game::move_::Move;

pub struct DummyLayer {
    input: [f32; INPUT_SIZE],
}

impl InputLayer for DummyLayer {
    fn set_input(&mut self, index: usize) {
        self.input[index] = 1.0;
    }

    fn unset_input(&mut self, index: usize) {
        self.input[index] = 0.0;
    }

    fn output(&self) -> &[f32] {
        &self.input
    }

    fn reset(&mut self) {
        self.input = [0.0; INPUT_SIZE];
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DataLoader {
    #[allow(clippy::type_complexity)]
    local_buffer: Arc<Mutex<Vec<(Array2<f32>, Array2<f32>)>>>,
    batch_size: usize,
    target_buffer_size: Arc<AtomicUsize>,
}

#[pymethods]
impl DataLoader {
    #[new]
    pub fn new(url: &str, batch_size: usize) -> Self {
        let local_buffer = Arc::new(Mutex::new(Vec::new()));
        let target_buffer_size = Arc::new(AtomicUsize::new(0));

        let local_buffer_clone = local_buffer.clone();
        let target_buffer_size_clone = target_buffer_size.clone();
        let url = url.to_string();
        thread::spawn(move || {
            let client: ReplayBufferClient = ReplayBufferClient::new(&url);
            loop {
                let current_length;
                let target_length;
                {
                    let lock = local_buffer_clone.lock().unwrap();
                    current_length = lock.len();
                    target_length = target_buffer_size_clone.load(Ordering::Relaxed);
                }
                if current_length < target_length {
                    let entries = if let Ok(result) = client.sample_entries(batch_size) {
                        result
                    } else {
                        println!("Failed to sample entries");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    };

                    let mut batch_x = Array2::zeros((batch_size, INPUT_SIZE));
                    let mut batch_y = Array2::zeros((batch_size, 1));
                    // let mut mask_y = Array2::zeros((batch_size, 1080));
                    for (i, entry) in entries.iter().enumerate() {
                        let mut dummy_layer = DummyLayer {
                            input: [0.0; INPUT_SIZE],
                        };
                        encode_game_state_original(
                            &entry.game_state,
                            &mut dummy_layer,
                            &mut [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
                        );
                        batch_x.slice_mut(s![i, ..]).assign(
                            &Array2::from_shape_vec((1, INPUT_SIZE), dummy_layer.output().to_vec())
                                .unwrap()
                                .into_shape((INPUT_SIZE,))
                                .unwrap(),
                        );

                        // let mut y = [0.0_f32; NUM_PLAYERS];
                        // for (action, value) in &entry.action_value_pairs {
                        //     let factory_index =
                        //         if action.factory_index as usize != CENTER_FACTORY_INDEX {
                        //             hash_factory(
                        //                 &entry.game_state.factories[action.factory_index as usize],
                        //             )
                        //         } else {
                        //             INDEX_TO_FACTORY.len()
                        //         };
                        //     let value = value[current_player];
                        //     let index = move_lookup
                        //         [&(factory_index, action.color as u8, action.pattern_line_index)];
                        //     // y[index] = value;
                        //     y[[index, current_player]] = value;
                        //     mask_y[[i, index]] = 1.0;
                        // }
                        // y[current_player] = entry.value[current_player];
                        batch_y[[i, 0]] = entry.value[0];
                    }

                    {
                        let mut lock = local_buffer_clone.lock().unwrap();
                        lock.push((batch_x, batch_y));
                        println!("Added batch to local buffer");
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
            }
        });

        DataLoader {
            local_buffer,
            batch_size,
            target_buffer_size,
        }
    }

    pub fn __next__(&mut self) -> PyResult<Py<PyTuple>> {
        let batch;
        loop {
            let mut lock = self.local_buffer.lock().unwrap();
            if let Some(b) = lock.pop() {
                batch = b;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        Python::with_gil(|py| {
            let batch_tuple = PyTuple::new_bound(
                py,
                &[
                    batch.0.to_pyarray_bound(py).unbind(),
                    batch.1.to_pyarray_bound(py).unbind(),
                ],
            );
            Ok(batch_tuple.into())
        })
    }

    pub fn set_target_buffer_size(&self, target_buffer_size: usize) {
        self.target_buffer_size
            .store(target_buffer_size, Ordering::Relaxed);
    }

    pub fn get_target_buffer_size(&self) -> usize {
        self.target_buffer_size.load(Ordering::Relaxed)
    }

    pub fn get_batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn __iter__(&self) -> DataLoader {
        self.clone()
    }
}

#[pyfunction]
pub fn encode_game_state(game_state: &GameState) -> Py<PyArray2<f32>> {
    let mut dummy_layer = DummyLayer {
        input: [0.0; INPUT_SIZE],
    };
    encode_game_state_original(
        &game_state.0,
        &mut dummy_layer,
        &mut [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
    );

    let mut x = Array2::zeros((1, INPUT_SIZE));
    x.slice_mut(s![0, ..]).assign(
        &Array2::from_shape_vec((1, INPUT_SIZE), dummy_layer.output().to_vec())
            .unwrap()
            .into_shape((INPUT_SIZE,))
            .unwrap(),
    );

    Python::with_gil(|py| x.to_pyarray_bound(py).unbind())
}

#[pyfunction]
pub fn decode_move(game_state: &GameState, output: Vec<f32>) -> PyResult<Move> {
    let move_lookup = build_move_lookup();
    // let (factory_index, color, pattern_line_index) = move_lookup[&move_index];

    // let mut move_list = MoveList::default();
    // let mut rng = SmallRng::from_entropy();
    // game_state
    //     .0
    //     .clone()
    //     .get_possible_moves(&mut move_list, &mut rng);

    // for i in 0..move_list.len() {
    //     let mov = move_list[i];
    //     if mov.factory_index == factory_index as u8
    //         && mov.color as usize == color as usize
    //         && mov.pattern_line_index as usize == pattern_line_index as usize
    //     {
    //         return Ok(Move(mov));
    //     }
    // }

    // Err(pyo3::exceptions::PyValueError::new_err(
    //     "Invalid move index",
    // ))

    // Iterate over each possible move and find the highest valued legal move
    let mut max_value = f32::NEG_INFINITY;
    let mut best_move = None;

    let mut move_list = MoveList::default();
    let mut rng = SmallRng::from_entropy();
    game_state
        .0
        .clone()
        .get_possible_moves(&mut move_list, &mut rng);

    for i in 0..move_list.len() {
        let mov = move_list[i];
        let factory_index = if mov.factory_index as usize != CENTER_FACTORY_INDEX {
            hash_factory(&game_state.0.factories[mov.factory_index as usize])
        } else {
            INDEX_TO_FACTORY.len()
        };

        let index = move_lookup.get(&(factory_index, mov.color as u8, mov.pattern_line_index));

        if index.is_none() {
            println!("Invalid move index");
            continue;
        }
        let index = *index.unwrap();

        let value = output[index];
        if value > max_value {
            max_value = value;
            best_move = Some(mov);
        }
    }

    if let Some(mov) = best_move {
        Ok(Move(mov))
    } else {
        Err(pyo3::exceptions::PyValueError::new_err(
            "Invalid move index",
        ))
    }
}
