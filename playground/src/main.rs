#![allow(unused_imports)]

use game::{match_::run_match, *};
use player::{
    command_line_player::HumanCommandLinePlayer,
    mcts::{
        edge::Edge,
        tree::{Root, Tree},
        HeuristicMoveGenerationPlayer, MonteCarloTreeSearch,
    },
    random_player::RandomPlayer,
};
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rayon::prelude::*;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    time::Duration,
};
use std::{collections::HashMap, sync::Mutex};

// struct MoveEvaluation {
//     move_name: Move,
//     score: f32,
// }

// fn choose_best_move(combined_results: &HashMap<Move, Vec<f32>>) -> MoveEvaluation {
//     combined_results
//         .iter()
//         .map(|(move_name, scores)| MoveEvaluation {
//             move_name: move_name.clone(),
//             score: scores.iter().sum::<f32>() / scores.len() as f32, // Average score
//         })
//         .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
//         .unwrap()
// }

// fn aggregate_results(all_evaluations: Vec<Vec<MoveEvaluation>>) -> HashMap<Move, Vec<f32>> {
//     let mut combined_results = HashMap::new();
//     for thread_results in all_evaluations {
//         for eval in thread_results {
//             combined_results
//                 .entry(eval.move_name)
//                 .or_insert_with(Vec::new)
//                 .push(eval.score);
//         }
//     }
//     combined_results
// }

fn main() {
    let mut rng = SmallRng::from_entropy();
    let game_state = GameState::new(&mut rng);
    // let game_state = GameState::from_fen("2_1_0_21542077700_30233003015_0-0-0-0-0-33554432_66520066_4_34157462-134791574_12952207873-21474968065_4328588288-4278452739_1").unwrap();

    let mut players: Vec<Box<dyn Player>> = Vec::new();
    for _ in 0..NUM_PLAYERS {
        let mut player = MonteCarloTreeSearch::default();
        player.set_time(TimeControl::SuddenDeath {
            total_milliseconds: 30_000,
        });

        players.push(Box::new(player));
    }

    run_match(game_state, &mut players, true).unwrap();
}

// game_state.get_possible_moves(&mut move_list, &mut rng);
// let random_move = move_list[rng.gen_range(0..move_list.len())];
// println!("Random move: {}", random_move);
// game_state.do_move(random_move);

// println!("{}", game_state);

// let mut mcts = MonteCarloTreeSearch::default();
// let best_move = mcts.get_move(&game_state);
// println!("Best move: {}", best_move);

// mcts.advance_root(&game_state, None);

// mcts.start_working();

// std::thread::sleep(Duration::from_secs(1));

// mcts.stop_working();
// game_state.get_possible_moves(&mut move_list, &mut rng);
// let random_move = move_list[rng.gen_range(0..move_list.len())];
// println!("Random move: {}", random_move);
// game_state.do_move(random_move);

// tree.advance_root(&game_state, Some(Edge::Deterministic(random_move)));
// std::thread::sleep(Duration::from_secs(5));

// tree.stop_working();

// let policy = mcts.policy().unwrap();
// let value = mcts.value();
// println!("Policy: {}", policy);
// println!("Value: {}", value);

// println!("Rated moves:");

// let mut rated_moves: Vec<(Move, f32)> = mcts.rated_moves();
// // rated_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
// for (move_, rating) in rated_moves {
//     println!("{} -> {}", move_, rating);
// }
// }
// let mut root = Root::for_game_state(&game_state);
// let mut move_list = MoveList::default();
// for _ in 0..100_000 {
//     root.get_node_mut()
//         .iteration(&mut game_state.clone(), &mut move_list, &mut rng);
// }

// game_state.get_possible_moves(&mut move_list, &mut rng);

// let random_move = move_list[rng.gen_range(0..move_list.len())];
// println!("Random move: {}", random_move);

// game_state.do_move(random_move);

// // root.get_node_mut().get_children().iter().for_each(|node| {
// //     println!("{}", node.get_edge());
// // });

// root.advance(&game_state, Some(Edge::Deterministic(random_move)));

// let mut rng = SmallRng::from_entropy();
// // let game_state = GameState::from_fen("2_1_1_30266756869_0_4099-8449-196864-196609-65539-0_65864703_0_1099599-68440990_12901679104-8590065664_50331647-12868452351_0").unwrap();
// // let game_state = GameState::new(&mut rng);
// // let mut players: Vec<Box<dyn Player>> = Vec::new();
// // players.push(Box::<HumanCommandLinePlayer>::default());
// for _i in 0..NUM_PLAYERS {
//     let mut player = MonteCarloTreeSearch::default();
//     // player.set_time(TimeControl::ConstantTimePerMove {
//     //     milliseconds_per_move: 5000,
//     // });
//     // player.set_time(TimeControl::ConstantIterationsPerMove {
//     //     iterations_per_move: 10000,
//     // });
//     // player.set_time(TimeControl::SuddenDeath {
//     //     total_milliseconds: 60_000 * 5,
//     // });
//     // player.set_time(TimeControl::Incremental {
//     //     total_milliseconds: 120_000,
//     //     increment_milliseconds: 24_000,
//     // });
//     // player.set_time(TimeControl::RealTimeIncremental {
//     //     base_time_milliseconds: 60_000 * 2 + 12_000,
//     //     increment_milliseconds: 24_000,
//     //     max_time_milliseconds: 60_000 * 2 + 12_000,
//     // });

//     // let edge = Edge::Deterministic(random_move);
//     // root.advance(&game_state, Some(edge));

// }

// let game_state = GameState::from_fen("2_0_1_47497088265_12884902658_65809-0-0-0-131074-8606777602_65668075_0_32900-8322_4328521729-12935233537_8606711555-8657043202_0").unwrap();
// let mut mcts = MonteCarloTreeSearch::default();
// mcts.set_time(TimeControl::ConstantTimePerMove {
//     milliseconds_per_move: 6000,
// });

// mcts.get_move(&game_state);
// }

// fn main() {
//     let mut rng = SmallRng::from_entropy();
//     let game_state = GameState::from_fen("2_0_0_69022387730_65792_0-0-0-0-0-4328521728_65537000_258_0-0_33751553-33686017_1095283900676-1095216726529_1").unwrap();
//     // let game_state = GameState::new(&mut rng);
//     let all_evaluations = Mutex::new(Vec::new());
//     (0..15).into_par_iter().for_each(|_| {
//         let mut mcts = MonteCarloTreeSearch::default();
//         // mcts.set_time(TimeControl::ConstantTimePerMove {
//         //     milliseconds_per_move: 1000,
//         // });

//         // mcts.get_move(&game_state);
//         // mcts.set_root(&game_state);
//         // mcts.set_pondering(true);
//         // mcts.start_pondering();
//         mcts.get_move(&game_state);
//         std::thread::sleep(Duration::from_secs(20));
//         // mcts.stop_pondering();
//         // mcts.set_pondering(false);
//         let mut evaluations = mcts.rated_moves();

//         evaluations.sort_by(|a, b| {
//             let a_score = a.1;
//             let b_score = b.1;
//             b_score.partial_cmp(&a_score).unwrap()
//         });
//         let evaluations: Vec<MoveEvaluation> = evaluations
//             .into_iter()
//             .map(|(m, e)| MoveEvaluation {
//                 move_name: m,
//                 score: e,
//             })
//             .collect();

//         let mut all_evals = all_evaluations.lock().unwrap();
//         all_evals.push(evaluations);
//     });

//     std::thread::sleep(Duration::from_secs(1));

//     println!("{}", game_state);

//     let all_evaluations = all_evaluations.into_inner().unwrap();
//     // Print all next to each other
//     for row in 0..all_evaluations[0].len() {
//         for evaluations in all_evaluations.iter() {
//             // let (m, e) = &evaluations[row];
//             let m = &evaluations[row].move_name;
//             let e = &evaluations[row].score;
//             print!("{:23} -> {:6.4} | ", m.to_string(), e);
//         }
//         println!();
//     }

//     // Aggregate results
//     let combined_results = aggregate_results(all_evaluations);

//     // Decide on the best move
//     let best_move = choose_best_move(&combined_results);
//     println!("Best Move: {}", best_move.move_name);
// }

// 32371883
//  1269858

// pub struct Node {
//     sibling: Option<Weak<RefCell<Node>>>,
//     child: Option<Weak<RefCell<Node>>>,
//     n: f32,
//     q: [f32; 2],
//     is_game_over: bool,
//     has_probibalistic_children: bool,
// }

// pub struct OldNode {
//     children: Vec<OldNode>,
//     n: f32,
//     q: [f32; 2],
//     is_game_over: bool,
//     has_probibalistic_children: bool,
// }

// impl Default for Node {
//     fn default() -> Self {
//         Node {
//             sibling: None,
//             child: None,
//             n: 0.0,
//             q: [0.0, 0.0],
//             is_game_over: false,
//             has_probibalistic_children: false,
//         }
//     }
// }

// impl Default for OldNode {
//     fn default() -> Self {
//         OldNode {
//             children: Vec::new(),
//             n: 0.0,
//             q: [0.0, 0.0],
//             is_game_over: false,
//             has_probibalistic_children: false,
//         }
//     }
// }

// impl Node {
//     fn add_child(&mut self, child_node: Rc<RefCell<Node>>) {
//         match self.child {
//             Some(ref weak) => {
//                 let mut last = weak.upgrade().unwrap();
//                 loop {
//                     let next = {
//                         let last_borrow = last.borrow();
//                         if let Some(ref sibling) = last_borrow.sibling {
//                             sibling.upgrade()
//                         } else {
//                             None
//                         }
//                     };

//                     if let Some(next) = next {
//                         last = next;
//                     } else {
//                         break;
//                     }
//                 }
//                 last.borrow_mut().sibling = Some(Rc::downgrade(&child_node));
//             }
//             None => {
//                 self.child = Some(Rc::downgrade(&child_node));
//             }
//         }
//     }

//     fn children(&self) -> Children {
//         Children {
//             next: self.child.clone(),
//         }
//     }
// }

// impl OldNode {
//     fn add_child(&mut self, child_node: OldNode) {
//         self.children.push(child_node);
//     }

//     fn children(&self) -> &Vec<OldNode> {
//         &self.children
//     }

//     fn count_nodes(&self) -> usize {
//         let mut count = 1;
//         for child in self.children() {
//             count += child.count_nodes();
//         }
//         count
//     }
// }

// pub struct Children {
//     next: Option<Weak<RefCell<Node>>>,
// }

// impl Iterator for Children {
//     type Item = Rc<RefCell<Node>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.next.take().and_then(|weak| {
//             let strong = weak.upgrade();
//             if let Some(node) = strong {
//                 self.next = node.borrow().sibling.clone();
//                 Some(node)
//             } else {
//                 None
//             }
//         })
//     }
// }

// pub struct NodeStorage {
//     nodes: Vec<Rc<RefCell<Node>>>,
// }

// impl NodeStorage {
//     pub fn with_capacity(capacity: usize) -> Self {
//         NodeStorage {
//             nodes: Vec::with_capacity(capacity),
//         }
//     }

//     pub fn add_node(&mut self, node: Node) -> Rc<RefCell<Node>> {
//         let rc_node = Rc::new(RefCell::new(node));
//         self.nodes.push(rc_node.clone());
//         rc_node
//     }

//     pub fn count_nodes(&self) -> usize {
//         self.nodes.len()
//     }
// }

// fn main() {
//     let mut storage = NodeStorage::with_capacity(1000);
//     let root = storage.add_node(Node::default());

//     let mut rng = thread_rng();

//     let start_time = std::time::Instant::now();
//     for _ in 0..1000 {
//         // Perform a random walk
//         let mut current_node = root.clone();

//         // Random walk until a leaf is found or a random chance stops us.
//         loop {
//             let children: Vec<_> = current_node.borrow().children().collect();
//             if children.is_empty() || !rng.gen_bool(0.5) {
//                 break;
//             }

//             // Choose a random child to walk into
//             let index = rng.gen_range(0..children.len());
//             current_node = children[index].clone();
//         }

//         // Add children to the current node (leaf node)
//         let children_count = rng.gen_range(20..=200);
//         for _ in 0..children_count {
//             let new_child = storage.add_node(Node::default());
//             current_node.borrow_mut().add_child(new_child);
//         }
//     }

//     let node_count = storage.count_nodes();
//     println!("New Node count: {}", node_count);
//     println!("Elapsed time: {:?}", start_time.elapsed());

//     let mut old_root = OldNode::default();
//     let start_time = std::time::Instant::now();
//     for _ in 0..1000 {
//         // Perform a random walk
//         let mut current_node = &mut old_root;

//         // Random walk until a leaf is found or a random chance stops us.
//         loop {
//             if current_node.children().is_empty() || !rng.gen_bool(0.5) {
//                 break;
//             }

//             // Choose a random child to walk into
//             let index = rng.gen_range(0..current_node.children().len());
//             current_node = &mut current_node.children[index];
//         }

//         // Add children to the current node (leaf node)
//         let children_count = rng.gen_range(20..=200);
//         for _ in 0..children_count {
//             current_node.add_child(OldNode::default());
//         }
//     }
//     let end = start_time.elapsed();
//     let nodes = old_root.count_nodes();
//     println!("Old Node count: {}", nodes);
//     println!("Elapsed time: {:?}", end);
// }

// fn main() {
//     let mut rng = SmallRng::from_entropy();
//     // let game_state = GameState::deserialize_string("2_1_1_30266756869_0_4099-8449-196864-196609-65539-0_65864703_0_1099599-68440990_12901679104-8590065664_50331647-12868452351_0").unwrap();
//     let game_state = GameState::new(&mut rng);
//     let mut players: Vec<Box<dyn Player>> = Vec::new();
//     // players.push(Box::<HumanCommandLinePlayer>::default());
//     for _i in 0..NUM_PLAYERS {
//         let mut player = MonteCarloTreeSearch::default();
//         // player.set_time(TimeControl::ConstantTimePerMove {
//         //     milliseconds_per_move: 5000,
//         // });
//         // player.set_time(TimeControl::ConstantIterationsPerMove {
//         //     iterations_per_move: 10000,
//         // });
//         player.set_time(TimeControl::SuddenDeath {
//             total_milliseconds: 60_000 * 5,
//         });
//         // player.set_time(TimeControl::Incremental {
//         //     total_milliseconds: 120_000,
//         //     increment_milliseconds: 24_000,
//         // });
//         // player.set_time(TimeControl::RealTimeIncremental {
//         //     base_time_milliseconds: 60_000 * 2 + 12_000,
//         //     increment_milliseconds: 24_000,
//         //     max_time_milliseconds: 60_000 * 2 + 12_000,
//         // });

//         player.set_pondering(false);
//         players.push(Box::new(player));
//     }

// //     let start_time = std::time::Instant::now();
//     let _stats = run_match(game_state, &mut players, true).unwrap();
//     // let num_turns = stats.num_turns;
//     // let expected_duration = 0 * NUM_PLAYERS as u64 + 200 * (num_turns as u64);
//     let elapsed_time = start_time.elapsed();
//     println!("Elapsed time: {:?}", elapsed_time);
//     // println!(
//     "Expected duration: {:?}",
//     Duration::from_millis(expected_duration)
// );
// let mut game_state = GameState::deserialize_string("2_0_0_68955345170_0_65809-4354-0-0-0-8590000384_65537000_1_0-0_33751553-12918456832_1095216858113-16712447_1").unwrap();
// println!("{}", game_state);
//     player.set_pondering(false);
//     players.push(Box::new(player));
// }

// // let start_time = std::time::Instant::now();
// let _stats = run_match(game_state, &mut players, true).unwrap();
// let num_turns = stats.num_turns;
// let expected_duration = 0 * NUM_PLAYERS as u64 + 200 * (num_turns as u64);
// let elapsed_time = start_time.elapsed();
// println!("Elapsed time: {:?}", elapsed_time);
// println!(
//     "Expected duration: {:?}",
//     Duration::from_millis(expected_duration)
// );
// let mut game_state = GameState::deserialize_string("2_0_0_68955345170_0_65809-4354-0-0-0-8590000384_65537000_1_0-0_33751553-12918456832_1095216858113-16712447_1").unwrap();
// println!("{}", game_state);

// // 1G4->1@1

// let move_ = Move {
//     take_from_factory_index: 3,
//     color: TileColor::Green,
//     pattern: [1, 0, 0, 0, 0, 0],
// };
// println!("{}", move_);

// game_state.do_move(move_);

// let result = game_state.check_integrity().unwrap();

// // Sleep 5s
// std::thread::sleep(Duration::from_secs(50));

// let _stats = run_match(game_state, &mut players, true).await.unwrap();
// let scores = stats
//     .player_statistics
//     .iter()
//     .map(|s| s.final_score)
//     .collect::<Vec<_>>();

// // let game_state = GameState::deserialize_string("4_0_3_3_16777218_0-0-0-0-0-65584-0-0-0-512_294990263901094930_33554434_140551134-68478855-34203535-70308830_4295164417-8590131712-50397697-17247175168_17163092736-8573289727-1095250412548-17230332927_1").unwrap();
// // let game_state = GameState::deserialize_string("4_0_3_3_16777216_0-0-0-0-0-65584-0-0-259-8606712320_294990263901094930_33554432_140551134-68478855-34203535-70308830_4295098881-8590066176-50332161-8657240576_17163092736-8573289727-1095266927620-17230332927_1").unwrap();
// // let game_state = GameState::deserialize_string("4_2_3_3_16777218_0-0-0-0-0-65584-0-0-0-8606712320_294990263901094930_33554434_140551134-68478855-34203535-70308830_4295164417-8590131712-50332161-8657240576_17163092736-8573289727-1095266927620-17230332927_1").unwrap();

// // let game_state = GameState::deserialize_string("2_1_1_25954879495_30098588674_4609-66064-289-274-135184-0_65864678_0_1081740-8856_12885032960-4345364480_17163157503-17180000255_0").unwrap();
// // let game_state = GameState::deserialize_string("2_0_1_56086956810_197377_0-65554-8209-0-0-8623554817_65799146_0_132-295172_4311875840-12918456832_12952011263-17196581631_0").unwrap();
// let game_state = GameState::deserialize_string("2_0_0_64694194190_0_0-0-65569-0-0-8623555072_65537000_0_0-0_4295163905-33554944_17163157251-1095300481279_0").unwrap();
// let mut mcts = MonteCarloTreeSearch::default();
// mcts.set_pondering(true).await;
// mcts.get_move(&game_state).await;
// // mcts.start_pondering();

// tokio::time::sleep(Duration::from_secs(10)).await;

// mcts.stop_pondering();

// let pv = mcts.get_principal_variation().await;
// for event in pv.iter() {
//     println!("{}", event);
// }

// mcts.set_time(60000 * 30).await;
// mcts.get_move(&game_state).await;

// mcts.store_tree(0.0);
// // let value = Value::from_game_scores([71_i16, 54_i16, 71_i16, 81_i16]);
// // println!("{}", value);
// }
