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
    pyclass, pymethods,
    types::{PyAnyMethods, PyList},
    Bound, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};

use lox_bodies::python::PyPlanet;
use lox_bodies::*;
use lox_time::python::deltas::PyTimeDelta;
use lox_time::python::ut1::{PyNoOpOffsetProvider, PyUt1Provider};
use lox_time::{python::time::PyTime, ut1::DeltaUt1Tai};
use python::PyBody;

use crate::elements::{Keplerian, ToKeplerian};
use crate::events::{Event, Window};
use crate::frames::{CoordinateSystem, Icrf};
use crate::ground::{GroundLocation, GroundPropagator, GroundPropagatorError};
use crate::origins::CoordinateOrigin;
use crate::propagators::semi_analytical::{Vallado, ValladoError};
use crate::propagators::Propagator;
use crate::states::ToCartesian;
use crate::{
    frames::FrameTransformationProvider,
    states::State,
    trajectories::{Trajectory, TrajectoryError},
};

mod generated;

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
    fn new(planet: PyPlanet, latitude: f64, longitude: f64, altitude: f64) -> Self {
        PyGroundLocation(GroundLocation::new(latitude, longitude, altitude, planet))
    }
}

#[pyclass(name = "Ground", module = "lox_space", frozen)]
pub struct PyGround(GroundPropagator<PyPlanet, PyUt1Provider>);

impl From<GroundPropagatorError> for PyErr {
    fn from(err: GroundPropagatorError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

#[pymethods]
impl PyGround {
    #[new]
    fn new(location: PyGroundLocation, provider: PyUt1Provider) -> Self {
        PyGround(GroundPropagator::new(location.0, provider))
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
