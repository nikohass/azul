use pyo3::{pyclass, pymethods};

#[pyclass]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PlayerMarker(pub game::PlayerMarker);

#[pymethods]
impl PlayerMarker {
    #[new]
    fn new(id: u8) -> Self {
        Self(game::PlayerMarker::new(id))
    }

    fn next(&self) -> Self {
        Self(self.0.next())
    }

    fn __int__(&self) -> u8 {
        self.0.into()
    }

    fn __index__(&self) -> usize {
        self.0.into()
    }

    fn __str__(&self) -> String {
        format!("PlayerMarker({})", self.__int__())
    }

    fn __repr__(&self) -> String {
        format!("PlayerMarker({})", self.__int__())
    }
}
