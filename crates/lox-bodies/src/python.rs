/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::{
    Adrastea, Amalthea, Ariel, Atlas, Barycenter, Body, Callisto, Ceres, Charon, Davida, Deimos,
    Despina, Dione, Earth, EarthBarycenter, Ellipsoid, Enceladus, Epimetheus, Eros, Europa,
    Galatea, Ganymede, Helene, Himalia, Hyperion, Iapetus, Io, Janus, Jupiter, JupiterBarycenter,
    Larissa, Mars, MarsBarycenter, Mercury, MercuryBarycenter, Metis, Mimas, MinorBody, Miranda,
    Moon, Naiad, NaifId, Neptune, NeptuneBarycenter, NutationPrecessionCoefficients, Oberon,
    Pandora, Phobos, Phoebe, Planet, Pluto, PlutoBarycenter, PointMass, PolynomialCoefficients,
    Prometheus, Proteus, Psyche, Rhea, RotationalElements, Satellite, Saturn, SaturnBarycenter,
    SolarSystemBarycenter, Spheroid, Sun, Tethys, Thalassa, Thebe, Titan, Titania, Triton, Umbriel,
    Uranus, UranusBarycenter, Venus, VenusBarycenter, Vesta,
};

#[pyclass(name = "Sun", module = "lox_space", frozen)]
#[derive(Clone, Debug, Default)]
pub struct PySun;

#[pymethods]
impl PySun {
    #[new]
    pub fn new() -> Self {
        Self
    }

    fn __repr__(&self) -> &str {
        "Sun()"
    }

    fn __str__(&self) -> &str {
        "Sun"
    }

    fn __eq__(&self, _other: &Self) -> bool {
        true
    }

    fn __getnewargs__(&self) -> Py<PyTuple> {
        // A unit return type would be converted to `None` on the Python side,
        // but we actually want an empty tuple here.
        Python::with_gil(|py| PyTuple::empty_bound(py).into_py(py))
    }

    pub fn id(&self) -> i32 {
        Sun.id().0
    }

    pub fn name(&self) -> &'static str {
        Sun.name()
    }

    pub fn gravitational_parameter(&self) -> f64 {
        Sun.gravitational_parameter()
    }

    pub fn mean_radius(&self) -> f64 {
        Sun.mean_radius()
    }

    pub fn polar_radius(&self) -> f64 {
        Sun.polar_radius()
    }

    pub fn equatorial_radius(&self) -> f64 {
        Sun.equatorial_radius()
    }
}

#[pyclass(name = "Barycenter", module = "lox_space", frozen)]
#[derive(Debug, Clone)]
pub struct PyBarycenter(Box<dyn Barycenter + Send + Sync>);

#[pymethods]
impl PyBarycenter {
    #[new]
    pub fn new(name: &str) -> PyResult<Self> {
        let barycenter: Option<Box<dyn Barycenter + Send + Sync>> = match name {
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
            None => Err(PyValueError::new_err(format!(
                "unknown barycenter: {}",
                name
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!("Barycenter(\"{}\")", self.name())
    }

    fn __str__(&self) -> &str {
        self.name()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.name(),)
    }

    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }
}

#[pyclass(name = "Planet", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyPlanet(Box<dyn Planet + Send + Sync>);

#[pymethods]
impl PyPlanet {
    #[new]
    pub fn new(name: &str) -> PyResult<Self> {
        let planet: Option<Box<dyn Planet + Send + Sync>> = match name {
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
            None => Err(PyValueError::new_err(format!("unknown planet: {}", name))),
        }
    }

    fn __repr__(&self) -> String {
        format!("Planet(\"{}\")", self.name())
    }

    fn __str__(&self) -> &str {
        self.name()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.name(),)
    }

    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }

    pub fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }

    pub fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    pub fn equatorial_radius(&self) -> f64 {
        self.0.equatorial_radius()
    }
}

impl Body for PyPlanet {
    fn id(&self) -> NaifId {
        self.0.id()
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}

impl Ellipsoid for PyPlanet {
    fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }
}

impl Spheroid for PyPlanet {
    fn equatorial_radius(&self) -> f64 {
        self.0.equatorial_radius()
    }
}

impl RotationalElements for PyPlanet {
    fn nutation_precession_coefficients(&self) -> NutationPrecessionCoefficients {
        self.0.nutation_precession_coefficients()
    }

    fn right_ascension_coefficients(&self) -> PolynomialCoefficients {
        self.0.right_ascension_coefficients()
    }

    fn declination_coefficients(&self) -> PolynomialCoefficients {
        self.0.declination_coefficients()
    }

    fn prime_meridian_coefficients(&self) -> PolynomialCoefficients {
        self.0.prime_meridian_coefficients()
    }
}

#[pyclass(name = "Satellite", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PySatellite(Box<dyn Satellite + Send + Sync>);

#[pymethods]
impl PySatellite {
    #[new]
    pub fn new(name: &str) -> PyResult<Self> {
        let satellite: Option<Box<dyn Satellite + Send + Sync>> = match name {
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
            None => Err(PyValueError::new_err(format!(
                "unknown satellite: {}",
                name
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!("Satellite(\"{}\")", self.name())
    }

    fn __str__(&self) -> &str {
        self.name()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.name(),)
    }

    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }

    pub fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }

    pub fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    pub fn subplanetary_radius(&self) -> f64 {
        self.0.subplanetary_radius()
    }

    pub fn along_orbit_radius(&self) -> f64 {
        self.0.along_orbit_radius()
    }
}

#[pyclass(name = "MinorBody", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyMinorBody(Box<dyn MinorBody + Send + Sync>);

#[pymethods]
impl PyMinorBody {
    #[new]
    pub fn new(name: &str) -> PyResult<Self> {
        let minor: Option<Box<dyn MinorBody + Send + Sync>> = match name {
            "ceres" | "Ceres" => Some(Box::new(Ceres)),
            "vesta" | "Vesta" => Some(Box::new(Vesta)),
            "psyche" | "Psyche" => Some(Box::new(Psyche)),
            "eros" | "Eros" => Some(Box::new(Eros)),
            "davida" | "Davida" => Some(Box::new(Davida)),
            _ => None,
        };
        match minor {
            Some(minor) => Ok(Self(minor)),
            None => Err(PyValueError::new_err(format!(
                "unknown minor body: {}",
                name
            ))),
        }
    }

    fn __repr__(&self) -> String {
        format!("MinorBody(\"{}\")", self.name())
    }

    fn __str__(&self) -> &str {
        self.name()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn __getnewargs__(&self) -> (&str,) {
        (self.name(),)
    }

    pub fn id(&self) -> i32 {
        self.0.id().0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn gravitational_parameter(&self) -> f64 {
        self.0.gravitational_parameter()
    }

    pub fn mean_radius(&self) -> f64 {
        self.0.mean_radius()
    }

    pub fn polar_radius(&self) -> f64 {
        self.0.polar_radius()
    }

    pub fn subplanetary_radius(&self) -> f64 {
        self.0.subplanetary_radius()
    }

    pub fn along_orbit_radius(&self) -> f64 {
        self.0.along_orbit_radius()
    }
}

#[derive(Debug, Clone)]
pub enum PyBody {
    Barycenter(PyBarycenter),
    Sun(PySun),
    Planet(PyPlanet),
    Satellite(PySatellite),
    MinorBody(PyMinorBody),
}

impl From<PyBody> for PyObject {
    fn from(body: PyBody) -> Self {
        Python::with_gil(|py| match body {
            PyBody::Barycenter(barycenter) => barycenter.clone().into_py(py),
            PyBody::Sun(sun) => sun.clone().into_py(py),
            PyBody::Planet(planet) => planet.clone().into_py(py),
            PyBody::Satellite(satellite) => satellite.clone().into_py(py),
            PyBody::MinorBody(minor_body) => minor_body.clone().into_py(py),
        })
    }
}

impl TryFrom<&Bound<'_, PyAny>> for PyBody {
    type Error = PyErr;

    fn try_from(body: &Bound<'_, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(body) = body.extract::<PyBarycenter>() {
            Ok(PyBody::Barycenter(body))
        } else if let Ok(body) = body.extract::<PySun>() {
            Ok(PyBody::Sun(body))
        } else if let Ok(body) = body.extract::<PyPlanet>() {
            Ok(PyBody::Planet(body))
        } else if let Ok(body) = body.extract::<PySatellite>() {
            Ok(PyBody::Satellite(body))
        } else if let Ok(body) = body.extract::<PyMinorBody>() {
            Ok(PyBody::MinorBody(body))
        } else {
            Err(PyValueError::new_err("Invalid body"))
        }
    }
}

impl TryFrom<Option<&Bound<'_, PyAny>>> for PyBody {
    type Error = PyErr;

    fn try_from(body: Option<&Bound<'_, PyAny>>) -> Result<Self, Self::Error> {
        if let Some(body) = body {
            return PyBody::try_from(body);
        }
        Ok(PyBody::Planet(PyPlanet::new("Earth").unwrap()))
    }
}

impl TryFrom<PyObject> for PyBody {
    type Error = PyErr;

    // TODO: Use `Bound` API
    fn try_from(body: PyObject) -> Result<Self, Self::Error> {
        Python::with_gil(|py| {
            if let Ok(body) = body.extract::<PyBarycenter>(py) {
                Ok(PyBody::Barycenter(body))
            } else if let Ok(body) = body.extract::<PySun>(py) {
                Ok(PyBody::Sun(body))
            } else if let Ok(body) = body.extract::<PyPlanet>(py) {
                Ok(PyBody::Planet(body))
            } else if let Ok(body) = body.extract::<PySatellite>(py) {
                Ok(PyBody::Satellite(body))
            } else if let Ok(body) = body.extract::<PyMinorBody>(py) {
                Ok(PyBody::MinorBody(body))
            } else {
                Err(PyValueError::new_err("Invalid body"))
            }
        })
    }
}

impl Body for PyBody {
    fn id(&self) -> NaifId {
        match &self {
            PyBody::Barycenter(barycenter) => NaifId(barycenter.id()),
            PyBody::Sun(sun) => NaifId(sun.id()),
            PyBody::Planet(planet) => NaifId(planet.id()),
            PyBody::Satellite(satellite) => NaifId(satellite.id()),
            PyBody::MinorBody(minor_body) => NaifId(minor_body.id()),
        }
    }

    fn name(&self) -> &'static str {
        match &self {
            PyBody::Barycenter(barycenter) => barycenter.name(),
            PyBody::Sun(sun) => sun.name(),
            PyBody::Planet(planet) => planet.name(),
            PyBody::Satellite(satellite) => satellite.name(),
            PyBody::MinorBody(minor_body) => minor_body.name(),
        }
    }
}

impl PointMass for PyBody {
    fn gravitational_parameter(&self) -> f64 {
        match &self {
            PyBody::Barycenter(barycenter) => barycenter.gravitational_parameter(),
            PyBody::Sun(sun) => sun.gravitational_parameter(),
            PyBody::Planet(planet) => planet.gravitational_parameter(),
            PyBody::Satellite(satellite) => satellite.gravitational_parameter(),
            PyBody::MinorBody(minor_body) => minor_body.gravitational_parameter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_sun() {
        let sun = PySun::new();
        assert_eq!(sun.__repr__(), "Sun()");
        assert_eq!(sun.__str__(), "Sun");
        assert_eq!(sun.id(), Sun.id().0);
        assert_eq!(sun.name(), Sun.name());
        assert_eq!(sun.gravitational_parameter(), Sun.gravitational_parameter());
        assert_eq!(sun.mean_radius(), Sun.mean_radius());
        assert_eq!(sun.polar_radius(), Sun.polar_radius());
        assert_eq!(sun.equatorial_radius(), Sun.equatorial_radius());
        assert!(sun.__eq__(&sun));
        let sun_args = sun.__getnewargs__();
        let empty: Py<PyTuple> = Python::with_gil(|py| PyTuple::empty_bound(py).into_py(py));
        assert!(sun_args.is(&empty));
    }

    #[rstest]
    #[case("Solar System Barycenter", SolarSystemBarycenter)]
    #[case("SSB", SolarSystemBarycenter)]
    #[case("Mercury Barycenter", MercuryBarycenter)]
    #[case("Venus Barycenter", VenusBarycenter)]
    #[case("Earth Barycenter", EarthBarycenter)]
    #[case("Mars Barycenter", MarsBarycenter)]
    #[case("Jupiter Barycenter", JupiterBarycenter)]
    #[case("Saturn Barycenter", SaturnBarycenter)]
    #[case("Uranus Barycenter", UranusBarycenter)]
    #[case("Neptune Barycenter", NeptuneBarycenter)]
    #[case("Pluto Barycenter", PlutoBarycenter)]
    fn test_barycenter(#[case] name: &str, #[case] barycenter: impl Barycenter) {
        let py_barycenter = PyBarycenter::new(name).expect("barycenter should be valid");
        assert_eq!(
            py_barycenter.__repr__(),
            format!("Barycenter(\"{}\")", barycenter.name())
        );
        assert_eq!(py_barycenter.__str__(), barycenter.name());
        assert_eq!(py_barycenter.name(), barycenter.name());
        let py_barycenter =
            PyBarycenter::new(&name.to_lowercase()).expect("barycenter should be valid");
        assert_eq!(
            py_barycenter.__repr__(),
            format!("Barycenter(\"{}\")", barycenter.name())
        );
        assert_eq!(py_barycenter.__str__(), barycenter.name());
        assert_eq!(py_barycenter.name(), barycenter.name());
        assert_eq!(py_barycenter.id(), barycenter.id().0);
        assert_eq!(
            py_barycenter.gravitational_parameter(),
            barycenter.gravitational_parameter()
        );
        assert_eq!(py_barycenter.__getnewargs__(), (barycenter.name(),));
        assert!(py_barycenter.__eq__(&py_barycenter));
    }

    #[test]
    fn test_invalid_barycenter() {
        let barycenter = PyBarycenter::new("Rupert Barycenter");
        assert!(barycenter.is_err());
    }

    #[rstest]
    #[case("Mercury", Mercury)]
    #[case("Venus", Venus)]
    #[case("Earth", Earth)]
    #[case("Mars", Mars)]
    #[case("Jupiter", Jupiter)]
    #[case("Saturn", Saturn)]
    #[case("Uranus", Uranus)]
    #[case("Neptune", Neptune)]
    #[case("Pluto", Pluto)]
    fn test_planet(#[case] name: &str, #[case] planet: impl Planet) {
        let py_planet = PyPlanet::new(name).expect("planet should be valid");
        assert_eq!(py_planet.__repr__(), format!("Planet(\"{}\")", name));
        assert_eq!(py_planet.__str__(), name);
        assert_eq!(py_planet.name(), name);
        let py_planet = PyPlanet::new(&name.to_lowercase()).expect("planet should be valid");
        assert_eq!(py_planet.__repr__(), format!("Planet(\"{}\")", name));
        assert_eq!(py_planet.__str__(), name);
        assert_eq!(py_planet.name(), name);
        assert_eq!(py_planet.id(), planet.id().0);
        assert_eq!(
            py_planet.gravitational_parameter(),
            planet.gravitational_parameter()
        );
        assert_eq!(py_planet.mean_radius(), planet.mean_radius());
        assert_eq!(py_planet.polar_radius(), planet.polar_radius());
        assert_eq!(py_planet.equatorial_radius(), planet.equatorial_radius());
        assert_eq!(py_planet.__getnewargs__(), (planet.name(),));
        assert!(py_planet.__eq__(&py_planet));
    }

    #[test]
    fn test_invalid_planet() {
        let planet = PyPlanet::new("Rupert");
        assert!(planet.is_err());
    }

    #[rstest]
    #[case("Moon", Moon)]
    #[case("Luna", Moon)]
    #[case("Phobos", Phobos)]
    #[case("Deimos", Deimos)]
    #[case("Io", Io)]
    #[case("Europa", Europa)]
    #[case("Ganymede", Ganymede)]
    #[case("Callisto", Callisto)]
    #[case("Amalthea", Amalthea)]
    #[case("Himalia", Himalia)]
    #[case("Thebe", Thebe)]
    #[case("Adrastea", Adrastea)]
    #[case("Metis", Metis)]
    #[case("Mimas", Mimas)]
    #[case("Enceladus", Enceladus)]
    #[case("Tethys", Tethys)]
    #[case("Dione", Dione)]
    #[case("Rhea", Rhea)]
    #[case("Titan", Titan)]
    #[case("Hyperion", Hyperion)]
    #[case("Iapetus", Iapetus)]
    #[case("Phoebe", Phoebe)]
    #[case("Janus", Janus)]
    #[case("Epimetheus", Epimetheus)]
    #[case("Helene", Helene)]
    #[case("Atlas", Atlas)]
    #[case("Prometheus", Prometheus)]
    #[case("Pandora", Pandora)]
    #[case("Ariel", Ariel)]
    #[case("Umbriel", Umbriel)]
    #[case("Titania", Titania)]
    #[case("Oberon", Oberon)]
    #[case("Miranda", Miranda)]
    #[case("Triton", Triton)]
    #[case("Naiad", Naiad)]
    #[case("Thalassa", Thalassa)]
    #[case("Despina", Despina)]
    #[case("Galatea", Galatea)]
    #[case("Larissa", Larissa)]
    #[case("Proteus", Proteus)]
    #[case("Charon", Charon)]
    fn test_satellite(#[case] name: &str, #[case] satellite: impl Satellite) {
        let py_satellite = PySatellite::new(name).expect("satellite should be valid");
        assert_eq!(
            py_satellite.__repr__(),
            format!("Satellite(\"{}\")", satellite.name())
        );
        assert_eq!(py_satellite.__str__(), satellite.name());
        assert_eq!(py_satellite.name(), satellite.name());
        let py_satellite =
            PySatellite::new(&name.to_lowercase()).expect("satellite should be valid");
        assert_eq!(
            py_satellite.__repr__(),
            format!("Satellite(\"{}\")", satellite.name())
        );
        assert_eq!(py_satellite.__str__(), satellite.name());
        assert_eq!(py_satellite.name(), satellite.name());
        assert_eq!(py_satellite.id(), satellite.id().0);
        assert_eq!(
            py_satellite.gravitational_parameter(),
            satellite.gravitational_parameter()
        );
        assert_eq!(py_satellite.mean_radius(), satellite.mean_radius());
        assert_eq!(py_satellite.polar_radius(), satellite.polar_radius());
        assert_eq!(
            py_satellite.subplanetary_radius(),
            satellite.subplanetary_radius()
        );
        assert_eq!(
            py_satellite.along_orbit_radius(),
            satellite.along_orbit_radius()
        );
        assert_eq!(py_satellite.__getnewargs__(), (satellite.name(),));
        assert!(py_satellite.__eq__(&py_satellite));
    }

    #[test]
    fn test_invalid_satellite() {
        let satellite = PySatellite::new("Endor");
        assert!(satellite.is_err());
    }

    #[rstest]
    #[case("Ceres", Ceres)]
    #[case("Vesta", Vesta)]
    #[case("Psyche", Psyche)]
    #[case("Eros", Eros)]
    #[case("Davida", Davida)]
    fn test_minor_body(#[case] name: &str, #[case] minor_body: impl MinorBody) {
        let py_minor_body = PyMinorBody::new(name).expect("minor body should be valid");
        assert_eq!(
            py_minor_body.__repr__(),
            format!("MinorBody(\"{}\")", minor_body.name())
        );
        assert_eq!(py_minor_body.__str__(), minor_body.name());
        assert_eq!(py_minor_body.name(), minor_body.name());
        let py_minor_body =
            PyMinorBody::new(&name.to_lowercase()).expect("minor body should be valid");
        assert_eq!(
            py_minor_body.__repr__(),
            format!("MinorBody(\"{}\")", minor_body.name())
        );
        assert_eq!(py_minor_body.__str__(), minor_body.name());
        assert_eq!(py_minor_body.name(), minor_body.name());
        assert_eq!(py_minor_body.id(), minor_body.id().0);
        assert_eq!(
            py_minor_body.gravitational_parameter(),
            minor_body.gravitational_parameter()
        );
        assert_eq!(py_minor_body.mean_radius(), minor_body.mean_radius());
        assert_eq!(py_minor_body.polar_radius(), minor_body.polar_radius());
        assert_eq!(
            py_minor_body.subplanetary_radius(),
            minor_body.subplanetary_radius()
        );
        assert_eq!(
            py_minor_body.along_orbit_radius(),
            minor_body.along_orbit_radius()
        );
        assert_eq!(py_minor_body.__getnewargs__(), (minor_body.name(),));
        assert!(py_minor_body.__eq__(&py_minor_body));
    }

    #[test]
    fn test_invalid_minor_body() {
        let minor_body = PyMinorBody::new("Bielefeld");
        assert!(minor_body.is_err());
    }

    #[test]
    fn test_body() {
        let sun = PyBody::Sun(PySun::new());
        assert_eq!(sun.id(), Sun.id());
        assert_eq!(sun.name(), Sun.name());
        assert_eq!(sun.gravitational_parameter(), Sun.gravitational_parameter());
        let sun1: PyBody = PyObject::from(sun.clone())
            .try_into()
            .expect("sun is valid");
        assert_eq!(sun1.id(), sun.id());

        let barycenter = PyBody::Barycenter(PyBarycenter::new("ssb").expect("barycenter is valid"));
        assert_eq!(barycenter.id(), SolarSystemBarycenter.id());
        assert_eq!(barycenter.name(), SolarSystemBarycenter.name());
        assert_eq!(
            barycenter.gravitational_parameter(),
            SolarSystemBarycenter.gravitational_parameter()
        );
        let barycenter1: PyBody = PyObject::from(barycenter.clone())
            .try_into()
            .expect("barycenter is valid");
        assert_eq!(barycenter1.id(), barycenter.id());

        let planet = PyBody::Planet(PyPlanet::new("earth").expect("planet is valid"));
        assert_eq!(planet.id(), Earth.id());
        assert_eq!(planet.name(), Earth.name());
        assert_eq!(
            planet.gravitational_parameter(),
            Earth.gravitational_parameter()
        );
        let planet1: PyBody = PyObject::from(planet.clone())
            .try_into()
            .expect("planet is valid");
        assert_eq!(planet1.id(), planet.id());

        let satellite = PyBody::Satellite(PySatellite::new("moon").expect("satellite is valid"));
        assert_eq!(satellite.id(), Moon.id());
        assert_eq!(satellite.name(), Moon.name());
        assert_eq!(
            satellite.gravitational_parameter(),
            Moon.gravitational_parameter()
        );
        let satellite1: PyBody = PyObject::from(satellite.clone())
            .try_into()
            .expect("satellite is valid");
        assert_eq!(satellite1.id(), satellite.id());

        let minor_body = PyBody::MinorBody(PyMinorBody::new("ceres").expect("minor body is valid"));
        assert_eq!(minor_body.id(), Ceres.id());
        assert_eq!(minor_body.name(), Ceres.name());
        assert_eq!(
            minor_body.gravitational_parameter(),
            Ceres.gravitational_parameter()
        );
        let minor_body1: PyBody = PyObject::from(minor_body.clone())
            .try_into()
            .expect("minor_body is valid");
        assert_eq!(minor_body1.id(), minor_body.id());

        let obj = Python::with_gil(|py| 1.into_py(py));
        let body = PyBody::try_from(obj);
        assert!(body.is_err());
    }
}
