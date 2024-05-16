use super::tile_color::TileColor;
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyResult};

#[pyclass]
#[derive(Clone, Copy)]
pub struct Move(pub game::Move);

#[pymethods]
impl Move {
    #[new]
    fn new(factory_index: u8, tile_color: TileColor, pattern: [u8; 6]) -> Self {
        Self(game::Move {
            take_from_factory_index: factory_index,
            color: tile_color.0,
            pattern,
        })
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{}", self.0)
    }

    fn is_discard_only(&self) -> bool {
        self.0.is_discard_only()
    }

    fn serialize_string(&self) -> String {
        self.0.serialize_string()
    }

    #[staticmethod]
    fn deserialize_string(string: &str) -> PyResult<Self> {
        Ok(Self(
            game::Move::deserialize_string(string).map_err(PyValueError::new_err)?,
        ))
    }
}
