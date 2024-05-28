use super::edge::{Edge, ProbabilisticOutcome};
use super::value::Value;
use game::*;
use rand::rngs::SmallRng;
use rand::Rng as _;

const MIN_VISITS_BEFORE_EXPANSION: f32 = 20.;

mod constants {
    pub const C: f32 = 0.2;
    pub const C_BASE: f32 = 30_000.0;
    pub const C_FACTOR: f32 = std::f32::consts::SQRT_2;
}

use constants::*;

pub struct Node {
    children: Vec<Node>,
    edge: Edge, // The edge from the parent to this node
    n: f32,
    q: Value,
    is_game_over: bool,
    has_probabilistic_children: bool,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.edge)
    }
}

impl Node {
    pub fn new_deterministic(previous_move: Move) -> Self {
        Node {
            children: Vec::new(),
            edge: Edge::Deterministic(previous_move),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    pub fn new_probabilistic(outcome: ProbabilisticOutcome) -> Self {
        Node {
            children: Vec::new(),
            edge: Edge::Probabilistic(outcome),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    // pub fn store_node(
    //     &self,
    //     parent_id: usize,
    //     current_id: &mut usize,
    //     data: &mut String,
    //     min_visits: f32,
    // ) {
    //     let local_id: usize = *current_id;
    //     if self.n < min_visits {
    //         return;
    //     }

    //     // Write node definition
    //     data.push_str(&format!("{} [label=\"{}\"];\n", local_id, self.n,));

    //     // Write edge definition
    //     if parent_id != local_id {
    //         // avoid linking root to itself
    //         data.push_str(&format!("{} -> {};\n", parent_id, local_id));
    //     }

    //     for child in &self.children {
    //         *current_id += 1;
    //         child.store_node(local_id, current_id, data, min_visits);
    //     }
    // }

    pub fn previous_move(&self) -> Option<Move> {
        match self.edge {
            Edge::Deterministic(move_) => Some(move_),
            Edge::Probabilistic(_) => None,
        }
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    // pub fn edge(&self) -> &Edge {
    //     &self.edge
    // }

    // pub fn take_child_with_move(self, move_: Move) -> Option<Node> {
    //     let mut children = self.children;
    //     let mut index = None;
    //     for (i, child) in children.iter().enumerate() {
    //         if let Some(child_move) = child.previous_move() {
    //             if child_move == move_ {
    //                 index = Some(i);
    //                 break;
    //             }
    //         }
    //     }

    //     index.map(|index| children.remove(index))
    // }

    pub fn take_child_with_edge(self, edge: &Edge) -> Option<Node> {
        let mut children = self.children;
        let mut index = None;
        for (i, child) in children.iter().enumerate() {
            if &child.edge == edge {
                index = Some(i);
                break;
            }
        }

        index.map(|index| children.remove(index))
    }

    #[inline]
    pub fn value(&self) -> Value {
        if self.n > 0. {
            self.q / self.n
        } else {
            Value::from([std::f32::NEG_INFINITY; NUM_PLAYERS])
        }
    }

    fn uct_value(&self, player_index: usize, parent_n: f32, c: f32) -> f32 {
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
            let value = child.uct_value(player_index, self.n, c_adjusted);
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

        // If there are multiple identical factories, the moves originating from them are identical.
        // Remove them to avoid evaluating what is essentially the same move multiple times.
        // In case we need values for every move, we can add the removed moves back later with the values of their identical counterparts.
        // filter_identical_moves(game_state, move_list);

        // Create children for each possible move
        let mut children = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            children.push(Node::new_deterministic(move_list[i]))
        }

        if probabilistic_event {
            // Create a probabilistic child for the probabilistic event that just happend during the move generation
            // Since it is not possible to expand all outcomes of a probabilistic event, we will only expand one of them
            // and dynamically expand the other outcomes later
            self.expand_probabilistic_child(game_state, children);
        } else {
            // Expand the current node with the children we just created
            self.children = children;
        }
    }

    fn expand_probabilistic_child(&mut self, game_state: &mut GameState, children: Vec<Node>) {
        let outcome = ProbabilisticOutcome {
            factories: game_state.factories.clone(),
            out_of_bag: game_state.out_of_bag,
            bag: game_state.bag,
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
    ) -> (Value, u16) {
        #[cfg(debug_assertions)]
        game_state.check_integrity().unwrap();

        let current_player = u8::from(game_state.current_player);
        if self.has_probabilistic_children {
            // All children of this node are probabilistic. When this node was "expanded", we only expanded one probabilistic outcome.
            // There would be too many possible outcomes to expand all of them, so we just expanded one.
            // Now we need to adjust for this and dynamically expand the other outcomes.
            // Here we also need to balance exploration and exploitation.
            // If we only visit the only child and never expand further, our strategy will be quite bad because we basically assume that the probabilistic event will always happen.
            // If we expand a new child every time we iterate this node, we would never visit the same child twice. This would cause our estimations of the value of the child to be very inaccurate.

            // Let's just try this:
            let desired_number_of_children = self.n.sqrt().ceil() as usize / 8;
            if desired_number_of_children > self.children.len() {
                // We will expand a new child
                let mut game_state_clone = game_state.clone(); // Clone here because we don't want to modify the game state
                game_state_clone.evaluate_round();
                game_state_clone.fill_factories(rng);

                let outcome = ProbabilisticOutcome {
                    factories: game_state_clone.factories.clone(),
                    out_of_bag: game_state_clone.out_of_bag,
                    bag: game_state_clone.bag,
                };
                let child = Node::new_probabilistic(outcome);
                self.children.push(child);
            }
        }

        let (delta, depth) = if self.children.is_empty() {
            if self.n > MIN_VISITS_BEFORE_EXPANSION {
                self.expand(game_state, move_list, rng);
                if !self.is_game_over {
                    super::playout::playout(game_state.clone(), rng)
                } else if self.n == 0. {
                    self.q = Value::from_game_scores(game_state.scores);
                    self.n = 1.;
                    (self.q, 0)
                } else {
                    (self.q / self.n, 0)
                }
            } else {
                super::playout::playout(game_state.clone(), rng)
            }
        } else {
            let next_child = self.select_child(current_player as usize, rng);
            next_child.edge.apply_to_game_state(game_state);
            next_child.iteration(game_state, move_list, rng)
        };

        self.backpropagate(delta);

        (delta, depth + 1)
    }

    pub fn build_principal_variation(&mut self, game_state: &mut GameState, pv: &mut Vec<Edge>) {
        if self.children.is_empty() {
            return;
        }

        let player_index = usize::from(game_state.current_player);
        let child = self.best_child(player_index);

        child.edge.apply_to_game_state(game_state);
        pv.push(child.edge.clone());

        child.build_principal_variation(game_state, pv);
    }

    pub fn best_child(&mut self, player_index: usize) -> &mut Node {
        let mut best_child_index = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value: Value = child.value();
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
        match child.edge {
            Edge::Deterministic(move_) => Some(move_),
            Edge::Probabilistic(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn count_nodes(&self) -> ChildCount {
        let mut total_child_count = match self.edge {
            Edge::Probabilistic(_) => ChildCount {
                deterministic: 0,
                probabilistic: 1,
            },
            Edge::Deterministic(_) => ChildCount {
                deterministic: 1,
                probabilistic: 0,
            },
        };

        for child in self.children.iter() {
            total_child_count += child.count_nodes();
        }

        total_child_count
    }

    pub fn top_two_ratio(&self, player_index: usize) -> f32 {
        let mut best_value = std::f32::NEG_INFINITY;
        let mut second_best_value = std::f32::NEG_INFINITY;

        for child in self.children.iter() {
            let value = child.value()[player_index];
            if value > best_value {
                second_best_value = best_value;
                best_value = value;
            } else if value > second_best_value {
                second_best_value = value;
            }
        }

        if best_value == std::f32::NEG_INFINITY {
            0.
        } else if second_best_value == std::f32::NEG_INFINITY {
            1.
        } else {
            best_value / second_best_value
        }
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

// pub fn filter_identical_moves(game_state: &GameState, move_list: &mut MoveList) {
//     let mut unique = [false; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
//     let mut duplicates = [0; NUM_NON_CENTER_FACTORIES];
//     let mut num_duplicates = 0;
//     for (i, factory) in game_state
//         .factories
//         .iter()
//         .take(NUM_NON_CENTER_FACTORIES)
//         .enumerate()
//     {
//         let hash = hash_factory(factory);
//         if unique[hash] {
//             duplicates[num_duplicates] = i;
//             num_duplicates += 1;
//         } else {
//             unique[hash] = true;
//         }
//     }

//     if num_duplicates == 0 {
//         return;
//     }

//     for factory_index in duplicates.iter().take(num_duplicates) {
//         let factory_index = *factory_index as u8;
//         let mut index = 0;
//         while index < move_list.len() {
//             let move_ = move_list[index];
//             if move_.factory_index == factory_index {
//                 move_list.remove(index);
//             } else {
//                 index += 1;
//             }
//         }
//     }
// }
