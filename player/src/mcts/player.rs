use super::node::Node;
use super::value::Value;
use crate::mcts::edge::Edge;
use game::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::mpsc;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct MonteCarloTreeSearch {
    name: String,
    time_control: TimeControl,
    elapsed_time: u64,
    bonus_time: u64,

    root_node: Arc<Mutex<Option<Node>>>,
    root_game_state: GameState,

    stop_pondering_sender: Option<mpsc::Sender<()>>,
    allow_pondering: bool,
    is_pondering: Arc<Mutex<bool>>,
}

fn do_iterations(
    root_node: &mut Node,
    root_game_state: &GameState,
    iterations: u64,
    rng: &mut SmallRng,
) -> f32 {
    let mut move_list = MoveList::new();
    let mut sum_game_length = 0.0;
    for _ in 0..iterations {
        let (_, game_length) =
            root_node.iteration(&mut root_game_state.clone(), &mut move_list, rng);
        sum_game_length += game_length as f32;
    }

    sum_game_length
}

impl MonteCarloTreeSearch {
    fn set_root(&mut self, game_state: &GameState) {
        game_state
            .check_integrity()
            .expect("Trying to set root with invalid game state.");

        if self.root_game_state.to_fen() == game_state.to_fen()
            && self.root_node.lock().unwrap().is_some()
        {
            #[cfg(not(feature = "mute"))]
            println!("Keeping parts of the tree from previous search.");
        } else {
            self.root_game_state = game_state.clone();
            *self.root_node.lock().unwrap() = Some(Node::new_deterministic(Move::DUMMY));
        }
    }

    pub fn start_pondering(&mut self) {
        if !self.allow_pondering {
            return;
        }

        let mut rng = SmallRng::from_entropy();
        let root_node = self.root_node.clone();
        let root_game_state = self.root_game_state.clone();
        let (tx, rx) = mpsc::channel::<()>();

        #[cfg(not(feature = "mute"))]
        let mut pv: Vec<Edge> = Vec::with_capacity(100);
        let is_pondering = self.is_pondering.clone();

        std::thread::spawn(move || {
            {
                let mut is_pondering = is_pondering.lock().unwrap();
                if *is_pondering {
                    #[cfg(not(feature = "mute"))]
                    println!("Already pondering. Cannot start pondering again.");
                    return;
                }
                *is_pondering = true;
            }

            #[cfg(not(feature = "mute"))]
            let ponder_start_time = Instant::now();
            #[cfg(not(feature = "mute"))]
            let mut last_log_time = Instant::now();
            #[cfg(not(feature = "mute"))]
            let mut iterations = 0;
            while rx.try_recv().is_err() {
                let mut root_node = root_node.lock().unwrap();
                if let Some(root_node) = root_node.as_mut() {
                    #[cfg(not(feature = "mute"))]
                    {
                        iterations += 100;
                    }
                    do_iterations(root_node, &root_game_state, 100, &mut rng);
                    #[cfg(not(feature = "mute"))]
                    // if last_log_time.elapsed().as_secs() > 30 {
                    if last_log_time.elapsed().as_millis() > 500 {
                        pv.truncate(0);
                        root_node.build_pv(&mut root_game_state.clone(), &mut pv);
                        println!(
                            "Pondering - Value: {:7} PV-Depth: {} PV: {}",
                            root_node.get_value(),
                            pv.len(),
                            pv.iter()
                                .map(|edge| edge.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        last_log_time = Instant::now();
                    }
                } else {
                    break;
                }
            }

            #[cfg(not(feature = "mute"))]
            println!(
                "Pondering stopped after {}ms with {} iterations.",
                ponder_start_time.elapsed().as_millis(),
                iterations
            );
            {
                let mut is_pondering = is_pondering.lock().unwrap();
                *is_pondering = false;
            }
        });
        self.stop_pondering_sender = Some(tx);
    }

    pub fn stop_pondering(&mut self) {
        if let Some(sender) = self.stop_pondering_sender.take() {
            let _ = sender.send(());
        }
    }

    fn search(&mut self, game_state: &GameState) -> Move {
        #[cfg(not(feature = "mute"))]
        println!("Searching move using MCTS. Fen: {}", game_state.to_fen());

        let start_time = Instant::now();
        self.stop_pondering();
        self.set_root(game_state);
        let mut rng = SmallRng::from_entropy();
        let mut pv: Vec<Edge> = Vec::with_capacity(100);
        let mut iterations_per_ms = 1.; // Initial guess on the lower end for four players, will be adjusted later
        let mut completed_iterations: u64 = 0;
        let search_start_time = Instant::now();

        let factories: &[[u8; 5]; NUM_FACTORIES] = game_state.get_factories();
        let all_empty = factories
            .iter()
            .all(|factory| factory.iter().all(|&tile| tile == 0));
        if all_empty {
            panic!("Monte Carlo Tree search was started in a position where it is not possible to make a move.");
        }

        #[cfg(not(feature = "mute"))]
        println!(
            "    Left PV-Depth Iterations Value{}Principal variation",
            " ".repeat(NUM_PLAYERS * 5 - "Value".len())
        );

        // Scope in which the root node is locked
        let best_move;
        let mut sum_game_length = 0.0;
        let mut expected_game_length = f32::NAN;
        {
            let mut root_node = self.root_node.lock().unwrap();
            let root_node = root_node.as_mut().unwrap();

            loop {
                pv.truncate(0);
                root_node.build_pv(&mut self.root_game_state.clone(), &mut pv);

                let (iterations, time_info) = match self.time_control {
                    TimeControl::ConstantTimePerMove {
                        milliseconds_per_move,
                    } => {
                        let time_left: i64 =
                            milliseconds_per_move as i64 - start_time.elapsed().as_millis() as i64;
                        let time_info = format!("{}ms", time_left);
                        if time_left < 30 {
                            (0, time_info)
                        } else {
                            (
                                ((time_left as f32 / 6.).min(5000.) * iterations_per_ms).max(1.)
                                    as u64,
                                time_info,
                            )
                        }
                    }
                    TimeControl::ConstantIterationsPerMove {
                        iterations_per_move,
                    } => {
                        let remaining_iterations = iterations_per_move - completed_iterations;
                        let next_iterations =
                            remaining_iterations.min(200_000.min(iterations_per_move / 10));
                        (next_iterations, format!("{}", remaining_iterations))
                    }
                    TimeControl::SuddenDeath { total_milliseconds } => {
                        // let total_remaining_time = total_milliseconds as i64 - self.elapsed_time as i64;// - start_time.elapsed().as_millis() as i64;
                        let total_time_for_this_search = (total_milliseconds as f32
                            - self.elapsed_time as f32)
                            / (expected_game_length - 1.0)
                            * NUM_PLAYERS as f32;
                        let remaining_time =
                            total_time_for_this_search - start_time.elapsed().as_millis() as f32;
                        let time_info: String = format!(
                            "{:.0}ms",
                            remaining_time + total_milliseconds as f32 - self.elapsed_time as f32
                        );
                        if expected_game_length.is_nan() {
                            (1000, time_info)
                        } else if remaining_time < 30.0 {
                            (0, time_info)
                        } else {
                            // let time_left = total_remaining_time as f32 / expected_game_length;
                            // println!("{}", remaining_time);
                            (
                                ((remaining_time / 6.).min(5000.) * iterations_per_ms).max(1.)
                                    as u64,
                                time_info,
                            )
                        }
                    }
                    _ => panic!("Time control not implemented."),
                };

                #[cfg(not(feature = "mute"))]
                println!(
                    "{:>8} {:>8} {:10} {:18} {}",
                    time_info,
                    format!("{}/≈{:.1}", pv.len(), expected_game_length - 1.0),
                    completed_iterations,
                    root_node.get_value(),
                    pv.iter()
                        .map(|edge| edge.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                if iterations == 0 {
                    break;
                }

                sum_game_length +=
                    do_iterations(root_node, &self.root_game_state, iterations, &mut rng);
                completed_iterations += iterations;
                expected_game_length = sum_game_length / completed_iterations as f32;

                let elapsed_time = search_start_time.elapsed().as_micros() as f32 / 1000.;
                if elapsed_time > 0. {
                    iterations_per_ms = completed_iterations as f32 / elapsed_time
                }
            }

            #[cfg(not(feature = "mute"))]
            println!(
                "Search finished after {}ms. Value: {:7} PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
                start_time.elapsed().as_millis(),
                root_node.get_value(),
                pv.len(),
                completed_iterations,
                iterations_per_ms * 1000.,
                pv.iter()
                    .map(|edge| edge.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            let player_index = usize::from(game_state.get_current_player());
            #[cfg(not(feature = "mute"))]
            println!("{:?}", root_node.count_nodes());
            best_move = root_node.best_move(player_index).unwrap();
        }

        {
            let mut game_state = game_state.clone();
            game_state.do_move(best_move);
            self.notify_move(&game_state, best_move);
        }

        self.elapsed_time += start_time.elapsed().as_millis() as u64;

        best_move
    }

    pub fn store_tree(&self, min_visits: f32) {
        let mut current_id = 0;
        let mut data = String::from("digraph G {\n"); // Start of the DOT graph
                                                      // Settings for the graph
        data.push_str("graph [overlap=scale, scale=2];\n");
        data.push_str("node [width=.3, height=.3, fixedsize=true];\n");
        data.push_str("edge [penwidth=0.5];\n");

        if let Some(root_node) = &self.root_node.lock().unwrap().as_ref() {
            root_node.store_node(0, &mut current_id, &mut data, min_visits);
        }
        data.push_str("}\n"); // End of the DOT graph

        std::fs::write("logs/tree.dot", data).expect("Unable to write file");
        println!("Tree stored in logs/tree.dot");
    }

    pub fn get_principal_variation(&mut self) -> Vec<Edge> {
        let mut pv: Vec<Edge> = Vec::new();
        if let Some(root_node) = &mut self.root_node.lock().unwrap().as_mut() {
            root_node.build_pv(&mut self.root_game_state.clone(), &mut pv);
        }
        pv
    }

    pub fn get_value(&self) -> Option<Value> {
        self.root_node
            .lock()
            .unwrap()
            .as_ref()
            .as_ref()
            .map(|root_node| root_node.get_value())
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            name: "Monte Carlo Tree Search".to_string(),
            root_node: Arc::new(Mutex::new(None)),
            root_game_state: GameState::new(&mut rng),
            time_control: TimeControl::ConstantTimePerMove {
                milliseconds_per_move: 6000,
            },
            elapsed_time: 0,
            bonus_time: 0,
            stop_pondering_sender: None,
            allow_pondering: false,
            is_pondering: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl Player for MonteCarloTreeSearch {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_move(&mut self, game_state: &GameState) -> Move {
        self.search(game_state)
    }

    fn set_time(&mut self, time_control: TimeControl) {
        self.elapsed_time = 0;
        self.bonus_time = 0;
        self.time_control = time_control;
    }

    fn set_pondering(&mut self, pondering: bool) {
        self.allow_pondering = pondering;
        if !pondering {
            self.start_pondering();
        } else {
            self.stop_pondering();
        }
    }

    fn notify_move(&mut self, new_game_state: &GameState, last_move: Move) {
        new_game_state
            .check_integrity()
            .expect("Trying to set root with invalid game state.");
        if new_game_state.to_fen() == self.root_game_state.to_fen() {
            return;
        }

        #[cfg(not(feature = "mute"))]
        println!(
            "Notifying MCTS of move {}. Fen: {}",
            last_move,
            new_game_state.to_fen()
        );
        self.stop_pondering();
        let mut root_node = self.root_node.lock().unwrap();
        let mut success = false;
        if let Some(new_root_node) = root_node
            .take()
            .and_then(|root_node| root_node.take_child_with_move(last_move))
        {
            self.root_game_state.do_move(last_move);
            if new_game_state.to_fen() == self.root_game_state.to_fen() {
                self.root_game_state = new_game_state.clone();
                *root_node = Some(new_root_node);
                #[cfg(not(feature = "mute"))]
                println!("Successfully applied the move {} to the tree.", last_move);
                success = true;
            }
        }

        if !success {
            #[cfg(not(feature = "mute"))]
            println!("Could not apply the move {} to the tree.", last_move);
            *root_node = None;
            self.root_game_state = new_game_state.clone();
        }

        drop(root_node);
        self.start_pondering();
    }

    fn notify_game_over(&mut self, _game_state: &GameState) {
        self.stop_pondering();
    }

    fn reset(&mut self) {
        self.stop_pondering();
        *self.root_node.lock().unwrap() = None;
        self.elapsed_time = 0;
        self.bonus_time = 0;
    }

    fn notify_factories_refilled(&mut self, game_state: &GameState) {
        self.stop_pondering();
        *self.root_node.lock().unwrap() = Some(Node::new_deterministic(Move::DUMMY));
        self.root_game_state = game_state.clone();
        self.start_pondering();
    }
}
