/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::dynamic::{DynOrigin, UnknownOriginId, UnknownOriginName};
use crate::{
    Elements, Origin, TryMeanRadius, TryPointMass, TryRotationalElements, TrySpheroid,
    TryTriaxialEllipsoid,
};
use crate::{Radii, UndefinedOriginPropertyError as RsUndefinedPropertyError};
use lox_math::types::units::Seconds;
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyTypeError, PyValueError};
use pyo3::prelude::*;
use std::str::FromStr;

create_exception!(lox_space, UndefinedOriginPropertyError, PyException);

impl From<RsUndefinedPropertyError> for PyErr {
    fn from(err: RsUndefinedPropertyError) -> Self {
        UndefinedOriginPropertyError::new_err(err.to_string())
    }
}

impl From<UnknownOriginId> for PyErr {
    fn from(err: UnknownOriginId) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

impl From<UnknownOriginName> for PyErr {
    fn from(err: UnknownOriginName) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "Origin", module = "lox_space", frozen, eq)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PyOrigin(pub DynOrigin);

#[pymethods]
impl PyOrigin {
    #[new]
    fn new(origin: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(origin) = origin.extract::<i32>() {
            return Ok(Self(origin.try_into()?));
        }
        if let Ok(origin) = origin.extract::<&str>() {
            return Ok(Self(DynOrigin::from_str(origin)?));
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

    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn gravitational_parameter(&self) -> PyResult<f64> {
        Ok(self.0.try_gravitational_parameter()?)
    }

    pub fn mean_radius(&self) -> PyResult<f64> {
        Ok(self.0.try_mean_radius()?)
    }

    pub fn radii(&self) -> PyResult<Radii> {
        Ok(self.0.try_radii()?)
    }

    pub fn equatorial_radius(&self) -> PyResult<f64> {
        Ok(self.0.try_equatorial_radius()?)
    }

    pub fn polar_radius(&self) -> PyResult<f64> {
        Ok(self.0.try_polar_radius()?)
    }

    pub fn rotational_elements(&self, et: Seconds) -> PyResult<Elements> {
        Ok(self.0.try_rotational_elements(et)?)
    }

    pub fn rotational_element_rates(&self, et: Seconds) -> PyResult<Elements> {
        Ok(self.0.try_rotational_element_rates(et)?)
    }

    pub fn right_ascension(&self, et: Seconds) -> PyResult<f64> {
        Ok(self.0.try_right_ascension(et)?)
    }

    pub fn right_ascension_rate(&self, et: Seconds) -> PyResult<f64> {
        Ok(self.0.try_right_ascension_rate(et)?)
    }

    pub fn declination(&self, et: Seconds) -> PyResult<f64> {
        Ok(self.0.try_declination(et)?)
    }

    pub fn declination_rate(&self, et: Seconds) -> PyResult<f64> {
        Ok(self.0.try_declination_rate(et)?)
    }

    pub fn rotation_angle(&self, et: Seconds) -> PyResult<f64> {
        Ok(self.0.try_rotation_angle(et)?)
    }

    pub fn rotation_rate(&self, et: Seconds) -> PyResult<f64> {
        Ok(self.0.try_rotation_rate(et)?)
    }
}
