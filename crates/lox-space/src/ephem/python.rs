// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use pyo3::{
    Bound, PyAny, PyErr, PyResult, exceptions::PyValueError, pyclass, pymethods,
    types::PyAnyMethods,
};

use crate::ephem::spk::parser::{DafSpkError, Spk, parse_daf_spk};

pub struct PyDafSpkError(pub DafSpkError);

impl From<PyDafSpkError> for PyErr {
    fn from(err: PyDafSpkError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// SPICE SPK (Spacecraft and Planet Kernel) ephemeris data.
///
/// SPK files contain position and velocity data for celestial bodies and
/// spacecraft. They are used to compute accurate positions for orbit
/// propagation, frame transformations, and visibility analysis.
///
/// SPK files can be obtained from:
///
/// - NASA NAIF: https://naif.jpl.nasa.gov/naif/data.html
/// - ESA SPICE Service: https://spice.esac.esa.int/
///
/// Args:
///     path: Path to the SPK file (.bsp).
///
/// Raises:
///     ValueError: If the file cannot be parsed or is invalid.
///     OSError: If the file cannot be read.
///
/// Examples:
///     >>> import lox_space as lox
///     >>> # Load planetary ephemeris
///     >>> spk = lox.SPK("/path/to/de440.bsp")
///
///     >>> # Use with propagation
///     >>> trajectory = propagator.propagate(times, ephemeris=spk)
///
///     >>> # Use with visibility analysis
///     >>> passes = lox.visibility(times, gs, mask, trajectory, ephemeris=spk)
#[pyclass(name = "SPK", module = "lox_space", frozen)]
pub struct PySpk(pub Spk);

#[pymethods]
impl PySpk {
    #[new]
    fn new(path: &Bound<'_, PyAny>) -> PyResult<Self> {
        let path = path.extract::<PathBuf>()?;
        let data = std::fs::read(path)?;
        let spk = parse_daf_spk(&data).map_err(PyDafSpkError)?;
        Ok(PySpk(spk))
    }
}
