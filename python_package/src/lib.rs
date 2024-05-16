use pyo3::{pymodule, types::PyModule, Bound, PyResult, Python};

mod python_game;
mod python_player;

#[pymodule]
fn azul4(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("NUM_PLAYERS", game::NUM_PLAYERS)?;
    m.add("NUM_TILE_COLORS", game::NUM_TILE_COLORS)?;
    m.add("NUM_FACTORIES", game::NUM_FACTORIES)?;

    m.add_class::<python_game::player_marker::PlayerMarker>()?;
    m.add_class::<python_game::game_state::GameState>()?;
    m.add_class::<python_game::tile_color::TileColor>()?;
    m.add_class::<python_game::move_::Move>()?;
    m.add_class::<python_game::game_state::MoveGenerationResult>()?;
    m.add_class::<python_game::time_control::TimeControl>()?;
    m.add_class::<python_player::mcts::MonteCarloTreeSearch>()?;
    Ok(())
}
