use super::{
    edge::Edge, node::Node, playout::PlayoutPolicy, time_control::RemainingTimeInfo, value::Value,
};
use game::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex, MutexGuard, RwLock,
    },
    thread::JoinHandle,
    time::Instant,
};

const ROOT_WORKER_STEP_TIME: u64 = 10; // Adjust speed and check for new commands every 10 ms

#[derive(Debug, Clone, Default)]
pub struct RootStatistics {
    pub visits: u64,
    pub sum_plies: u64,
    pub principal_variation: Vec<Edge>,
    pub value: Value,
    pub speed: f64,
    pub top_two_ratio: f64,
    pub best_move: Option<Move>,
    pub branching_factor: usize,
}

impl RootStatistics {
    pub fn average_plies(&self) -> Option<f64> {
        if self.visits == 0 {
            None
        } else {
            Some(self.sum_plies as f64 / self.visits as f64)
        }
    }

    pub fn print_header() {
        let player_string = (0..NUM_PLAYERS)
            .map(|i| format!("  V{}", i + 1))
            .collect::<Vec<String>>()
            .join(" ");
        log::debug!(
            "{:>10} {:>3} {:>9} {} {:>8} {:>8} {:>8} {:>9} {:>2}",
            "Iterations",
            "PVd",
            "Avg.Plies",
            player_string,
            "Stop",
            "Total",
            "Speed",
            "Top2Ratio",
            "Principal Variation"
        );
    }

    pub fn print(&self, remaining_time_info: &RemainingTimeInfo, search_start_time: Instant) {
        log::debug!(
            "{:10} {:3} {:>9} {} {:>8} {:>8} {:>9} {:>8} {}",
            self.visits,
            self.principal_variation.len(),
            format!("{:.1}", self.average_plies().unwrap_or(0.0)),
            self.value,
            remaining_time_info
                .current_search_allocated_time
                .map(|t| (t as f32) / 1000. - search_start_time.elapsed().as_secs_f32())
                .map_or("N/A".to_string(), |v| format!("{:.1}s", v)),
            remaining_time_info
                .game_remaining_time
                .map(|t| (t as f32) / 1000. - search_start_time.elapsed().as_secs_f32())
                .map_or("N/A".to_string(), |v| format!("{:.1}s", v)),
            format!("{:.0}/s", self.speed * 1000.),
            format!("{:.2}", self.top_two_ratio),
            self.principal_variation
                .iter()
                .map(|edge| edge.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
}

pub struct Root {
    node: Node,
    game_state: GameState,
    statistics: RootStatistics,
}

impl Root {
    pub fn new(node: Node, game_state: &GameState) -> Self {
        Self {
            node,
            game_state: game_state.clone(),
            statistics: RootStatistics::default(),
        }
    }

    pub fn for_game_state(game_state: &GameState) -> Self {
        Self::new(Node::new_deterministic(Move::DUMMY), game_state)
    }

    pub fn advance(mut self, game_state: &GameState, edge: Option<Edge>) -> Self {
        if game_state.to_fen() == self.game_state.to_fen() {
            return self;
        }

        self.statistics = RootStatistics::default();

        let edge = match edge {
            Some(edge) => edge,
            None => {
                return Self::for_game_state(game_state);
            }
        };

        // Find the edge in the current tree
        let new_root_node = self.node.take_child_with_edge(&edge);

        match new_root_node {
            Some(new_root_node) => {
                edge.apply_to_game_state(&mut self.game_state);
                Self::new(new_root_node, &self.game_state)
            }
            None => Self::for_game_state(game_state),
        }
    }

    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn value(&self) -> Value {
        self.node.value()
    }

    pub fn do_iterations<P: PlayoutPolicy>(
        &mut self,
        move_list: &mut MoveList,
        num_iterations: u64,
        rng: &mut SmallRng,
        playout_policy: &mut P,
    ) {
        let mut sum_played_plies: u64 = 0;
        for _ in 0..num_iterations {
            let (_, played_plies) =
                self.node
                    .iteration(&mut self.game_state.clone(), move_list, rng, playout_policy);
            sum_played_plies += played_plies as u64;
        }

        self.statistics.visits += num_iterations;
        self.statistics.sum_plies += sum_played_plies;
    }

    pub fn update_statistics(&mut self) {
        self.statistics.value = self.value();
        self.statistics.principal_variation.clear();
        self.node.build_principal_variation(
            &mut self.game_state.clone(),
            &mut self.statistics.principal_variation,
        );
        self.statistics.speed = f64::NAN;
        let current_player = usize::from(self.game_state.current_player);
        self.statistics.top_two_ratio = self.node.top_two_ratio(current_player);
        self.statistics.best_move = self.node.best_move(current_player);
        self.statistics.branching_factor = self.node.children().len();
    }
}

#[derive(Debug, Clone)]
enum Command {
    StartWorking,
    StopWorking,
    AdvanceRoot(GameState, Option<Edge>),
    TerminateThread,
    Verbose(bool),
}

pub struct Tree<P: PlayoutPolicy> {
    root: Arc<Mutex<Option<Root>>>,
    thread_handle: Option<JoinHandle<()>>,
    sender: Sender<Command>,
    root_statistics: Arc<RwLock<Option<RootStatistics>>>,

    _playout_policy: std::marker::PhantomData<P>,
}

impl<P: PlayoutPolicy> Tree<P> {
    pub fn start_working(&self) {
        self.sender.send(Command::StartWorking).unwrap();
    }

    pub fn stop_working(&self) {
        self.sender.send(Command::StopWorking).unwrap();
    }

    pub fn policy(&mut self) -> Option<Move> {
        let stats = self.root_statistics.read().unwrap();
        stats.as_ref().and_then(|stats| stats.best_move)
    }

    pub fn value(&mut self) -> Option<Value> {
        let stats = self.root_statistics.read().unwrap();
        stats.as_ref().map(|stats| stats.value)
    }

    pub fn advance_root(&self, game_state: &GameState, edge: Option<Edge>) {
        self.sender
            .send(Command::AdvanceRoot(game_state.clone(), edge))
            .unwrap();
        if let Ok(mut root_statistics) = self.root_statistics.write() {
            *root_statistics = None;
        }
    }

    pub fn verbose(&self, verbose: bool) {
        self.sender.send(Command::Verbose(verbose)).unwrap();
    }

    pub fn rated_moves(&mut self) -> Vec<(Move, f64)> {
        self.sender.send(Command::StopWorking).unwrap();
        let root = self.root.lock().unwrap();
        let root = root.as_ref().unwrap();
        let current_player = root.game_state().current_player;
        let mut rated_moves = Vec::new();
        for node in root.node.children() {
            let move_ = if let Some(move_) = node.previous_move() {
                move_
            } else {
                continue;
            };
            rated_moves.push((move_, node.value()[usize::from(current_player)]));
        }
        rated_moves
    }

    pub fn action_value_pairs(&mut self) -> Vec<(Move, Value)> {
        self.sender.send(Command::StopWorking).unwrap();
        let root = self.root.lock().unwrap();
        let root = root.as_ref().unwrap();
        let mut action_value_pairs = Vec::new();
        for node in root.node.children() {
            let move_ = if let Some(move_) = node.previous_move() {
                move_
            } else {
                continue;
            };
            action_value_pairs.push((move_, node.value()));
        }
        action_value_pairs
    }

    pub fn principal_variation(&mut self) -> Vec<Edge> {
        if let Some(stats) = self.root_statistics.read().unwrap().as_ref() {
            stats.principal_variation.clone()
        } else {
            Vec::new()
        }
    }

    pub fn root_statistics(&self) -> Option<RootStatistics> {
        self.root_statistics.read().unwrap().clone()
    }
}

impl<P: PlayoutPolicy> Default for Tree<P> {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel::<Command>();
        let root = Arc::new(Mutex::new(None));
        let root_clone = root.clone();
        let root_statistics = Arc::new(RwLock::new(None));
        let root_statistics_clone = root_statistics.clone();

        let thread_handle = std::thread::spawn(move || {
            let mut playout_policy = P::default();
            let mut running = false;
            let mut iterations_per_step: u64 = 100;
            let mut completed_iterations: u64 = 0;
            let mut start_time = Instant::now();
            let mut root_lock: Option<MutexGuard<Option<Root>>> = None;
            let mut move_list = MoveList::default();
            let mut rng = SmallRng::from_entropy();
            let mut verbose = false;
            let mut last_print_time = Instant::now();

            loop {
                if let Ok(command) = receiver.try_recv() {
                    match command {
                        Command::StartWorking => {
                            running = true;
                            // Reset speed tracking variables
                            iterations_per_step = 100;
                            completed_iterations = 0;
                            start_time = Instant::now();
                            let mut start_time = Instant::now();
                            while root_lock.is_none() {
                                if let Ok(lock) = root_clone.try_lock() {
                                    root_lock = Some(lock);
                                } else {
                                    let elapsed = start_time.elapsed().as_millis();
                                    if elapsed > 1000 {
                                        log::warn!("Tree worker is waiting for root lock for more than 1 second");
                                        start_time = Instant::now();
                                    }
                                }
                            }
                            root_statistics_clone.write().unwrap().replace(RootStatistics::default());
                        }
                        Command::StopWorking => {
                            running = false;
                            root_lock = None;
                            root_statistics_clone.write().unwrap().replace(RootStatistics::default());
                        }
                        Command::AdvanceRoot(game_state, edge) => {
                            if let Some(root_lock) = &mut root_lock {
                                if let Some(root) = root_lock.take() {
                                    let new_root = root.advance(&game_state, edge);
                                    **root_lock = Some(new_root);
                                }
                            } else {
                                let mut root_lock = root_clone.lock().unwrap();
                                if let Some(root) = root_lock.take() {
                                    let new_root = root.advance(&game_state, edge);
                                    *root_lock = Some(new_root);
                                } else {
                                    *root_lock = Some(Root::for_game_state(&game_state));
                                }
                            }
                            #[cfg(not(feature = "mute"))]
                            if verbose {
                                RootStatistics::print_header();
                            }
                        }
                        Command::TerminateThread => {
                            #[cfg(not(feature = "mute"))]
                            log::info!("Terminating thread");
                            break;
                        }
                        Command::Verbose(v) => {
                            verbose = v;
                            if verbose {
                                RootStatistics::print_header();
                            }
                        }
                    }
                }

                if running {
                    if let Some(root_lock) = &mut root_lock {
                        if let Some(root) = root_lock.as_mut() {
                            root.do_iterations(
                                &mut move_list,
                                iterations_per_step,
                                &mut rng,
                                &mut playout_policy,
                            );
                            completed_iterations += iterations_per_step;
                            // Adjust the number of iterations per step based on the time it took to complete the last step
                            let elapsed_time = start_time.elapsed().as_micros() as f64 / 1000.;
                            let iterations_per_ms = completed_iterations as f64 / elapsed_time;
                            iterations_per_step =
                                (iterations_per_ms as u64 * ROOT_WORKER_STEP_TIME).max(1);

                            root.update_statistics();
                            root.statistics.speed = iterations_per_ms;
                            if let Ok(mut root_statistics) = root_statistics_clone.try_write() {
                                if root_statistics.is_some() {
                                    *root_statistics = Some(root.statistics.clone());
                                }
                            }

                            if verbose && last_print_time.elapsed().as_millis() >= 500 {
                                root.statistics.print(
                                    &RemainingTimeInfo {
                                        current_search_allocated_time: None,
                                        game_remaining_time: None,
                                    },
                                    start_time,
                                );
                                last_print_time = Instant::now();
                            }

                            continue;
                        }
                    }
                }

                iterations_per_step = 100;
                completed_iterations = 0;
                start_time = Instant::now();
                std::thread::sleep(std::time::Duration::from_millis(ROOT_WORKER_STEP_TIME));
            }
        });

        Self {
            root,
            thread_handle: Some(thread_handle),
            sender,
            root_statistics,

            _playout_policy: std::marker::PhantomData,
        }
    }
}

impl<P: PlayoutPolicy> Drop for Tree<P> {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::TerminateThread);
        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().unwrap();
        }
    }
}
