use std::time::Instant;

use game::{GameState, Move, MoveList, PlayerTrait};
use rand::{rngs::SmallRng, Rng, SeedableRng};

const C: f32 = 0.0;
const C_BASE: f32 = 120.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;
// const B_SQUARED: f32 = 0.8;
const FPU_R: f32 = 0.3;

pub struct Node {
    pub children: Vec<Node>,
    pub move_to_reach: Option<Move>,
    pub n: f32,
    pub q: f32,
    pub is_game_over: bool,
    pub refill_factories: bool,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            move_to_reach: None,
            n: 0.0,
            q: 0.0,
            is_game_over: false,
            refill_factories: false,
        }
    }
}

impl Node {
    #[inline]
    pub fn get_value(&self) -> f32 {
        if self.n > 0. {
            self.q / self.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    fn get_uct_value(&self, parent_n: f32, c: f32, fpu_base: f32, is_root: bool) -> f32 {
        if is_root {
            return if self.n > 0. {
                self.q / self.n + c * (parent_n.ln() / self.n).sqrt()
            } else {
                std::f32::INFINITY
            };
        }
        if self.n > 0. {
            self.q / self.n + c * (parent_n.ln() / self.n).sqrt()
        } else {
            fpu_base + c * parent_n.ln().sqrt()
        }
    }

    fn child_with_max_uct_value(&mut self, is_root: bool) -> &mut Node {
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();
        let fpu_base = (self.n - self.q) / self.n - FPU_R;
        let mut best_child = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_uct_value(self.n, c_adjusted, fpu_base, is_root);
            if value > best_value {
                best_value = value;
                best_child = i;
            }
        }
        &mut self.children[best_child]
    }

    #[inline]
    fn backpropagate(&mut self, q: f32) {
        self.n += 1.;
        self.q += q;
    }

    fn expand(&mut self, game_state: &mut GameState, move_list: &mut MoveList) -> bool {
        let (is_game_over, refill_factories) = game_state.get_possible_moves(move_list);
        // Cancel if game is over and for now also if we need to refill the factories
        self.is_game_over = is_game_over;
        self.refill_factories = refill_factories;
        if is_game_over || refill_factories {
            self.children = Vec::new();
            return true;
        }

        // Expand the node
        self.children = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            self.children.push(Node {
                children: Vec::new(),
                move_to_reach: Some(move_list[i]),
                n: 0.,
                q: 0.,
                is_game_over: false,
                refill_factories: false, //refill_factories,
            })
        }
        false
    }

    pub fn iteration(
        &mut self,
        move_list: &mut MoveList,
        game_state: &mut GameState,
        rng: &mut SmallRng,
        is_root: bool,
    ) -> f32 {
        let delta: f32;

        let mut is_game_over = self.is_game_over;
        let do_playout = self.refill_factories; // For now just do the playout

        if do_playout {
            let result = playout(&mut game_state.clone(), rng, move_list);
            // Invert the score based on the player
            let delta = if u8::from(game_state.get_current_player()) == 0 {
                1. - result
            } else {
                result
            };
            self.backpropagate(delta);
            return 1. - delta;
        }

        if self.children.is_empty() {
            is_game_over |= if self.n == 1. {
                self.expand(game_state, move_list)
            } else {
                self.is_game_over
            };
            if !is_game_over {
                let result = playout(&mut game_state.clone(), rng, move_list);
                // Invert the score based on the player
                delta = if u8::from(game_state.get_current_player()) == 0 {
                    1. - result
                } else {
                    result
                };
            } else if self.n == 0. {
                let side = if u8::from(game_state.get_current_player()) == 0 {
                    -1
                } else {
                    1
                };
                let result = game_result(game_state) * side;
                self.q = result_to_value(result);
                self.n = 1.;
                delta = self.q;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta);
            1. - delta
        } else {
            let next_child: &mut Node = self.child_with_max_uct_value(is_root);
            game_state.do_move(next_child.move_to_reach.unwrap());
            delta = next_child.iteration(move_list, game_state, rng, false);
            self.backpropagate(delta);
            1. - delta
        }
    }

    pub fn pv(&mut self, game_state: &mut GameState, move_list: &mut MoveList) {
        if self.children.is_empty() {
            return;
        }
        let child = self.best_child();
        let move_to_reach = child.move_to_reach.unwrap();
        move_list.push(move_to_reach);
        game_state.do_move(move_to_reach);
        child.pv(game_state, move_list);
    }

    pub fn best_child(&mut self) -> &mut Node {
        let mut best_child: usize = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (i, child) in self.children.iter().enumerate() {
            let child_value = child.get_value();
            if child_value > best_value {
                best_value = child_value;
                best_child = i;
            }
        }
        &mut self.children[best_child]
    }

    pub fn best_move(&mut self) -> Option<Move> {
        self.best_child().move_to_reach
    }
}

fn playout(game_state: &mut GameState, rng: &mut SmallRng, move_list: &mut MoveList) -> f32 {
    loop {
        if game_state.get_possible_moves(move_list).0 {
            let result = game_result(game_state);
            return result_to_value(result);
        }
        let move_ = move_list[rng.gen_range(0..move_list.len())];
        game_state.do_move(move_);
    }
}

fn game_result(game_state: &GameState) -> i16 {
    let scores = game_state.get_scores();
    scores[0] - scores[1]
}

pub fn result_to_value(result: i16) -> f32 {
    let abs = result.abs() as f32 / 100_000.; // Encourages the player to win with a large score difference
    match result {
        r if r > 0 => 0.999 + abs,
        r if r < 0 => 0.001 - abs,
        _ => 0.5,
    }
}

/*
    Completing a row on my board.
    Preventing my opponent from completing a high-scoring row.
    The likelihood that certain tiles will be available in later turns.
*/

pub struct MonteCarloTreeSearch {
    root_node: Node,
    root_state: GameState,
    time_limit: Option<i64>,
    iteration_limit: Option<usize>,
}

impl MonteCarloTreeSearch {
    pub fn get_value(&self) -> f32 {
        1. - self.root_node.get_value()
    }

    pub fn get_root_node(&mut self) -> &mut Node {
        &mut self.root_node
    }

    fn set_root(&mut self, game_state: &GameState) {
        // // let mut move_list = MoveList::default();
        // // let mut game_state = game_state.clone();
        // // game_state.get_possible_moves(&mut move_list);
        // for child in &mut self.root_node.children {
        //     let mut cloned_game_state = game_state.clone();
        //     cloned_game_state.do_move(child.move_to_reach.unwrap());
        //     cloned_game_state.get_possible_moves(&mut MoveList::default());
        //     // println!("Comparing {} with {}", cloned_game_state.serialize_string(), game_state.serialize_string());

        let is_current_player_equal =
            game_state.get_current_player() == self.root_state.get_current_player();
        let is_next_round_starting_player_equal = game_state.get_next_round_starting_player()
            == self.root_state.get_next_round_starting_player();
        let is_pattern_line_equal =
            game_state.get_pattern_lines_colors() == self.root_state.get_pattern_lines_colors();
        let is_pattern_line_equal = is_pattern_line_equal
            && game_state.get_pattern_lines_occupancy()
                == self.root_state.get_pattern_lines_occupancy();
        let is_bag_equal = game_state.get_bag() == self.root_state.get_bag();
        let states_equal = is_current_player_equal
            && is_next_round_starting_player_equal
            && is_pattern_line_equal
            && is_bag_equal;
        //     println!("{} {} {} {}", is_current_player_equal, is_next_round_starting_player_equal, is_pattern_line_equal, is_bag_equal);

        //     if is_current_player_equal && is_next_round_starting_player_equal && is_pattern_line_equal && is_bag_equal {
        //         // Take the node and replace it with a default node
        //         let new_root_node = std::mem::take(child);
        //         self.root_node = new_root_node;
        //         self.root_state = cloned_game_state;
        //         return;
        //     }
        // }
        // println!("Could not find the given game state in the tree. Falling back to the default root node.");
        // self.root_node = Node::default();

        // if !states_equal {
        //     println!("WARNING: The given game state is not equal to the root node's game state.");
        //     println!("ROOT:\n{}", self.root_state);
        //     println!("GAME:\n{}", game_state);
        // }

        if self.root_node.children.is_empty() || !states_equal {
            self.root_state = game_state.clone();
            self.root_node = Node::default();
            println!("Could not find the given game state in the tree. Falling back to the default root node.");
        } else {
            println!("Found the given game state in the tree. Setting it as root.");
        }
    }

    fn do_iterations(&mut self, n: usize, rng: &mut SmallRng) {
        let mut move_list = MoveList::default();
        for _ in 0..n {
            self.root_node
                .iteration(&mut move_list, &mut self.root_state.clone(), rng, true);
        }
    }
    pub fn search_action(&mut self, game_state: &GameState) -> Move {
        println!(
            "Searching action using MCTS. Fen: {}",
            game_state.serialize_string()
        );
        println!("    Left Depth Iterations Value PV");
        let start_time = Instant::now();
        self.set_root(game_state);
        let mut rng = SmallRng::from_entropy();
        let mut pv = MoveList::default();
        let mut iterations_per_ms = 5.;
        let mut iterations: usize = 0;

        let search_start_time = Instant::now();
        loop {
            pv.clear();
            self.root_node.pv(&mut self.root_state.clone(), &mut pv);

            let (next_iterations, stop) = if let Some(time_limit) = self.time_limit {
                let time_left = time_limit - start_time.elapsed().as_millis() as i64;
                println!(
                    "{:6}ms {:5} {:10} {:4.0}% {}",
                    time_left,
                    pv.len(),
                    iterations,
                    (1. - self.root_node.get_value()).min(1.0) * 100.,
                    pv
                );
                let next_iterations =
                    ((time_left as f64 / 6.).min(5000.) * iterations_per_ms).max(1.) as usize;
                (next_iterations, time_left < 30)
            } else if let Some(iteration_limit) = self.iteration_limit {
                if iterations >= iteration_limit {
                    (0, true)
                } else {
                    let iterations_left = iteration_limit - iterations;
                    println!(
                        "{:6}it {:5} {:10} {:4.0}% {}",
                        iterations_left,
                        pv.len(),
                        iterations,
                        (1. - self.root_node.get_value()).min(1.0) * 100.,
                        pv
                    );
                    let next_iterations = iterations_left / 2;
                    (next_iterations, next_iterations < 100)
                }
            } else {
                panic!("Mcts has neither a time limit nor a node limit");
            };
            if stop {
                break;
            }
            self.do_iterations(next_iterations, &mut rng);
            iterations += next_iterations;
            let elapsed = search_start_time.elapsed().as_micros() as f64;
            if elapsed > 0. {
                iterations_per_ms = iterations as f64 / elapsed * 1000.;
            }
        }

        println!(
            "Search finished after {}ms. Value: {:.0}% PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            (1. - self.root_node.get_value()).min(1.0) * 100.,
            pv.len(),
            iterations,
            iterations_per_ms * 1000.,
            pv,
        );
        self.root_node.best_move().unwrap()
    }
}

#[async_trait::async_trait]
impl PlayerTrait for MonteCarloTreeSearch {
    fn name(&self) -> &str {
        "MCTS"
    }

    async fn get_move(&mut self, game_state: GameState) -> Move {
        self.search_action(&game_state)
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        for child in &mut self.root_node.children {
            if child.move_to_reach.unwrap() == move_ {
                let new_root_node = std::mem::take(child);
                self.root_node = new_root_node;
                self.root_state = new_game_state.clone();
                println!("Found move in tree. Setting it as root.");
                return;
            }
        }
        println!("Could not find move in tree. Falling back to the default root node.");
        self.root_node = Node::default();
        self.root_state = new_game_state.clone();
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        Self {
            root_node: Node::default(),
            root_state: GameState::default(),
            time_limit: Some(20_000),
            iteration_limit: None,
        }
    }
}
