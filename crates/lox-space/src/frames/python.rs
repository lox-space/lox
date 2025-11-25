// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::frames::{
    dynamic::{DynFrame, UnknownFrameError},
    traits::ReferenceFrame,
};
use lox_earth::itrf::DynTransformEopError;
use lox_frames::dynamic::DynTransformError;
use pyo3::{
    PyErr, PyResult, create_exception,
    exceptions::{PyException, PyValueError},
    pyclass, pymethods,
};

pub struct PyUnknownFrameError(pub UnknownFrameError);

impl From<PyUnknownFrameError> for PyErr {
    fn from(err: PyUnknownFrameError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

create_exception!(lox_space, FrameTransformationError, PyException);

pub struct PyDynTransformError(pub DynTransformError);

impl From<PyDynTransformError> for PyErr {
    fn from(err: PyDynTransformError) -> Self {
        FrameTransformationError::new_err(err.0.to_string())
    }
}

pub struct PyDynTransformEopError(pub DynTransformEopError);

impl From<PyDynTransformEopError> for PyErr {
    fn from(err: PyDynTransformEopError) -> Self {
        FrameTransformationError::new_err(err.0.to_string())
    }
}

/// Represents a reference frame for positioning and transformations.
///
/// Reference frames define coordinate systems for expressing positions and
/// velocities. Lox supports both inertial (non-rotating) and rotating frames.
///
/// Supported frames:
///
/// - **ICRF**: International Celestial Reference Frame (inertial)
/// - **GCRF**: Geocentric Celestial Reference Frame (inertial, Earth-centered)
/// - **CIRF**: Celestial Intermediate Reference Frame
/// - **TIRF**: Terrestrial Intermediate Reference Frame
/// - **ITRF**: International Terrestrial Reference Frame (Earth-fixed)
/// - **Body-fixed frames**: IAU_EARTH, IAU_MOON, IAU_MARS, etc.
///
/// Args:
///     abbreviation: Frame abbreviation (e.g., "ICRF", "ITRF", "IAU_MOON").
///
/// Raises:
///     ValueError: If the frame abbreviation is not recognized.
///
/// Examples:
///     >>> import lox_space as lox
///     >>> icrf = lox.Frame("ICRF")
///     >>> itrf = lox.Frame("ITRF")
///     >>> moon_fixed = lox.Frame("IAU_MOON")
///
///     >>> icrf.name()
///     'International Celestial Reference Frame'
///     >>> icrf.abbreviation()
///     'ICRF'
#[pyclass(name = "Frame", module = "lox_space", frozen)]
#[pyo3(eq)]
#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct PyFrame(pub DynFrame);

#[pymethods]
impl PyFrame {
    #[new]
    fn new(abbreviation: &str) -> PyResult<Self> {
        Ok(Self(abbreviation.parse().map_err(PyUnknownFrameError)?))
    }

    fn __getnewargs__(&self) -> (String,) {
        (self.abbreviation(),)
    }

    /// Return the full name of this reference frame.
    ///
    /// Returns:
    ///     The descriptive name (e.g., "International Celestial Reference Frame").
    fn name(&self) -> String {
        self.0.name()
    }

    /// Return the abbreviation of this reference frame.
    ///
    /// Returns:
    ///     The short abbreviation (e.g., "ICRF", "ITRF").
    fn abbreviation(&self) -> String {
        self.0.abbreviation()
    }
}
