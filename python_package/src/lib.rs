use pyo3::{pymodule, types::PyModule, Bound, PyResult, Python};

mod python_game;

#[pymodule]
fn azul(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("NUM_PLAYERS", game::NUM_PLAYERS)?;
    m.add("NUM_TILE_COLORS", game::NUM_TILE_COLORS)?;
    m.add("NUM_FACTORIES", game::NUM_FACTORIES)?;

    m.add_class::<python_game::player_marker::PlayerMarker>()?;
    m.add_class::<python_game::game_state::GameState>()?;
    m.add_class::<python_game::tile_color::TileColor>()?;
    Ok(())
}
