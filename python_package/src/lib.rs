use pyo3::{pyfunction, pymodule, types::PyModule, wrap_pyfunction, Bound, PyResult, Python};
use shared::logging::init_logging;

mod python_game;
mod python_player;

#[pymodule]
fn azul4(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    init_logging("python");

    m.add_function(wrap_pyfunction!(info, m)?)?;
    m.add_function(wrap_pyfunction!(warn, m)?)?;
    m.add_function(wrap_pyfunction!(error, m)?)?;
    m.add_function(wrap_pyfunction!(debug, m)?)?;

    m.add("NUM_PLAYERS", game::NUM_PLAYERS)?;
    m.add("NUM_TILE_COLORS", game::NUM_TILE_COLORS)?;
    m.add("NUM_FACTORIES", game::NUM_FACTORIES)?;
    m.add("WALL_COLOR_MASKS", game::WALL_COLOR_MASKS)?;

    m.add_class::<python_game::player_marker::PlayerMarker>()?;
    m.add_class::<python_game::game_state::GameState>()?;
    m.add_class::<python_game::tile_color::TileColor>()?;
    m.add_class::<python_game::move_::Move>()?;
    m.add_class::<python_game::game_state::MoveGenerationResult>()?;
    m.add_class::<python_game::time_control::TimeControl>()?;
    m.add_class::<python_player::mcts::MonteCarloTreeSearch>()?;

    m.add_class::<python_player::dataloader::DataLoader>()?;
    m.add_function(wrap_pyfunction!(
        python_player::dataloader::encode_game_state,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(python_player::dataloader::decode_move, m)?)?;
    m.add_function(wrap_pyfunction!(python_player::dataloader::encode_move, m)?)?;
    Ok(())
}

#[pyfunction]
fn info(message: &str) {
    log::info!("{}", message);
}

#[pyfunction]
fn warn(message: &str) {
    log::warn!("{}", message);
}

#[pyfunction]
fn error(message: &str) {
    log::error!("{}", message);
}

#[pyfunction]
fn debug(message: &str) {
    log::debug!("{}", message);
}
