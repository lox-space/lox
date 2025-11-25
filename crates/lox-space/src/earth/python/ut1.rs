// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use lox_earth::eop::{self, EopParser, EopProvider};
use pyo3::exceptions::PyException;
use pyo3::types::{PyAnyMethods, PyTuple};
use pyo3::{Bound, PyErr, PyResult, create_exception, pyclass, pymethods};

create_exception!(lox_space, EopParserError, PyException);

pub struct PyEopParserError(pub eop::EopParserError);

impl From<PyEopParserError> for PyErr {
    fn from(err: PyEopParserError) -> Self {
        EopParserError::new_err(err.0.to_string())
    }
}

create_exception!(lox_space, EopProviderError, PyException);

pub struct PyEopProviderError(pub lox_earth::eop::EopProviderError);

impl From<PyEopProviderError> for PyErr {
    fn from(err: PyEopProviderError) -> Self {
        EopProviderError::new_err(err.0.to_string())
    }
}

/// Earth Orientation Parameters (EOP) data provider.
///
/// EOP data is required for accurate transformations involving UT1 and
/// polar motion corrections. The data can be loaded from IERS finals2000A
/// files (CSV format).
///
/// EOP data files can be obtained from:
///
/// - IERS: https://www.iers.org/IERS/EN/DataProducts/EarthOrientationData/eop.html
/// - Celestrak: https://celestrak.org/SpaceData/
///
/// Args:
///     path: Path to the EOP data file (CSV format).
///     path2: Optional second path for separate polar motion and UT1 files.
///
/// Raises:
///     EopParserError: If the file cannot be parsed.
///     OSError: If the file cannot be read.
///
/// Examples:
///     >>> import lox_space as lox
///     >>> # Load EOP data from a single file
///     >>> eop = lox.EOPProvider("/path/to/finals2000A.all.csv")
///
///     >>> # Use with time scale conversions
///     >>> t_tai = lox.Time("TAI", 2024, 1, 1)
///     >>> t_ut1 = t_tai.to_scale("UT1", provider=eop)
///
///     >>> # Use with frame transformations
///     >>> state_itrf = state.to_frame(lox.Frame("ITRF"), provider=eop)
#[pyclass(name = "EOPProvider", module = "lox_space", frozen)]
#[derive(Debug)]
pub struct PyEopProvider(pub EopProvider);

#[pymethods]
impl PyEopProvider {
    #[pyo3(signature = (*args))]
    #[new]
    pub fn new(args: &Bound<'_, PyTuple>) -> PyResult<PyEopProvider> {
        let (path1, path2) = if let Ok((path1, path2)) = args.extract::<(PathBuf, PathBuf)>() {
            (path1, path2)
        } else if let Ok((path,)) = args.extract::<(PathBuf,)>() {
            (path.clone(), path)
        } else {
            return Err(PyEopParserError(eop::EopParserError::NoFiles).into());
        };
        Ok(PyEopProvider(
            EopParser::new()
                .from_paths(path1, path2)
                .parse()
                .map_err(PyEopParserError)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::time::python::time::PyTime;
    use lox_test_utils::data_file;
    use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, Python};

    #[test]
    #[should_panic(expected = "No such file")]
    fn test_ut1_provider_invalid_path() {
        Python::attach(|py| {
            let path = (data_file("invalid"),).into_pyobject(py).unwrap();
            let _provider = PyEopProvider::new(&path).unwrap();
        })
    }

    #[test]
    #[should_panic(expected = "extrapolated")]
    fn test_ut1_provider_extrapolated() {
        Python::attach(|py| {
            let path = (data_file("iers/finals2000A.all.csv"),)
                .into_pyobject(py)
                .unwrap();
            let provider = Bound::new(py, PyEopProvider::new(&path).unwrap()).unwrap();
            let tai =
                PyTime::new(&"TAI".into_bound_py_any(py).unwrap(), 2100, 1, 1, 0, 0, 0.0).unwrap();
            let _ut1 = tai
                .to_scale(&"UT1".into_bound_py_any(py).unwrap(), Some(&provider))
                .unwrap();
        })
    }
}
