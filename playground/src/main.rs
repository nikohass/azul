#![allow(unused_imports)]

use game::*;
use match_::run_match;
use player::{
    command_line_player::HumanCommandLinePlayer,
    mcts::{
        apply_parameters,
        edge::Edge,
        evaluate_single_move, get_heuristic_move,
        neural_network::{
            encoding::{build_move_lookup, TOTAL_ENCODING_SIZE},
            encoding_v2::{pattern_lines, Accumulator},
            layers::{
                apply_relu,
                dequantize_i8,
                quantize_i32,
                quantize_i8,
                DenseLayer,
                EfficentlyUpdatableDenseLayer,
                InputLayer as _,
                Layer, // QuantizedDenseLayer,
            },
            model::Model,
        },
        to_input_vec, Flatten as _, HeuristicGameState, HeuristicPlayoutParams,
        HeuristicPlayoutPolicy, MonteCarloTreeSearch, PlayoutPolicy as _, RandomPlayoutPolicy,
        Value, DEFAULT_PARAMS,
    },
    random_player::RandomPlayer,
};
use rand::rngs::{SmallRng, StdRng};
use rand::{thread_rng, Rng, SeedableRng};
use rayon::prelude::*;
use replay_buffer::{buffer::ReplayEntry, ReplayBufferClient};
use shared::logging::init_logging;
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

/*

BAG: B 5	Y 11	R 5	G 0	W 13
OUT OF BAG: B 7	Y 4	R 8	G 12	W 0

   [1] ....  [2] ....  [3] ....  [4] ....  [5] ....  [C]

 Neural network           46 |  Random                    0
-----------------------------+-----------------------------
                             |
 1         .  ->  B Y R G W  |  1         .  ->  B Y R G .
 2       . G  ->  . B Y R .  |  2       . .  ->  . B . R .
 3     . . .  ->  . W B Y R  |  3     . G G  ->  . . B Y .
 4   . . . .  ->  . . W B .  |  4   . . . G  ->  . . W B .
 5 . . . . R  ->  . . G . .  |  5 . . W W W  ->  . R G . .
                             |
-----------------------------+-----------------------------
 -1 -1 -2 -2 -2 -3 -3      0 |  -1 -1 -2 -2 -2 -3 -3      0
*/

use std::arch::x86_64::*;

// pub fn run_match(
//     mut game_state: GameState,
//     players: &mut [MonteCarloTreeSearch],
//     verbose: bool,
// ) -> Result<Vec<ReplayEntry>, GameError> {
//     let num_players = players.len();
//     if num_players != NUM_PLAYERS {
//         return Err(GameError::PlayerCountMismatch);
//     }

//     let player_names = players
//         .iter()
//         .map(|player| player.get_name().to_string())
//         .collect::<Vec<_>>();

//     game_state.check_integrity()?;
//     for player in players.iter_mut() {
//         player.notify_factories_refilled(&game_state);
//     }
//     let mut stats = Vec::new();

//     let mut move_list = MoveList::default();
//     let mut rng = SmallRng::from_entropy();
//     loop {
//         if verbose {
//             println!("{}", display_gamestate(&game_state, Some(&player_names)));
//             println!("{}", game_state.to_fen());
//         }
//         let result = game_state.get_possible_moves(&mut move_list, &mut rng);
//         let is_game_over = matches!(result, MoveGenerationResult::GameOver);
//         let refilled_factories = matches!(result, MoveGenerationResult::RoundOver);
//         if is_game_over {
//             break;
//         }
//         if refilled_factories && verbose {
//             println!("Factories refilled");
//             println!("{}", display_gamestate(&game_state, Some(&player_names)));
//         }
//         if refilled_factories {
//             for player in players.iter_mut() {
//                 player.notify_factories_refilled(&game_state);
//             }
//         }

//         // stats.num_factory_refills += refilled_factories as u32;
//         // stats.num_turns += 1;

//         let current_player_marker: PlayerMarker = game_state.current_player;
//         let current_player = usize::from(current_player_marker);

//         let mut players_move: Move = players[current_player].get_move(&game_state);
//         if verbose {
//             println!(
//                 "{}: {} {:?}",
//                 player_names[current_player], players_move, players_move
//             );
//         }

//         if !move_list.contains(players_move) {
//             // If the move is not legal, return an error
//             println!(
//                 "Player {} made an illegal move: {:?}",
//                 current_player, players_move
//             );
//             println!("Move list: {:?}", move_list);
//             println!("{}", display_gamestate(&game_state, Some(&player_names)));
//             println!("{}", game_state.to_fen());
//             return Err(GameError::IllegalMove);
//         }

//         let value_f64 = players[current_player].value().unwrap();
//         let mut value: [f32; NUM_PLAYERS] = [0.0; NUM_PLAYERS];
//         for i in 0..NUM_PLAYERS {
//             value[i] = value_f64[i] as f32;
//         }

//         let action_value_pairs = players[current_player].action_value_pairs();
//         // .action_value_pairs()
//         // .iter()
//         // .map(|(action, value)| (action, value.into()))
//         // .collect();
//         let action_value_pairs = action_value_pairs
//             .iter()
//             .map(|(action, value)| {
//                 let action = *action;
//                 let value_f64: [f64; NUM_PLAYERS] = std::convert::From::from(*value);
//                 let mut value: [f32; NUM_PLAYERS] = [0.0; NUM_PLAYERS];
//                 for i in 0..NUM_PLAYERS {
//                     value[i] = value_f64[i] as f32;
//                 }
//                 (action, value)
//             })
//             .collect();
//         stats.push(ReplayEntry {
//             game_state: game_state.clone(),
//             value,
//             iterations: 0,
//             action_value_pairs,
//         });

//         if rng.gen_bool(0.6) {
//             players_move = move_list[rng.gen_range(0..move_list.len())];
//         }

//         game_state.do_move(players_move);

//         for player in players.iter_mut() {
//             player.notify_move(&game_state, players_move);
//         }

//         game_state.check_integrity()?;
//     }
//     if verbose {
//         println!("{}", display_gamestate(&game_state, Some(&player_names)));
//         println!("{}", game_state.to_fen());
//     }

//     // Reset the players
//     for player in players.iter_mut() {
//         player.notify_game_over(&game_state);
//         player.reset();
//     }

//     Ok(stats)
// }

// fn main() {
//     // let mut rng = SmallRng::from_entropy();
//     // // let game_state = GameState::new(&mut rng);
//     // let mut players: Vec<Box<dyn Player>> = vec![
//     //     Box::<MonteCarloTreeSearch>::default(),
//     //     Box::<MonteCarloTreeSearch>::default(),
//     // ];

//     let client = ReplayBufferClient::new("http://127.0.0.1:3044");
//     client.set_buffer_size(1_000_000).unwrap();

//     const THREADS: usize = 12;
//     let mut handles = vec![];

//     let (statistics_sender, statistics_receiver) = mpsc::channel();
//     let stop_flag = Arc::new(AtomicBool::new(false));

//     for _ in 0..THREADS {
//         let statistics_sender = statistics_sender.clone();
//         let stop_flag = stop_flag.clone();

//         let mut rng = SmallRng::from_entropy();
//         let handle = thread::spawn(move || loop {
//             let mut players = vec![
//                 MonteCarloTreeSearch::default(),
//                 MonteCarloTreeSearch::default(),
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

//     const NUM_STATE_ACTION_PAIRS: usize = 500_000;

//     let handle = thread::spawn(move || {
//         let mut state_action_pairs = Vec::new();
//         let mut saved: u64 = 0;
//         let client = ReplayBufferClient::new("http://127.0.0.1:3044");

//         while let Ok(statistics) = statistics_receiver.recv() {
//             println!(
//                 "Received statistics with {} state-action pairs",
//                 statistics.len()
//             );
//             state_action_pairs.extend(statistics);

//             if state_action_pairs.len() >= NUM_STATE_ACTION_PAIRS.min(100) {
//                 saved += state_action_pairs.len() as u64;
//                 client.add_entries(state_action_pairs.clone()).unwrap();
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

pub fn meta_playout(game_state: GameState, players: &mut [Box<dyn Player>]) -> Value {
    let match_statistics = run_match(game_state, players, false).unwrap();
    let final_state = match_statistics
        .state_action_pairs
        .last()
        .unwrap()
        .0
        .clone();
    Value::from_game_scores(final_state.scores)
}

fn main() {
    init_logging("playground");

    let mut rng = SmallRng::from_entropy();
    let game_state = GameState::new(&mut rng);
    let mut players: Vec<Box<dyn Player>> = vec![];

    let base_time_milliseconds = 12900;
    let increment_milliseconds = 2400;
    for _ in 0..NUM_PLAYERS {
        let mut mcts = MonteCarloTreeSearch::default();
        mcts.set_time(TimeControl::FischerTimingWithMaxTime {
            base_time_milliseconds,
            increment_milliseconds,
            max_time_milliseconds: base_time_milliseconds,
        });
        players.push(Box::new(mcts));
    }
    players[0].set_time(TimeControl::FischerTimingWithMaxTime {
        base_time_milliseconds: 129_000,
        increment_milliseconds: 24_000,
        max_time_milliseconds: 129_000,
    });

    let stats = run_match(game_state, &mut players, true).unwrap();

    for (player_index, player_stats) in stats.player_statistics.iter().enumerate() {
        let total_time_used = player_stats.total_time_used();
        // println!(
        //     "Player {} used {:.3}s",
        //     player_index, total_time_used as f64 / 1_000.0
        // );
        let moves_made = player_stats.executed_moves.len();
        let time_increment_gained = increment_milliseconds * moves_made as u64;
        let total_max_time = time_increment_gained + base_time_milliseconds;
        let time_percentage_used = total_time_used as f64 / total_max_time as f64;
        println!(
            "Player {} used {}s in total to make {} moves. The maximum time would have been {}s. The player used {:.2}%",
            player_index,
            total_time_used as f64 / 1_000.0,
            moves_made,
            total_max_time as f64 / 1_000.0,
            time_percentage_used * 100.0
        );
    }

    for (player_index, player_stats) in stats.player_statistics.iter().enumerate() {
        println!("Used time for player {}", player_index);
        for time in player_stats.executed_moves.iter().map(|(_, _, time)| time) {
            println!("{:.3}s", *time as f64 / 1_000.0);
        }
    }
}
