use crate::mcts::event::Event;

use super::node::Node;
use game::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

pub struct MonteCarloTreeSearch {
    root_node: SharedState<Node>,
    root_game_state: SharedState<GameState>,
    time_limit: u64,
}

fn do_iterations(
    root_node: &mut Node,
    root_game_state: &GameState,
    iterations: usize,
    rng: &mut SmallRng,
) {
    let mut move_list = MoveList::new();
    for _ in 0..iterations {
        root_node.iteration(&mut root_game_state.clone(), &mut move_list, rng);
    }
}

impl MonteCarloTreeSearch {
    async fn set_root(&mut self, game_state: &GameState) {
        let mut root_game_state = self.root_game_state.lock().await;
        let mut root_node = self.root_node.lock().await;
        // Check game state for equality. TODO: Implement PartialEq for GameState
        let is_current_player_equal =
            game_state.get_current_player() == root_game_state.get_current_player();
        let is_next_round_starting_player_equal = game_state.get_next_round_starting_player()
            == root_game_state.get_next_round_starting_player();
        let is_pattern_line_equal =
            game_state.get_pattern_lines_colors() == root_game_state.get_pattern_lines_colors();
        let is_pattern_line_equal = is_pattern_line_equal
            && game_state.get_pattern_lines_occupancy()
                == root_game_state.get_pattern_lines_occupancy();
        let is_bag_equal = game_state.get_bag() == root_game_state.get_bag();
        let is_factory_equal = game_state.get_factories() == root_game_state.get_factories();
        let is_discard_equal =
            game_state.get_floor_line_progress() == root_game_state.get_floor_line_progress();
        let states_equal = is_current_player_equal
            && is_next_round_starting_player_equal
            && is_pattern_line_equal
            && is_bag_equal
            && is_factory_equal
            && is_discard_equal;

        if root_node.get_children().is_empty() || !states_equal {
            println!("Could not find the given game state in the tree. Falling back to the default root node.");
            //root_node = Node::new_deterministic(Move::DUMMY);
            *root_node = Node::new_deterministic(Move::DUMMY);
        } else {
            println!("Found the given game state in the tree. Setting it as the new root node.");
        }
        // self.root_node = Node::new_deterministic(Move::DUMMY);
        *root_game_state = game_state.clone();
    }

    async fn search(&mut self, game_state: &GameState) -> Move {
        println!(
            "Searching move using MCTS. Fen: {}",
            game_state.serialize_string()
        );
        let start_time = Instant::now();
        self.set_root(game_state).await;
        let mut rng = SmallRng::from_entropy();
        let mut pv: Vec<Event> = Vec::with_capacity(100);
        let mut iterations_per_ms = 1.; // Initial guess on the lower end for four players, will be adjusted later
        let mut completed_iterations: usize = 0;
        let search_start_time = Instant::now();

        let factories: &[[u8; 5]; NUM_FACTORIES] = game_state.get_factories();
        let all_empty = factories
            .iter()
            .all(|factory| factory.iter().all(|&tile| tile == 0));
        if all_empty {
            panic!("Monte Carlo Tree search was started in a position where it is not possible to make a move.");
        }

        let mut root_node = self.root_node.lock().await;
        let root_game_state = self.root_game_state.lock().await;
        println!("    Left Depth Iterations          Value PV");

        loop {
            pv.truncate(0);
            root_node.build_pv(&mut root_game_state.clone(), &mut pv);

            let time_left: i64 = self.time_limit as i64 - start_time.elapsed().as_millis() as i64;

            println!(
                "{:6}ms {:5} {:10} {:7} {}",
                time_left,
                pv.len(),
                completed_iterations,
                root_node.get_value(),
                pv.iter()
                    .map(|event| event.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            if time_left < 30 {
                break;
            }

            let iterations =
                ((time_left as f32 / 6.).min(5000.) * iterations_per_ms).max(1.) as usize;
            do_iterations(&mut root_node, &root_game_state, iterations, &mut rng);
            completed_iterations += iterations;

            let elapsed_time = search_start_time.elapsed().as_micros() as f32 / 1000.;
            if elapsed_time > 0. {
                iterations_per_ms = completed_iterations as f32 / elapsed_time
            }
        }

        println!(
            "Search finished after {}ms. Value: {:7} PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            root_node.get_value(),
            pv.len(),
            completed_iterations,
            iterations_per_ms * 1000.,
            pv.iter()
                .map(|event| event.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
        let player_index = usize::from(game_state.get_current_player());
        root_node.best_move(player_index).unwrap()
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            root_node: SharedState::new(Node::new_deterministic(Move::DUMMY)),
            root_game_state: SharedState::new(GameState::new(&mut rng)),
            time_limit: 6000,
        }
    }
}

#[async_trait::async_trait]
impl Player for MonteCarloTreeSearch {
    fn get_name(&self) -> &str {
        "MCTS"
    }

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        let move_ = self.search(game_state).await;
        self.root_game_state.lock().await.do_move(move_);
        let mut root_node = self.root_node.lock().await;
        let player_index = usize::from(game_state.get_current_player());
        println!("Setting child as new root node.");
        let new_root_node = std::mem::replace(
            &mut *root_node.best_child(player_index).lock().unwrap(),
            Node::new_deterministic(Move::DUMMY),
        );
        *root_node = new_root_node;
        println!("New root node set.");
        drop(root_node);
        println!("Move: {}", move_);
        move_
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        let mut invalid = false;
        let mut root_node_guard = self.root_node.lock().await;
        let mut root_game_state_guard = self.root_game_state.lock().await;

        // Find the child node that matches the move
        if let Some((i, _)) =
            root_node_guard
                .get_children()
                .iter()
                .enumerate()
                .find(|(_, child)| {
                    matches!(
                        *child.lock().unwrap().get_previous_event(),
                        Event::Deterministic(child_move) if child_move == move_
                    )
                })
        {
            // Found the matching child, now make it the new root
            println!("Setting child as new root node.");

            // Swap the root node with the matching child node
            let child_arc = root_node_guard.get_children_mut().remove(i);
            let mut child_node = child_arc.lock().unwrap();

            // Take the data out of the child to avoid cloning the whole subtree
            let new_root_node = std::mem::take(&mut *child_node);

            // Update the root node with the data from the matching child
            *root_node_guard = new_root_node;
            *root_game_state_guard = new_game_state.clone();
            println!("New root node set.");
        } else {
            // No matching child was found, or there was a probabilistic event
            for child in root_node_guard.get_children() {
                if let Event::Probabilistic(_) = child.lock().unwrap().get_previous_event() {
                    invalid = true;
                    break;
                }
            }

            if invalid {
                // Replace the root node with a new dummy node if an invalid state was encountered
                *root_node_guard = Node::new_deterministic(Move::DUMMY); // Assuming a new method for creating a Node
            }
            *root_game_state_guard = new_game_state.clone();
        }

        // Explicitly drop the guards to release the locks
        drop(root_node_guard);
        drop(root_game_state_guard);
    }

    async fn set_time(&mut self, time: u64) {
        self.time_limit = time;
    }
}
