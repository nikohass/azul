use super::node::Node;
use crate::mcts::event::Event;
use game::*;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

pub struct MonteCarloTreeSearch {
    root_node: Node,
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
        if self.root_node.get_children().is_empty()
            || game_state.to_string() != self.root_game_state.to_string()
        {
            println!("Could not find the given game state in the tree. Falling back to the default root node.");
            self.root_node = Node::new_deterministic(Move::DUMMY);
        } else {
            println!("Found the given game state in the tree. Setting it as the new root node.");
        }
        self.root_game_state = game_state.clone();
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

        println!("    Left Depth Iterations          Value PV");

        loop {
            pv.truncate(0);
            self.root_node
                .build_pv(&mut self.root_game_state.clone(), &mut pv);

            let time_left: i64 = self.time_limit as i64 - start_time.elapsed().as_millis() as i64;

            println!(
                "{:6}ms {:5} {:10} {:7} {}",
                time_left,
                pv.len(),
                completed_iterations,
                self.root_node.get_value(),
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
            do_iterations(
                &mut self.root_node,
                &self.root_game_state,
                iterations,
                &mut rng,
            );
            completed_iterations += iterations;

            let elapsed_time = search_start_time.elapsed().as_micros() as f32 / 1000.;
            if elapsed_time > 0. {
                iterations_per_ms = completed_iterations as f32 / elapsed_time
            }
        }

        println!(
            "Search finished after {}ms. Value: {:7} PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            self.root_node.get_value(),
            pv.len(),
            completed_iterations,
            iterations_per_ms * 1000.,
            pv.iter()
                .map(|event| event.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );

        let player_index = usize::from(game_state.get_current_player());
        println!("{:#?}", self.root_node.count_nodes());
        self.root_node.best_move(player_index).unwrap()
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            root_node: Node::new_deterministic(Move::DUMMY),
            root_game_state: GameState::new(&mut rng),
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
        self.root_game_state.do_move(move_);
        move_
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {}

    async fn set_time(&mut self, time: u64) {
        self.time_limit = time;
    }
}
