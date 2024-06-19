/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::TryFrom;

use glam::DVec3;
use numpy::PyArray1;
use pyo3::{
    exceptions::PyValueError,
    pyclass, pyfunction, pymethods,
    types::{PyAnyMethods, PyList},
    Bound, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use sgp4::Elements;

use lox_bodies::python::PyPlanet;
use lox_bodies::*;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::time_scales::PyTimeScale;
use lox_time::python::ut1::{PyNoOpOffsetProvider, PyUt1Provider};
use lox_time::time_scales::Tai;
use lox_time::transformations::TryToScale;
use lox_time::{python::time::PyTime, ut1::DeltaUt1Tai, Time};
use lox_utils::roots::Brent;
use python::PyBody;

use crate::elements::{Keplerian, ToKeplerian};
use crate::events::{Event, FindEventError, Window};
use crate::frames::{CoordinateSystem, Icrf, ReferenceFrame, Topocentric};
use crate::ground::{GroundLocation, GroundPropagator, GroundPropagatorError};
use crate::origins::CoordinateOrigin;
use crate::propagators::semi_analytical::{Vallado, ValladoError};
use crate::propagators::sgp4::{Sgp4, Sgp4Error};
use crate::propagators::Propagator;
use crate::states::ToCartesian;
use crate::{
    frames::FrameTransformationProvider,
    states::State,
    trajectories::{Trajectory, TrajectoryError},
};

mod generated;

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
                .unwrap_or(f64::NAN.to_object(py).into_bound(py))
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
                .unwrap_or(f64::NAN.to_object(py).into_bound(py))
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

#[pyclass(name = "Frame", module = "lox_space", frozen)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PyFrame {
    Icrf,
    Sun,
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    Moon,
    Phobos,
    Deimos,
    Io,
    Europa,
    Ganymede,
    Callisto,
    Amalthea,
    Himalia,
    Elara,
    Pasiphae,
    Sinope,
    Lysithea,
    Carme,
    Ananke,
    Leda,
    Thebe,
    Adrastea,
    Metis,
    Callirrhoe,
    Themisto,
    Magaclite,
    Taygete,
    Chaldene,
    Harpalyke,
    Kalyke,
    Iocaste,
    Erinome,
    Isonoe,
    Praxidike,
    Autonoe,
    Thyone,
    Hermippe,
    Aitne,
    Eurydome,
    Euanthe,
    Euporie,
    Orthosie,
    Sponde,
    Kale,
    Pasithee,
    Hegemone,
    Mneme,
    Aoede,
    Thelxinoe,
    Arche,
    Kallichore,
    Helike,
    Carpo,
    Eukelade,
    Cyllene,
    Kore,
    Herse,
    Dia,
    Mimas,
    Enceladus,
    Tethys,
    Dione,
    Rhea,
    Titan,
    Hyperion,
    Iapetus,
    Phoebe,
    Janus,
    Epimetheus,
    Helene,
    Telesto,
    Calypso,
    Atlas,
    Prometheus,
    Pandora,
    Pan,
    Ymir,
    Paaliaq,
    Tarvos,
    Ijiraq,
    Suttungr,
    Kiviuq,
    Mundilfari,
    Albiorix,
    Skathi,
    Erriapus,
    Siarnaq,
    Thrymr,
    Narvi,
    Methone,
    Pallene,
    Polydeuces,
    Daphnis,
    Aegir,
    Bebhionn,
    Bergelmir,
    Bestla,
    Farbauti,
    Fenrir,
    Fornjot,
    Hati,
    Hyrrokkin,
    Kari,
    Loge,
    Skoll,
    Surtur,
    Anthe,
    Jarnsaxa,
    Greip,
    Tarqeq,
    Aegaeon,
    Ariel,
    Umbriel,
    Titania,
    Oberon,
    Miranda,
    Cordelia,
    Ophelia,
    Bianca,
    Cressida,
    Desdemona,
    Juliet,
    Portia,
    Rosalind,
    Belinda,
    Puck,
    Caliban,
    Sycorax,
    Prospero,
    Setebos,
    Stephano,
    Trinculo,
    Francisco,
    Margaret,
    Ferdinand,
    Perdita,
    Mab,
    Cupid,
    Triton,
    Nereid,
    Naiad,
    Thalassa,
    Despina,
    Galatea,
    Larissa,
    Proteus,
    Halimede,
    Psamathe,
    Sao,
    Laomedeia,
    Neso,
    Charon,
    Nix,
    Hydra,
    Kerberos,
    Styx,
    Gaspra,
    Ida,
    Dactyl,
    Ceres,
    Pallas,
    Vesta,
    Psyche,
    Lutetia,
    Kleopatra,
    Eros,
    Davida,
    Mathilde,
    Steins,
    Braille,
    WilsonHarrington,
    Toutatis,
    Itokawa,
    Bennu,
}

#[pymethods]
impl PyFrame {
    #[new]
    fn new(name: &str) -> PyResult<Self> {
        name.parse()
    }
    fn __getnewargs__(&self) -> (String,) {
        (self.abbreviation(),)
    }

    fn name(&self) -> String {
        ReferenceFrame::name(self)
    }

    fn abbreviation(&self) -> String {
        ReferenceFrame::abbreviation(self)
    }
}

impl FrameTransformationProvider for DeltaUt1Tai {}

impl FrameTransformationProvider for PyNoOpOffsetProvider {}

impl FrameTransformationProvider for PyUt1Provider {}

#[pyclass(name = "State", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyState(pub State<PyTime, PyBody, PyFrame>);

#[pymethods]
impl PyState {
    #[new]
    fn new(
        time: PyTime,
        position: (f64, f64, f64),
        velocity: (f64, f64, f64),
        origin: Option<&Bound<'_, PyAny>>,
        frame: Option<PyFrame>,
    ) -> PyResult<Self> {
        let origin: PyBody = if let Some(origin) = origin {
            PyBody::try_from(origin)?
        } else {
            PyBody::Planet(PyPlanet::new("Earth").unwrap())
        };
        let frame = frame.unwrap_or(PyFrame::Icrf);

        Ok(PyState(State::new(
            time,
            DVec3::new(position.0, position.1, position.2),
            DVec3::new(velocity.0, velocity.1, velocity.2),
            origin,
            frame,
        )))
    }

    fn time(&self) -> PyTime {
        self.0.time().clone()
    }

    fn origin(&self) -> PyObject {
        self.0.origin().into()
    }

    fn reference_frame(&self) -> PyFrame {
        self.0.reference_frame()
    }

    fn position<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let pos = self.0.position().to_array();
        PyArray1::from_slice_bound(py, &pos)
    }

    fn velocity<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let vel = self.0.velocity().to_array();
        PyArray1::from_slice_bound(py, &vel)
    }

    fn to_frame(
        &self,
        frame: PyFrame,
        provider: Option<&Bound<'_, PyUt1Provider>>,
    ) -> PyResult<Self> {
        self.to_frame_generated(frame, provider)
    }

    fn to_keplerian(&self) -> PyResult<PyKeplerian> {
        if self.0.reference_frame() != PyFrame::Icrf {
            return Err(PyValueError::new_err(
                "only inertial frames are supported for conversion to Keplerian elements",
            ));
        }
        Ok(PyKeplerian(self.0.to_keplerian()))
    }
}

#[pyclass(name = "Keplerian", module = "lox_space", frozen)]
pub struct PyKeplerian(pub Keplerian<PyTime, PyBody>);

#[pymethods]
impl PyKeplerian {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: PyTime,
        semi_major_axis: f64,
        eccentricity: f64,
        inclination: f64,
        longitude_of_ascending_node: f64,
        argument_of_periapsis: f64,
        true_anomaly: f64,
        origin: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let origin: PyBody = origin.try_into()?;
        Ok(PyKeplerian(Keplerian::new(
            time,
            origin,
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node,
            argument_of_periapsis,
            true_anomaly,
        )))
    }

    fn time(&self) -> PyTime {
        self.0.time()
    }

    fn origin(&self) -> PyObject {
        self.0.origin().into()
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
        Ok(PyState(self.0.to_cartesian().with_frame(PyFrame::Icrf)))
    }

    fn orbital_period(&self) -> PyTimeDelta {
        PyTimeDelta(self.0.orbital_period())
    }
}

#[pyclass(name = "Trajectory", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyTrajectory(pub Trajectory<PyTime, PyBody, PyFrame>);

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
        let states: Vec<State<PyTime, PyBody, PyFrame>> = states.into_iter().map(|s| s.0).collect();
        Ok(PyTrajectory(Trajectory::new(&states)?))
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
                    .unwrap_or(f64::NAN.to_object(py).into_bound(py))
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
                    .unwrap_or(f64::NAN.to_object(py).into_bound(py))
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
}

#[pyclass(name = "Vallado", module = "lox_space", frozen)]
pub struct PyVallado(pub Vallado<PyTime, PyBody>);

impl From<ValladoError> for PyErr {
    fn from(err: ValladoError) -> Self {
        // TODO: Use better error type
        PyValueError::new_err(err.to_string())
    }
}

#[pymethods]
impl PyVallado {
    #[new]
    fn new(initial_state: PyState, max_iter: Option<i32>) -> PyResult<Self> {
        if initial_state.0.reference_frame() != PyFrame::Icrf {
            return Err(PyValueError::new_err(
                "only inertial frames are supported for the Vallado propagator",
            ));
        }
        let mut vallado = Vallado::new(initial_state.0.with_frame(Icrf));
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
            return Ok(Bound::new(
                py,
                PyState(self.0.propagate(time)?.with_frame(PyFrame::Icrf)),
            )?
            .into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            return Ok(Bound::new(
                py,
                PyTrajectory(self.0.propagate_all(steps)?.with_frame(PyFrame::Icrf)),
            )?
            .into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

#[pyclass(name = "GroundLocation", module = "lox_space", frozen)]
#[derive(Clone)]
pub struct PyGroundLocation(pub GroundLocation<PyPlanet>);

#[pymethods]
impl PyGroundLocation {
    #[new]
    fn new(planet: PyPlanet, longitude: f64, latitude: f64, altitude: f64) -> Self {
        PyGroundLocation(GroundLocation::new(longitude, latitude, altitude, planet))
    }
}

#[pyclass(name = "GroundPropagator", module = "lox_space", frozen)]
pub struct PyGroundPropagator(GroundPropagator<PyPlanet, PyUt1Provider>);

impl From<GroundPropagatorError> for PyErr {
    fn from(err: GroundPropagatorError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pymethods]
impl PyGroundPropagator {
    #[new]
    fn new(location: PyGroundLocation, provider: PyUt1Provider) -> Self {
        PyGroundPropagator(GroundPropagator::new(location.0, provider))
    }

    fn propagate<'py>(
        &self,
        py: Python<'py>,
        steps: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(time) = steps.extract::<PyTime>() {
            return Ok(Bound::new(
                py,
                PyState(
                    self.0
                        .propagate(time)?
                        .with_origin_and_frame(PyBody::Planet(self.0.origin()), PyFrame::Icrf),
                ),
            )?
            .into_any());
        }
        if let Ok(steps) = steps.extract::<Vec<PyTime>>() {
            return Ok(Bound::new(
                py,
                PyTrajectory(
                    self.0
                        .propagate_all(steps)?
                        .with_frame(PyFrame::Icrf)
                        .with_origin_and_frame(PyBody::Planet(self.0.origin()), PyFrame::Icrf),
                ),
            )?
            .into_any());
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
                    PyBody::Planet(PyPlanet::new("Earth").unwrap()),
                    PyFrame::Icrf,
                )),
            )?
            .into_any());
        }
        if let Ok(pysteps) = steps.extract::<Vec<PyTime>>() {
            let mut steps: Vec<Time<Tai>> = Vec::with_capacity(pysteps.len());
            for step in pysteps {
                let time = match provider {
                    None => step.try_to_scale(Tai, &PyNoOpOffsetProvider)?,
                    Some(provider) => step.try_to_scale(Tai, provider.get())?,
                };
                steps.push(time);
            }
            let trajectory = self
                .0
                .propagate_all(steps)?
                .with_frame(PyFrame::Icrf)
                .with_origin_and_frame(
                    PyBody::Planet(PyPlanet::new("Earth").unwrap()),
                    PyFrame::Icrf,
                );
            let states: Vec<State<PyTime, PyBody, PyFrame>> = trajectory
                .states()
                .iter()
                .map(|s| {
                    State::new(
                        PyTime(s.time().with_scale(PyTimeScale::Tai)),
                        s.position(),
                        s.velocity(),
                        s.origin(),
                        s.reference_frame(),
                    )
                })
                .collect();
            return Ok(Bound::new(py, PyTrajectory(Trajectory::new(&states)?))?.into_any());
        }
        Err(PyValueError::new_err("invalid time delta(s)"))
    }
}

#[pyfunction]
pub fn visibility(
    times: &Bound<'_, PyList>,
    location: PyGroundLocation,
    min_elevation: f64,
    gs: &Bound<'_, PyTrajectory>,
    sc: &Bound<'_, PyTrajectory>,
    provider: &Bound<'_, PyUt1Provider>,
) -> PyResult<Vec<PyWindow>> {
    let gs = gs.get();
    let sc = sc.get();
    if gs.0.reference_frame() != PyFrame::Icrf || sc.0.reference_frame() != PyFrame::Icrf {
        return Err(PyValueError::new_err(
            "only inertial frames are supported for visibility analysis",
        ));
    }
    if gs.0.origin().name() != sc.0.origin().name() {
        return Err(PyValueError::new_err(
            "ground station and spacecraft must have the same origin",
        ));
    }
    let gs_origin = match gs.0.origin() {
        PyBody::Planet(planet) => planet,
        _ => return Err(PyValueError::new_err("invalid origin")),
    };
    let sc_origin = match sc.0.origin() {
        PyBody::Planet(planet) => planet,
        _ => return Err(PyValueError::new_err("invalid origin")),
    };
    let times: Vec<PyTime> = times.extract()?;
    let frame = Topocentric::new(location.0);
    let provider = provider.get();
    let gs = gs.0.with_origin_and_frame(gs_origin, Icrf);
    let sc = sc.0.with_origin_and_frame(sc_origin, Icrf);
    Ok(
        crate::analysis::visibility(&times, &frame, min_elevation, &gs, &sc, provider)
            .into_iter()
            .map(PyWindow)
            .collect(),
    )
}

#[pyclass(name = "Topocentric", module = "lox_space", frozen)]
pub struct PyTopocentric(pub Topocentric<PyPlanet>);

#[pymethods]
impl PyTopocentric {
    #[new]
    fn new(location: PyGroundLocation) -> Self {
        PyTopocentric(Topocentric::new(location.0))
    }
}

#[pyfunction]
pub fn elevation(
    time: PyTime,
    frame: &Bound<'_, PyTopocentric>,
    gs: &Bound<'_, PyTrajectory>,
    sc: &Bound<'_, PyTrajectory>,
    provider: &Bound<'_, PyUt1Provider>,
) -> f64 {
    let gs = gs.get();
    let sc = sc.get();
    let frame = frame.get();
    // FIXME
    if gs.0.reference_frame() != PyFrame::Icrf || sc.0.reference_frame() != PyFrame::Icrf {
        return f64::NAN;
    }
    if gs.0.origin().name() != sc.0.origin().name() {
        return f64::NAN;
    }
    let gs_origin = match gs.0.origin() {
        PyBody::Planet(planet) => planet,
        _ => return f64::NAN,
    };
    let sc_origin = match sc.0.origin() {
        PyBody::Planet(planet) => planet,
        _ => return f64::NAN,
    };
    let provider = provider.get();
    let gs = gs.0.with_origin_and_frame(gs_origin, Icrf);
    let sc = sc.0.with_origin_and_frame(sc_origin, Icrf);
    crate::analysis::elevation(time, &frame.0, &gs, &sc, provider)
}
