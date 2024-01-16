/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use numpy::{pyarray, PyArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use lox_core::time::epochs::Epoch;
use lox_core::two_body::elements::cartesian_to_keplerian;
use lox_core::two_body::{DVec3, Elements, TwoBody};

use crate::bodies::{PyBarycenter, PyBody, PyMinorBody, PyPlanet, PySatellite, PySun};
use crate::time::PyEpoch;

#[pyclass(name = "Cartesian")]
pub struct PyCartesian {
    t: Epoch,
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    center: PyBody,
}

impl TwoBody for PyCartesian {
    fn epoch(&self) -> Epoch {
        self.t
    }

    fn position(&self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }

    fn velocity(&self) -> DVec3 {
        DVec3::new(self.vx, self.vy, self.vz)
    }

    fn cartesian(&self) -> (DVec3, DVec3) {
        (TwoBody::position(self), TwoBody::velocity(self))
    }

    fn keplerian(&self) -> Elements {
        let mu = match &self.center {
            PyBody::Barycenter(barycenter) => barycenter.gravitational_parameter(),
            PyBody::Sun(sun) => sun.gravitational_parameter(),
            PyBody::Planet(planet) => planet.gravitational_parameter(),
            PyBody::Satellite(satellite) => satellite.gravitational_parameter(),
            PyBody::MinorBody(minor_body) => minor_body.gravitational_parameter(),
        };
        cartesian_to_keplerian(mu, TwoBody::position(self), TwoBody::velocity(self))
    }

    fn semi_major(&self) -> f64 {
        self.keplerian().0
    }

    fn eccentricity(&self) -> f64 {
        self.keplerian().1
    }

    fn inclination(&self) -> f64 {
        self.keplerian().2
    }

    fn ascending_node(&self) -> f64 {
        self.keplerian().3
    }

    fn periapsis_arg(&self) -> f64 {
        self.keplerian().4
    }

    fn true_anomaly(&self) -> f64 {
        self.keplerian().5
    }
}

#[pymethods]
impl PyCartesian {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        t: &PyEpoch,
        x: f64,
        y: f64,
        z: f64,
        vx: f64,
        vy: f64,
        vz: f64,
        body: PyObject,
    ) -> PyResult<Self> {
        let body = Python::with_gil(|py| {
            if let Ok(body) = body.extract::<PyBarycenter>(py) {
                Some(PyBody::Barycenter(body))
            } else if let Ok(body) = body.extract::<PySun>(py) {
                Some(PyBody::Sun(body))
            } else if let Ok(body) = body.extract::<PyPlanet>(py) {
                Some(PyBody::Planet(body))
            } else if let Ok(body) = body.extract::<PySatellite>(py) {
                Some(PyBody::Satellite(body))
            } else if let Ok(body) = body.extract::<PyMinorBody>(py) {
                Some(PyBody::MinorBody(body))
            } else {
                None
            }
        });
        match body {
            None => Err(PyValueError::new_err("foo")),
            Some(body) => Ok(Self {
                t: t.0,
                x,
                y,
                z,
                vx,
                vy,
                vz,
                center: body,
            }),
        }
    }
    fn center(&self) -> PyObject {
        Python::with_gil(|py| match &self.center {
            PyBody::Barycenter(barycenter) => barycenter.clone().into_py(py),
            PyBody::Sun(sun) => sun.clone().into_py(py),
            PyBody::Planet(planet) => planet.clone().into_py(py),
            PyBody::Satellite(satellite) => satellite.clone().into_py(py),
            PyBody::MinorBody(minor_body) => minor_body.clone().into_py(py),
        })
    }

    fn position(&self) -> Py<PyArray1<f64>> {
        Python::with_gil(|py| pyarray![py, self.x, self.y, self.z].into_py(py))
    }

    fn velocity(&self) -> Py<PyArray1<f64>> {
        Python::with_gil(|py| pyarray![py, self.vx, self.vy, self.vz].into_py(py))
    }

    fn cartesian(&self) -> (Py<PyArray1<f64>>, Py<PyArray1<f64>>) {
        (self.position(), self.velocity())
    }

    fn keplerian(&self) -> Elements {
        TwoBody::keplerian(self)
    }

    fn semi_major(&self) -> f64 {
        self.keplerian().0
    }

    fn eccentricity(&self) -> f64 {
        self.keplerian().1
    }

    fn inclination(&self) -> f64 {
        self.keplerian().2
    }

    fn ascending_node(&self) -> f64 {
        self.keplerian().3
    }

    fn periapsis_arg(&self) -> f64 {
        self.keplerian().4
    }

    fn true_anomaly(&self) -> f64 {
        self.keplerian().5
    }
}
