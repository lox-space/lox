/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::str::FromStr;

use numpy::{pyarray, PyArray1};
use pyo3::prelude::*;

use lox_core::bodies::PointMass;
use lox_core::coords::states::{CartesianState, KeplerianState, TwoBodyState};
use lox_core::coords::DVec3;

use crate::bodies::PyBody;
use crate::frames::PyFrame;
use crate::time::PyEpoch;

#[pyclass(name = "Cartesian")]
pub struct PyCartesian {
    state: CartesianState,
    origin: PyBody,
    frame: PyFrame,
}

#[pymethods]
impl PyCartesian {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: &PyEpoch,
        body: PyObject,
        frame: &str,
        x: f64,
        y: f64,
        z: f64,
        vx: f64,
        vy: f64,
        vz: f64,
    ) -> PyResult<Self> {
        let origin: PyBody = body.try_into()?;
        let frame = PyFrame::from_str(frame)?;
        let state = CartesianState::new(time.0, DVec3::new(x, y, z), DVec3::new(vx, vy, vz));
        Ok(Self {
            state,
            origin,
            frame,
        })
    }

    fn to_keplerian(&self) -> PyKeplerian {
        let mu = self.origin.gravitational_parameter();
        let state = self.state.to_keplerian_state(mu);
        PyKeplerian {
            origin: self.origin.clone(),
            frame: self.frame.clone(),
            state,
        }
    }

    fn time(&self) -> PyEpoch {
        PyEpoch(self.state.time())
    }

    fn reference_frame(&self) -> String {
        format!("{}", self.frame)
    }

    fn origin(&self) -> PyObject {
        self.origin.clone().into()
    }

    fn position(&self) -> Py<PyArray1<f64>> {
        let position = self.state.position();
        Python::with_gil(|py| pyarray![py, position.x, position.y, position.z].into_py(py))
    }

    fn velocity(&self) -> Py<PyArray1<f64>> {
        let velocity = self.state.velocity();
        Python::with_gil(|py| pyarray![py, velocity.x, velocity.y, velocity.z].into_py(py))
    }
}

#[pyclass(name = "Keplerian")]
pub struct PyKeplerian {
    state: KeplerianState,
    origin: PyBody,
    frame: PyFrame,
}

#[pymethods]
impl PyKeplerian {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        t: &PyEpoch,
        body: PyObject,
        frame: &str,
        semi_major: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_arg: f64,
        true_anomaly: f64,
    ) -> PyResult<Self> {
        let origin: PyBody = body.try_into()?;
        let frame = PyFrame::from_str(frame)?;
        let state = KeplerianState::new(
            t.0,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        );
        Ok(Self {
            state,
            origin,
            frame,
        })
    }

    fn to_cartesian(&self) -> PyCartesian {
        let mu = self.origin.gravitational_parameter();
        let state = self.state.to_cartesian_state(mu);
        PyCartesian {
            state,
            origin: self.origin.clone(),
            frame: self.frame.clone(),
        }
    }

    fn time(&self) -> PyEpoch {
        PyEpoch(self.state.time())
    }

    fn reference_frame(&self) -> String {
        format!("{}", self.frame)
    }

    fn origin(&self) -> PyObject {
        self.origin.clone().into()
    }

    fn keplerian(&self) -> Py<PyArray1<f64>> {
        Python::with_gil(|py| {
            pyarray![
                py,
                self.state.true_anomaly(),
                self.state.eccentricity(),
                self.state.inclination(),
                self.state.ascending_node(),
                self.state.periapsis_argument(),
                self.state.true_anomaly()
            ]
            .into_py(py)
        })
    }

    fn semi_major(&self) -> f64 {
        self.state.semi_major_axis()
    }

    fn eccentricity(&self) -> f64 {
        self.state.eccentricity()
    }

    fn inclination(&self) -> f64 {
        self.state.inclination()
    }

    fn ascending_node(&self) -> f64 {
        self.state.ascending_node()
    }

    fn periapsis_arg(&self) -> f64 {
        self.state.periapsis_argument()
    }

    fn true_anomaly(&self) -> f64 {
        self.state.true_anomaly()
    }
}
