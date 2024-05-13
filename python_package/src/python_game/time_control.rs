use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Clone)]
pub struct TimeControl(pub game::TimeControl);

#[pymethods]
impl TimeControl {
    #[staticmethod]
    fn sudden_death(total_milliseconds: u64) -> Self {
        Self(game::TimeControl::SuddenDeath { total_milliseconds })
    }

    #[staticmethod]
    fn incremental(total_milliseconds: u64, increment_milliseconds: u64) -> Self {
        Self(game::TimeControl::Incremental {
            total_milliseconds,
            increment_milliseconds,
        })
    }

    #[staticmethod]
    fn constant_time_per_move(milliseconds_per_move: u64) -> Self {
        Self(game::TimeControl::ConstantTimePerMove {
            milliseconds_per_move,
        })
    }

    #[staticmethod]
    fn constant_iterations_per_move(iterations_per_move: u64) -> Self {
        Self(game::TimeControl::ConstantIterationsPerMove {
            iterations_per_move,
        })
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
