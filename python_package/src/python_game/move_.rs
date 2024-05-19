use super::tile_color::TileColor;
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyResult};

#[pyclass]
#[derive(Clone, Copy)]
pub struct Move(pub game::Move);

#[pymethods]
impl Move {
    #[new]
    fn new(
        factory_index: u8,
        tile_color: TileColor,
        pattern_line_index: u8,
        discards: u8,
        places: u8,
    ) -> Self {
        Self(game::Move {
            factory_index,
            color: tile_color.0,
            pattern_line_index,
            discards,
            places,
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

    #[getter]
    fn factory_index(&self) -> u8 {
        self.0.factory_index
    }

    #[getter]
    fn color(&self) -> TileColor {
        TileColor(self.0.color)
    }

    #[getter]
    fn pattern_line_index(&self) -> u8 {
        self.0.pattern_line_index
    }

    #[getter]
    fn discards(&self) -> u8 {
        self.0.discards
    }

    #[getter]
    fn places(&self) -> u8 {
        self.0.places
    }
}
