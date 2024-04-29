use super::node::Node;
use crate::mcts::event::Event;
use game::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

pub struct MonteCarloTreeSearch {
    name: String,
    root_node: Option<Node>,
    root_game_state: GameState,
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
        game_state
            .check_integrity()
            .expect("Trying to set root with invalid game state.");

        if self.root_game_state.serialize_string() == game_state.serialize_string()
            && self.root_node.is_some()
        {
            println!("Keeping parts of the tree from previous search.");
        } else {
            self.root_game_state = game_state.clone();
            self.root_node = Some(Node::new_deterministic(Move::DUMMY));
        }
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

        println!(
            "    Left Depth Iterations Value{} Principal variation",
            " ".repeat(NUM_PLAYERS * 5 - "Value".len())
        );

        let root_node = self.root_node.as_mut().unwrap();

        loop {
            pv.truncate(0);
            root_node.build_pv(&mut self.root_game_state.clone(), &mut pv);

            let time_left: i64 = self.time_limit as i64 - start_time.elapsed().as_millis() as i64;

            println!(
                "{:6}ms {:5} {:10} {:18} {}",
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
            do_iterations(root_node, &self.root_game_state, iterations, &mut rng);
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
        println!("{:?}", root_node.count_nodes());
        let best_move = root_node.best_move(player_index).unwrap();
        {
            let mut game_state = game_state.clone();
            game_state.do_move(best_move);
            self.notify_move(&game_state, best_move).await;
        }
        best_move
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            name: "Monte Carlo Tree Search".to_string(),
            root_node: None,
            root_game_state: GameState::new(&mut rng),
            time_limit: 6000,
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

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        self.search(game_state).await
    }

    async fn set_time(&mut self, time: u64) {
        self.time_limit = time;
    }

    async fn notify_move(&mut self, new_game_state: &GameState, last_move: Move) {
        new_game_state
            .check_integrity()
            .expect("Trying to set root with invalid game state.");
        if new_game_state.serialize_string() == self.root_game_state.serialize_string() {
            return;
        }

        if let Some(new_root_node) = self
            .root_node
            .take()
            .and_then(|root_node| root_node.take_child_with_move(last_move))
        {
            self.root_game_state.do_move(last_move);
            if new_game_state.serialize_string() == self.root_game_state.serialize_string() {
                self.root_game_state = new_game_state.clone();
                self.root_node = Some(new_root_node);
                println!("Successfully applied the move {} to the tree.", last_move);
                return;
            }
        }

        self.root_node = None;
        self.root_game_state = new_game_state.clone();
    }
}
