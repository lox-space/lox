// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::bodies::dynamic::{DynOrigin, UnknownOriginId, UnknownOriginName};
use crate::bodies::{
    Elements, Origin, TryMeanRadius, TryPointMass, TryRotationalElements, TrySpheroid,
    TryTriaxialEllipsoid,
};
use lox_core::types::units::Seconds;
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::str::FromStr;

create_exception!(lox_space, UndefinedOriginPropertyError, PyException);

pub struct PyUndefinedOriginPropertyError(pub crate::bodies::UndefinedOriginPropertyError);

impl From<PyUndefinedOriginPropertyError> for PyErr {
    fn from(err: PyUndefinedOriginPropertyError) -> Self {
        UndefinedOriginPropertyError::new_err(err.0.to_string())
    }
}

pub struct PyUnknownOriginId(pub UnknownOriginId);

impl From<PyUnknownOriginId> for PyErr {
    fn from(err: PyUnknownOriginId) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

struct PyUnknownOriginName(UnknownOriginName);

impl From<PyUnknownOriginName> for PyErr {
    fn from(err: PyUnknownOriginName) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

/// Represents a celestial body (planet, moon, barycenter, etc.).
///
/// Origin objects represent celestial bodies using NAIF/SPICE identifiers.
/// They provide access to physical properties such as gravitational parameters,
/// radii, and rotational elements.
///
/// Args:
///     origin: Body name (e.g., "Earth", "Moon") or NAIF ID (e.g., 399 for Earth).
///
/// Raises:
///     ValueError: If the origin name or ID is not recognized.
///     TypeError: If the argument is neither a string nor an integer.
#[pyclass(name = "Origin", module = "lox_space", frozen, eq)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PyOrigin(pub DynOrigin);

#[pymethods]
impl PyOrigin {
    #[new]
    fn new(origin: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(origin) = origin.extract::<i32>() {
            return Ok(Self(origin.try_into().map_err(PyUnknownOriginId)?));
        }
        if let Ok(origin) = origin.extract::<&str>() {
            return Ok(Self(
                DynOrigin::from_str(origin).map_err(PyUnknownOriginName)?,
            ));
        }
        Err(PyTypeError::new_err(
            "`origin` must be either a string or an integer",
        ))
    }

    fn __repr__(&self) -> String {
        format!("Origin(\"{}\")", self.name())
    }

    fn __str__(&self) -> &str {
        self.name()
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.name(),)
    }

    /// Return the NAIF ID of this body.
    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    /// Return the name of this body.
    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    /// Return the gravitational parameter (GM) in km³/s².
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn gravitational_parameter(&self) -> PyResult<f64> {
        let gp = self
            .0
            .try_gravitational_parameter()
            .map_err(PyUndefinedOriginPropertyError)?;
        Ok(gp.as_f64() * 1e-9)
    }

    /// Return the mean radius in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn mean_radius(&self) -> PyResult<f64> {
        Ok(self
            .0
            .try_mean_radius()
            .map_err(PyUndefinedOriginPropertyError)?
            .to_kilometers())
    }

    /// Return the triaxial radii (x, y, z) in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn radii(&self) -> PyResult<(f64, f64, f64)> {
        let (a, b, c) = self.0.try_radii().map_err(PyUndefinedOriginPropertyError)?;
        Ok((a.to_kilometers(), b.to_kilometers(), c.to_kilometers()))
    }

    /// Return the equatorial radius in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn equatorial_radius(&self) -> PyResult<f64> {
        Ok(self
            .0
            .try_equatorial_radius()
            .map_err(PyUndefinedOriginPropertyError)?
            .to_kilometers())
    }

    /// Return the polar radius in km.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn polar_radius(&self) -> PyResult<f64> {
        Ok(self
            .0
            .try_polar_radius()
            .map_err(PyUndefinedOriginPropertyError)?
            .to_kilometers())
    }

    /// Return rotational elements (right ascension, declination, rotation angle) in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    ///
    /// Returns:
    ///     Tuple of (right_ascension, declination, rotation_angle) in radians.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn rotational_elements(&self, et: Seconds) -> PyResult<Elements> {
        Ok(self
            .0
            .try_rotational_elements(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return rotational element rates in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    ///
    /// Returns:
    ///     Tuple of (ra_rate, dec_rate, rotation_rate) in radians/second.
    ///
    /// Raises:
    ///     UndefinedOriginPropertyError: If not defined for this body.
    pub fn rotational_element_rates(&self, et: Seconds) -> PyResult<Elements> {
        Ok(self
            .0
            .try_rotational_element_rates(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return the right ascension of the pole in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn right_ascension(&self, et: Seconds) -> PyResult<f64> {
        Ok(self
            .0
            .try_right_ascension(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return the rate of change of right ascension in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn right_ascension_rate(&self, et: Seconds) -> PyResult<f64> {
        Ok(self
            .0
            .try_right_ascension_rate(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return the declination of the pole in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn declination(&self, et: Seconds) -> PyResult<f64> {
        Ok(self
            .0
            .try_declination(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return the rate of change of declination in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn declination_rate(&self, et: Seconds) -> PyResult<f64> {
        Ok(self
            .0
            .try_declination_rate(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return the rotation angle (prime meridian) in radians.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn rotation_angle(&self, et: Seconds) -> PyResult<f64> {
        Ok(self
            .0
            .try_rotation_angle(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }

    /// Return the rotation rate in radians/second.
    ///
    /// Args:
    ///     et: Ephemeris time in seconds from J2000.
    pub fn rotation_rate(&self, et: Seconds) -> PyResult<f64> {
        Ok(self
            .0
            .try_rotation_rate(et)
            .map_err(PyUndefinedOriginPropertyError)?)
    }
}
