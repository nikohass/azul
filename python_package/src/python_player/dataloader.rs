use game::{hash_factory, MoveList, CENTER_FACTORY_INDEX, INDEX_TO_FACTORY};
use ndarray::{s, Array2};
use numpy::{PyArray2, PyArrayMethods, PyReadonlyArray, ToPyArray};
use player::mcts::neural_network::encoding_v2::{Accumulator, INPUT_SIZE};
use player::mcts::neural_network::layers::InputLayer;
use player::mcts::neural_network::model;
use pyo3::exceptions::{PyKeyboardInterrupt, PyValueError};
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
    local_buffer: Arc<Mutex<Vec<(Array2<f32>, Array2<f32>, Array2<f32>)>>>,
    batch_size: usize,
    target_buffer_size: Arc<AtomicUsize>,
}

const OUTPUT_SIZE: usize = 1080;

#[pymethods]
impl DataLoader {
    #[new]
    pub fn new(url: &str, batch_size: usize) -> Self {
        let local_buffer = Arc::new(Mutex::new(Vec::new()));
        let target_buffer_size = Arc::new(AtomicUsize::new(0));

        for _ in 0..2 {
            let local_buffer_clone = local_buffer.clone();
            let target_buffer_size_clone = target_buffer_size.clone();
            let url = url.to_string();
            thread::spawn(move || {
                let mut accumulator = Accumulator::new(DummyLayer {
                    input: [0.0; INPUT_SIZE],
                });
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
                        // let start_time = std::time::Instant::now();
                        let entries = if let Ok(result) = client.sample_entries(batch_size) {
                            result
                        } else {
                            println!("Failed to sample entries");
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            continue;
                        };
                        // println!("Sampled entries in {:?}", start_time.elapsed());
                        // let start_time = std::time::Instant::now();

                        let mut batch_x = Array2::zeros((batch_size, INPUT_SIZE));
                        let mut batch_y = Array2::zeros((batch_size, OUTPUT_SIZE));
                        let mut batch_y_mask = Array2::zeros((batch_size, OUTPUT_SIZE));
                        // let mut mask_y = Array2::zeros((batch_size, 1080));
                        for (i, entry) in entries.iter().enumerate() {
                            // let mut dummy_layer = DummyLayer {
                            //     input: [0.0; INPUT_SIZE],
                            // };
                            println!("{}", entry.game_state.to_fen());
                            // encode_game_state_original(
                            //     &entry.game_state,
                            //     &mut dummy_layer,
                            //     &mut [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
                            // );
                            let current_player = usize::from(entry.game_state.current_player);
                            accumulator.set_game_state(&entry.game_state, current_player);
                            batch_x.slice_mut(s![i, ..]).assign(
                                &Array2::from_shape_vec(
                                    (1, INPUT_SIZE),
                                    accumulator.layer().output().to_vec(),
                                )
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
                            // batch_y[[i, 0]] = entry.value[0];
                            for (action, value) in entry.action_value_pairs.iter() {
                                let index = player::mcts::neural_network::encoding::encode_move(
                                    &entry.game_state,
                                    *action,
                                );
                                if let Some(index) = index {
                                    batch_y[[i, index]] = value[current_player];
                                    batch_y_mask[[i, index]] = 1.0;
                                }
                            }
                        }
                        // println!("Encoded entries after {:?}", start_time.elapsed());

                        {
                            let mut lock = local_buffer_clone.lock().unwrap();
                            lock.push((batch_x, batch_y, batch_y_mask));
                            println!("Added batch to local buffer");
                        }
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                    }
                }
            });
        }

        DataLoader {
            local_buffer,
            batch_size,
            target_buffer_size,
        }
    }

    pub fn __next__(&mut self) -> PyResult<Py<PyTuple>> {
        let batch;
        loop {
            {
                let mut lock = self.local_buffer.lock().unwrap();
                if let Some(b) = lock.pop() {
                    batch = b;
                    break;
                }
            }

            Python::with_gil(|py| {
                py.check_signals()
                    .map_err(|_| PyKeyboardInterrupt::new_err("Keyboard interrupt received"))
            })?;
        }
        Python::with_gil(|py| {
            let batch_tuple = PyTuple::new_bound(
                py,
                &[
                    batch.0.to_pyarray_bound(py).unbind(),
                    batch.1.to_pyarray_bound(py).unbind(),
                    batch.2.to_pyarray_bound(py).unbind(),
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
pub fn encode_game_state(game_state: &GameState, player: usize) -> Py<PyArray2<f32>> {
    // let mut dummy_layer = DummyLayer {
    //     input: [0.0; INPUT_SIZE],
    // };
    // encode_game_state_original(
    //     &game_state.0,
    //     &mut dummy_layer,
    //     &mut [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
    // );
    let mut accumulator = Accumulator::new(DummyLayer {
        input: [0.0; INPUT_SIZE],
    });
    accumulator.set_game_state(&game_state.0, player);

    let mut x = Array2::zeros((1, INPUT_SIZE));
    x.slice_mut(s![0, ..]).assign(
        &Array2::from_shape_vec((1, INPUT_SIZE), accumulator.layer().output().to_vec())
            .unwrap()
            .into_shape((INPUT_SIZE,))
            .unwrap(),
    );

    Python::with_gil(|py| x.to_pyarray_bound(py).unbind())
}

#[pyfunction]
pub fn decode_move(game_state: &GameState, output: Vec<f32>) -> PyResult<Move> {
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

        let index = player::mcts::neural_network::encoding::MOVE_LOOKUP.get(&(
            factory_index,
            mov.color as u8,
            mov.pattern_line_index,
        ));

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

#[pyfunction]
pub fn encode_move(game_state: &GameState, mov: Move) -> Option<usize> {
    // let factory_index = if mov.0.factory_index as usize != CENTER_FACTORY_INDEX {
    //     hash_factory(&game_state.0.factories[mov.0.factory_index as usize])
    // } else {
    //     INDEX_TO_FACTORY.len()
    // };
    // MOVE_LOOKUP
    //     .get(&(factory_index, mov.0.color as u8, mov.0.pattern_line_index))
    //     .copied()
    let game_state = &game_state.0;
    let mov = mov.0;

    player::mcts::neural_network::encoding::encode_move(game_state, mov)
}

#[pyclass]
pub struct WeightsBiases(pub player::mcts::neural_network::model::WeightsBiases);

#[pymethods]
impl WeightsBiases {
    #[new]
    pub fn new(
        weights: PyReadonlyArray<f32, ndarray::Ix2>,
        biases: PyReadonlyArray<f32, ndarray::Ix1>,
    ) -> Self {
        let weights = weights.to_owned_array();
        let biases = biases.to_owned_array();
        WeightsBiases(player::mcts::neural_network::model::WeightsBiases { weights, biases })
    }

    #[getter]
    pub fn weights(
        &self,
        py: Python,
    ) -> Py<numpy::PyArray<f32, ndarray::prelude::Dim<[usize; 2]>>> {
        self.0.weights.to_pyarray_bound(py).unbind()
    }

    #[getter]
    pub fn biases(&self, py: Python) -> Py<numpy::PyArray<f32, ndarray::prelude::Dim<[usize; 1]>>> {
        self.0.biases.to_pyarray_bound(py).unbind()
    }
}

impl<'source> FromPyObject<'source> for WeightsBiases {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let weights = ob
            .getattr("weights")?
            .extract::<PyReadonlyArray<f32, ndarray::Ix2>>()?;
        let biases = ob
            .getattr("biases")?
            .extract::<PyReadonlyArray<f32, ndarray::Ix1>>()?;
        Ok(WeightsBiases::new(weights, biases))
    }
}

#[pyfunction]
pub fn store_model(file_path: &str, layers: Vec<WeightsBiases>) -> PyResult<()> {
    let layers = layers.into_iter().map(|layer| layer.0).collect::<Vec<_>>();
    model::store_model(file_path, layers)
        .map_err(|e| PyValueError::new_err(format!("Failed to store model: {}", e)))
}

#[pyfunction]
pub fn load_model(file_path: &str) -> PyResult<Vec<WeightsBiases>> {
    model::load_model(file_path)
        .map(|layers| layers.into_iter().map(WeightsBiases).collect())
        .map_err(|e| PyValueError::new_err(format!("Failed to load model: {}", e)))
}
