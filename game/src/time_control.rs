use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimeControl {
    SuddenDeath {
        total_milliseconds: u64, // Total number of milliseconds that the player has for the entire game
    },
    Incremental {
        // The player gets a total time and an increment for each move
        total_milliseconds: u64,
        increment_milliseconds: u64,
    },
    ConstantTimePerMove {
        // The player will spend a constant amount of time per move without any time control
        milliseconds_per_move: u64,
    },
    ConstantIterationsPerMove {
        // The player will spend a constant amount of iterations per move without any time control
        iterations_per_move: u64,
    },
}

impl Display for TimeControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeControl::SuddenDeath { total_milliseconds } => {
                write!(f, "Sudden Death: {}ms", total_milliseconds)
            }
            TimeControl::Incremental {
                total_milliseconds,
                increment_milliseconds,
            } => write!(
                f,
                "Incremental: {}ms total, {}ms increment",
                total_milliseconds, increment_milliseconds
            ),
            TimeControl::ConstantTimePerMove {
                milliseconds_per_move,
            } => {
                write!(f, "Constant Time Per Move: {}ms", milliseconds_per_move)
            }
            TimeControl::ConstantIterationsPerMove {
                iterations_per_move,
            } => {
                write!(f, "Constant Iterations Per Move: {}", iterations_per_move)
            }
        }
    }
}
