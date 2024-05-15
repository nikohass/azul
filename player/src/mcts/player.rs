use super::node::Node;
use super::time_control::MctsTimeControl;
use super::value::Value;
use crate::mcts::edge::Edge;
use game::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::{
    sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex},
    time::Instant,
};

pub struct MonteCarloTreeSearch {
    name: String,
    time_control: MctsTimeControl,
    root_node: Arc<Mutex<Option<Node>>>,
    root_game_state: GameState,

    stop_flag: Arc<AtomicBool>,
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
        sum_game_length += game_length as f32 - 1.0;
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
        let stop_flag = Arc::new(AtomicBool::new(false));

        #[cfg(not(feature = "mute"))]
        let mut pv: Vec<Edge> = Vec::with_capacity(100);
        let is_pondering = self.is_pondering.clone();
        let stop_flag_clone = stop_flag.clone();
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
            let mut root_node = root_node.lock().unwrap();
            if let Some(root_node) = root_node.as_mut() {
                while !stop_flag_clone.load(Ordering::Relaxed) {
                    #[cfg(not(feature = "mute"))]
                    {
                        iterations += 100;
                    }
                    do_iterations(root_node, &root_game_state, 100, &mut rng);
                    #[cfg(not(feature = "mute"))]
                    if last_log_time.elapsed().as_secs() > 30 {
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
        self.stop_flag = stop_flag;
    }

    pub fn stop_pondering(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    fn search(&mut self, game_state: &GameState) -> Move {
        #[cfg(not(feature = "mute"))]
        println!(
            "Searching move using MCTS. Fen: {}, Time Control: {}",
            game_state.to_fen(),
            self.time_control
        );

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
        {
            let player_string = (0..NUM_PLAYERS)
                .map(|i| format!("  V{}", i + 1))
                .collect::<Vec<String>>()
                .join(" ");
            println!(
                "{:>10} {:>3} {:>9} {} {:>8} {:>8} {:>8} {:>2}",
                "Iterations",
                "PVd",
                "Avg.Plies",
                player_string,
                "Stop",
                "Total",
                "Speed",
                "Principal Variation"
            );
        }

        // Scope in which the root node is locked
        let best_move;
        let mut sum_game_length = 0.0;
        let mut estimated_remaining_plies = f32::NAN;
        {
            let mut root_node = self.root_node.lock().unwrap();
            let root_node = root_node.as_mut().unwrap();

            loop {
                pv.truncate(0);
                root_node.build_pv(&mut self.root_game_state.clone(), &mut pv);

                let (iterations, remaining_time_info) = self.time_control.get_num_next_iterations(
                    search_start_time,
                    completed_iterations,
                    iterations_per_ms,
                    estimated_remaining_plies,
                );

                #[cfg(not(feature = "mute"))]
                println!(
                    "{:10} {:3} {:>9} {} {:>8} {:>8} {:>8} {}",
                    completed_iterations,
                    pv.len(),
                    format!("{:.1}", estimated_remaining_plies),
                    root_node.get_value(),
                    remaining_time_info
                        .time_left_for_search
                        .map_or("N/A".to_string(), |v| format!("{}ms", v)),
                    remaining_time_info
                        .time_left_for_game
                        .map_or("N/A".to_string(), |v| format!("{}ms", v)),
                    format!("{:.0}/ms", iterations_per_ms),
                    pv.iter()
                        .map(|edge| edge.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );

                if iterations == 0 {
                    break;
                }

                sum_game_length +=
                    do_iterations(root_node, &self.root_game_state, iterations, &mut rng);
                completed_iterations += iterations;
                estimated_remaining_plies = sum_game_length / completed_iterations as f32;

                let elapsed_time = search_start_time.elapsed().as_micros() as f64 / 1000.;
                if elapsed_time > 0. {
                    iterations_per_ms = completed_iterations as f64 / elapsed_time
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
            time_control: MctsTimeControl::new(TimeControl::ConstantTimePerMove {
                milliseconds_per_move: 6000,
            }),
            stop_flag: Arc::new(AtomicBool::new(false)),
            allow_pondering: false,
            is_pondering: Arc::new(Mutex::new(false)),
        }
    }
}

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
        self.time_control = MctsTimeControl::new(time_control);
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
        self.time_control.reset();
    }

    fn notify_factories_refilled(&mut self, game_state: &GameState) {
        self.stop_pondering();
        *self.root_node.lock().unwrap() = Some(Node::new_deterministic(Move::DUMMY));
        self.root_game_state = game_state.clone();
        self.start_pondering();
    }
}
