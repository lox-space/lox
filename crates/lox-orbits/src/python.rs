/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rayon::prelude::*;

use glam::DVec3;
use hashbrown::HashMap;
use lox_ephem::Ephemeris;
use lox_ephem::python::PySpk;
use lox_time::DynTime;
use lox_time::ut1::DeltaUt1TaiProvider;
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::types::PyType;
use pyo3::{
    Bound, IntoPyObjectExt, PyAny, PyErr, PyResult, Python,
    exceptions::PyValueError,
    pyclass, pyfunction, pymethods,
    types::{PyAnyMethods, PyList},
};
use python::PyOrigin;
use sgp4::Elements;

use lox_bodies::*;
use lox_math::roots::Brent;
use lox_math::series::SeriesError;
use lox_time::deltas::TimeDelta;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time::PyTime;
use lox_time::python::ut1::PyUt1Provider;
use lox_time::time_scales::{DynTimeScale, Tai};

use crate::analysis::{DynPass, ElevationMask, ElevationMaskError, Pass, visibility_combined};
use crate::elements::{DynKeplerian, Keplerian};
use crate::events::{Event, FindEventError, Window};
use crate::frames::iau::IauFrameTransformationError;
use crate::frames::{DynFrame, ReferenceFrame, TryRotateTo, UnknownFrameError};
use crate::ground::{DynGroundLocation, DynGroundPropagator, GroundPropagatorError, Observables};
use crate::propagators::Propagator;
use crate::propagators::semi_analytical::{DynVallado, Vallado, ValladoError};
use crate::propagators::sgp4::{Sgp4, Sgp4Error};
use crate::states::DynState;
use crate::trajectories::{DynTrajectory, TrajectoryTransformationError};
use crate::{
    states::State,
    trajectories::{Trajectory, TrajectoryError},
};

impl From<TrajectoryTransformationError> for PyErr {
    fn from(err: TrajectoryTransformationError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.to_string())
    }
}

impl From<FindEventError> for PyErr {
    fn from(err: FindEventError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.to_string())
    }
}

impl From<IauFrameTransformationError> for PyErr {
    fn from(err: IauFrameTransformationError) -> Self {
        // FIXME: wrong error type
        PyValueError::new_err(err.to_string())
    }
}

#[pyfunction]
pub fn find_events(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    start: PyTime,
    times: Vec<f64>,
) -> PyResult<Vec<PyEvent>> {
    let root_finder = Brent::default();
    Ok(crate::events::find_events(
        |t| {
            func.call((t,), None)
                .unwrap_or(f64::NAN.into_bound_py_any(py).unwrap())
                .extract()
                .unwrap_or(f64::NAN)
        },
        start.0,
        &times,
        root_finder,
    )?
    .into_iter()
    .map(PyEvent)
    .collect())
}

#[pyfunction]
pub fn find_windows(
    py: Python<'_>,
    func: &Bound<'_, PyAny>,
    start: PyTime,
    end: PyTime,
    times: Vec<f64>,
) -> PyResult<Vec<PyWindow>> {
    let root_finder = Brent::default();
    Ok(crate::events::find_windows(
        |t| {
            func.call((t,), None)
                .unwrap_or(f64::NAN.into_bound_py_any(py).unwrap())
                .extract()
                .unwrap_or(f64::NAN)
        },
        start.0,
        end.0,
        &times,
        root_finder,
    )
    .into_iter()
    .map(PyWindow)
    .collect())
}

impl From<UnknownFrameError> for PyErr {
    fn from(err: UnknownFrameError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "Frame", module = "lox_space", frozen)]
#[pyo3(eq)]
#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct PyFrame(DynFrame);

#[pymethods]
impl PyFrame {
    #[new]
    fn new(abbreviation: &str) -> PyResult<Self> {
        Ok(Self(abbreviation.parse()?))
    }

    fn __getnewargs__(&self) -> (String,) {
        (self.abbreviation(),)
    }

    fn name(&self) -> String {
        self.0.name()
    }

    fn abbreviation(&self) -> String {
        self.0.abbreviation()
    }
}

#[pyclass(name = "State", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyState(pub DynState);

#[pymethods]
impl PyState {
    #[new]
    #[pyo3(signature = (time, position, velocity, origin=None, frame=None))]
    fn new(
        time: PyTime,
        position: (f64, f64, f64),
        velocity: (f64, f64, f64),
        origin: Option<PyOrigin>,
        frame: Option<PyFrame>,
    ) -> PyResult<Self> {
        let origin = origin.unwrap_or_default();
        let frame = frame.unwrap_or_default();

        Ok(PyState(State::new(
            time.0,
            DVec3::new(position.0, position.1, position.2),
            DVec3::new(velocity.0, velocity.1, velocity.2),
            origin.0,
            frame.0,
        )))
    }

    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    fn reference_frame(&self) -> PyFrame {
        PyFrame(self.0.reference_frame())
    }

    fn position<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let pos = self.0.position().to_array();
        PyArray1::from_slice(py, &pos)
    }

    fn velocity<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let vel = self.0.velocity().to_array();
        PyArray1::from_slice(py, &vel)
    }

    #[pyo3(signature = (frame, provider=None))]
    fn to_frame(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<Self> {
        let provider = provider.map(|p| &p.get().0);
        let rot = self
            .0
            .reference_frame()
            .try_rotation(frame.0, self.0.time(), provider)?;
        let (r1, v1) = rot.rotate_state(self.0.position(), self.0.velocity());
        Ok(PyState(State::new(
            self.0.time(),
            r1,
            v1,
            self.0.origin(),
            frame.0,
        )))
    }

    fn to_origin(&self, target: PyOrigin, ephemeris: &Bound<'_, PySpk>) -> PyResult<Self> {
        let frame = self.reference_frame();
        let s = if frame.0 != DynFrame::Icrf {
            self.to_frame(PyFrame(DynFrame::Icrf), None)?
        } else {
            self.clone()
        };
        let spk = &ephemeris.borrow().0;
        let mut s1 = Self(s.0.to_origin_dynamic(target.0, spk)?);
        if frame.0 != DynFrame::Icrf {
            s1 = s1.to_frame(frame, None)?
        }
        Ok(s1)
    }

    fn to_keplerian(&self) -> PyResult<PyKeplerian> {
        if self.0.reference_frame() != DynFrame::Icrf {
            return Err(PyValueError::new_err(
                "only inertial frames are supported for conversion to Keplerian elements",
            ));
        }
        Ok(PyKeplerian(self.0.try_to_keplerian()?))
    }

    fn rotation_lvlh<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        if self.0.reference_frame() != DynFrame::Icrf {
            return Err(PyValueError::new_err(
                "only inertial frames are supported for the LVLH rotation matrix",
            ));
        }
        let rot = self.0.try_rotation_lvlh().map_err(PyValueError::new_err)?;
        let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
        Ok(PyArray2::from_vec2(py, &rot)?)
    }

    fn to_ground_location(&self) -> PyResult<PyGroundLocation> {
        Ok(PyGroundLocation(
            self.0
                .to_dyn_ground_location()
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }
}

#[pyclass(name = "Keplerian", module = "lox_space", frozen)]
pub struct PyKeplerian(pub DynKeplerian);

#[pymethods]
impl PyKeplerian {
    #[new]
    #[pyo3(signature = (
        time,
        semi_major_axis,
        eccentricity,
        inclination,
        longitude_of_ascending_node,
        argument_of_periapsis,
        true_anomaly,
        origin=None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: PyTime,
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        longitude_of_ascending_node: f64,
        argument_of_periapsis: f64,
        true_anomaly: f64,
        origin: Option<PyOrigin>,
    ) -> PyResult<Self> {
        let origin = origin.map(|origin| origin.0).unwrap_or_default();
        Ok(PyKeplerian(
            Keplerian::with_dynamic(
                time.0,
                origin,
                semi_major_axis,
                eccentricity,
                inclination,
                longitude_of_ascending_node,
                argument_of_periapsis,
                true_anomaly,
            )
            .map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }

    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    fn semi_major_axis(&self) -> f64 {
        self.0.semi_major_axis()
    }

    fn eccentricity(&self) -> f64 {
        self.0.eccentricity()
    }

    fn inclination(&self) -> f64 {
        self.0.inclination()
    }

    fn longitude_of_ascending_node(&self) -> f64 {
        self.0.longitude_of_ascending_node()
    }

    fn argument_of_periapsis(&self) -> f64 {
        self.0.argument_of_periapsis()
    }

    fn true_anomaly(&self) -> f64 {
        self.0.true_anomaly()
    }

    fn to_cartesian(&self) -> PyResult<PyState> {
        Ok(PyState(self.0.to_cartesian()))
    }

    fn orbital_period(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.orbital_period())
    }
}

#[pyclass(name = "Trajectory", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyTrajectory(pub DynTrajectory);

impl From<TrajectoryError> for PyErr {
    fn from(err: TrajectoryError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pymethods]
impl PyTrajectory {
    #[new]
    fn new(states: &Bound<'_, PyList>) -> PyResult<Self> {
        let states: Vec<PyState> = states.extract()?;
        let states: Vec<DynState> = states.into_iter().map(|s| s.0).collect();
        Ok(PyTrajectory(Trajectory::new(&states)?))
    }

    #[classmethod]
    #[pyo3(signature = (start_time, array, origin=None, frame=None))]
    fn from_numpy(
        _cls: &Bound<'_, PyType>,
        start_time: PyTime,
        array: &Bound<'_, PyArray2<f64>>,
        origin: Option<PyOrigin>,
        frame: Option<PyFrame>,
    ) -> PyResult<Self> {
        let origin = origin.unwrap_or_default();
        let frame = frame.unwrap_or_default();
        let array = array.to_owned_array();
        if array.ncols() != 7 {
            return Err(PyValueError::new_err("invalid shape"));
        }
        let mut states: Vec<DynState> = Vec::with_capacity(array.nrows());
        for row in array.rows() {
            let time = start_time.0 + TimeDelta::try_from_decimal_seconds(row[0])?;
            let position = DVec3::new(row[1], row[2], row[3]);
            let velocity = DVec3::new(row[4], row[5], row[6]);
            states.push(State::new(time, position, velocity, origin.0, frame.0));
        }
        Ok(PyTrajectory(Trajectory::new(&states)?))
    }

    fn origin(&self) -> PyOrigin {
        PyOrigin(self.0.origin())
    }

    fn reference_frame(&self) -> PyFrame {
        PyFrame(self.0.reference_frame())
    }

    fn to_numpy<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        Ok(PyArray2::from_vec2(py, &self.0.to_vec())?)
    }

    fn states(&self) -> Vec<PyState> {
        self.0.states().into_iter().map(PyState).collect()
    }

    fn find_events(&self, py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<Vec<PyEvent>> {
        Ok(self
            .0
            .find_events(|s| {
                func.call((PyState(s),), None)
                    // FIXME: Bad idea
                    .unwrap_or(f64::NAN.into_bound_py_any(py).unwrap())
                    .extract()
                    .unwrap_or(f64::NAN)
            })
            .into_iter()
            .map(PyEvent)
            .collect())
    }

    fn find_windows(&self, py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<Vec<PyWindow>> {
        Ok(self
            .0
            .find_windows(|s| {
                func.call((PyState(s),), None)
                    // FIXME: Bad idea
                    .unwrap_or(f64::NAN.into_bound_py_any(py).unwrap())
                    .extract()
                    .unwrap_or(f64::NAN)
            })
            .into_iter()
            .map(PyWindow)
            .collect())
    }

    fn interpolate(&self, time: &Bound<'_, PyAny>) -> PyResult<PyState> {
        if let Ok(delta) = time.extract::<PyTimeDelta>() {
            return Ok(PyState(self.0.interpolate(delta.0)));
        }
        if let Ok(time) = time.extract::<PyTime>() {
            return Ok(PyState(self.0.interpolate_at(time.0)));
        }
        Err(PyValueError::new_err("invalid time argument"))
    }

    #[pyo3(signature = (frame, provider=None))]
    fn to_frame(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<Self> {
        let mut states: Vec<DynState> = Vec::with_capacity(self.0.states().len());
        for s in self.0.states() {
            states.push(PyState(s).to_frame(frame.clone(), provider)?.0);
        }
        Ok(PyTrajectory(Trajectory::new(&states)?))
    }

    fn to_origin(&self, target: PyOrigin, ephemeris: &Bound<'_, PySpk>) -> PyResult<Self> {
        let mut states: Vec<PyState> = Vec::with_capacity(self.states().len());
        for s in self.states() {
            states.push(s.to_origin(target.clone(), ephemeris)?)
        }
        let states: Vec<DynState> = states.into_iter().map(|s| s.0).collect();
        Ok(Self(Trajectory::new(&states)?))
    }
}

#[pyclass(name = "Event", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyEvent(pub Event<DynTimeScale>);

#[pymethods]
impl PyEvent {
    fn __repr__(&self) -> String {
        format!("Event({}, {})", self.time().__str__(), self.crossing())
    }

    fn __str__(&self) -> String {
        format!(
            "Event - {}crossing at {}",
            self.crossing(),
            self.time().__str__()
        )
    }
    fn time(&self) -> PyTime {
        PyTime(self.0.time())
    }

    fn crossing(&self) -> String {
        self.0.crossing().to_string()
    }
}

#[pyclass(name = "Window", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyWindow(pub Window<DynTimeScale>);

#[pymethods]
impl PyWindow {
    fn __repr__(&self) -> String {
        format!(
            "Window({}, {})",
            self.start().__str__(),
            self.end().__str__()
        )
    }

    fn start(&self) -> PyTime {
        PyTime(self.0.start())
    }

    fn end(&self) -> PyTime {
        PyTime(self.0.end())
    }

    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.duration())
    }
}

#[pyclass(name = "Vallado", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyVallado(pub DynVallado);

impl From<ValladoError> for PyErr {
    fn from(err: ValladoError) -> Self {
        // TODO: Use better error type
        PyValueError::new_err(err.to_string())
    }
}

#[pymethods]
impl PyVallado {
    #[new]
    #[pyo3(signature =(initial_state, max_iter=None))]
    fn new(initial_state: PyState, max_iter: Option<i32>) -> PyResult<Self> {
        let mut vallado = Vallado::with_dynamic(initial_state.0).map_err(|_| {
            PyValueError::new_err("only inertial frames are supported for the Vallado propagator")
        })?;
        if let Some(max_iter) = max_iter {
            vallado.with_max_iter(max_iter);
        }
        Ok(PyVallado(vallado))
    }

    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(time) = steps.extract::<PyTime>() {
            return Ok(Bound::new(py, PyState(self.0.propagate(time.0)?))?.into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            let steps = steps.into_iter().map(|s| s.0);
            return Ok(Bound::new(py, PyTrajectory(self.0.propagate_all(steps)?))?.into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

#[pyclass(name = "GroundLocation", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyGroundLocation(pub DynGroundLocation);

#[pymethods]
impl PyGroundLocation {
    #[new]
    fn new(origin: PyOrigin, longitude: f64, latitude: f64, altitude: f64) -> PyResult<Self> {
        Ok(PyGroundLocation(
            DynGroundLocation::with_dynamic(longitude, latitude, altitude, origin.0)
                .map_err(PyValueError::new_err)?,
        ))
    }

    #[pyo3(signature = (state, provider=None, frame=None))]
    fn observables(
        &self,
        state: PyState,
        provider: Option<&Bound<'_, PyUt1Provider>>,
        frame: Option<PyFrame>,
    ) -> PyResult<PyObservables> {
        let frame = frame.unwrap_or(PyFrame(DynFrame::Iau(state.0.origin())));
        let state = state.to_frame(frame, provider)?;
        let rot = self.0.rotation_to_topocentric();
        let position = rot * (state.0.position() - self.0.body_fixed_position());
        let velocity = rot * state.0.velocity();
        let range = position.length();
        let range_rate = position.dot(velocity) / range;
        let elevation = (position.z / range).asin();
        let azimuth = position.y.atan2(-position.x);
        Ok(PyObservables(Observables::new(
            azimuth, elevation, range, range_rate,
        )))
    }

    fn rotation_to_topocentric<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let rot = self.0.rotation_to_topocentric();
        let rot: Vec<Vec<f64>> = rot.to_cols_array_2d().iter().map(|v| v.to_vec()).collect();
        Ok(PyArray2::from_vec2(py, &rot)?)
    }

    fn longitude(&self) -> f64 {
        self.0.longitude()
    }

    fn latitude(&self) -> f64 {
        self.0.latitude()
    }

    fn altitude(&self) -> f64 {
        self.0.altitude()
    }
}

#[pyclass(name = "GroundPropagator", module = "lox_space", frozen)]
pub struct PyGroundPropagator(DynGroundPropagator<PyUt1Provider>);

impl From<GroundPropagatorError> for PyErr {
    fn from(err: GroundPropagatorError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pymethods]
impl PyGroundPropagator {
    #[new]
    #[pyo3(signature = (location, provider=None))]
    fn new(location: PyGroundLocation, provider: Option<PyUt1Provider>) -> Self {
        PyGroundPropagator(DynGroundPropagator::with_dynamic(location.0, provider))
    }

    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(time) = steps.extract::<PyTime>() {
            return Ok(Bound::new(py, PyState(self.0.propagate_dyn(time.0)?))?.into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            let steps = steps.into_iter().map(|s| s.0);
            return Ok(Bound::new(py, PyTrajectory(self.0.propagate_all_dyn(steps)?))?.into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

impl From<Sgp4Error> for PyErr {
    fn from(err: Sgp4Error) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "SGP4", module = "lox_space", frozen)]
pub struct PySgp4(pub Sgp4);

#[pymethods]
impl PySgp4 {
    #[new]
    fn new(tle: &str) -> PyResult<Self> {
        let lines: Vec<&str> = tle.trim().split('\n').collect();
        let elements = if lines.len() == 3 {
            Elements::from_tle(
                Some(lines[0].to_string()),
                lines[1].as_bytes(),
                lines[2].as_bytes(),
            )
            .map_err(|err| PyValueError::new_err(err.to_string()))?
        } else if lines.len() == 2 {
            Elements::from_tle(None, lines[0].as_bytes(), lines[1].as_bytes())
                .map_err(|err| PyValueError::new_err(err.to_string()))?
        } else {
            return Err(PyValueError::new_err("invalid TLE"));
        };
        Ok(PySgp4(
            Sgp4::new(elements).map_err(|err| PyValueError::new_err(err.to_string()))?,
        ))
    }

    fn time(&self) -> PyTime {
        PyTime(
            self.0
                .time()
                .try_to_scale(DynTimeScale::Tai, None::<&PyUt1Provider>)
                .unwrap(),
        )
    }

    #[pyo3(signature = (steps, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let provider = provider.map(|p| &p.get().0);
        if let Ok(pytime) = steps.extract::<PyTime>() {
            let time = pytime.0.try_to_scale(Tai, provider)?;
            let s1 = self.0.propagate(time)?;
            let time = time.try_to_scale(DynTimeScale::Tai, provider)?;
            return Ok(Bound::new(
                py,
                PyState(State::new(
                    time,
                    s1.position(),
                    s1.velocity(),
                    DynOrigin::default(),
                    DynFrame::default(),
                )),
            )?
            .into_any());
        }
        if let Ok(pysteps) = steps.extract::<Vec<PyTime>>() {
            let mut states: Vec<DynState> = Vec::with_capacity(pysteps.len());
            for step in pysteps {
                let time = step.0.try_to_scale(Tai, provider)?;
                let s = self.0.propagate(time)?;
                let time = time.try_to_scale(DynTimeScale::Tai, provider)?;
                let s = State::new(
                    time,
                    s.position(),
                    s.velocity(),
                    DynOrigin::default(),
                    DynFrame::default(),
                );
                states.push(s);
            }
            return Ok(Bound::new(py, PyTrajectory(Trajectory::new(&states)?))?.into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

#[pyfunction]
#[pyo3(signature = (times, gs, mask, sc, ephemeris, bodies=None, provider=None))]
pub fn visibility(
    times: &Bound<'_, PyList>,
    gs: PyGroundLocation,
    mask: &Bound<'_, PyElevationMask>,
    sc: &Bound<'_, PyTrajectory>,
    ephemeris: &Bound<'_, PySpk>,
    bodies: Option<Vec<PyOrigin>>,
    provider: Option<&Bound<'_, PyUt1Provider>>,
) -> PyResult<Vec<PyPass>> {
    let sc = sc.get();
    if gs.0.origin().name() != sc.0.origin().name() {
        return Err(PyValueError::new_err(
            "ground station and spacecraft must have the same origin",
        ));
    }
    let times: Vec<DynTime> = times
        .extract::<Vec<PyTime>>()?
        .into_iter()
        .map(|s| s.0)
        .collect();
    let provider = provider.map(|p| &p.get().0);
    let mask = &mask.borrow().0;
    let ephemeris = &ephemeris.get().0;
    let bodies: Vec<DynOrigin> = bodies
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.0)
        .collect();
    Ok(crate::analysis::visibility_combined(
        &times, &gs.0, mask, &bodies, &sc.0, ephemeris, provider,
    )?
    .into_iter()
    .map(PyPass)
    .collect())
}

#[pyclass(name = "Ensemble", module = "lox_space", frozen)]
pub struct PyEnsemble(pub HashMap<String, DynTrajectory>);

#[pymethods]
impl PyEnsemble {
    #[new]
    pub fn new(ensemble: HashMap<String, PyTrajectory>) -> Self {
        Self(
            ensemble
                .into_iter()
                .map(|(name, trajectory)| (name, trajectory.0))
                .collect(),
        )
    }
}

#[pyfunction]
#[pyo3(signature = (
    times,
    ground_stations,
    spacecraft,
    ephemeris,
    bodies=None,
    provider=None,
))]
pub fn visibility_all(
    _py: Python<'_>,
    times: &Bound<'_, PyList>,
    ground_stations: HashMap<String, (PyGroundLocation, PyElevationMask)>,
    spacecraft: &Bound<'_, PyEnsemble>,
    ephemeris: &Bound<'_, PySpk>,
    bodies: Option<Vec<PyOrigin>>,
    provider: Option<&Bound<'_, PyUt1Provider>>,
) -> PyResult<HashMap<String, HashMap<String, Vec<PyPass>>>> {
    let times: Vec<DynTime> = times
        .extract::<Vec<PyTime>>()?
        .into_iter()
        .map(|s| s.0)
        .collect();
    let bodies: Vec<DynOrigin> = bodies
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.0)
        .collect();
    let provider = provider.map(|p| &p.get().0);
    let spacecraft = &spacecraft.get().0;
    let ephemeris = &ephemeris.get().0;

    let _total_combinations = spacecraft.len() * ground_stations.len();

    // Adaptive strategy based on workload size
    if should_use_parallel(spacecraft.len(), ground_stations.len()) {
        visibility_all_parallel_optimized(
            _py,
            &times,
            &ground_stations,
            spacecraft,
            ephemeris,
            &bodies,
            provider,
        )
    } else {
        visibility_all_sequential_optimized(
            _py,
            &times,
            &ground_stations,
            spacecraft,
            ephemeris,
            &bodies,
            provider,
        )
    }
}

/// Determine if parallel processing should be used based on workload characteristics
fn should_use_parallel(spacecraft_count: usize, ground_station_count: usize) -> bool {
    let total_combinations = spacecraft_count * ground_station_count;

    // Use parallel processing if:
    // 1. We have enough work to justify overhead (>100 combinations)
    // 2. AND either enough spacecraft (>10) OR enough ground stations (>8)
    total_combinations > 100 && (spacecraft_count > 10 || ground_station_count > 8)
}

/// Sequential implementation optimized for small workloads
fn visibility_all_sequential_optimized<P, E>(
    _py: Python<'_>,
    times: &[DynTime],
    ground_stations: &HashMap<String, (PyGroundLocation, PyElevationMask)>,
    spacecraft: &HashMap<String, DynTrajectory>,
    ephemeris: &E,
    bodies: &[DynOrigin],
    provider: Option<&P>,
) -> PyResult<HashMap<String, HashMap<String, Vec<PyPass>>>>
where
    P: DeltaUt1TaiProvider + Clone + Send + Sync,
    E: Ephemeris + Send + Sync,
{
    // Pre-allocate result with known capacity
    let mut result = HashMap::with_capacity(spacecraft.len());

    for (sc_name, sc_trajectory) in spacecraft {
        let mut gs_results = HashMap::with_capacity(ground_stations.len());

        for (gs_name, (gs_location, gs_mask)) in ground_stations {
            let passes = visibility_combined(
                times,
                &gs_location.0,
                &gs_mask.0,
                bodies,
                sc_trajectory,
                ephemeris,
                provider,
            )?;

            gs_results.insert(gs_name.clone(), passes.into_iter().map(PyPass).collect());
        }

        result.insert(sc_name.clone(), gs_results);
    }

    Ok(result)
}

/// Parallel implementation optimized for large workloads
fn visibility_all_parallel_optimized<P, E>(
    py: Python<'_>,
    times: &[DynTime],
    ground_stations: &HashMap<String, (PyGroundLocation, PyElevationMask)>,
    spacecraft: &HashMap<String, DynTrajectory>,
    ephemeris: &E,
    bodies: &[DynOrigin],
    provider: Option<&P>,
) -> PyResult<HashMap<String, HashMap<String, Vec<PyPass>>>>
where
    P: DeltaUt1TaiProvider + Clone + Send + Sync,
    E: Ephemeris + Send + Sync,
{
    // Create all combinations upfront for better work distribution
    let combinations: Vec<_> = spacecraft
        .iter()
        .flat_map(|(sc_name, sc_trajectory)| {
            ground_stations
                .iter()
                .map(move |(gs_name, (gs_location, gs_mask))| {
                    (sc_name, sc_trajectory, gs_name, gs_location, gs_mask)
                })
        })
        .collect();

    // Determine optimal chunk size based on number of combinations and available cores
    let num_threads = rayon::current_num_threads();
    let chunk_size = (combinations.len() / (num_threads * 2)).clamp(1, 50);

    // Process combinations in chunks with periodic GIL release
    let results: Result<Vec<_>, _> = py.allow_threads(|| {
        combinations
            .par_chunks(chunk_size)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|(sc_name, sc_trajectory, gs_name, gs_location, gs_mask)| {
                        let passes = visibility_combined(
                            times,
                            &gs_location.0,
                            &gs_mask.0,
                            bodies,
                            sc_trajectory,
                            ephemeris,
                            provider,
                        )?;

                        let py_passes = passes.into_iter().map(PyPass).collect();
                        Ok(((*sc_name).clone(), (*gs_name).clone(), py_passes))
                    })
                    .collect::<Result<Vec<_>, SeriesError>>()
            })
            .collect()
    });

    // Convert the flat results to nested hashmap structure
    let flat_results: Vec<_> = results?.into_iter().flatten().collect();
    let mut final_result: HashMap<String, HashMap<String, Vec<PyPass>>> = HashMap::new();

    for (sc_name, gs_name, passes) in flat_results {
        final_result
            .entry(sc_name)
            .or_insert_with(HashMap::new)
            .insert(gs_name, passes);
    }

    Ok(final_result)
}

impl From<ElevationMaskError> for PyErr {
    fn from(err: ElevationMaskError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pyclass(name = "ElevationMask", module = "lox_space", frozen, eq)]
#[derive(Debug, Clone, PartialEq)]
pub struct PyElevationMask(pub ElevationMask);

#[pymethods]
impl PyElevationMask {
    #[new]
    #[pyo3(signature = (azimuth=None, elevation=None, min_elevation=None))]
    fn new(
        azimuth: Option<&Bound<'_, PyArray1<f64>>>,
        elevation: Option<&Bound<'_, PyArray1<f64>>>,
        min_elevation: Option<f64>,
    ) -> PyResult<Self> {
        if let Some(min_elevation) = min_elevation {
            return Ok(PyElevationMask(ElevationMask::with_fixed_elevation(
                min_elevation,
            )));
        }
        if let (Some(azimuth), Some(elevation)) = (azimuth, elevation) {
            let azimuth = azimuth.to_vec()?;
            let elevation = elevation.to_vec()?;
            return Ok(PyElevationMask(ElevationMask::new(azimuth, elevation)?));
        }
        Err(PyValueError::new_err(
            "invalid argument combination, either `min_elevation` or `azimuth` and `elevation` arrays need to be present",
        ))
    }

    #[classmethod]
    fn fixed(_cls: &Bound<'_, PyType>, min_elevation: f64) -> Self {
        PyElevationMask(ElevationMask::with_fixed_elevation(min_elevation))
    }

    #[classmethod]
    fn variable(
        _cls: &Bound<'_, PyType>,
        azimuth: &Bound<'_, PyArray1<f64>>,
        elevation: &Bound<'_, PyArray1<f64>>,
    ) -> PyResult<Self> {
        let azimuth = azimuth.to_vec()?;
        let elevation = elevation.to_vec()?;
        Ok(PyElevationMask(ElevationMask::new(azimuth, elevation)?))
    }

    fn __getnewargs__(&self) -> (Option<Vec<f64>>, Option<Vec<f64>>, Option<f64>) {
        (self.azimuth(), self.elevation(), self.fixed_elevation())
    }

    fn azimuth(&self) -> Option<Vec<f64>> {
        match &self.0 {
            ElevationMask::Fixed(_) => None,
            ElevationMask::Variable(series) => Some(series.x().to_vec()),
        }
    }

    fn elevation(&self) -> Option<Vec<f64>> {
        match &self.0 {
            ElevationMask::Fixed(_) => None,
            ElevationMask::Variable(series) => Some(series.y().to_vec()),
        }
    }

    fn fixed_elevation(&self) -> Option<f64> {
        match &self.0 {
            ElevationMask::Fixed(min_elevation) => Some(*min_elevation),
            ElevationMask::Variable(_) => None,
        }
    }

    fn min_elevation(&self, azimuth: f64) -> f64 {
        self.0.min_elevation(azimuth)
    }
}

#[pyclass(name = "Observables", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyObservables(pub Observables);

#[pymethods]
impl PyObservables {
    #[new]
    fn new(azimuth: f64, elevation: f64, range: f64, range_rate: f64) -> Self {
        PyObservables(Observables::new(azimuth, elevation, range, range_rate))
    }

    fn azimuth(&self) -> f64 {
        self.0.azimuth()
    }

    fn elevation(&self) -> f64 {
        self.0.elevation()
    }

    fn range(&self) -> f64 {
        self.0.range()
    }

    fn range_rate(&self) -> f64 {
        self.0.range_rate()
    }
}

#[pyclass(name = "Pass", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyPass(pub DynPass);

#[pymethods]
impl PyPass {
    #[new]
    fn new(
        window: PyWindow,
        times: Vec<PyTime>,
        observables: Vec<PyObservables>,
    ) -> PyResult<Self> {
        let times: Vec<DynTime> = times.into_iter().map(|t| t.0).collect();
        let observables: Vec<Observables> = observables.into_iter().map(|o| o.0).collect();

        let pass = Pass::new(window.0, times, observables)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyPass(pass))
    }

    fn window(&self) -> PyWindow {
        PyWindow(*self.0.window())
    }

    fn times(&self) -> Vec<PyTime> {
        self.0.times().iter().map(|&t| PyTime(t)).collect()
    }

    fn observables(&self) -> Vec<PyObservables> {
        self.0
            .observables()
            .iter()
            .map(|o| PyObservables(o.clone()))
            .collect()
    }

    fn interpolate(&self, time: PyTime) -> Option<PyObservables> {
        self.0.interpolate(time.0).map(PyObservables)
    }

    fn __repr__(&self) -> String {
        let window = self.0.window();
        format!(
            "Pass(window=Window({}, {}), {} observables)",
            window.start(),
            window.end(),
            self.0.observables().len()
        )
    }
}
