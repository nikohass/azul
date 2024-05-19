use super::time_control::{MctsTimeControl, TimeControlResult};
use super::tree::Tree;
use super::value::Value;
use crate::mcts::edge::Edge;
use crate::mcts::time_control::RemainingTimeInfo;
use game::*;
use std::time::Instant;

pub struct MonteCarloTreeSearch {
    name: String,
    tree: Tree,
    time_control: MctsTimeControl,
}

impl MonteCarloTreeSearch {
    pub fn advance_root(&self, game_state: &GameState, edge: Option<Edge>) {
        self.tree.advance_root(game_state, edge);
    }

    pub fn start_working(&self) {
        self.tree.start_working();
    }

    pub fn stop_working(&self) {
        self.tree.stop_working();
    }

    pub fn rated_moves(&mut self) -> Vec<(Move, f32)> {
        self.tree.rated_moves()
    }

    pub fn value(&mut self) -> Option<Value> {
        self.tree.value()
    }

    pub fn policy(&mut self) -> Option<Move> {
        self.tree.policy()
    }

    pub fn principal_variation(&mut self) -> Vec<Edge> {
        self.tree.principal_variation()
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        Self {
            tree: Tree::default(),
            name: "Monte Carlo Tree Search".to_string(),
            time_control: MctsTimeControl::new(TimeControl::ConstantTimePerMove {
                milliseconds_per_move: 6000,
            }),
        }
    }
}

impl Player for MonteCarloTreeSearch {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn set_time(&mut self, time: TimeControl) {
        self.time_control = MctsTimeControl::new(time);
    }

    fn get_move(&mut self, game_state: &GameState) -> Move {
        self.advance_root(game_state, None);

        let search_start_time = Instant::now();
        self.start_working();
        std::thread::sleep(std::time::Duration::from_millis(15));

        #[cfg(not(feature = "mute"))]
        {
            let player_string = (0..NUM_PLAYERS)
                .map(|i| format!("  V{}", i + 1))
                .collect::<Vec<String>>()
                .join(" ");
            println!(
                "{:>10} {:>3} {:>9} {} {:>8} {:>8} {:>8} {:>2}",
                "Iterations",
                "PVd",
                "Avg.Plies",
                player_string,
                "Stop",
                "Total",
                "Speed",
                "Principal Variation"
            );
        }
        loop {
            let statistics = self.tree.root_statistics();
            let time_control_result = self.time_control.how_long(search_start_time, &statistics);
            if let Some(statistics) = statistics.as_ref() {
                let remaining_time_info = match time_control_result {
                    TimeControlResult::ContinueFor(_, remaining_time_info) => remaining_time_info,
                    TimeControlResult::Stop => RemainingTimeInfo {
                        current_search_allocated_time: Some(0),
                        game_remaining_time: None,
                    },
                };

                #[cfg(not(feature = "mute"))]
                println!(
                    "{:10} {:3} {:>9} {} {:>8} {:>8} {:>8} {}",
                    statistics.visits,
                    statistics.principal_variation.len(),
                    format!("{:.1}", statistics.average_plies().unwrap_or(0.0)),
                    statistics.value,
                    remaining_time_info
                        .current_search_allocated_time
                        .map(|t| t - search_start_time.elapsed().as_millis() as i64)
                        .map_or("N/A".to_string(), |v| format!("{}ms", v)),
                    remaining_time_info
                        .game_remaining_time
                        .map(|t| t - search_start_time.elapsed().as_millis() as i64)
                        .map_or("N/A".to_string(), |v| format!("{}ms", v)),
                    format!("{:.0}/ms", statistics.speed),
                    statistics
                        .principal_variation
                        .iter()
                        .map(|edge| edge.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
            match time_control_result {
                TimeControlResult::ContinueFor(duration, ..) => {
                    std::thread::sleep(duration);
                }
                TimeControlResult::Stop => {
                    break;
                }
            }
        }

        self.stop_working();

        self.policy().unwrap()
    }

    fn reset(&mut self) {
        self.tree = Tree::default();
        self.time_control.reset();
    }

    fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        self.advance_root(new_game_state, Some(Edge::Deterministic(move_)));
    }
}
