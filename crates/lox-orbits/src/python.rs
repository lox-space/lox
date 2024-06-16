/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::TryFrom;

use glam::DVec3;
use pyo3::{
    exceptions::PyValueError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyList},
    Bound, PyAny, PyErr, PyObject, PyResult,
};

use lox_bodies::python::PyPlanet;
use lox_bodies::*;
use lox_time::python::ut1::{PyNoOpOffsetProvider, PyUt1Provider};
use lox_time::{python::time::PyTime, ut1::DeltaUt1Tai};
use python::PyBody;

use crate::elements::{Keplerian, ToKeplerian};
use crate::frames::CoordinateSystem;
use crate::origins::CoordinateOrigin;
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
            origin,
            frame,
            DVec3::new(position.0, position.1, position.2),
            DVec3::new(velocity.0, velocity.1, velocity.2),
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

    fn position(&self) -> (f64, f64, f64) {
        let pos = self.0.position();
        (pos.x, pos.y, pos.z)
    }

    fn velocity(&self) -> (f64, f64, f64) {
        let vel = self.0.velocity();
        (vel.x, vel.y, vel.z)
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
}
