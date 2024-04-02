/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::str::FromStr;

use pyo3::prelude::*;

use lox_bodies::PointMass;
use lox_orbits::base::{BaseCartesian, BaseKeplerian, BaseTwoBody};
use lox_orbits::DVec3;

use crate::bodies::PyBody;
use crate::frames::PyFrame;
use crate::time::PyTime;

#[pyclass(name = "Cartesian")]
pub struct PyCartesian {
    time: PyTime,
    state: BaseCartesian,
    origin: PyBody,
    frame: PyFrame,
}

#[pymethods]
impl PyCartesian {
    #[allow(clippy::too_many_arguments)]
    #[new]
    fn new(
        time: PyTime,
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
        let state = BaseCartesian::new(DVec3::new(x, y, z), DVec3::new(vx, vy, vz));
        Ok(Self {
            time,
            state,
            origin,
            frame,
        })
    }

    fn time(&self) -> PyTime {
        self.time
    }

    fn reference_frame(&self) -> String {
        format!("{}", self.frame)
    }

    fn origin(&self) -> PyObject {
        self.origin.clone().into()
    }

    fn position(&self) -> (f64, f64, f64) {
        let position = self.state.position();
        (position.x, position.y, position.z)
    }

    fn velocity(&self) -> (f64, f64, f64) {
        let velocity = self.state.velocity();
        (velocity.x, velocity.y, velocity.z)
    }

    fn to_keplerian(&self) -> PyKeplerian {
        let mu = self.origin.gravitational_parameter();
        let state = self.state.to_keplerian_state(mu);
        PyKeplerian {
            state,
            time: self.time,
            origin: self.origin.clone(),
            frame: self.frame.clone(),
        }
    }
}

#[pyclass(name = "Keplerian")]
pub struct PyKeplerian {
    time: PyTime,
    state: BaseKeplerian,
    origin: PyBody,
    frame: PyFrame,
}

#[pymethods]
impl PyKeplerian {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: PyTime,
        body: PyObject,
        frame: &str,
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        ascending_node: f64,
        periapsis_argument: f64,
        true_anomaly: f64,
    ) -> PyResult<Self> {
        let origin: PyBody = body.try_into()?;
        let frame = PyFrame::from_str(frame)?;
        let state = BaseKeplerian::new(
            semi_major_axis,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_argument,
            true_anomaly,
        );
        Ok(Self {
            time,
            state,
            origin,
            frame,
        })
    }

    fn time(&self) -> PyTime {
        self.time
    }

    fn reference_frame(&self) -> String {
        format!("{}", self.frame)
    }

    fn origin(&self) -> PyObject {
        self.origin.clone().into()
    }

    fn semi_major_axis(&self) -> f64 {
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

    fn periapsis_argument(&self) -> f64 {
        self.state.periapsis_argument()
    }

    fn true_anomaly(&self) -> f64 {
        self.state.true_anomaly()
    }

    fn to_cartesian(&self) -> PyCartesian {
        let mu = self.origin.gravitational_parameter();
        let state = self.state.to_cartesian_state(mu);
        PyCartesian {
            state,
            time: self.time,
            origin: self.origin.clone(),
            frame: self.frame.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use crate::bodies::PyPlanet;
    use crate::time::{PySubsecond, PyTimeScale};

    use super::*;

    #[test]
    fn test_cartesian() {
        let time = PyTime::new(
            PyTimeScale::Tdb,
            2023,
            3,
            25,
            Some(21),
            Some(8),
            Some(0),
            Some(PySubsecond::default()),
        )
        .expect("time should be valid");
        let body = Python::with_gil(|py| {
            PyPlanet::new("Earth")
                .expect("body should be valid")
                .into_py(py)
        });
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        ) * 1e-3;
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        ) * 1e-3;

        let cartesian = PyCartesian::new(
            time,
            body.clone(),
            "ICRF",
            pos.x,
            pos.y,
            pos.z,
            vel.x,
            vel.y,
            vel.z,
        )
        .expect("cartesian state should be valid");
        let cartesian1 = cartesian.to_keplerian().to_cartesian();

        let origin =
            Python::with_gil(|py| body.extract::<PyPlanet>(py)).expect("origin should be a planet");
        let origin1 = Python::with_gil(|py| cartesian1.origin().extract::<PyPlanet>(py))
            .expect("origin should be a planet");
        assert_eq!(cartesian1.time(), time);
        assert_eq!(origin1.name(), origin.name());
        assert_eq!(cartesian1.reference_frame(), "ICRF");

        assert_float_eq!(cartesian.position().0, cartesian1.position().0, rel <= 1e-8);
        assert_float_eq!(cartesian.position().1, cartesian1.position().1, rel <= 1e-8);
        assert_float_eq!(cartesian.position().2, cartesian1.position().2, rel <= 1e-8);
        assert_float_eq!(cartesian.velocity().0, cartesian1.velocity().0, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().1, cartesian1.velocity().1, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity().2, cartesian1.velocity().2, rel <= 1e-6);
    }

    #[test]
    fn test_keplerian() {
        let time = PyTime::new(
            PyTimeScale::Tdb,
            2023,
            3,
            25,
            Some(21),
            Some(8),
            Some(0),
            Some(PySubsecond::default()),
        )
        .expect("time should be valid");
        let body = Python::with_gil(|py| {
            PyPlanet::new("Earth")
                .expect("body should be valid")
                .into_py(py)
        });
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;

        let keplerian = PyKeplerian::new(
            time,
            body.clone(),
            "ICRF",
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        )
        .expect("Keplerian state should be valid");
        let keplerian1 = keplerian.to_cartesian().to_keplerian();

        let origin =
            Python::with_gil(|py| body.extract::<PyPlanet>(py)).expect("origin should be a planet");
        let origin1 = Python::with_gil(|py| keplerian1.origin().extract::<PyPlanet>(py))
            .expect("origin should be a planet");
        assert_eq!(keplerian1.time(), time);
        assert_eq!(origin1.name(), origin.name());
        assert_eq!(keplerian1.reference_frame(), "ICRF");

        assert_float_eq!(
            keplerian.semi_major_axis(),
            keplerian1.semi_major_axis(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.eccentricity(),
            keplerian1.eccentricity(),
            abs <= 1e-6
        );
        assert_float_eq!(
            keplerian.inclination(),
            keplerian1.inclination(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.ascending_node(),
            keplerian1.ascending_node(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.periapsis_argument(),
            keplerian1.periapsis_argument(),
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.true_anomaly(),
            keplerian1.true_anomaly(),
            rel <= 1e-6
        );
    }
}
