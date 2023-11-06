use std::time::Instant;

use game::{GameState, Move, MoveList, PlayerTrait};
use rand::{rngs::SmallRng, Rng, SeedableRng};

const C: f32 = 0.0;
const C_BASE: f32 = 220.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;
// const B_SQUARED: f32 = 0.8;
const FPU_R: f32 = 0.1;

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
        if is_game_over {
            self.children = Vec::new();
            self.is_game_over = true;
            return true;
        }
        self.children = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            self.children.push(Node {
                children: Vec::new(),
                move_to_reach: Some(move_list[i]),
                n: 0.,
                q: 0.,
                is_game_over: false,
                refill_factories: refill_factories,
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
        let delta;
        // println!("Iteration");
        // println!("{}", game_state);
        game_state.check_integrity();
        if self.children.is_empty() {
            #[allow(clippy::float_cmp)]
            let is_game_over = if self.n == 1. {
                self.expand(game_state, move_list)
            } else {
                self.is_game_over
            };
            if !is_game_over {
                let result = playout(&mut game_state.clone(), rng, move_list);
                // TODO: this only works for 2 players
                delta = if u8::from(game_state.get_current_player()) == 0 {
                    1. - result
                } else {
                    result
                };
            } else if self.n == 0. {
                let side = if u8::from(game_state.get_current_player()) == 0 {
                    1
                } else {
                    -1
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
            let next_child = self.child_with_max_uct_value(is_root);
            // let is_game_over_check = game_state.get_possible_moves(move_list);
            // assert!(!is_game_over_check, "Game is over in iteration");
            // let next_move = next_child.move_to_reach.unwrap();
            // Make sure the move is actually legal
            // assert!(
            //     move_list.contains(next_move),
            //     "Illegam move in state\n{}\nMove: {}",
            //     game_state,
            //     next_move
            // );
            // println!(
            //     "Do move {}\n{}",
            //     next_child.move_to_reach.unwrap(),
            //     game_state
            // );
            if next_child.refill_factories {
                game_state.evaluate_round();
                game_state.fill_factories();
            }
            // OMG i am so stupid its the rng
            game_state.do_move(next_child.move_to_reach.unwrap());
            // println!("Move: {}", next_child.move_to_reach.unwrap());
            game_state.check_integrity();
            if next_child.refill_factories {}
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
    // println!("Integrity check before playout");
    game_state.check_integrity();
    loop {
        if game_state.get_possible_moves(move_list).0 {
            let result = game_result(game_state);
            return result_to_value(result);
        }
        let move_ = move_list[rng.gen_range(0..move_list.len())];
        // println!("Do move playout: {}", move_);
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

    fn set_root(&mut self, state: &GameState) {
        // TODO: Reuse the root node instead of creating a new one
        self.root_node = Node::default();
        self.root_state = state.clone();
    }

    fn do_iterations(&mut self, n: usize, rng: &mut SmallRng) {
        let mut move_list = MoveList::default();
        for _ in 0..n {
            self.root_node
                .iteration(&mut move_list, &mut self.root_state.clone(), rng, true);
        }
    }
    pub fn search_action(&mut self, state: &GameState) -> Move {
        println!(
            "Searching action using MCTS. Fen: {}",
            state.serialize_string()
        );
        println!("    Left Depth Iterations Value PV");
        let start_time = Instant::now();
        self.set_root(state);
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
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        Self {
            root_node: Node::default(),
            root_state: GameState::default(),
            time_limit: Some(1500),
            iteration_limit: None,
        }
    }
}
