/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use glam::DVec3;
use lox_ephem::python::PySpk;
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::types::PyType;
use pyo3::{
    exceptions::PyValueError,
    pyclass, pyfunction, pymethods,
    types::{PyAnyMethods, PyList},
    Bound, IntoPyObjectExt, PyAny, PyErr, PyResult, Python,
};
use python::PyOrigin;
use sgp4::Elements;

use lox_bodies::*;
use lox_math::roots::Brent;
use lox_time::deltas::TimeDelta;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time_scales::PyTimeScale;
use lox_time::python::ut1::{PyNoOpOffsetProvider, PyUt1Provider};
use lox_time::time_scales::Tai;
use lox_time::transformations::TryToScale;
use lox_time::{python::time::PyTime, ut1::DeltaUt1Tai};

use crate::analysis::{ElevationMask, ElevationMaskError};
use crate::elements::{DynKeplerian, Keplerian};
use crate::events::{Event, FindEventError, Window};
use crate::frames::{CoordinateSystem, DynFrame, ReferenceFrame, TryRotateTo, UnknownFrameError};
use crate::ground::{DynGroundLocation, DynGroundPropagator, GroundPropagatorError, Observables};
use crate::propagators::semi_analytical::{Vallado, ValladoError};
use crate::propagators::sgp4::{Sgp4, Sgp4Error};
use crate::propagators::Propagator;
use crate::states::DynState;
use crate::trajectories::TrajectoryTransformationError;
use crate::{
    frames::FrameTransformationProvider,
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
        start,
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
        start,
        end,
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

impl FrameTransformationProvider for DeltaUt1Tai {}

impl FrameTransformationProvider for PyNoOpOffsetProvider {}

impl FrameTransformationProvider for PyUt1Provider {}

#[pyclass(name = "State", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyState(pub State<PyTime, DynOrigin, DynFrame>);

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
            time,
            DVec3::new(position.0, position.1, position.2),
            DVec3::new(velocity.0, velocity.1, velocity.2),
            origin.0,
            frame.0,
        )))
    }

    fn time(&self) -> PyTime {
        self.0.time().clone()
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
        let rot = match provider {
            Some(provider) => {
                self.0
                    .reference_frame()
                    .try_rotation(&frame.0, self.0.time(), provider.get())
            }
            None => self.0.reference_frame().try_rotation(
                &frame.0,
                self.0.time(),
                &PyNoOpOffsetProvider,
            ),
        }
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let (r1, v1) = rot.rotate_state(self.0.position(), self.0.velocity());
        Ok(PyState(State::new(
            self.time(),
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
pub struct PyKeplerian(pub DynKeplerian<PyTime>);

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
                time,
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
        self.0.time()
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
pub struct PyTrajectory(pub Trajectory<PyTime, DynOrigin, DynFrame>);

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
        let states: Vec<DynState<PyTime>> = states.into_iter().map(|s| s.0).collect();
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
        let mut states: Vec<DynState<PyTime>> = Vec::with_capacity(array.nrows());
        for row in array.rows() {
            let time = PyTime(start_time.0 + TimeDelta::from_decimal_seconds(row[0])?);
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
            return Ok(PyState(self.0.interpolate_at(time)));
        }
        Err(PyValueError::new_err("invalid time argument"))
    }

    #[pyo3(signature = (frame, provider=None))]
    fn to_frame(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<Self> {
        let mut states: Vec<DynState<PyTime>> = Vec::with_capacity(self.0.states().len());
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
        let states: Vec<DynState<PyTime>> = states.into_iter().map(|s| s.0).collect();
        Ok(Self(Trajectory::new(&states)?))
    }
}

#[pyclass(name = "Event", module = "lox_space", frozen)]
pub struct PyEvent(pub Event<PyTime>);

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
        self.0.time().clone()
    }

    fn crossing(&self) -> String {
        self.0.crossing().to_string()
    }
}

#[pyclass(name = "Window", module = "lox_space", frozen)]
pub struct PyWindow(pub Window<PyTime>);

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
        self.0.start().clone()
    }

    fn end(&self) -> PyTime {
        self.0.end().clone()
    }

    fn duration(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.duration())
    }
}

#[pyclass(name = "Vallado", module = "lox_space", frozen)]
pub struct PyVallado(pub Vallado<PyTime, DynOrigin, DynFrame>);

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
            return Ok(Bound::new(py, PyState(self.0.propagate(time)?))?.into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            return Ok(Bound::new(py, PyTrajectory(self.0.propagate_all(steps)?))?.into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

#[pyclass(name = "GroundLocation", module = "lox_space", frozen)]
#[derive(Clone)]
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
        let frame = frame.unwrap_or(PyFrame(DynFrame::BodyFixed(state.0.origin())));
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
    fn new(location: PyGroundLocation, provider: PyUt1Provider) -> Self {
        PyGroundPropagator(DynGroundPropagator::with_dynamic(location.0, provider))
    }

    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(time) = steps.extract::<PyTime>() {
            return Ok(Bound::new(py, PyState(self.0.propagate_dyn(time)?))?.into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
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
        PyTime(self.0.time().with_scale(PyTimeScale::Tai))
    }

    #[pyo3(signature = (steps, provider=None))]
    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(pytime) = steps.extract::<PyTime>() {
            let time = match provider {
                None => pytime.try_to_scale(Tai, &PyNoOpOffsetProvider)?,
                Some(provider) => pytime.try_to_scale(Tai, provider.get())?,
            };
            let s1 = self.0.propagate(time)?;
            return Ok(Bound::new(
                py,
                PyState(State::new(
                    pytime,
                    s1.position(),
                    s1.velocity(),
                    DynOrigin::default(),
                    DynFrame::default(),
                )),
            )?
            .into_any());
        }
        if let Ok(pysteps) = steps.extract::<Vec<PyTime>>() {
            let mut states: Vec<DynState<PyTime>> = Vec::with_capacity(pysteps.len());
            for step in pysteps {
                let time = match provider {
                    None => step.try_to_scale(Tai, &PyNoOpOffsetProvider)?,
                    Some(provider) => step.try_to_scale(Tai, provider.get())?,
                };
                let s = self.0.propagate(time)?;
                let s = State::new(
                    step,
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
pub fn visibility(
    times: &Bound<'_, PyList>,
    gs: PyGroundLocation,
    mask: &Bound<'_, PyElevationMask>,
    sc: &Bound<'_, PyTrajectory>,
    provider: &Bound<'_, PyUt1Provider>,
) -> PyResult<Vec<PyWindow>> {
    let sc = sc.get();
    if gs.0.origin().name() != sc.0.origin().name() {
        return Err(PyValueError::new_err(
            "ground station and spacecraft must have the same origin",
        ));
    }
    let times: Vec<PyTime> = times.extract()?;
    let provider = provider.get();
    let mask = &mask.borrow().0;
    Ok(
        crate::analysis::visibility_dyn(&times, &gs.0, mask, &sc.0, provider)
            .into_iter()
            .map(PyWindow)
            .collect(),
    )
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
        Err(PyValueError::new_err("invalid argument combination, either `min_elevation` or `azimuth` and `elevation` arrays need to be present"))
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
        (self.azimuth(), self.elevation(), self.min_elevation())
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

    fn min_elevation(&self) -> Option<f64> {
        match &self.0 {
            ElevationMask::Fixed(min_elevation) => Some(*min_elevation),
            ElevationMask::Variable(_) => None,
        }
    }
}

#[pyclass(name = "Observables", module = "lox_space", frozen)]
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
