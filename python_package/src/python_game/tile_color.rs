use pyo3::prelude::PyAnyMethods;
use pyo3::{
    basic::CompareOp, exceptions::PyValueError, pyclass, pymethods, Bound, PyAny, PyResult,
};

#[pyclass]
#[derive(Clone, Copy)]
pub struct TileColor(pub game::TileColor);

#[pymethods]
impl TileColor {
    #[new]
    fn new(color: &Bound<PyAny>) -> PyResult<Self> {
        if let Ok(color_str) = color.extract::<&str>() {
            Ok(match color_str.to_ascii_uppercase().as_str() {
                "B" => Self(game::TileColor::Blue),
                "Y" => Self(game::TileColor::Yellow),
                "R" => Self(game::TileColor::Red),
                "G" => Self(game::TileColor::Green),
                "W" => Self(game::TileColor::White),
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid color: {}",
                        color_str
                    )))
                }
            })
        } else if let Ok(color_int) = color.extract::<i32>() {
            Ok(match color_int {
                0 => Self(game::TileColor::Blue),
                1 => Self(game::TileColor::Yellow),
                2 => Self(game::TileColor::Red),
                3 => Self(game::TileColor::Green),
                4 => Self(game::TileColor::White),
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid color: {}",
                        color_int
                    )))
                }
            })
        } else {
            Err(PyValueError::new_err(format!("Invalid color: {}", color)))
        }
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{}", self.0)
    }

    fn as_char(&self) -> char {
        self.0.into()
    }

    fn __int__(&self) -> u8 {
        self.0.into()
    }

    fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.0 == other.0,
            CompareOp::Ne => self.0 != other.0,
            _ => false,
        }
    }
}
