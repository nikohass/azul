use std::time::Instant;

use game::{TimeControl, NUM_PLAYERS};

use super::tree::RootStatistics;

// How much of the remaining time we use until we recalculate the speed.
// If the speed of the iterations changes a lot lower this value.
const FACTOR: f64 = 1.0 / 5.0;

#[derive(Debug, Clone, Copy)]
pub struct RemainingTimeInfo {
    pub current_search_allocated_time: Option<i64>,
    pub game_remaining_time: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeControlResult {
    ContinueFor(std::time::Duration, RemainingTimeInfo),
    Stop,
}

pub struct MctsTimeControl {
    time_control: TimeControl,
    remaining_time: i64,
}

impl MctsTimeControl {
    pub fn new(time_control: TimeControl) -> Self {
        let mut ret = Self {
            time_control,
            remaining_time: 0,
        };
        ret.reset();
        ret
    }

    pub fn how_long(
        &mut self,
        search_start_time: Instant,
        stats: &Option<RootStatistics>,
    ) -> TimeControlResult {
        let elapsed_time = search_start_time.elapsed().as_millis() as i64;
        // Calculate how much time we want to allocate for this search
        let (allocated_time, remaining_time_info) = match self.time_control {
            TimeControl::ConstantTimePerMove {
                milliseconds_per_move,
            } => (
                milliseconds_per_move as i64,
                RemainingTimeInfo {
                    current_search_allocated_time: Some(milliseconds_per_move as i64),
                    game_remaining_time: None,
                },
            ),
            TimeControl::SuddenDeath { .. } => {
                let average_plies = if let Some(average_plies) =
                    stats.as_ref().and_then(|stats| stats.average_plies())
                {
                    average_plies
                } else {
                    30.0 // Just assume 30 plies for now if we don't have any statistics
                };
                let remaining_searches = f64::floor(average_plies / NUM_PLAYERS as f64);
                let allocated_time_per_search = self.remaining_time as f64 / remaining_searches;
                (
                    allocated_time_per_search as i64,
                    RemainingTimeInfo {
                        current_search_allocated_time: Some(allocated_time_per_search as i64),
                        game_remaining_time: Some(self.remaining_time),
                    },
                )
            }
            TimeControl::FischerTimingWithMaxTime {
                base_time_milliseconds,
                increment_milliseconds,
                max_time_milliseconds,
            } => {
                let average_plies = if let Some(average_plies) =
                    stats.as_ref().and_then(|stats| stats.average_plies())
                {
                    average_plies
                } else {
                    30.0 // Just assume 30 plies for now if we don't have any statistics
                };
                // Make sure max time is not smaller than base time
                let max_time_milliseconds = max_time_milliseconds.max(base_time_milliseconds);
                let remaining_searches = f64::floor(average_plies / NUM_PLAYERS as f64);

                let expected_remaining_time = self.remaining_time
                    + (increment_milliseconds as i64 * remaining_searches as i64);
                println!("Remaining searches: {}", remaining_searches);
                println!(
                    "Expected remaining time: {}",
                    expected_remaining_time - elapsed_time
                );

                let allocated_time_per_search = expected_remaining_time as f64 / remaining_searches;
                let allocated_time_per_search =
                    allocated_time_per_search.min(max_time_milliseconds as f64);
                println!("Allocated time per search: {}", allocated_time_per_search);
                (
                    allocated_time_per_search as i64,
                    RemainingTimeInfo {
                        current_search_allocated_time: Some(allocated_time_per_search as i64),
                        game_remaining_time: Some(expected_remaining_time),
                    },
                )
            }
            _ => panic!("Time control not implemented"),
        };

        let effective_remaining_time = allocated_time - elapsed_time;
        let effective_remaining_time = effective_remaining_time.max(0);

        if effective_remaining_time < 10 {
            self.remaining_time -= elapsed_time;

            match self.time_control {
                TimeControl::FischerTimingWithMaxTime {
                    base_time_milliseconds: _,
                    increment_milliseconds,
                    max_time_milliseconds,
                } => {
                    self.remaining_time += increment_milliseconds as i64;
                    self.remaining_time = self.remaining_time.min(max_time_milliseconds as i64);
                }
                _ => {}
            }

            TimeControlResult::Stop
        } else {
            let time_until_next_check = (effective_remaining_time as f64 * FACTOR) as u64;
            let time_until_next_check = time_until_next_check.min(5000); // Never wait more than 5 seconds
            TimeControlResult::ContinueFor(
                std::time::Duration::from_millis(time_until_next_check),
                remaining_time_info,
            )
        }
    }

    // Calculates the number of iterations the Monte Carlo Tree Search (MCTS) should execute next.
    // Takes into account the start time of the search, number of completed iterations,
    // search speed (iterations per millisecond), and the estimated remaining moves (plies) in the game.
    // pub fn get_num_next_iterations(
    //     &mut self,
    //     search_start_time: Instant,
    //     completed_iterations: u64,
    //     search_speed: f64,
    //     estimated_remaining_plies: f32,
    // ) -> (u64, RemainingTimeInfo) {
    //     match self.time_control {
    //         TimeControl::ConstantIterationsPerMove {
    //             iterations_per_move,
    //         } => {
    //             // Do this in at least 10 steps, but never complete more than 200k iterations without logging the stats
    //             let iterations_left = iterations_per_move - completed_iterations;
    //             let iterations_to_run = iterations_left.min(200_000.min(iterations_per_move / 10));
    //             (
    //                 iterations_to_run,
    //                 RemainingTimeInfo {
    //                     time_left_for_search: None,
    //                     time_left_for_game: None,
    //                     iterations_left_for_search: Some(iterations_left),
    //                 },
    //             )
    //         }
    //         TimeControl::ConstantTimePerMove {
    //             milliseconds_per_move,
    //         } => {
    //             let elapsed_time = search_start_time.elapsed().as_millis() as i64;
    //             let remaining_time = milliseconds_per_move as i64 - elapsed_time;
    //             let time_info = RemainingTimeInfo {
    //                 time_left_for_search: Some(remaining_time),
    //                 time_left_for_game: None,
    //                 iterations_left_for_search: None,
    //             };
    //             if remaining_time < 10 {
    //                 (0, time_info)
    //             } else {
    //                 // At least every 5000 ms we log the stats
    //                 // We always return at least 1 iteration
    //                 let iterations = ((remaining_time as f64 * FACTOR).min(5000.) * search_speed)
    //                     .max(100.0) as u64;
    //                 (iterations, time_info)
    //             }
    //         }
    //         TimeControl::SuddenDeath { total_milliseconds } => {
    //             let search_elapsed_time = search_start_time.elapsed().as_millis() as i64;
    //             let remaining_time = total_milliseconds as i64 - self.my_used_time as i64;
    //             let remaining_searches =
    //                 f64::round(estimated_remaining_plies as f64 / NUM_PLAYERS as f64);
    //             let allocated_time_per_search = remaining_time as f64 / remaining_searches;

    //             let effective_remaining_time =
    //                 allocated_time_per_search - search_elapsed_time as f64;

    //             let time_info = RemainingTimeInfo {
    //                 time_left_for_search: Some(effective_remaining_time as i64),
    //                 time_left_for_game: Some(remaining_time - search_elapsed_time),
    //                 iterations_left_for_search: None,
    //             };
    //             if effective_remaining_time < 10. || effective_remaining_time.is_infinite() {
    //                 self.my_used_time += search_elapsed_time as u64;
    //                 (0, time_info)
    //             } else {
    //                 let iterations = ((effective_remaining_time * FACTOR).min(5000.) * search_speed)
    //                     .max(100.0) as u64;
    //                 (iterations, time_info)
    //             }
    //         }
    //         TimeControl::Incremental {
    //             total_milliseconds,
    //             increment_milliseconds,
    //         } => {
    //             let elapsed_search_time = search_start_time.elapsed().as_millis() as i64;
    //             let remaining_searches =
    //                 f64::round(estimated_remaining_plies as f64 / NUM_PLAYERS as f64);
    //             let remaining_time = total_milliseconds as i64 - self.my_used_time as i64
    //                 + self.my_bonus_time as i64
    //                 + (remaining_searches as i64 * increment_milliseconds as i64);
    //             let allocated_time_per_search = remaining_time as f64 / remaining_searches;
    //             let effective_remaining_time =
    //                 allocated_time_per_search - elapsed_search_time as f64;

    //             let time_info = RemainingTimeInfo {
    //                 time_left_for_search: Some(if effective_remaining_time.is_finite() {
    //                     effective_remaining_time as i64
    //                 } else {
    //                     0
    //                 }),
    //                 time_left_for_game: Some(
    //                     total_milliseconds as i64 - self.my_used_time as i64
    //                         + self.my_bonus_time as i64,
    //                 ),
    //                 iterations_left_for_search: None,
    //             };
    //             if effective_remaining_time < 10. || effective_remaining_time.is_infinite() {
    //                 self.my_used_time += elapsed_search_time as u64;
    //                 self.my_bonus_time += increment_milliseconds;
    //                 (0, time_info)
    //             } else {
    //                 let iterations =
    //                     (effective_remaining_time * FACTOR * search_speed).max(100.0) as u64;
    //                 (iterations, time_info)
    //             }
    //         }
    //         // TimeControl::RealTimeIncremental {
    //         //     base_time_milliseconds,
    //         //     increment_milliseconds,
    //         //     max_time_milliseconds,
    //         // } => {
    //         //     if completed_iterations == 0 {
    //         //         self.my_bonus_time += increment_milliseconds;
    //         //         self.my_bonus_time = self.my_bonus_time.min(max_time_milliseconds);
    //         //     }

    //         //     let elapsed_search_time = search_start_time.elapsed().as_millis() as i64;
    //         //     let remaining_searches =
    //         //         f64::round(estimated_remaining_plies as f64 / NUM_PLAYERS as f64);
    //         //     let remaining_time = base_time_milliseconds as i64 - self.my_used_time as i64
    //         //         + self.my_bonus_time as i64
    //         //         + (remaining_searches as i64 * increment_milliseconds as i64);
    //         //     let allocated_time_per_search = remaining_time as f64 / remaining_searches;

    //         //     let effective_remaining_time =
    //         //         allocated_time_per_search - elapsed_search_time as f64;
    //         //     let time_left_for_game = base_time_milliseconds as i64 - self.my_used_time as i64
    //         //         + self.my_bonus_time as i64
    //         //         - elapsed_search_time;
    //         //     let effective_remaining_time =
    //         //         effective_remaining_time.min(time_left_for_game as f64);
    //         //     let time_info = RemainingTimeInfo {
    //         //         time_left_for_search: Some(if effective_remaining_time.is_finite() {
    //         //             effective_remaining_time as i64
    //         //         } else {
    //         //             0
    //         //         }),
    //         //         time_left_for_game: Some(time_left_for_game),
    //         //         iterations_left_for_search: None,
    //         //     };
    //         //     if effective_remaining_time < 10. || effective_remaining_time.is_infinite() {
    //         //         self.my_used_time += elapsed_search_time as u64;
    //         //         (0, time_info)
    //         //     } else {
    //         //         let iterations =
    //         //             (effective_remaining_time * FACTOR * search_speed).max(100.0) as u64;
    //         //         (iterations, time_info)
    //         //     }
    //         // }
    //         _ => panic!("Time control not implemented"),
    //     }
    // }

    pub fn reset(&mut self) {
        self.remaining_time = self.time_control.get_total_time() as i64;
    }

    pub fn set_remaining_time(&mut self, remaining_time: i64) {
        self.remaining_time = remaining_time;
    }
}

impl std::fmt::Display for MctsTimeControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.time_control)
    }
}
