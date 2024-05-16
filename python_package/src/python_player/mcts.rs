use crate::python_game::{game_state::GameState, move_::Move, time_control::TimeControl};
use game::{Player, NUM_PLAYERS};
use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct MonteCarloTreeSearch(pub player::mcts::MonteCarloTreeSearch);

#[pymethods]
impl MonteCarloTreeSearch {
    #[new]
    fn new() -> Self {
        Self(player::mcts::MonteCarloTreeSearch::default())
    }

    fn start_pondering(&mut self) {
        self.0.start_pondering();
    }

    fn stop_pondering(&mut self) {
        self.0.stop_pondering();
    }

    fn get_principal_variation(&mut self) -> Vec<String> {
        self.0
            .get_principal_variation()
            .iter()
            .map(|e| format!("{}", e))
            .collect()
    }

    #[getter]
    fn name(&self) -> String {
        self.0.get_name().to_string()
    }

    fn set_name(&mut self, name: &str) {
        self.0.set_name(name);
    }

    fn get_move(&mut self, state: &GameState) -> Move {
        Move(self.0.get_move(&state.0))
    }

    fn set_time(&mut self, time: TimeControl) {
        self.0.set_time(time.0);
    }

    fn set_pondering(&mut self, pondering: bool) {
        self.0.set_pondering(pondering);
    }

    fn notify_move(&mut self, state: &GameState, move_: &Move) {
        self.0.notify_move(&state.0, move_.0);
    }

    fn notify_game_over(&mut self, state: &GameState) {
        self.0.notify_game_over(&state.0);
    }

    fn notify_factories_refilled(&mut self, state: &GameState) {
        self.0.notify_factories_refilled(&state.0);
    }

    fn reset(&mut self) {
        self.0.reset();
    }

    #[getter]
    fn value(&self) -> Option<[f32; NUM_PLAYERS]> {
        self.0.get_value().map(|v| v.into())
    }

    fn set_root(&mut self, state: &GameState) {
        self.0.set_root(&state.0);
    }

    fn get_best_move(&mut self) -> Option<Move> {
        self.0.get_best_move().map(Move)
    }

    fn get_evaluated_moves(&mut self) -> Vec<(Move, [f32; NUM_PLAYERS])> {
        self.0
            .get_evaluated_moves()
            .iter()
            .map(|(m, v)| (Move(*m), <[f32; NUM_PLAYERS]>::from(*v)))
            .collect()
    }
}
