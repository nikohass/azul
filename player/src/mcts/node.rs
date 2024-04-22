use super::event::{Event, ProbabilisticOutcome};
use super::value::Value;
use game::*;
use rand::rngs::SmallRng;
use rand::Rng as _;

const C: f32 = 0.1;
const C_BASE: f32 = 30120.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;

pub struct Node {
    children: Vec<Node>,
    previous_event: Event, // The edge from the parent to this node
    n: f32,
    q: Value,
    is_game_over: bool,
    has_probabilistic_children: bool,
}

impl Node {
    pub fn new_deterministic(previous_move: Move) -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Deterministic(previous_move),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    pub fn new_probabilistic(outcome: ProbabilisticOutcome) -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Probabilistic(outcome),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    pub fn get_children(&self) -> &Vec<Node> {
        &self.children
    }

    #[inline]
    pub fn get_value(&self) -> Value {
        if self.n > 0. {
            self.q / self.n
        } else {
            Value::from([std::f32::NEG_INFINITY; NUM_PLAYERS])
        }
    }

    fn get_uct_value(&self, player_index: usize, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            let mean_value = self.q[player_index] / self.n;
            mean_value + c * (parent_n.ln() / self.n).sqrt()
        } else {
            std::f32::INFINITY
        }
    }

    fn child_with_max_uct_value(&mut self, player_index: usize) -> &mut Node {
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();

        let mut best_child_index = 0;
        let mut best_chuld_uct_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_uct_value(player_index, self.n, c_adjusted);
            if value > best_chuld_uct_value {
                best_child_index = i;
                best_chuld_uct_value = value;
            }
        }

        &mut self.children[best_child_index]
    }

    fn select_child(&mut self, player_index: usize, rng: &mut SmallRng) -> &mut Node {
        if self.has_probabilistic_children {
            let index = rng.gen_range(0..self.children.len());
            &mut self.children[index]
        } else {
            // Use the existing UCT method for deterministic children
            self.child_with_max_uct_value(player_index)
        }
    }

    fn backpropagate(&mut self, value: Value) {
        self.n += 1.;
        self.q += value;
    }

    fn expand(&mut self, game_state: &mut GameState, move_list: &mut MoveList, rng: &mut SmallRng) {
        let result = game_state.get_possible_moves(move_list, rng);
        let is_game_over = matches!(result, MoveGenerationResult::GameOver);
        let probabilistic_event = matches!(result, MoveGenerationResult::RoundOver);

        self.is_game_over = is_game_over;
        if is_game_over {
            // If the game is over, we don't need to expand any children
            return;
        }

        // Create children nodes for each possible move
        let mut children = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            children.push(Node::new_deterministic(move_list[i]))
        }

        if probabilistic_event {
            // Create a probabilistic child for the probabilistic event that just happend during the move generation
            // Since it is not possible to expand all possible outcomes of a probabilistic event, we will only expand one of them
            // and dynamically expand the other outcomes later
            self.expand_probabilistic_child(game_state, children);
        } else {
            // Expand the current node with the children we just created
            self.children = children;
        }
    }

    fn expand_probabilistic_child(&mut self, game_state: &mut GameState, children: Vec<Node>) {
        let outcome = ProbabilisticOutcome {
            factories: game_state.get_factories().clone(),
            out_of_bag: game_state.get_out_of_bag(),
            bag: game_state.get_bag(),
        };
        let mut child = Node::new_probabilistic(outcome);
        child.children = children;
        self.children.push(child);
        self.has_probabilistic_children = true;
    }

    pub fn iteration(
        &mut self,
        game_state: &mut GameState,
        move_list: &mut MoveList,
        rng: &mut SmallRng,
    ) -> Value {
        let current_player = u8::from(game_state.get_current_player());
        if self.has_probabilistic_children {
            // All children of this node are probabilistic. When this node was "expanded", we only expanded one probabilistic outcome.
            // There would be too many possible outcomes to expand all of them, so we just expanded one.
            // Now we need to adjust for this and dynamically expand the other outcomes.
            // Here we also need to balance exploration and exploitation.
            // If we only visit the only child and never expand further, our strategy will be quite bad because we basically assume that the probabilistic event will always happen.
            // If we expand a new child every time we iterate this node, we would never visit the same child twice. This would cause our estimations of the value of the child to be very inaccurate.

            // Let's just try this:
            let desired_number_of_children = self.n.sqrt() as usize / 2; //self.n.sqrt().ceil() as usize / 2;
            if desired_number_of_children > self.children.len() {
                // We will expand a new child
                let mut game_state_clone = game_state.clone(); // Clone here because we don't want to modify the game state
                game_state_clone.fill_factories(rng);

                let outcome = ProbabilisticOutcome {
                    factories: game_state_clone.get_factories().clone(),
                    out_of_bag: game_state_clone.get_out_of_bag(),
                    bag: game_state_clone.get_bag(),
                };
                let child = Node::new_probabilistic(outcome);
                self.children.push(child);
            }
        }

        let delta: Value = if self.children.is_empty() {
            if rng.gen_bool(0.05) {
                self.expand(game_state, move_list, rng);
                if !self.is_game_over {
                    super::heuristic_move_generation::playout(&mut game_state.clone(), rng)
                } else if self.n == 0. {
                    let game_result = Value::from_game_scores(game_state.get_scores());
                    self.q = Value::from_game_scores(game_state.get_scores());
                    self.n = 1.;
                    game_result
                } else {
                    self.q / self.n
                }
            } else {
                super::heuristic_move_generation::playout(&mut game_state.clone(), rng)
            }
        } else {
            let next_child = self.select_child(current_player as usize, rng);
            next_child.previous_event.apply_to_game_state(game_state);
            next_child.iteration(game_state, move_list, rng)
        };

        self.backpropagate(delta);

        delta
    }

    pub fn build_pv(&mut self, game_state: &mut GameState, pv: &mut Vec<Event>) {
        if self.children.is_empty() {
            return;
        }

        let player_index = usize::from(game_state.get_current_player());
        let child = self.best_child(player_index);

        child.previous_event.apply_to_game_state(game_state);
        pv.push(child.previous_event.clone());

        child.build_pv(game_state, pv);
    }

    pub fn best_child(&mut self, player_index: usize) -> &mut Node {
        let mut best_child_index = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value: Value = child.get_value();
            if value[player_index] > best_child_value {
                best_child_index = i;
                best_child_value = value[player_index];
            }
        }

        &mut self.children[best_child_index]
    }

    pub fn best_move(&mut self, player_index: usize) -> Option<Move> {
        if self.children.is_empty() {
            return None;
        }

        let child = self.best_child(player_index);
        match child.previous_event {
            Event::Deterministic(move_) => Some(move_),
            Event::Probabilistic(_) => None,
        }
    }

    pub fn count_nodes(&self) -> ChildCount {
        let mut total_child_count = match self.previous_event {
            Event::Probabilistic(_) => ChildCount {
                deterministic: 0,
                probabilistic: 1,
            },
            Event::Deterministic(_) => ChildCount {
                deterministic: 1,
                probabilistic: 0,
            },
        };

        for child in self.children.iter() {
            total_child_count += child.count_nodes();
        }

        total_child_count
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChildCount {
    pub deterministic: usize,
    pub probabilistic: usize,
}

impl std::ops::AddAssign for ChildCount {
    fn add_assign(&mut self, other: Self) {
        self.deterministic += other.deterministic;
        self.probabilistic += other.probabilistic;
    }
}
