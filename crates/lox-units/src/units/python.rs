use pyo3::{pyclass, pymethods};

use crate::Angle;

#[pyclass(name = "Angle", frozen)]
pub struct PyAngle(pub Angle);

#[pymethods]
impl PyAngle {
    #[new]
    fn new(value: f64) -> Self {
        Self(Angle(value))
    }

    fn __rmul__(&self, other: f64) -> Self {
        Self(Angle(other * self.0.0))
    }
}
