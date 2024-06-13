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
    fn fischer_timing_with_max_time(
        base_time_milliseconds: u64,
        increment_milliseconds: u64,
        max_time_milliseconds: u64,
    ) -> Self {
        Self(game::TimeControl::FischerTimingWithMaxTime {
            base_time_milliseconds,
            increment_milliseconds,
            max_time_milliseconds,
        })
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
