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
        let (mut allocated_time, remaining_time_info) = match self.time_control {
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
                // println!("Remaining searches: {}", remaining_searches);
                // println!(
                //     "Expected remaining time: {}",
                //     expected_remaining_time - elapsed_time
                // );

                let allocated_time_per_search = expected_remaining_time as f64 / remaining_searches;
                // let allocated_time_per_search = allocated_time_per_search * 3.0; // Assumption is that we wont need much time at the end of the game
                let allocated_time_per_search =
                    allocated_time_per_search.min(max_time_milliseconds as f64);
                // println!("Allocated time per search: {}", allocated_time_per_search);
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

        if let Some(stats) = stats {
            if early_stopping_heuristic(stats) {
                #[cfg(debug_assertions)]
                {
                    log::debug!("Early stopping.");
                }
                allocated_time = 0;
            }
        }

        let effective_remaining_time = allocated_time - elapsed_time;
        let effective_remaining_time = effective_remaining_time.max(0);

        if effective_remaining_time < 10 {
            self.remaining_time -= elapsed_time;

            #[allow(clippy::single_match)]
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
            let time_until_next_check = time_until_next_check.min(1000); // Never wait more than 1 second
            TimeControlResult::ContinueFor(
                std::time::Duration::from_millis(time_until_next_check),
                remaining_time_info,
            )
        }
    }

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

fn early_stopping_heuristic(statistics: &RootStatistics) -> bool {
    // If we can't make any decisions, we should stop early to avoid wasting time
    if statistics.branching_factor == 1 {
        return true;
    }

    // Certainty based on the number of visits
    let certainty_visits = ((statistics.visits as f64 + 1.0).log10() / 6.0).min(1.0);
    // Certainty based on the win probability
    let value: [f64; NUM_PLAYERS] = statistics.value.into();
    let max_value = value.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let outcome_certainty = 4.0 * (max_value - 0.5).powi(2);

    // let certainty_ratio = if statistics.top_two_ratio > 1.05 && statistics.visits > 100_000 {
    //     1.0
    // } else {
    //     0.0
    // };

    let branching_factor = statistics.branching_factor;
    // let branching_factor_bonus = if branching_factor > 0 && branching_factor < 20 {
    //     0.5 * certainty_visits
    // } else {
    //     0.0
    // };
    let branching_factor_bonus = 0.25 - (branching_factor as f64 / 325.).sqrt();

    // Combine the certainties
    let combined_certainty =
        certainty_visits * outcome_certainty + branching_factor_bonus + (statistics.top_two_ratio.min(2.0).max(1.0) - 1.0) * 2.0 * certainty_visits;
    // let combined_certainty =
    //     certainty_visits * certainty_value * (1.0 + certainty_ratio) + branching_factor_bonus;

    combined_certainty >= 0.45
}
