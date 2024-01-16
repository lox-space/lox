/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::prelude::*;

use lox_core::bodies::*;

use crate::LoxPyError;

#[pyclass(name = "Sun")]
pub struct PySun;

#[pymethods]
impl PySun {
    #[new]
    fn new() -> Self {
        Self
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok("Sun()".to_string())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Sun".to_string())
    }

    fn id(&self) -> i32 {
        Sun.id().0
    }

    fn name(&self) -> &'static str {
        Sun.name()
    }

    fn gravitational_parameter(&self) -> f64 {
        Sun.gravitational_parameter()
    }

    fn mean_radius(&self) -> f64 {
        Sun.mean_radius()
    }

    fn polar_radius(&self) -> f64 {
        Sun.polar_radius()
    }

    fn equatorial_radius(&self) -> f64 {
        Sun.equatorial_radius()
    }
}

#[pyclass(name = "Barycenter")]
pub struct PyBarycenter(Box<dyn PointMass + Send>);

#[pymethods]
impl PyBarycenter {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        let barycenter: Option<Box<dyn PointMass + Send>> = match name {
            "ssb" | "SSB" | "solar system barycenter" | "Solar System Barycenter" => {
                Some(Box::new(SolarSystemBarycenter))
            }
            "mercury barycenter" | "Mercury Barycenter" => Some(Box::new(MercuryBarycenter)),
            "venus barycenter" | "Venus Barycenter" => Some(Box::new(VenusBarycenter)),
            "earth barycenter" | "Earth Barycenter" => Some(Box::new(EarthBarycenter)),
            "mars barycenter" | "Mars Barycenter" => Some(Box::new(MarsBarycenter)),
            "jupiter barycenter" | "Jupiter Barycenter" => Some(Box::new(JupiterBarycenter)),
            "saturn barycenter" | "Saturn Barycenter" => Some(Box::new(SaturnBarycenter)),
            "uranus barycenter" | "Uranus Barycenter" => Some(Box::new(UranusBarycenter)),
            "neptune barycenter" | "Neptune Barycenter" => Some(Box::new(NeptuneBarycenter)),
            "pluto barycenter" | "Pluto Barycenter" => Some(Box::new(PlutoBarycenter)),
            _ => None,
        };
        match barycenter {
            Some(barycenter) => Ok(Self(barycenter)),
            None => Err(LoxPyError::InvalidBody(name.to_string())),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Barycenter(\"{}\")", self.name()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.name().to_string())
    }

    fn id(&self) -> i32 {
        self.0.id().0
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }
}

#[pyclass(name = "Planet")]
pub struct PyPlanet(Box<dyn Planet + Send>);

#[pymethods]
impl PyPlanet {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        let planet: Option<Box<dyn Planet + Send>> = match name {
            "mercury" | "Mercury" => Some(Box::new(Mercury)),
            "venus" | "Venus" => Some(Box::new(Venus)),
            "earth" | "Earth" => Some(Box::new(Earth)),
            "mars" | "Mars" => Some(Box::new(Mars)),
            "jupiter" | "Jupiter" => Some(Box::new(Jupiter)),
            "saturn" | "Saturn" => Some(Box::new(Saturn)),
            "uranus" | "Uranus" => Some(Box::new(Uranus)),
            "neptune" | "Neptune" => Some(Box::new(Neptune)),
            "pluto" | "Pluto" => Some(Box::new(Pluto)),
            _ => None,
        };
        match planet {
            Some(planet) => Ok(Self(planet)),
            None => Err(LoxPyError::InvalidBody(name.to_string())),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Planet(\"{}\")", self.name()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.name().to_string())
    }

    fn id(&self) -> i32 {
        self.0.id().0
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }

    fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }

    fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    fn equatorial_radius(&self) -> f64 {
        self.0.equatorial_radius()
    }
}

#[pyclass(name = "Satellite")]
pub struct PySatellite(Box<dyn Satellite + Send>);

#[pymethods]
impl PySatellite {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        let satellite: Option<Box<dyn Satellite + Send>> = match name {
            "moon" | "Moon" | "luna" | "Luna" => Some(Box::new(Moon)),
            "phobos" | "Phobos" => Some(Box::new(Phobos)),
            "deimos" | "Deimos" => Some(Box::new(Deimos)),
            "io" | "Io" => Some(Box::new(Io)),
            "europa" | "Europa" => Some(Box::new(Europa)),
            "ganymede" | "Ganymede" => Some(Box::new(Ganymede)),
            "callisto" | "Callisto" => Some(Box::new(Callisto)),
            "amalthea" | "Amalthea" => Some(Box::new(Amalthea)),
            "himalia" | "Himalia" => Some(Box::new(Himalia)),
            "thebe" | "Thebe" => Some(Box::new(Thebe)),
            "adrastea" | "Adrastea" => Some(Box::new(Adrastea)),
            "metis" | "Metis" => Some(Box::new(Metis)),
            "mimas" | "Mimas" => Some(Box::new(Mimas)),
            "enceladus" | "Enceladus" => Some(Box::new(Enceladus)),
            "tethys" | "Tethys" => Some(Box::new(Tethys)),
            "dione" | "Dione" => Some(Box::new(Dione)),
            "rhea" | "Rhea" => Some(Box::new(Rhea)),
            "titan" | "Titan" => Some(Box::new(Titan)),
            "hyperion" | "Hyperion" => Some(Box::new(Hyperion)),
            "iapetus" | "Iapetus" => Some(Box::new(Iapetus)),
            "phoebe" | "Phoebe" => Some(Box::new(Phoebe)),
            "janus" | "Janus" => Some(Box::new(Janus)),
            "epimetheus" | "Epimetheus" => Some(Box::new(Epimetheus)),
            "helene" | "Helene" => Some(Box::new(Helene)),
            "atlas" | "Atlas" => Some(Box::new(Atlas)),
            "prometheus" | "Prometheus" => Some(Box::new(Prometheus)),
            "pandora" | "Pandora" => Some(Box::new(Pandora)),
            "ariel" | "Ariel" => Some(Box::new(Ariel)),
            "umbriel" | "Umbriel" => Some(Box::new(Umbriel)),
            "titania" | "Titania" => Some(Box::new(Titania)),
            "oberon" | "Oberon" => Some(Box::new(Oberon)),
            "miranda" | "Miranda" => Some(Box::new(Miranda)),
            "triton" | "Triton" => Some(Box::new(Triton)),
            "naiad" | "Naiad" => Some(Box::new(Naiad)),
            "thalassa" | "Thalassa" => Some(Box::new(Thalassa)),
            "despina" | "Despina" => Some(Box::new(Despina)),
            "galatea" | "Galatea" => Some(Box::new(Galatea)),
            "larissa" | "Larissa" => Some(Box::new(Larissa)),
            "proteus" | "Proteus" => Some(Box::new(Proteus)),
            "charon" | "Charon" => Some(Box::new(Charon)),
            _ => None,
        };
        match satellite {
            Some(satellite) => Ok(Self(satellite)),
            None => Err(LoxPyError::InvalidBody(name.to_string())),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Satellite(\"{}\")", self.name()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.name().to_string())
    }

    fn id(&self) -> i32 {
        self.0.id().0
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }

    fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }

    fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    fn subplanetary_radius(&self) -> f64 {
        self.0.subplanetary_radius()
    }

    fn along_orbit_radius(&self) -> f64 {
        self.0.along_orbit_radius()
    }
}

#[pyclass(name = "MinorBody")]
pub struct PyMinorBody(Box<dyn MinorBody + Send>);

#[pymethods]
impl PyMinorBody {
    #[new]
    fn new(name: &str) -> Result<Self, LoxPyError> {
        let minor: Option<Box<dyn MinorBody + Send>> = match name {
            "ceres" | "Ceres" => Some(Box::new(Ceres)),
            "vesta" | "Vesta" => Some(Box::new(Vesta)),
            "psyche" | "Psyche" => Some(Box::new(Psyche)),
            "eros" | "Eros" => Some(Box::new(Eros)),
            "davida" | "Davida" => Some(Box::new(Davida)),
            _ => None,
        };
        match minor {
            Some(minor) => Ok(Self(minor)),
            None => Err(LoxPyError::InvalidBody(name.to_string())),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("MinorBody(\"{}\")", self.name()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.name().to_string())
    }

    fn id(&self) -> i32 {
        self.0.id().0
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }

    fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }

    fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    fn subplanetary_radius(&self) -> f64 {
        self.0.subplanetary_radius()
    }

    fn along_orbit_radius(&self) -> f64 {
        self.0.along_orbit_radius()
    }
}
