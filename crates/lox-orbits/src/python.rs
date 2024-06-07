/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    frames::FrameTransformationProvider,
    states::State,
    trajectories::{Trajectory, TrajectoryError},
};
use glam::DVec3;
use lox_bodies::*;
use lox_time::{python::time::PyTime, ut1::DeltaUt1Tai};
use pyo3::{
    exceptions::PyValueError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyList},
    Bound, PyAny, PyErr, PyResult,
};
use python::{PyBody, PyPlanet};

mod generated;

#[pyclass]
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

impl FrameTransformationProvider for DeltaUt1Tai {}

#[pyclass(name = "State", module = "lox_space")]
#[derive(Debug, Clone)]
pub struct PyState(pub State<PyTime, PyBody, PyFrame>);

#[pymethods]
impl PyState {
    #[new]
    #[pyo3(signature = (time, position, velocity, origin = "Earth", frame = "ICRF"))]
    fn new(
        time: PyTime,
        position: (f64, f64, f64),
        velocity: (f64, f64, f64),
        origin: &str,
        frame: &str,
    ) -> PyResult<Self> {
        let origin: PyBody = origin.parse()?;
        let frame: PyFrame = frame.parse()?;

        Ok(PyState(State::new(
            time,
            origin,
            frame,
            DVec3::new(position.0, position.1, position.2),
            DVec3::new(velocity.0, velocity.1, velocity.2),
        )))
    }
}

#[pyclass(name = "Trajectory", module = "lox_space")]
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
