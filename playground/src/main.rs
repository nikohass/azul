#![allow(unused_imports)]

use game::{match_::run_match, *};
use player::{
    command_line_player::HumanCommandLinePlayer,
    mcts::{
        edge::Edge,
        neural_network::{
            layers::{DenseLayer, EfficentlyUpdatableDenseLayer, InputLayer as _},
            model::{Model, INPUT_SIZE},
        },
        MonteCarloTreeSearch,
    },
    random_player::RandomPlayer,
};
use rand::rngs::{SmallRng, StdRng};
use rand::{thread_rng, Rng, SeedableRng};
use rayon::prelude::*;
use replay_buffer::{buffer::ReplayEntry, ReplayBufferClient};
use std::{
    cell::RefCell,
    collections::HashSet,
    fmt,
    fs::OpenOptions,
    io::BufRead as _,
    rc::{Rc, Weak},
    sync::{
        atomic::{AtomicI64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
use std::{collections::HashMap, sync::Mutex};
use std::{io::Write, sync::atomic::AtomicBool};

// fn main() {
//     let mut handles = vec![];

//     let (statistics_sender, statistics_receiver) = mpsc::channel();
//     let stop_flag = Arc::new(AtomicBool::new(false));

//     for _ in 0..12 {
//         let statistics_sender = statistics_sender.clone();
//         let stop_flag = stop_flag.clone();

//         let mut rng = SmallRng::from_entropy();
//         let handle = thread::spawn(move || loop {
//             let mut players = vec![
//                 Box::<MonteCarloTreeSearch>::default() as Box<dyn Player>,
//                 Box::<MonteCarloTreeSearch>::default() as Box<dyn Player>,
//             ];
//             for player in players.iter_mut() {
//                 player.set_time(TimeControl::ConstantTimePerMove {
//                     milliseconds_per_move: 700,
//                 });
//             }

//             let game_state = GameState::new(&mut rng);
//             let statistics = run_match(game_state, &mut players, false).unwrap();
//             if stop_flag.load(Ordering::Relaxed) {
//                 return;
//             }
//             statistics_sender.send(statistics).unwrap();
//         });
//         handles.push(handle);
//     }

//     const NUM_STATE_ACTION_PAIRS: usize = 2_000_000;

//     let handle = thread::spawn(move || {
//         let mut state_action_pairs = Vec::new();
//         let mut saved: u64 = 0;

//         while let Ok(statistics) = statistics_receiver.recv() {
//             println!(
//                 "Received statistics with {} state-action pairs",
//                 statistics.state_action_pairs.len()
//             );
//             state_action_pairs.extend(statistics.state_action_pairs);

//             if state_action_pairs.len() >= NUM_STATE_ACTION_PAIRS.min(1000) {
//                 println!("Writing to file");
//                 let mut file = OpenOptions::new()
//                     .append(true)
//                     .create(true)
//                     .open("logs/state_action_pairs.csv")
//                     .unwrap();
//                 for (state, action) in state_action_pairs.iter() {
//                     writeln!(file, "{},{}", state.to_fen(), action.serialize_string()).unwrap();
//                 }
//                 saved += state_action_pairs.len() as u64;
//                 state_action_pairs.clear();

//                 println!("Saved {} state-action pairs in total", saved);

//                 if saved >= NUM_STATE_ACTION_PAIRS as u64 {
//                     println!("Stopping");
//                     stop_flag.store(true, Ordering::Relaxed);
//                     break;
//                 }
//             }
//         }
//     });
//     handles.push(handle);

//     for handle in handles {
//         handle.join().unwrap();
//     }
// }
use std::arch::x86_64::*;

fn main() {
    // let mut rng = SmallRng::from_entropy();
    // let game_state = GameState::new(&mut rng);
    // let mut players: Vec<Box<dyn Player>> = vec![
    //     Box::<MonteCarloTreeSearch>::default(),
    //     Box::<MonteCarloTreeSearch>::default(),
    //     Box::<MonteCarloTreeSearch>::default(),
    // ];

    let mut client = ReplayBufferClient::new("http://127.0.0.1:3044");
    client.set_buffer_size(100_000).unwrap();
    client.set_buffer_size(90_000).unwrap();
    // let result = client.sample_entries(10);
    // match result {
    //     ApiResponse::Entries(_, entries) => {
    //         println!("Received {} entries", entries.len());
    //     }
    //     _ => panic!("Unexpected response"),
    // }

    let mut rng = SmallRng::from_entropy();
    let game_state = GameState::new(&mut rng);
    println!("{}", game_state);

    let mut model = Model::default();
    let output = model.forward();
    println!("{:?}", output);
    model.set_game_state(&game_state);
    let output = model.forward();
    println!("{:?}", output);

    println!("Input size: {}", INPUT_SIZE);

    // let mut players: Vec<Box<dyn Player>> = vec![
    //     Box::<MonteCarloTreeSearch>::default(),
    //     Box::<MonteCarloTreeSearch>::default(),
    //     Box::<MonteCarloTreeSearch>::default(),
    // ];
    // run_match(game_state, &mut players, true).unwrap();

    // let move_lookup = build_move_lookup();
    // const OUTPUT_SIZE: usize = 1080;
    // assert_eq!(OUTPUT_SIZE, move_lookup.len());
    // assert_eq!(OUTPUT_SIZE % 8, 0, "Output size must be a multiple of 8.");
    // const INPUT_SIZE: usize = TOTAL_ENCODING_SIZE + (8 - TOTAL_ENCODING_SIZE % 8);

    // let mut rng = SmallRng::from_entropy();
    // // let mut weights = vec![0.0; INPUT_SIZE * output_size];
    // // for col in 0..output_size {
    // //     for row in 0..INPUT_SIZE {
    // //         weights[col * INPUT_SIZE + row] = col as f32 * 0.1 + row as f32 * 0.01;
    // //     }
    // // }
    // // let mut biases = [0.0; output_size];
    // // let mut default_layer = DenseLayer::new(INPUT_SIZE, output_size);
    // // default_layer.initialize_random(&mut rng);
    // // default_layer.set_weights(&weights);
    // // default_layer.set_biases(&biases);
    // let move_lookup: HashMap<(usize, u8, u8), usize> = build_move_lookup();

    // let mut efficient_layer = EfficentlyUpdatableDenseLayer::<OUTPUT_SIZE>::new(INPUT_SIZE);
    // efficient_layer.initialize_random(&mut rng);

    // let mut move_list = MoveList::default();

    // // let start_time = std::time::Instant::now();
    // let mut accumulator = Accumulator::new(efficient_layer);
    // // for _ in 0..1000 {
    // let mut game_state = GameState::new(&mut rng);

    // accumulator.set_state(&game_state);
    // loop {
    //     let result = game_state.get_possible_moves(&mut move_list, &mut rng);
    //     if result == MoveGenerationResult::GameOver {
    //         break;
    //     }

    //     if result == MoveGenerationResult::RoundOver {
    //         accumulator.set_state(&game_state);
    //     }
    //     println!("{}", game_state);
    //     // efficient_layer.reset();
    //     // player::mcts::neural_network::encoding::encode_game_state(
    //     //     &game_state,
    //     //     &mut efficient_layer,
    //     // );
    //     // let move_ = player::mcts::neural_network::factory_encoding::decode_move(
    //     //     &mut game_state,
    //     //     accumulator.output(),
    //     //     &move_lookup,
    //     // );
    //     // game_state.do_move(move_);
    //     accumulator.do_move(&mut game_state, move_);
    // }

    // println!("{}", game_state);
    // }

    // 3 - 4 ms per game
    // println!("{}", game_state);
    // println!(
    //     "Time taken: {:?}ms",
    //     start_time.elapsed().as_millis() as f64 / 1000.0
    // );
}

// fn main() {
//     let mut rng = SmallRng::from_entropy();
//     let mut all_unique_factories: HashSet<[u8; NUM_TILE_COLORS]> = HashSet::new();
//     for _ in 0..10_000 {
//         let mut factory = [0; NUM_TILE_COLORS];
//         for _ in 0..4 {
//             factory[rng.gen_range(0..NUM_TILE_COLORS)] += 1;
//         }
//         all_unique_factories.insert(factory);
//     }
//     assert_eq!(all_unique_factories.len(), 70);
//     let mut all_unique_factories: Vec<[u8; NUM_TILE_COLORS]> =
//         all_unique_factories.into_iter().collect();
//     all_unique_factories.push([0; NUM_TILE_COLORS]);

//     let mut reverse_lookup = [[0_u8; NUM_TILE_COLORS]; 71];
//     let mut lookup = [0_u8; 123];
//     for (index, factory) in all_unique_factories.iter().enumerate() {
//         let index = player::mcts::neural_network::encoding::hash_factory(factory) as usize;
//         // lookup[index] = index as u8;
//         reverse_lookup[index] = *factory;
//     }
//     println!("{:?}", reverse_lookup);
// }

// fn check_hash_function(
//     function: impl Fn(&[u8; NUM_TILE_COLORS]) -> u64,
//     all_unique_factories: &[[u8; NUM_TILE_COLORS]],
// ) -> (bool, u64) {
//     let mut hashes = HashSet::new();

//     // let mut result: [usize; 100] = [0; 100];
//     let mut max_hash = 0;
//     // let mut current_index = 0;
//     for factory in all_unique_factories.iter() {
//         let hash = function(factory);
//         // result[hash as usize] = current_index;
//         // current_index += 1;
//         if !hashes.insert(hash) {
//             return (false, u64::MAX);
//         }
//         max_hash = max_hash.max(hash);
//     }

//     // println!("{:?}", result);

//     (true, max_hash)
// }

// // fn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> usize {
// //     // let mut hash = 0u8;
// //     // let mut values = *factory;

// //     // values[1] *= 246;
// //     // values[2] ^= 159;
// //     // values[2] *= 120;
// //     // values[3] *= 21;
// //     // values[4] *= 202;
// //     // values[4] += 54;

// //     // for &value in values.iter() {
// //     //     hash += value;
// //     // }
// //     (((factory[1] * 246) as u64
// //         + ((factory[2] ^ 159) * 120) as u64
// //         + (factory[3] * 21) as u64
// //         + (factory[4] * 202 + 54) as u64)
// //         % 131) as usize

// //     // hash % 131
// // }

// // fn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> u8 {
// //     // let mut hash = 0u8;
// //     // let mut values = *factory;

// //     // values[0] *= 223;
// //     // values[1] >>= 1;
// //     // values[1] *= 80;
// //     // values[2] += 16;
// //     // values[2] <<= 2;
// //     // values[3] *= 84;

// //     // for &value in values.iter() {
// //     //     hash += value;
// //     // }

// //     // hash % 137

// //     ((factory[0] * 223) + ((factory[1] >> 1) * 80) + ((factory[2] + 16) << 2) + (factory[4] * 84))
// //         % 137
// // }

// // Thread found new best function! Loss: 98 Total loss: 98.11
// // fn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
// //     let mut hash = 0u64;
// //     let mut values = *factory;

// //     values[1] = values[1].rotate_left(6);
// //     values[0] *= 187;
// //     values[3] = values[3].rotate_right(3);
// //     values[3] ^= 87;
// //     values[3] = values[3].wrapping_mul(219);
// //     values[4] *= 100;
// //     values[3] >>= 1;
// //     values[0] ^= 200;
// //     values[0] -= 11;
// //     values[0] = values[0].rotate_right(5);
// //     values[1] -= 151;

// //     for &value in values.iter() {
// //         hash += value as u64;
// //     }

// //     hash % 103
// // }

// // fn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
// //     // ((((factory[0] * 187) ^ 200) - 11).rotate_right(5) as u64
// //     //     + ((factory[1].rotate_left(6) - 151) as u64)
// //     //     + ((factory[3].rotate_right(3) ^ 87).wrapping_mul(219) >> 1) as u64
// //     //     + (factory[4] * 100) as u64)
// //     //     % 103;
// //     ((factory[0].rotate_right(7) as u64)
// //         + (factory[1].rotate_right(2) as u64)
// //         + ((factory[2] * 106).rotate_left(7) ^ 228) as u64
// //         + ((factory[3] * 120).rotate_left(7) + 14) as u64
// //         + (factory[4] as u64))
// //         % 113
// // }

// // [RotateLeft(1, 6), Multiply(0, 187), RotateRight(3, 3), Xor(3, 87), WrappingMultiply(3, 219), Multiply(4, 100), ShiftRight(3, 1), Xor(0, 200), Subtract(0, 11), RotateRight(0, 5), Subtract(1, 151)]
// // 103

// // // 119
// // fn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
// //     let mut hash = 0u64;
// //     let mut values = *factory; // Clone factory values to hash_factory operations

// //     values[0] *= 52;
// //     values[4] = values[4].wrapping_mul(157);
// //     values[2] = values[2].wrapping_mul(224);
// //     values[3] = values[3].rotate_right(5);
// //     values[4] ^= 145;
// //     values[1] -= 84;
// //     values[1] ^= 208;
// //     values[0] = values[0].rotate_left(1);
// //     values[0] = values[0].wrapping_mul(42);

// //     // Combine all values into a single hash
// //     for &value in values.iter() {
// //         hash += value as u64;
// //     }

// //     hash % 139
// // }

// // 118
// // fn hash_factory2(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
// //     let mut hash = 0u64;
// //     let mut values = *factory; // Clone factory values to hash_factory operations

// //     values[0] *= 52;
// //     values[4] = values[4].wrapping_mul(157);
// //     values[2] = values[2].wrapping_mul(18);
// //     values[3] = values[3].rotate_right(5);
// //     values[4] ^= 145;
// //     values[1] -= 84;
// //     values[1] ^= 208;
// //     values[0] = values[0].rotate_left(1);
// //     values[0] = values[0].wrapping_mul(42);
// //     values[2] = values[2].rotate_left(2);
// //     values[2] = values[2].wrapping_mul(28);

// //     // Combine all values into a single hash
// //     for &value in values.iter() {
// //         hash += value as u64;
// //     }

// //     hash % 139
// // }

// /*
// 125
// fn hash_factory(&self, factory: &[u8; NUM_TILE_COLORS]) -> u64 {
//     let mut hash = 0u64;
//     let mut values = *factory; // Clone factory values to hash_factory operations

//     values[3] *= 59;
//     values[0] ^= 135;
//     values[0] = values[0].rotate_left(4);
//     values[1] >>= 7;
//     values[4] *= 155;
//     values[3] = values[3].wrapping_add(29);
//     values[3] ^= 8;
//     values[2] -= 186;
//     values[2] ^= 235;
//     values[1] = values[1].wrapping_sub(178);

//     // Combine all values into a single hash
//     for &value in values.iter() {
//         hash += value as u64;
//     }

//     hash % 163
// }
// */
// // Thread found new best function! Loss: 108 Total loss: 109
// // fn hash_factory3(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
// //     let mut hash = 0u64;
// //     let mut values = *factory; // Clone factory values to hash_factory operations

// //     values[3] >>= 1;
// //     values[1] ^= 207;
// //     values[2] <<= 3;
// //     values[2] -= 30;
// //     values[4] = values[4].wrapping_mul(110);
// //     values[4] ^= 135;
// //     values[2] ^= 22;
// //     values[3] *= 75;
// //     values[3] = values[3].rotate_right(7);
// //     values[3] = values[3].wrapping_mul(134);

// //     // Combine all values into a single hash
// //     for &value in values.iter() {
// //         hash += value as u64;
// //     }

// //     hash % 163
// // }

// /*
// Thread found new best function! Loss: 99 Total loss: 99
// fn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
//     let mut hash = 0u64;
//     let mut values = *factory;

//     values[4] = values[4].rotate_right(5);
//     values[0] ^= 158;
//     values[3] ^= 216;
//     values[0] *= 139;
//     values[3] *= 169;
//     values[2] = values[2].rotate_right(6);
//     values[0] += 14;
//     values[1] -= 232;
//     values[1] += 228;
//     values[0] = values[0].wrapping_sub(145);
//     values[0] = values[0].rotate_right(1);
//     values[0] ^= 82;
//     values[3] ^= 200;
//     values[2] += 120;
//     values[0] *= 28;
//     values[1] ^= 224;

//     for &value in values.iter() {
//         hash += value as u64;
//     }

//     hash % 109
// } */
// // Thread found new best function! Loss: 99 Total loss: 99.7
// // fn hash_factory4(factory: &[u8; NUM_TILE_COLORS]) -> u64 {
// //     let mut hash = 0u64;
// //     let mut values = *factory; // Clone factory values to hash_factory operations

// //     values[1] = values[1].rotate_left(7);
// //     values[4] *= 191;
// //     values[4] ^= 4;
// //     values[0] = values[0].wrapping_mul(9);
// //     values[4] += 234;
// //     values[2] ^= 15;
// //     values[1] = values[1].wrapping_mul(54);

// //     // Combine all values into a single hash
// //     for &value in values.iter() {
// //         hash += value as u64;
// //     }

// //     hash % 113
// // }

// /*
// New best function found! Loss: 123 Length penalty: 6 Total loss: 126
// fn hash_factory(&self, factory: &[u8; NUM_TILE_COLORS]) -> u64 {
//     let mut hash = 0u64;
//     let mut values = *factory; // Clone factory values to hash_factory operations

//     values[0] = values[0].rotate_left(3);
//     values[3] = values[3].wrapping_mul(26);
//     values[1] = values[1].rotate_left(1);
//     values[3] ^= 61;
//     values[4] = values[4].wrapping_mul(60);
//     values[0] ^= 36;

//     // Combine all values into a single hash
//     for &value in values.iter() {
//         hash += value as u64;
//     }

//     hash % 137
// } */
// const RELEVANT_PRIMES: [u8; 19] = [
//     71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163,
// ];

// #[derive(Debug, Clone)]
// enum Operation {
//     // WrappingAdd(usize, u8),      // Index, value to add
//     // WrappingSubtract(usize, u8), // Index, value to subtract
//     // WrappingMultiply(usize, u8), // Index, value to multiply
//     Add(usize, u8),      // Index, value to add
//     Subtract(usize, u8), // Index, value to subtract
//     Multiply(usize, u8), // Index, value to multiply
//     Xor(usize, u8),      // Index, value to XOR with
//     // RotateLeft(usize, u32),  // Index, number of bits to rotate left
//     // RotateRight(usize, u32), // Index, number of bits to rotate right
//     ShiftLeft(usize, u32),  // Index, number of bits to shift left
//     ShiftRight(usize, u32), // Index, number of bits to shift right
//     Div(usize, u8),         // Index, value to divide by
// }

// impl Operation {
//     fn get_index(&self) -> usize {
//         match self {
//             // Operation::WrappingAdd(index, _)
//             // | Operation::WrappingSubtract(index, _)
//             // | Operation::WrappingMultiply(index, _)
//             Operation::Add(index, _)
//             | Operation::Subtract(index, _)
//             | Operation::Multiply(index, _)
//             | Operation::Xor(index, _)
//             // | Operation::RotateLeft(index, _)
//             // | Operation::RotateRight(index, _)
//             | Operation::ShiftLeft(index, _)
//             | Operation::ShiftRight(index, _)
//             | Operation::Div(index, _) => *index,
//         }
//     }

//     fn random(rng: &mut StdRng, num_tile_colors: usize) -> Operation {
//         match rng.gen_range(1..=7) {
//             // 0 => Operation::WrappingAdd(
//             //     rng.gen_range(0..num_tile_colors),
//             //     rng.gen_range(1..=u64::MAX),
//             // ),
//             // 1 => Operation::WrappingSubtract(
//             //     rng.gen_range(0..num_tile_colors),
//             //     rng.gen_range(1..=u64::MAX),
//             // ),
//             // 2 => Operation::WrappingMultiply(
//             //     rng.gen_range(0..num_tile_colors),
//             //     rng.gen_range(1..=u64::MAX),
//             // ),
//             1 => Operation::Xor(rng.gen_range(0..num_tile_colors), rng.gen()),
//             // 4 => Operation::RotateLeft(rng.gen_range(0..num_tile_colors), rng.gen_range(1..31)),
//             // 5 => Operation::RotateRight(rng.gen_range(0..num_tile_colors), rng.gen_range(1..31)),
//             2 => Operation::ShiftLeft(rng.gen_range(0..num_tile_colors), rng.gen_range(1..7)),
//             3 => Operation::ShiftRight(rng.gen_range(0..num_tile_colors), rng.gen_range(1..7)),
//             4 => Operation::Add(
//                 rng.gen_range(0..num_tile_colors),
//                 rng.gen_range(1..=u8::MAX),
//             ),
//             5 => Operation::Subtract(
//                 rng.gen_range(0..num_tile_colors),
//                 rng.gen_range(1..=u8::MAX),
//             ),
//             6 => Operation::Multiply(
//                 rng.gen_range(0..num_tile_colors),
//                 rng.gen_range(1..=u8::MAX),
//             ),
//             7 => Operation::Div(
//                 rng.gen_range(0..num_tile_colors),
//                 rng.gen_range(2..=u8::MAX),
//             ),
//             _ => unreachable!(),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct HashFunction {
//     operations: Vec<Operation>,
//     modulo: u8,
// }

// impl HashFunction {
//     fn hash_factory(&self, factory: &[u8; NUM_TILE_COLORS]) -> u64 {
//         let mut hash = 0u64;
//         let mut values = *factory;

//         for op in &self.operations {
//             match *op {
//                 // Operation::WrappingAdd(index, x) => values[index] = values[index].wrapping_add(x),
//                 // Operation::WrappingSubtract(index, x) => {
//                 //     values[index] = values[index].wrapping_sub(x)
//                 // }
//                 // Operation::WrappingMultiply(index, x) => {
//                 //     values[index] = values[index].wrapping_mul(x)
//                 // }
//                 Operation::Add(index, x) => values[index] += x,
//                 Operation::Subtract(index, x) => values[index] -= x,
//                 Operation::Multiply(index, x) => values[index] *= x,
//                 Operation::Xor(index, x) => values[index] ^= x,
//                 // Operation::RotateLeft(index, bits) => {
//                 //     values[index] = values[index].rotate_left(bits)
//                 // }
//                 // Operation::RotateRight(index, bits) => {
//                 //     values[index] = values[index].rotate_right(bits)
//                 // }
//                 Operation::ShiftLeft(index, bits) => values[index] <<= bits,
//                 Operation::ShiftRight(index, bits) => values[index] >>= bits,
//                 Operation::Div(index, x) => {
//                     values[index] /= x;
//                 }
//             }
//         }

//         // Combine all values into a single hash
//         for &value in values.iter() {
//             hash += value as u64;
//         }

//         hash % self.modulo as u64
//     }

//     fn mutate(&mut self, rng: &mut StdRng, num_tile_colors: usize) {
//         let num_changes = rng.gen_range(1..=self.operations.len() / 2); // Randomly decide to make 1 to 3 changes

//         if rng.gen_bool(0.2) {
//             // with 99% probability, change the modulo to a lower prime
//             if rng.gen_bool(0.5) {
//                 // Randomly change the modulo
//                 let lower_primes: Vec<_> = RELEVANT_PRIMES
//                     .iter()
//                     .filter(|&&prime| prime < self.modulo)
//                     .cloned()
//                     .collect();

//                 // Randomly change the modulo if there are any lower primes available
//                 if !lower_primes.is_empty() {
//                     self.modulo = lower_primes[rng.gen_range(0..lower_primes.len())];
//                 }
//             } else {
//                 self.modulo = RELEVANT_PRIMES[rng.gen_range(0..RELEVANT_PRIMES.len())];
//             }
//         }

//         for _ in 0..num_changes {
//             match rng.gen_range(0..4) {
//                 0 => {
//                     // Modify an existing operation
//                     if !self.operations.is_empty() {
//                         let index = rng.gen_range(0..self.operations.len());
//                         self.operations[index] = Operation::random(rng, num_tile_colors);
//                     }
//                 }
//                 1 => {
//                     // Add a new operation
//                     if self.operations.len() < 30 {
//                         // Limit the maximum number of operations
//                         self.operations
//                             .push(Operation::random(rng, num_tile_colors));
//                     }
//                 }
//                 2 => {
//                     // Remove an operation
//                     if !self.operations.is_empty() {
//                         let index = rng.gen_range(0..self.operations.len());
//                         self.operations.remove(index);
//                     }
//                 }
//                 3 => {
//                     // Slightly modify the values in an existing operation
//                     if !self.operations.is_empty() {
//                         let index = rng.gen_range(0..self.operations.len());
//                         let adjustment = rng.gen_range(-15..=15);
//                         self.operations[index] = match self.operations[index] {
//                             // Operation::WrappingAdd(idx, x) => {
//                             //     Operation::Add(idx, (x as i64 + adjustment) as u8)
//                             // }
//                             // Operation::WrappingSubtract(idx, x) => {
//                             //     Operation::Subtract(idx, (x as i64 + adjustment) as u8)
//                             // }
//                             // Operation::WrappingMultiply(idx, x) => {
//                             //     Operation::Multiply(idx, (x as i64 + adjustment).max(1) as u8)
//                             // } // Ensure it's at least 1
//                             Operation::Add(idx, x) => {
//                                 Operation::Add(idx, (x as i64 + adjustment) as u8)
//                             }
//                             Operation::Subtract(idx, x) => {
//                                 Operation::Subtract(idx, (x as i64 + adjustment) as u8)
//                             }
//                             Operation::Multiply(idx, x) => {
//                                 Operation::Multiply(idx, (x as i64 + adjustment).max(1) as u8)
//                             } // Ensure it's at least 1
//                             Operation::Xor(idx, x) => {
//                                 let bit = 1 << rng.gen_range(0..=7);
//                                 Operation::Xor(idx, x ^ bit)
//                             }
//                             // Operation::RotateLeft(idx, bits) => Operation::RotateLeft(
//                             //     idx,
//                             //     (bits as i64 + adjustment).max(1).min(7) as u32,
//                             // ), // Valid bit range
//                             // Operation::RotateRight(idx, bits) => Operation::RotateRight(
//                             //     idx,
//                             //     (bits as i64 + adjustment).max(1).min(7) as u32,
//                             // ), // Valid bit range
//                             Operation::ShiftLeft(idx, bits) => Operation::ShiftLeft(
//                                 idx,
//                                 (bits as i64 + adjustment).max(1).min(7) as u32,
//                             ), // Valid bit range
//                             Operation::ShiftRight(idx, bits) => Operation::ShiftRight(
//                                 idx,
//                                 (bits as i64 + adjustment).max(1).min(7) as u32,
//                             ), // Valid bit range
//                             Operation::Div(idx, x) => Operation::Div(
//                                 idx,
//                                 (x as i64 + adjustment).max(1).min(u8::MAX as i64) as u8,
//                             ),
//                         };
//                     }
//                 }
//                 _ => unreachable!(),
//             }
//         }
//     }
// }

// impl fmt::Display for HashFunction {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(
//             f,
//             "\nfn hash_factory(factory: &[u8; NUM_TILE_COLORS]) -> u8 {{"
//         )?;
//         writeln!(f, "    let mut hash = 0u8;")?;
//         writeln!(f, "    let mut values = *factory;")?;
//         writeln!(f)?;

//         let mut ops = self.operations.clone();
//         ops.sort_by_key(|op| op.get_index());
//         for op in &ops {
//             match *op {
//                 // Operation::WrappingAdd(index, x) => writeln!(
//                 //     f,
//                 //     "    values[{}] = values[{}].wrapping_add({});",
//                 //     index, index, x
//                 // )?,
//                 // Operation::WrappingSubtract(index, x) => writeln!(
//                 //     f,
//                 //     "    values[{}] = values[{}].wrapping_sub({});",
//                 //     index, index, x
//                 // )?,
//                 // Operation::WrappingMultiply(index, x) => writeln!(
//                 //     f,
//                 //     "    values[{}] = values[{}].wrapping_mul({});",
//                 //     index, index, x
//                 // )?,
//                 Operation::Add(index, x) => writeln!(f, "    values[{}] += {};", index, x)?,
//                 Operation::Subtract(index, x) => writeln!(f, "    values[{}] -= {};", index, x)?,
//                 Operation::Multiply(index, x) => writeln!(f, "    values[{}] *= {};", index, x)?,
//                 Operation::Xor(index, x) => writeln!(f, "    values[{}] ^= {};", index, x)?,
//                 // Operation::RotateLeft(index, bits) => writeln!(
//                 //     f,
//                 //     "    values[{}] = values[{}].rotate_left({});",
//                 //     index, index, bits
//                 // )?,
//                 // Operation::RotateRight(index, bits) => writeln!(
//                 //     f,
//                 //     "    values[{}] = values[{}].rotate_right({});",
//                 //     index, index, bits
//                 // )?,
//                 Operation::ShiftLeft(index, bits) => {
//                     writeln!(f, "    values[{}] <<= {};", index, bits)?
//                 }
//                 Operation::ShiftRight(index, bits) => {
//                     writeln!(f, "    values[{}] >>= {};", index, bits)?
//                 }
//                 Operation::Div(index, x) => writeln!(f, "    values[{}] /= {};", index, x)?,
//             }
//         }

//         writeln!(f)?;
//         writeln!(f, "    for &value in values.iter() {{")?;
//         writeln!(f, "        hash += value;")?;
//         writeln!(f, "    }}")?;
//         writeln!(f)?;
//         writeln!(f, "    hash % {}", self.modulo)?;
//         writeln!(f, "}}")
//     }
// }

// fn random_hash_function(rng: &mut StdRng, num_ops: usize, num_tile_colors: usize) -> HashFunction {
//     HashFunction {
//         operations: (0..num_ops)
//             .map(|_| Operation::random(rng, num_tile_colors))
//             .collect(),
//         modulo: RELEVANT_PRIMES[rng.gen_range(0..RELEVANT_PRIMES.len())],
//     }
// }
// // Loss:      147[Subtract(3, 143), Xor(3, 234), RotateRight(2, 6), Subtract(3, 114), Subtract(4, 98), RotateLeft(0, 1), Multiply(1, 48), RotateLeft(2, 1), RotateRight(3, 2), Multiply(2, 251)]
// // fn main() {
// //     let mut rng = StdRng::from_entropy();

// //     let mut all_unique_factories: HashSet<[u8; NUM_TILE_COLORS]> = HashSet::new();
// //     for _ in 0..10_000 {
// //         let mut factory = [0; NUM_TILE_COLORS];
// //         for _ in 0..4 {
// //             factory[rng.gen_range(0..NUM_TILE_COLORS)] += 1;
// //         }
// //         all_unique_factories.insert(factory);
// //     }

// //     all_unique_factories.insert([0; NUM_TILE_COLORS]);
// //     assert_eq!(all_unique_factories.len(), 71);
// //     let all_unique_factories: Vec<[u8; NUM_TILE_COLORS]> =
// //         all_unique_factories.into_iter().collect();

// //     // let mut min_loss = f64::INFINITY;
// //     // let mut best_function = random_hash_function(&mut rng, 10, NUM_TILE_COLORS);

// //     // let mut iterations = 0;
// //     // loop {
// //     //     let function = if min_loss != f64::INFINITY {
// //     //         let mut function = best_function.clone();
// //     //         function.mutate(&mut rng, NUM_TILE_COLORS);
// //     //         function
// //     //     } else {
// //     //         random_hash_function(&mut rng, 10, NUM_TILE_COLORS)
// //     //     };

// //     //     let (is_valid, loss) = check_hash_function(
// //     //         |factory: &[u8; 5]| function.hash_factory(factory),
// //     //         &all_unique_factories,
// //     //     );
// //     //     let length_penalty = function.operations.len() as u64;
// //     //     let final_loss = loss as f64 + (length_penalty as f64) * 0.5;

// //     //     if iterations % 1000 == 0 && is_valid {
// //     //         println!("iterations: {:10} Loss: {}", iterations, final_loss);
// //     //         println!("{}", function);
// //     //         iterations += 1;
// //     //     }

// //     //     if is_valid && final_loss < min_loss {
// //     //         println!("{}", "#".repeat(80));
// //     //         println!(
// //     //             "New best function found! Loss: {} Length penalty: {} Total loss: {}",
// //     //             loss, length_penalty, final_loss
// //     //         );
// //     //         println!("{}", function);
// //     //         // println!("Loss: {:8}", loss);
// //     //         // println!("{}", function);
// //     //         min_loss = final_loss;
// //     //         best_function = function;
// //     //     }
// //     // }
// //     const INITIAL_NUM_OPS: usize = 8;

// //     let best_function = Arc::new(Mutex::new((
// //         random_hash_function(&mut rng, INITIAL_NUM_OPS, NUM_TILE_COLORS),
// //         f64::INFINITY,
// //     )));

// //     let mut handles = vec![];

// //     for _ in 0..16 {
// //         let best_function = Arc::clone(&best_function);
// //         let all_unique_factories = all_unique_factories.clone();
// //         let mut rng = StdRng::from_entropy();

// //         let handle = thread::spawn(move || {
// //             let mut local_best_function =
// //                 random_hash_function(&mut rng, INITIAL_NUM_OPS, NUM_TILE_COLORS);
// //             let mut local_min_total_loss = f64::INFINITY;
// //             let mut local_min_loss = f64::INFINITY;
// //             let mut iterations = 0;

// //             loop {
// //                 iterations += 1;
// //                 let function = {
// //                     if local_min_total_loss != f64::INFINITY {
// //                         let mut function = local_best_function.clone();
// //                         function.mutate(&mut rng, NUM_TILE_COLORS);
// //                         function
// //                     } else {
// //                         random_hash_function(&mut rng, 10, NUM_TILE_COLORS)
// //                     }
// //                 };

// //                 let (is_valid, loss) = check_hash_function(
// //                     |factory: &[u8; 5]| function.hash_factory(factory),
// //                     &all_unique_factories,
// //                 );
// //                 if !is_valid {
// //                     continue;
// //                 }

// //                 let length_penalty = function.operations.len() as u64;
// //                 let final_loss = loss as f64 + (length_penalty as f64) * 4.0;
// //                 if is_valid && final_loss < local_min_total_loss {
// //                     local_min_total_loss = final_loss;
// //                     local_best_function = function.clone();
// //                     local_min_loss = loss as f64;
// //                 }

// //                 if iterations % 10_000 == 0 {
// //                     let mut global_best = best_function.lock().unwrap();
// //                     if local_min_total_loss < global_best.1 {
// //                         *global_best = (local_best_function.clone(), local_min_total_loss);
// //                         println!("{}", "#".repeat(80));
// //                         println!(
// //                             "Thread found new best function! Loss: {} Total loss: {}",
// //                             local_min_loss, local_min_total_loss
// //                         );
// //                         println!("{}", local_best_function);
// //                         println!("{:?}", local_best_function.operations);
// //                         println!("{}", local_best_function.modulo);
// //                     } else {
// //                         local_best_function = global_best.0.clone();
// //                         local_min_total_loss = global_best.1;
// //                     }
// //                 }
// //             }
// //         });

// //         handles.push(handle);
// //     }

// //     for handle in handles {
// //         handle.join().unwrap();
// //     }
// // }

// // Loss:      114 [Multiply(2, 32), Multiply(1, 202), Add(2, 22), RotateLeft(0, 5), Multiply(2, 122), RotateRight(3, 6), RotateRight(0, 2), Add(3, 52), Xor(3, 210), Add(0, 74), Subtract(0, 2), Add(4, 104), Xor(1, 33), Xor(2, 42), Subtract(2, 205), Xor(3, 64), Add(0, 166), Multiply(1, 161), Add(1, 77), Add(3, 91)]

// // let game_state = GameState::new(&mut rng);
// // // let game_state = GameState::from_fen("2_1_0_21542077700_30233003015_0-0-0-0-0-33554432_66520066_4_34157462-134791574_12952207873-21474968065_4328588288-4278452739_1").unwrap();

// // for factory in game_state.factories.iter() {
// //     let hash = hash_factory(factory);
// //     println!("Factory: {:?} -> {}", factory, hash);
// // }

// // let mut players: Vec<Box<dyn Player>> = Vec::new();
// // for _ in 0..NUM_PLAYERS {
// //     let mut player = MonteCarloTreeSearch::default();
// //     player.set_time(TimeControl::SuddenDeath {
// //         total_milliseconds: 30_000,
// //     });

// //     players.push(Box::new(player));
// // }

// // run_match(game_state, &mut players, true).unwrap();

// // game_state.get_possible_moves(&mut move_list, &mut rng);
// // let random_move = move_list[rng.gen_range(0..move_list.len())];
// // println!("Random move: {}", random_move);
// // game_state.do_move(random_move);

// // println!("{}", game_state);

// // let mut mcts = MonteCarloTreeSearch::default();
// // let best_move = mcts.get_move(&game_state);
// // println!("Best move: {}", best_move);

// // mcts.advance_root(&game_state, None);

// // mcts.start_working();

// // std::thread::sleep(Duration::from_secs(1));

// // mcts.stop_working();
// // game_state.get_possible_moves(&mut move_list, &mut rng);
// // let random_move = move_list[rng.gen_range(0..move_list.len())];
// // println!("Random move: {}", random_move);
// // game_state.do_move(random_move);

// // tree.advance_root(&game_state, Some(Edge::Deterministic(random_move)));
// // std::thread::sleep(Duration::from_secs(5));

// // tree.stop_working();

// // let policy = mcts.policy().unwrap();
// // let value = mcts.value();
// // println!("Policy: {}", policy);
// // println!("Value: {}", value);

// // println!("Rated moves:");

// // let mut rated_moves: Vec<(Move, f32)> = mcts.rated_moves();
// // // rated_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
// // for (move_, rating) in rated_moves {
// //     println!("{} -> {}", move_, rating);
// // }
// // }
// // let mut root = Root::for_game_state(&game_state);
// // let mut move_list = MoveList::default();
// // for _ in 0..100_000 {
// //     root.get_node_mut()
// //         .iteration(&mut game_state.clone(), &mut move_list, &mut rng);
// // }

// // game_state.get_possible_moves(&mut move_list, &mut rng);

// // let random_move = move_list[rng.gen_range(0..move_list.len())];
// // println!("Random move: {}", random_move);

// // game_state.do_move(random_move);

// // // root.get_node_mut().get_children().iter().for_each(|node| {
// // //     println!("{}", node.get_edge());
// // // });

// // root.advance(&game_state, Some(Edge::Deterministic(random_move)));

// // let mut rng = StdRng::from_entropy();
// // // let game_state = GameState::from_fen("2_1_1_30266756869_0_4099-8449-196864-196609-65539-0_65864703_0_1099599-68440990_12901679104-8590065664_50331647-12868452351_0").unwrap();
// // // let game_state = GameState::new(&mut rng);
// // // let mut players: Vec<Box<dyn Player>> = Vec::new();
// // // players.push(Box::<HumanCommandLinePlayer>::default());
// // for _i in 0..NUM_PLAYERS {
// //     let mut player = MonteCarloTreeSearch::default();
// //     // player.set_time(TimeControl::ConstantTimePerMove {
// //     //     milliseconds_per_move: 5000,
// //     // });
// //     // player.set_time(TimeControl::ConstantIterationsPerMove {
// //     //     iterations_per_move: 10000,
// //     // });
// //     // player.set_time(TimeControl::SuddenDeath {
// //     //     total_milliseconds: 60_000 * 5,
// //     // });
// //     // player.set_time(TimeControl::Incremental {
// //     //     total_milliseconds: 120_000,
// //     //     increment_milliseconds: 24_000,
// //     // });
// //     // player.set_time(TimeControl::RealTimeIncremental {
// //     //     base_time_milliseconds: 60_000 * 2 + 12_000,
// //     //     increment_milliseconds: 24_000,
// //     //     max_time_milliseconds: 60_000 * 2 + 12_000,
// //     // });

// //     // let edge = Edge::Deterministic(random_move);
// //     // root.advance(&game_state, Some(edge));

// // }

// // let game_state = GameState::from_fen("2_0_1_47497088265_12884902658_65809-0-0-0-131074-8606777602_65668075_0_32900-8322_4328521729-12935233537_8606711555-8657043202_0").unwrap();
// // let mut mcts = MonteCarloTreeSearch::default();
// // mcts.set_time(TimeControl::ConstantTimePerMove {
// //     milliseconds_per_move: 6000,
// // });

// // mcts.get_move(&game_state);
// // }

// // fn main() {
// //     let mut rng = StdRng::from_entropy();
// //     let game_state = GameState::from_fen("2_0_0_69022387730_65792_0-0-0-0-0-4328521728_65537000_258_0-0_33751553-33686017_1095283900676-1095216726529_1").unwrap();
// //     // let game_state = GameState::new(&mut rng);
// //     let all_evaluations = Mutex::new(Vec::new());
// //     (0..15).into_par_iter().for_each(|_| {
// //         let mut mcts = MonteCarloTreeSearch::default();
// //         // mcts.set_time(TimeControl::ConstantTimePerMove {
// //         //     milliseconds_per_move: 1000,
// //         // });

// //         // mcts.get_move(&game_state);
// //         // mcts.set_root(&game_state);
// //         // mcts.set_pondering(true);
// //         // mcts.start_pondering();
// //         mcts.get_move(&game_state);
// //         std::thread::sleep(Duration::from_secs(20));
// //         // mcts.stop_pondering();
// //         // mcts.set_pondering(false);
// //         let mut evaluations = mcts.rated_moves();

// //         evaluations.sort_by(|a, b| {
// //             let a_score = a.1;
// //             let b_score = b.1;
// //             b_score.partial_cmp(&a_score).unwrap()
// //         });
// //         let evaluations: Vec<MoveEvaluation> = evaluations
// //             .into_iter()
// //             .map(|(m, e)| MoveEvaluation {
// //                 move_name: m,
// //                 score: e,
// //             })
// //             .collect();

// //         let mut all_evals = all_evaluations.lock().unwrap();
// //         all_evals.push(evaluations);
// //     });

// //     std::thread::sleep(Duration::from_secs(1));

// //     println!("{}", game_state);

// //     let all_evaluations = all_evaluations.into_inner().unwrap();
// //     // Print all next to each other
// //     for row in 0..all_evaluations[0].len() {
// //         for evaluations in all_evaluations.iter() {
// //             // let (m, e) = &evaluations[row];
// //             let m = &evaluations[row].move_name;
// //             let e = &evaluations[row].score;
// //             print!("{:23} -> {:6.4} | ", m.to_string(), e);
// //         }
// //         println!();
// //     }

// //     // Aggregate results
// //     let combined_results = aggregate_results(all_evaluations);

// //     // Decide on the best move
// //     let best_move = choose_best_move(&combined_results);
// //     println!("Best Move: {}", best_move.move_name);
// // }

// // 32371883
// //  1269858

// // pub struct Node {
// //     sibling: Option<Weak<RefCell<Node>>>,
// //     child: Option<Weak<RefCell<Node>>>,
// //     n: f32,
// //     q: [f32; 2],
// //     is_game_over: bool,
// //     has_probibalistic_children: bool,
// // }

// // pub struct OldNode {
// //     children: Vec<OldNode>,
// //     n: f32,
// //     q: [f32; 2],
// //     is_game_over: bool,
// //     has_probibalistic_children: bool,
// // }

// // impl Default for Node {
// //     fn default() -> Self {
// //         Node {
// //             sibling: None,
// //             child: None,
// //             n: 0.0,
// //             q: [0.0, 0.0],
// //             is_game_over: false,
// //             has_probibalistic_children: false,
// //         }
// //     }
// // }

// // impl Default for OldNode {
// //     fn default() -> Self {
// //         OldNode {
// //             children: Vec::new(),
// //             n: 0.0,
// //             q: [0.0, 0.0],
// //             is_game_over: false,
// //             has_probibalistic_children: false,
// //         }
// //     }
// // }

// // impl Node {
// //     fn add_child(&mut self, child_node: Rc<RefCell<Node>>) {
// //         match self.child {
// //             Some(ref weak) => {
// //                 let mut last = weak.upgrade().unwrap();
// //                 loop {
// //                     let next = {
// //                         let last_borrow = last.borrow();
// //                         if let Some(ref sibling) = last_borrow.sibling {
// //                             sibling.upgrade()
// //                         } else {
// //                             None
// //                         }
// //                     };

// //                     if let Some(next) = next {
// //                         last = next;
// //                     } else {
// //                         break;
// //                     }
// //                 }
// //                 last.borrow_mut().sibling = Some(Rc::downgrade(&child_node));
// //             }
// //             None => {
// //                 self.child = Some(Rc::downgrade(&child_node));
// //             }
// //         }
// //     }

// //     fn children(&self) -> Children {
// //         Children {
// //             next: self.child.clone(),
// //         }
// //     }
// // }

// // impl OldNode {
// //     fn add_child(&mut self, child_node: OldNode) {
// //         self.children.push(child_node);
// //     }

// //     fn children(&self) -> &Vec<OldNode> {
// //         &self.children
// //     }

// //     fn count_nodes(&self) -> usize {
// //         let mut count = 1;
// //         for child in self.children() {
// //             count += child.count_nodes();
// //         }
// //         count
// //     }
// // }

// // pub struct Children {
// //     next: Option<Weak<RefCell<Node>>>,
// // }

// // impl Iterator for Children {
// //     type Item = Rc<RefCell<Node>>;

// //     fn next(&mut self) -> Option<Self::Item> {
// //         self.next.take().and_then(|weak| {
// //             let strong = weak.upgrade();
// //             if let Some(node) = strong {
// //                 self.next = node.borrow().sibling.clone();
// //                 Some(node)
// //             } else {
// //                 None
// //             }
// //         })
// //     }
// // }

// // pub struct NodeStorage {
// //     nodes: Vec<Rc<RefCell<Node>>>,
// // }

// // impl NodeStorage {
// //     pub fn with_capacity(capacity: usize) -> Self {
// //         NodeStorage {
// //             nodes: Vec::with_capacity(capacity),
// //         }
// //     }

// //     pub fn add_node(&mut self, node: Node) -> Rc<RefCell<Node>> {
// //         let rc_node = Rc::new(RefCell::new(node));
// //         self.nodes.push(rc_node.clone());
// //         rc_node
// //     }

// //     pub fn count_nodes(&self) -> usize {
// //         self.nodes.len()
// //     }
// // }

// // fn main() {
// //     let mut storage = NodeStorage::with_capacity(1000);
// //     let root = storage.add_node(Node::default());

// //     let mut rng = thread_rng();

// //     let start_time = std::time::Instant::now();
// //     for _ in 0..1000 {
// //         // Perform a random walk
// //         let mut current_node = root.clone();

// //         // Random walk until a leaf is found or a random chance stops us.
// //         loop {
// //             let children: Vec<_> = current_node.borrow().children().collect();
// //             if children.is_empty() || !rng.gen_bool(0.5) {
// //                 break;
// //             }

// //             // Choose a random child to walk into
// //             let index = rng.gen_range(0..children.len());
// //             current_node = children[index].clone();
// //         }

// //         // Add children to the current node (leaf node)
// //         let children_count = rng.gen_range(20..=200);
// //         for _ in 0..children_count {
// //             let new_child = storage.add_node(Node::default());
// //             current_node.borrow_mut().add_child(new_child);
// //         }
// //     }

// //     let node_count = storage.count_nodes();
// //     println!("New Node count: {}", node_count);
// //     println!("Elapsed time: {:?}", start_time.elapsed());

// //     let mut old_root = OldNode::default();
// //     let start_time = std::time::Instant::now();
// //     for _ in 0..1000 {
// //         // Perform a random walk
// //         let mut current_node = &mut old_root;

// //         // Random walk until a leaf is found or a random chance stops us.
// //         loop {
// //             if current_node.children().is_empty() || !rng.gen_bool(0.5) {
// //                 break;
// //             }

// //             // Choose a random child to walk into
// //             let index = rng.gen_range(0..current_node.children().len());
// //             current_node = &mut current_node.children[index];
// //         }

// //         // Add children to the current node (leaf node)
// //         let children_count = rng.gen_range(20..=200);
// //         for _ in 0..children_count {
// //             current_node.add_child(OldNode::default());
// //         }
// //     }
// //     let end = start_time.elapsed();
// //     let nodes = old_root.count_nodes();
// //     println!("Old Node count: {}", nodes);
// //     println!("Elapsed time: {:?}", end);
// // }

// // fn main() {
// //     let mut rng = StdRng::from_entropy();
// //     // let game_state = GameState::deserialize_string("2_1_1_30266756869_0_4099-8449-196864-196609-65539-0_65864703_0_1099599-68440990_12901679104-8590065664_50331647-12868452351_0").unwrap();
// //     let game_state = GameState::new(&mut rng);
// //     let mut players: Vec<Box<dyn Player>> = Vec::new();
// //     // players.push(Box::<HumanCommandLinePlayer>::default());
// //     for _i in 0..NUM_PLAYERS {
// //         let mut player = MonteCarloTreeSearch::default();
// //         // player.set_time(TimeControl::ConstantTimePerMove {
// //         //     milliseconds_per_move: 5000,
// //         // });
// //         // player.set_time(TimeControl::ConstantIterationsPerMove {
// //         //     iterations_per_move: 10000,
// //         // });
// //         player.set_time(TimeControl::SuddenDeath {
// //             total_milliseconds: 60_000 * 5,
// //         });
// //         // player.set_time(TimeControl::Incremental {
// //         //     total_milliseconds: 120_000,
// //         //     increment_milliseconds: 24_000,
// //         // });
// //         // player.set_time(TimeControl::RealTimeIncremental {
// //         //     base_time_milliseconds: 60_000 * 2 + 12_000,
// //         //     increment_milliseconds: 24_000,
// //         //     max_time_milliseconds: 60_000 * 2 + 12_000,
// //         // });

// //         player.set_pondering(false);
// //         players.push(Box::new(player));
// //     }

// // //     let start_time = std::time::Instant::now();
// //     let _stats = run_match(game_state, &mut players, true).unwrap();
// //     // let num_turns = stats.num_turns;
// //     // let expected_duration = 0 * NUM_PLAYERS as u64 + 200 * (num_turns as u64);
// //     let elapsed_time = start_time.elapsed();
// //     println!("Elapsed time: {:?}", elapsed_time);
// //     // println!(
// //     "Expected duration: {:?}",
// //     Duration::from_millis(expected_duration)
// // );
// // let mut game_state = GameState::deserialize_string("2_0_0_68955345170_0_65809-4354-0-0-0-8590000384_65537000_1_0-0_33751553-12918456832_1095216858113-16712447_1").unwrap();
// // println!("{}", game_state);
// //     player.set_pondering(false);
// //     players.push(Box::new(player));
// // }

// // // let start_time = std::time::Instant::now();
// // let _stats = run_match(game_state, &mut players, true).unwrap();
// // let num_turns = stats.num_turns;
// // let expected_duration = 0 * NUM_PLAYERS as u64 + 200 * (num_turns as u64);
// // let elapsed_time = start_time.elapsed();
// // println!("Elapsed time: {:?}", elapsed_time);
// // println!(
// //     "Expected duration: {:?}",
// //     Duration::from_millis(expected_duration)
// // );
// // let mut game_state = GameState::deserialize_string("2_0_0_68955345170_0_65809-4354-0-0-0-8590000384_65537000_1_0-0_33751553-12918456832_1095216858113-16712447_1").unwrap();
// // println!("{}", game_state);

// // // 1G4->1@1

// // let move_ = Move {
// //     take_from_factory_index: 3,
// //     color: TileColor::Green,
// //     pattern: [1, 0, 0, 0, 0, 0],
// // };
// // println!("{}", move_);

// // game_state.do_move(move_);

// // let result = game_state.check_integrity().unwrap();

// // // Sleep 5s
// // std::thread::sleep(Duration::from_secs(50));

// // let _stats = run_match(game_state, &mut players, true).await.unwrap();
// // let scores = stats
// //     .player_statistics
// //     .iter()
// //     .map(|s| s.final_score)
// //     .collect::<Vec<_>>();

// // // let game_state = GameState::deserialize_string("4_0_3_3_16777218_0-0-0-0-0-65584-0-0-0-512_294990263901094930_33554434_140551134-68478855-34203535-70308830_4295164417-8590131712-50397697-17247175168_17163092736-8573289727-1095250412548-17230332927_1").unwrap();
// // // let game_state = GameState::deserialize_string("4_0_3_3_16777216_0-0-0-0-0-65584-0-0-259-8606712320_294990263901094930_33554432_140551134-68478855-34203535-70308830_4295098881-8590066176-50332161-8657240576_17163092736-8573289727-1095266927620-17230332927_1").unwrap();
// // // let game_state = GameState::deserialize_string("4_2_3_3_16777218_0-0-0-0-0-65584-0-0-0-8606712320_294990263901094930_33554434_140551134-68478855-34203535-70308830_4295164417-8590131712-50332161-8657240576_17163092736-8573289727-1095266927620-17230332927_1").unwrap();

// // // let game_state = GameState::deserialize_string("2_1_1_25954879495_30098588674_4609-66064-289-274-135184-0_65864678_0_1081740-8856_12885032960-4345364480_17163157503-17180000255_0").unwrap();
// // // let game_state = GameState::deserialize_string("2_0_1_56086956810_197377_0-65554-8209-0-0-8623554817_65799146_0_132-295172_4311875840-12918456832_12952011263-17196581631_0").unwrap();
// // let game_state = GameState::deserialize_string("2_0_0_64694194190_0_0-0-65569-0-0-8623555072_65537000_0_0-0_4295163905-33554944_17163157251-1095300481279_0").unwrap();
// // let mut mcts = MonteCarloTreeSearch::default();
// // mcts.set_pondering(true).await;
// // mcts.get_move(&game_state).await;
// // // mcts.start_pondering();

// // tokio::time::sleep(Duration::from_secs(10)).await;

// // mcts.stop_pondering();

// // let pv = mcts.get_principal_variation().await;
// // for event in pv.iter() {
// //     println!("{}", event);
// // }

// // mcts.set_time(60000 * 30).await;
// // mcts.get_move(&game_state).await;

// // mcts.store_tree(0.0);
// // // let value = Value::from_game_scores([71_i16, 54_i16, 71_i16, 81_i16]);
// // // println!("{}", value);
// // }
