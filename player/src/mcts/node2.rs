use std::time::Instant;

use game::{GameState, Move, MoveList, Player, TileColor, NUM_FACTORIES, NUM_TILE_COLORS};
use rand::{rngs::SmallRng, Rng, SeedableRng};

#[derive(Debug, Clone, Default, Copy)] // TODO: Default just for swapping root
struct ProbabilisticOutcome {
    pub factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES],
    pub out_of_bag: [u8; NUM_TILE_COLORS],
    pub bag: [u8; NUM_TILE_COLORS],
}

impl ProbabilisticOutcome {
    pub fn apply_to_game_state(&self, game_state: &mut GameState) {
        let orig = game_state.clone();
        // Before each factory refill, the last round ends, so evaluate the round
        game_state.evaluate_round();
        let orig_out_of_bag = game_state.get_out_of_bag();
        let after_eval = game_state.clone();

        // Overwrite the factories with the outcome of the event
        game_state.set_factories(self.factories);
        // The number of tiles in and out of bag also changes when the factories are refilled, so overwrite those as well
        game_state.set_out_of_bag(self.out_of_bag);
        game_state.set_bag(self.bag);

        // For debugging purposes, check the integrity of the game state
        if game_state.check_integrity().is_err() {
            println!("Original game state:");
            println!("{}", orig);
            println!("After evaluation:");
            println!("{:?}", orig_out_of_bag);
            println!("{}", after_eval);
            println!("Probabilistic outcome:");
            println!("{:?}", game_state.get_out_of_bag());
            println!("{}", game_state);

            println!("Out of bag would have been {:?}", self.out_of_bag);
            panic!("Probabilistic outcome was applied to an invalid game state.");
        }
    }
}

const EXPLORATION_FACTOR: f32 = 0.0;
const EXPLORATION_BASELINE: f32 = 120.0;
const EXPLORATION_ADJUSTMENT_RATIO: f32 = std::f32::consts::SQRT_2;
const FIRST_PLAY_URGENCY_ADJUSTMENT: f32 = 0.3;

#[derive(Debug, Clone)]
enum Event {
    Deterministic(Move),
    Probabilistic(ProbabilisticOutcome),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Deterministic(move_) => write!(f, "{}", move_),
            Event::Probabilistic(outcome) => {
                let mut string = String::new();
                for (factory_index, factory) in
                    outcome.factories.iter().take(NUM_FACTORIES - 1).enumerate()
                {
                    if factory_index > 0 {
                        string.push(' ');
                    }
                    for (color, number_of_tiles) in factory.iter().enumerate() {
                        string.push_str(
                            &TileColor::from(color)
                                .to_string()
                                .repeat(*number_of_tiles as usize),
                        );
                    }
                }
                write!(f, "{}", string)
            }
        }
    }
}

struct Node {
    children: Vec<Node>,   // The children of this node
    previous_event: Event, // The edge from the parent to this node
    n: f32,
    q: f32,
    is_game_over: bool,
    has_probabilistic_children: bool,
}

impl Node {
    pub fn new_deterministic(previous_move: Move) -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Deterministic(previous_move),
            n: 0.,
            q: 0.,
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    pub fn new_probabilistic(outcome: ProbabilisticOutcome) -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Probabilistic(outcome),
            n: 0.,
            q: 0.,
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    #[inline]
    fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    fn get_uct_value(&self, parent_n: f32, c: f32, fpu_base: f32, is_root: bool) -> f32 {
        if is_root {
            if self.n > 0. {
                self.q / self.n + c * (parent_n.ln() / self.n).sqrt()
            } else {
                std::f32::INFINITY
            }
        } else if self.n > 0. {
            self.q / self.n + c * (parent_n.ln() / self.n).sqrt()
        } else {
            fpu_base + c * parent_n.ln().sqrt()
        }
    }

    fn child_with_max_uct_value(&mut self, is_root: bool) -> &mut Node {
        let c_adjusted = EXPLORATION_FACTOR
            + EXPLORATION_ADJUSTMENT_RATIO
                * ((1. + self.n + EXPLORATION_BASELINE) / EXPLORATION_BASELINE).ln();
        let fpu_base = (self.n - self.q) / self.n - FIRST_PLAY_URGENCY_ADJUSTMENT;

        let mut best_child_index = 0;
        let mut best_chuld_uct_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_uct_value(self.n, c_adjusted, fpu_base, is_root);
            if value > best_chuld_uct_value {
                best_child_index = i;
                best_chuld_uct_value = value;
            }
        }

        &mut self.children[best_child_index]
    }

    fn probabilistic_child_with_max_uct_value(&mut self, rng: &mut SmallRng) -> &mut Node {
        let num_children = self.children.len();
        &mut self.children[rng.gen_range(0..num_children)]
    }

    fn backpropagate(&mut self, value: f32) {
        self.n += 1.;
        self.q += value;
    }

    fn expand(&mut self, game_state: &mut GameState, move_list: &mut MoveList, rng: &mut SmallRng) {
        let (is_game_over, probabilistic_event) = game_state.get_possible_moves(move_list, rng);
        self.is_game_over = is_game_over;

        if is_game_over {
            return;
        }

        // Create a child for each possible move
        let mut children: Vec<Node> = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            children.push(Node::new_deterministic(move_list[i]))
        }

        if probabilistic_event {
            // If we have a probabilistic event, create a child for this outcome.
            // We do not expand all possible outcomes, because that would be too expensive.
            let outcome = ProbabilisticOutcome {
                factories: *game_state.get_factories(),
                out_of_bag: game_state.get_out_of_bag(),
                bag: game_state.get_bag(),
            };
            let mut child = Node::new_probabilistic(outcome);
            self.has_probabilistic_children = true;
            // We have already generated the moves for this outcome, so we can just copy them over
            child.children.append(&mut children);
            self.children.push(child);
        } else {
            // Otherwise, just add the children we generated
            std::mem::swap(&mut self.children, &mut children);
        }
    }

    fn iteration(
        &mut self,
        game_state: &mut GameState,
        move_list: &mut MoveList,
        rng: &mut SmallRng,
        is_root: bool,
    ) -> f32 {
        let current_player = u8::from(game_state.get_current_player());
        let mut invert_delta = false;

        if self.has_probabilistic_children {
            invert_delta = true; // This node doesnt change the current player, so we need to invert the delta

            // All children of this node are probabilistic. When this node was "expanded", we only expanded one probabilistic outcome.
            // There would be too many possible outcomes to expand all of them, so we just expanded one.
            // Now we need to adjust for this and dynamically expand the other outcomes.
            // Here we also need to balance exploration and exploitation.
            // If we only visit the only child and never expand further, our strategy will be quite bad because we basically assume that the probabilistic event will always happen.
            // If we expand a new child every time we iterate this node, we would never visit the same child twice. This would cause our estimations of the value of the child to be very inaccurate.

            let next_round_starting_player = game_state.get_next_round_starting_player();
            let next_round_starting_player = u8::from(next_round_starting_player);
            if next_round_starting_player != current_player {
                // The next round will start with the other player, so we need to invert the delta
                invert_delta = !invert_delta;
            }

            // Let's just try this:
            let desired_number_of_children = self.n.sqrt().ceil() as usize / 2;
            if desired_number_of_children > self.children.len() {
                // println!(
                //     "Expanding a new child. Desired number of children: {}",
                //     desired_number_of_children
                // );
                // We will expand a new child
                let mut game_state_clone = game_state.clone(); // Clone here because we don't want to modify the game state
                let (_is_game_over, probabilistic_event) =
                    game_state_clone.get_possible_moves(move_list, rng);

                assert!(probabilistic_event); // A probabilistic event must have happened, otherwise we wouldn't have any probabilistic children

                // Create a child for each possible move
                let mut children: Vec<Node> = Vec::with_capacity(move_list.len());
                for i in 0..move_list.len() {
                    children.push(Node::new_deterministic(move_list[i]))
                }

                let outcome = ProbabilisticOutcome {
                    factories: *game_state_clone.get_factories(),
                    out_of_bag: game_state_clone.get_out_of_bag(),
                    bag: game_state_clone.get_bag(),
                };
                let mut child = Node::new_probabilistic(outcome);
                // self.has_probabilistic_children = true;
                child.children.append(&mut children);
                self.children.push(child);
            }
            // let result: f32 = playout(&mut game_state.clone(), rng, move_list);
            // let delta = if u8::from(game_state.get_current_player()) == 0 {
            //     1. - result
            // } else {
            //     result
            // };
            // self.backpropagate(delta);
            // return 1. - delta;
        }

        let delta = if self.children.is_empty() {
            self.expand(game_state, move_list, rng);
            if !self.is_game_over {
                let playout_result = playout(&mut game_state.clone(), rng, move_list); // Playout returns 1 if player 1 wins, 0 if player 2 wins, and 0.5 if it's a draw
                if current_player == 0 {
                    1. - playout_result
                } else {
                    playout_result
                }
            } else if self.n == 0. {
                let mut game_result = get_game_result(game_state);
                game_result = if current_player == 0 {
                    1. - game_result
                } else {
                    game_result
                };
                self.q = game_result;
                self.n = 1.;
                game_result
            } else {
                self.q / self.n
            }
        } else {
            let next_child: &mut Node = if !self.has_probabilistic_children {
                self.child_with_max_uct_value(is_root)
            } else {
                self.probabilistic_child_with_max_uct_value(rng)
            };
            match next_child.previous_event {
                Event::Deterministic(move_) => {
                    game_state.do_move(move_);
                }
                Event::Probabilistic(outcome) => {
                    // println!("Probabilistic event encountered. Applying outcome.");
                    // println!("{}", game_state);
                    outcome.apply_to_game_state(game_state);
                }
            }
            next_child.iteration(game_state, move_list, rng, false)
        };

        let delta = if invert_delta { 1. - delta } else { delta };

        self.backpropagate(delta);

        1. - delta
    }

    fn build_pv(&mut self, game_state: &mut GameState, pv: &mut Vec<Event>) {
        if self.children.is_empty() {
            return;
        }
        let child: &mut Node = self.best_child();
        match child.previous_event {
            Event::Deterministic(move_) => {
                game_state.do_move(move_);
            }
            Event::Probabilistic(outcome) => {
                outcome.apply_to_game_state(game_state);
            }
        }
        pv.push(child.previous_event.clone());
        child.build_pv(game_state, pv);
    }

    fn best_child(&mut self) -> &mut Node {
        let mut best_child_index = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_value();
            if value > best_child_value {
                best_child_index = i;
                best_child_value = value;
            }
        }

        &mut self.children[best_child_index]
    }

    pub fn best_move(&mut self) -> Option<Move> {
        if self.children.is_empty() {
            return None;
        }

        let child: &mut Node = self.best_child();
        match child.previous_event {
            Event::Deterministic(move_) => Some(move_),
            Event::Probabilistic(_) => None,
        }
    }
}

const NORMALIZATION_SCORE_LIMIT: f32 = 100.;
const BASE_SCORE_ADJUSTMENT: f32 = 0.001;
const COMPLEMENTARY_SCORE_ADJUSTMENT: f32 = 1. - BASE_SCORE_ADJUSTMENT;

fn get_game_result(game_state: &GameState) -> f32 {
    let scores = game_state.get_scores();
    let score_difference = scores[0] as f32 - scores[1] as f32;
    let normalized_score_difference =
        score_difference.abs() / NORMALIZATION_SCORE_LIMIT * BASE_SCORE_ADJUSTMENT;
    match score_difference {
        x if x > 0. => COMPLEMENTARY_SCORE_ADJUSTMENT + normalized_score_difference, // Basically 1 + normalized_score_difference
        x if x < 0. => BASE_SCORE_ADJUSTMENT - normalized_score_difference, // Basically 0 - normalized_score_difference
        _ => 0.5,
    }
}

pub fn playout(game_state: &mut GameState, rng: &mut SmallRng, move_list: &mut MoveList) -> f32 {
    loop {
        match super::node::get_random_move(game_state, rng, move_list) {
            None => {
                return get_game_result(game_state);
            }
            Some(move_) => game_state.do_move(move_),
        }
    }
}

pub struct MonteCarloTreeSearch {
    root_node: Node,
    root_game_state: GameState,
    time_limit: u64,
}

impl MonteCarloTreeSearch {
    fn do_iterations(&mut self, iterations: usize, rng: &mut SmallRng) {
        let mut move_list = MoveList::new();
        for _ in 0..iterations {
            self.root_node
                .iteration(&mut self.root_game_state.clone(), &mut move_list, rng, true);
        }
    }

    fn set_root(&mut self, game_state: &GameState) {
        // // Check game state for equality. TODO: Implement PartialEq for GameState
        // let is_current_player_equal =
        //     game_state.get_current_player() == self.root_game_state.get_current_player();
        // let is_next_round_starting_player_equal = game_state.get_next_round_starting_player()
        //     == self.root_game_state.get_next_round_starting_player();
        // let is_pattern_line_equal = game_state.get_pattern_lines_colors()
        //     == self.root_game_state.get_pattern_lines_colors();
        // let is_pattern_line_equal = is_pattern_line_equal
        //     && game_state.get_pattern_lines_occupancy()
        //         == self.root_game_state.get_pattern_lines_occupancy();
        // let is_bag_equal = game_state.get_bag() == self.root_game_state.get_bag();
        // let is_factory_equal = game_state.get_factories() == self.root_game_state.get_factories();
        // let is_discard_equal =
        //     game_state.get_floor_line_progress() == self.root_game_state.get_floor_line_progress();
        // let states_equal = is_current_player_equal
        //     && is_next_round_starting_player_equal
        //     && is_pattern_line_equal
        //     && is_bag_equal
        //     && is_factory_equal
        //     && is_discard_equal;

        // if self.root_node.children.is_empty() || !states_equal {
        //     println!("Could not find the given game state in the tree. Falling back to the default root node.");
        // } else {
        //     println!("Found the given game state in the tree. Setting it as the new root node.");
        // }
        self.root_node = Node::new_deterministic(Move::DUMMY);
        self.root_game_state = game_state.clone();
    }

    fn search(&mut self, game_state: &GameState) -> Move {
        println!(
            "Searching move using MCTS. Fen: {}",
            game_state.serialize_string()
        );
        println!("    Left Depth Iterations Value PV");
        let start_time = Instant::now();
        self.set_root(game_state);
        let mut rng = SmallRng::from_entropy();
        let mut pv: Vec<Event> = Vec::with_capacity(100);
        let mut iterations_per_ms = 5.;
        let mut completed_iterations: usize = 0;
        let search_start_time = Instant::now();

        let factories: &[[u8; 5]; 6] = game_state.get_factories();
        let all_empty = factories
            .iter()
            .all(|factory| factory.iter().all(|&tile| tile == 0));
        if all_empty {
            panic!("Monte Carlo Tree search was started in a position where it is not possible to make a move.");
        }

        loop {
            pv.truncate(0);
            self.root_node
                .build_pv(&mut self.root_game_state.clone(), &mut pv);

            let time_left: i64 = self.time_limit as i64 - start_time.elapsed().as_millis() as i64;

            println!(
                "{:6}ms {:5} {:10} {:4.0}% {}",
                time_left,
                pv.len(),
                completed_iterations,
                (1. - self.root_node.get_value()).min(1.0) * 100.,
                pv.iter()
                    .map(|event| event.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            if time_left < 30 {
                break;
            }

            let iterations =
                ((time_left as f32 / 6.).min(5000.) * iterations_per_ms).max(1.) as usize;
            self.do_iterations(iterations, &mut rng);
            completed_iterations += iterations;

            let elapsed_time = search_start_time.elapsed().as_micros() as f32 / 1000.;
            if elapsed_time > 0. {
                iterations_per_ms = completed_iterations as f32 / elapsed_time
            }
        }

        println!(
            "Search finished after {}ms. Value: {:.0}% PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            (1. - self.root_node.get_value()).min(1.0) * 100.,
            pv.len(),
            completed_iterations,
            iterations_per_ms * 1000.,
            pv.iter()
                .map(|event| event.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
        self.root_node.best_move().unwrap()
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            root_node: Node::new_deterministic(Move::DUMMY),
            root_game_state: GameState::new(&mut rng),
            time_limit: 7000,
        }
    }
}

#[async_trait::async_trait]
impl Player for MonteCarloTreeSearch {
    fn name(&self) -> &str {
        "MCTS"
    }

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        self.search(game_state)
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        let mut invalid = false;
        for child in &mut self.root_node.children {
            match child.previous_event.clone() {
                Event::Deterministic(child_move) => {
                    if child_move == move_ {
                        let mut temp_node =
                            Node::new_probabilistic(ProbabilisticOutcome::default()); // Assuming a new method for creating a Node
                        std::mem::swap(child, &mut temp_node);
                        let new_root_node = temp_node;
                        self.root_node = new_root_node;
                        self.root_game_state = new_game_state.clone();
                        return;
                    }
                }
                Event::Probabilistic(_) => {
                    invalid = true;
                    // let mut move_list = MoveList::default();
                    // let mut rng = SmallRng::from_entropy();
                    // // self.root_node = Node::new_deterministic(Move::DUMMY); // Assuming a new method for creating a Node
                    // self.root_game_state.get_possible_moves(&mut move_list, &mut rng);
                    // Get all possible moves to trigger the refilling of the factories
                }
            }
        }
        if invalid {
            self.root_node = Node::new_deterministic(Move::DUMMY); // Assuming a new method for creating a Node
        }
        self.root_game_state = new_game_state.clone();
    }

    async fn set_time(&mut self, time: u64) {
        self.time_limit = time;
    }
}
