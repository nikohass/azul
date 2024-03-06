use super::event::{Event, ProbabilisticOutcome};
use super::value::Value;
use game::*;
use rand::rngs::SmallRng;
use std::sync::{Arc, Mutex};

const C: f32 = 0.2;
const C_BASE: f32 = 20120.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;

pub struct Node {
    children: Vec<Arc<Mutex<Node>>>,
    previous_event: Event, // The edge from the parent to this node
    n: f32,
    q: Value,
    is_game_over: bool,
    has_probabilistic_children: bool,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Deterministic(Move::DUMMY),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }
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

    pub fn get_children(&self) -> &Vec<Arc<Mutex<Node>>> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Arc<Mutex<Node>>> {
        &mut self.children
    }

    pub fn get_previous_event(&self) -> &Event {
        &self.previous_event
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

    fn child_with_max_uct_value(&mut self, player_index: usize) -> Arc<Mutex<Node>> {
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();

        let mut best_child_index = 0;
        let mut best_chuld_uct_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value = child
                .lock()
                .unwrap()
                .get_uct_value(player_index, self.n, c_adjusted);
            if value > best_chuld_uct_value {
                best_child_index = i;
                best_chuld_uct_value = value;
            }
        }

        self.children[best_child_index].clone()
    }

    fn backpropagate(&mut self, value: Value) {
        self.n += 1.;
        self.q += value;
    }

    fn expand(&mut self, game_state: &mut GameState, move_list: &mut MoveList, rng: &mut SmallRng) {
        let (is_game_over, probabilistic_event) = game_state.get_possible_moves(move_list, rng);
        self.is_game_over = is_game_over;

        if is_game_over {
            // If the game is over, we don't need to expand any children
            return;
        }

        // Create children nodes for each possible move
        let mut children = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            children.push(Arc::new(Mutex::new(Node::new_deterministic(move_list[i]))))
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

    fn expand_probabilistic_child(
        &mut self,
        game_state: &mut GameState,
        children: Vec<Arc<Mutex<Node>>>,
    ) {
        let outcome = ProbabilisticOutcome {
            factories: game_state.get_factories().clone(),
            out_of_bag: game_state.get_out_of_bag(),
            bag: game_state.get_bag(),
        };
        let mut child = Node::new_probabilistic(outcome);
        child.children = children;
        self.children.push(Arc::new(Mutex::new(child)));
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
            let desired_number_of_children = self.n.sqrt().ceil() as usize / 2;
            if desired_number_of_children > self.children.len() {
                // We will expand a new child
                let mut game_state_clone = game_state.clone(); // Clone here because we don't want to modify the game state
                let (_is_game_over, probabilistic_event) =
                    game_state_clone.get_possible_moves(move_list, rng);

                assert!(probabilistic_event); // A probabilistic event must have happened, otherwise we wouldn't have any probabilistic children

                // Create a child for each possible move
                let mut children: Vec<Arc<Mutex<Node>>> = Vec::with_capacity(move_list.len());
                for i in 0..move_list.len() {
                    children.push(Arc::new(Mutex::new(Node::new_deterministic(move_list[i]))))
                }

                let outcome = ProbabilisticOutcome {
                    factories: game_state_clone.get_factories().clone(),
                    out_of_bag: game_state_clone.get_out_of_bag(),
                    bag: game_state_clone.get_bag(),
                };
                let mut child = Node::new_probabilistic(outcome);
                child.children.append(&mut children);
                self.children.push(Arc::new(Mutex::new(child)));
            }
        }

        let delta: Value = if self.children.is_empty() {
            self.expand(game_state, move_list, rng);
            if !self.is_game_over {
                super::heuristic_move_generation::playout(&mut game_state.clone(), rng, move_list)
            } else if self.n == 0. {
                let game_result = Value::from_game_scores(game_state.get_scores());
                self.q = Value::from_game_scores(game_state.get_scores());
                self.n = 1.;
                game_result
            } else {
                self.q / self.n
            }
        } else {
            let next_child = self.child_with_max_uct_value(current_player as usize);
            let mut next_child = next_child.lock().unwrap();
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
        let mut child = child.lock().unwrap();

        child.previous_event.apply_to_game_state(game_state);
        pv.push(child.previous_event.clone());

        child.build_pv(game_state, pv);
    }

    pub fn best_child(&mut self, player_index: usize) -> Arc<Mutex<Node>> {
        let mut best_child_index = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value: Value = child.lock().unwrap().get_value();
            if value[player_index] > best_child_value {
                best_child_index = i;
                best_child_value = value[player_index];
            }
        }

        self.children[best_child_index].clone()
    }

    pub fn best_move(&mut self, player_index: usize) -> Option<Move> {
        if self.children.is_empty() {
            return None;
        }

        let child = self.best_child(player_index);
        let child = child.lock().unwrap();
        match child.previous_event {
            Event::Deterministic(move_) => Some(move_),
            Event::Probabilistic(_) => None,
        }
    }
}
