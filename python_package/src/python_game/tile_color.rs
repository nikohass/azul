use pyo3::{basic::CompareOp, exceptions::PyValueError, pyclass, pymethods, PyResult};

#[pyclass]
#[derive(Clone, Copy)]
pub struct TileColor(pub game::TileColor);

#[pymethods]
impl TileColor {
    #[new]
    fn new(color: char) -> PyResult<Self> {
        Ok(match color.to_ascii_uppercase() {
            'B' => Self(game::TileColor::Blue),
            'Y' => Self(game::TileColor::Yellow),
            'R' => Self(game::TileColor::Red),
            'G' => Self(game::TileColor::Green),
            'W' => Self(game::TileColor::White),
            _ => return Err(PyValueError::new_err(format!("Invalid color: {}", color))),
        })
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
