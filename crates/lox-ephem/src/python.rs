use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyErr, PyResult};

use crate::spk::parser::{parse_daf_spk, DafSpkError, Spk};

impl From<DafSpkError> for PyErr {
    fn from(err: DafSpkError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "SPK", module = "lox_space", frozen)]
pub struct PySpk(pub Spk);

#[pymethods]
impl PySpk {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let data = std::fs::read(path)?;
        let spk = parse_daf_spk(&data)?;
        Ok(PySpk(spk))
    }
}
