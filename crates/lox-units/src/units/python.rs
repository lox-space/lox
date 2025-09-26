use pyo3::{Bound, Python, pyclass, pymethods, types::PyComplex};

use crate::Angle;

#[pyclass(name = "Angle", module = "lox_space", frozen)]
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

    fn __repr__(&self) -> String {
        format!("Angle({})", self.0.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
        PyComplex::from_doubles(py, self.0.0, 0.0)
    }

    fn __float__(&self) -> f64 {
        self.0.0
    }

    fn __int__(&self) -> i64 {
        self.0.0.round_ties_even() as i64
    }
}
