use pyo3::{Bound, Python, pyclass, pymethods, types::PyComplex};
use std::format;
use std::string::{String, ToString};

use lox_units::{Angle, Distance, Frequency, Velocity};

macro_rules! py_unit {
    ($(($unit:ident, $name:literal, $pyunit:ident)),*) => {
        $(
            #[pyclass(name = $name, module = "lox_space", frozen)]
            pub struct $pyunit(pub $unit);

            #[pymethods]
            impl $pyunit {
                #[new]
                pub fn new(value: f64) -> Self {
                    Self($unit::new(value))
                }

                pub fn __rmul__(&self, other: f64) -> Self {
                    Self(other * self.0)
                }

                pub fn __repr__(&self) ->  String {
                    format!("{}({})", stringify!($unit), f64::from(self.0))
                }

                pub fn __str__(&self) -> String {
                    self.0.to_string()
                }

                pub fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
                    PyComplex::from_doubles(py, self.0.into(), 0.0)
                }

                pub fn __float__(&self) -> f64 {
                    self.0.into()
                }

                pub fn __int__(&self) -> i64 {
                   let val: f64 =  self.0.into();
                    val.round_ties_even() as i64
                }
            }
        )*
    };
}

py_unit!(
    (Angle, "Angle", PyAngle),
    (Distance, "Distance", PyDistance),
    (Frequency, "Frequency", PyFrequency),
    (Velocity, "Velocity", PyVelocity)
);
