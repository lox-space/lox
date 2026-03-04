// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_frames::DynFrame;
use lox_orbits::constellations::{
    ConstellationError, ConstellationPropagator,
    ConstellationSatellite as RustConstellationSatellite, DynConstellation, FlowerBuilder,
    StreetOfCoverageBuilder, WalkerDeltaBuilder, WalkerStarBuilder,
};
use lox_time::time_scales::Tai;
use lox_units::Angle;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::bodies::python::PyOrigin;
use crate::time::python::time::PyTime;
use crate::units::python::{PyAngle, PyDistance};

struct PyConstellationError(ConstellationError);

impl From<PyConstellationError> for PyErr {
    fn from(err: PyConstellationError) -> Self {
        PyValueError::new_err(err.0.to_string())
    }
}

fn parse_propagator(s: &str) -> PyResult<ConstellationPropagator> {
    match s {
        "vallado" => Ok(ConstellationPropagator::Vallado),
        "j2" => Ok(ConstellationPropagator::J2),
        _ => Err(PyValueError::new_err(format!(
            "unknown propagator \"{s}\", expected \"vallado\" or \"j2\""
        ))),
    }
}

/// A single satellite in a constellation.
#[pyclass(name = "ConstellationSatellite", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyConstellationSatellite(pub RustConstellationSatellite);

#[pymethods]
impl PyConstellationSatellite {
    /// Return the orbital plane index (0-based).
    #[getter]
    fn plane(&self) -> usize {
        self.0.plane
    }

    /// Return the index within the plane (0-based).
    #[getter]
    fn index_in_plane(&self) -> usize {
        self.0.index_in_plane
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstellationSatellite(plane={}, index={})",
            self.0.plane, self.0.index_in_plane
        )
    }
}

/// A named collection of satellites produced by a constellation design algorithm.
#[pyclass(name = "Constellation", module = "lox_space", frozen)]
#[derive(Clone, Debug)]
pub struct PyConstellation(pub DynConstellation);

#[pymethods]
impl PyConstellation {
    /// Create a Walker Delta constellation (RAAN spread = 360 deg).
    #[classmethod]
    #[pyo3(signature = (
        name,
        time,
        origin,
        *,
        nsats,
        nplanes,
        semi_major_axis,
        inclination,
        eccentricity=0.0,
        phasing=0,
        argument_of_periapsis=None,
        propagator="vallado",
    ))]
    #[allow(clippy::too_many_arguments)]
    fn walker_delta(
        _cls: &Bound<'_, PyType>,
        name: String,
        time: PyTime,
        origin: PyOrigin,
        nsats: usize,
        nplanes: usize,
        semi_major_axis: PyDistance,
        inclination: PyAngle,
        eccentricity: f64,
        phasing: usize,
        argument_of_periapsis: Option<PyAngle>,
        propagator: &str,
    ) -> PyResult<Self> {
        let aop = argument_of_periapsis.map(|a| a.0).unwrap_or(Angle::ZERO);
        let prop = parse_propagator(propagator)?;
        let epoch = time.0.to_scale(Tai);

        let constellation = WalkerDeltaBuilder::new(nsats, nplanes)
            .with_semi_major_axis(semi_major_axis.0, eccentricity)
            .with_inclination(inclination.0)
            .with_phasing(phasing)
            .with_argument_of_periapsis(aop)
            .build_constellation(name, epoch, origin.0, DynFrame::Icrf)
            .map_err(PyConstellationError)?;

        Ok(PyConstellation(
            constellation.with_propagator(prop).into_dyn(),
        ))
    }

    /// Create a Walker Star constellation (RAAN spread = 180 deg).
    #[classmethod]
    #[pyo3(signature = (
        name,
        time,
        origin,
        *,
        nsats,
        nplanes,
        semi_major_axis,
        inclination,
        eccentricity=0.0,
        phasing=0,
        argument_of_periapsis=None,
        propagator="vallado",
    ))]
    #[allow(clippy::too_many_arguments)]
    fn walker_star(
        _cls: &Bound<'_, PyType>,
        name: String,
        time: PyTime,
        origin: PyOrigin,
        nsats: usize,
        nplanes: usize,
        semi_major_axis: PyDistance,
        inclination: PyAngle,
        eccentricity: f64,
        phasing: usize,
        argument_of_periapsis: Option<PyAngle>,
        propagator: &str,
    ) -> PyResult<Self> {
        let aop = argument_of_periapsis.map(|a| a.0).unwrap_or(Angle::ZERO);
        let prop = parse_propagator(propagator)?;
        let epoch = time.0.to_scale(Tai);

        let constellation = WalkerStarBuilder::new(nsats, nplanes)
            .with_semi_major_axis(semi_major_axis.0, eccentricity)
            .with_inclination(inclination.0)
            .with_phasing(phasing)
            .with_argument_of_periapsis(aop)
            .build_constellation(name, epoch, origin.0, DynFrame::Icrf)
            .map_err(PyConstellationError)?;

        Ok(PyConstellation(
            constellation.with_propagator(prop).into_dyn(),
        ))
    }

    /// Create a Street-of-Coverage constellation.
    #[classmethod]
    #[pyo3(signature = (
        name,
        time,
        origin,
        *,
        nsats,
        nplanes,
        semi_major_axis,
        inclination,
        eccentricity=0.0,
        coverage_fold=1,
        argument_of_periapsis=None,
        propagator="vallado",
    ))]
    #[allow(clippy::too_many_arguments)]
    fn street_of_coverage(
        _cls: &Bound<'_, PyType>,
        name: String,
        time: PyTime,
        origin: PyOrigin,
        nsats: usize,
        nplanes: usize,
        semi_major_axis: PyDistance,
        inclination: PyAngle,
        eccentricity: f64,
        coverage_fold: usize,
        argument_of_periapsis: Option<PyAngle>,
        propagator: &str,
    ) -> PyResult<Self> {
        let aop = argument_of_periapsis.map(|a| a.0).unwrap_or(Angle::ZERO);
        let prop = parse_propagator(propagator)?;
        let epoch = time.0.to_scale(Tai);

        let constellation = StreetOfCoverageBuilder::new(nsats, nplanes)
            .with_semi_major_axis(semi_major_axis.0, eccentricity)
            .with_inclination(inclination.0)
            .with_coverage_fold(coverage_fold)
            .with_argument_of_periapsis(aop)
            .build_constellation(name, epoch, origin.0, DynFrame::Icrf)
            .map_err(PyConstellationError)?;

        Ok(PyConstellation(
            constellation.with_propagator(prop).into_dyn(),
        ))
    }

    /// Create a Flower constellation (repeating ground tracks).
    #[classmethod]
    #[pyo3(signature = (
        name,
        time,
        origin,
        *,
        n_petals,
        n_days,
        nsats,
        phasing_numerator,
        phasing_denominator,
        inclination,
        perigee_altitude=None,
        semi_major_axis=None,
        eccentricity=None,
        argument_of_periapsis=None,
        propagator="vallado",
    ))]
    #[allow(clippy::too_many_arguments)]
    fn flower(
        _cls: &Bound<'_, PyType>,
        name: String,
        time: PyTime,
        origin: PyOrigin,
        n_petals: u32,
        n_days: u32,
        nsats: usize,
        phasing_numerator: u32,
        phasing_denominator: u32,
        inclination: PyAngle,
        perigee_altitude: Option<PyDistance>,
        semi_major_axis: Option<PyDistance>,
        eccentricity: Option<f64>,
        argument_of_periapsis: Option<PyAngle>,
        propagator: &str,
    ) -> PyResult<Self> {
        let aop = argument_of_periapsis.map(|a| a.0).unwrap_or(Angle::ZERO);
        let prop = parse_propagator(propagator)?;
        let epoch = time.0.to_scale(Tai);

        let mut builder = FlowerBuilder::new(
            n_petals,
            n_days,
            nsats,
            phasing_numerator,
            phasing_denominator,
        )
        .with_inclination(inclination.0)
        .with_argument_of_periapsis(aop);

        match (perigee_altitude, semi_major_axis) {
            (Some(alt), None) => {
                builder = builder.with_perigee_altitude(alt.0);
            }
            (None, Some(sma)) => {
                let ecc = eccentricity.ok_or_else(|| {
                    PyValueError::new_err("eccentricity is required when using semi_major_axis")
                })?;
                builder = builder.with_semi_major_axis(sma.0, ecc);
            }
            (None, None) => {
                return Err(PyValueError::new_err(
                    "either perigee_altitude or semi_major_axis must be provided",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(PyValueError::new_err(
                    "perigee_altitude and semi_major_axis are mutually exclusive",
                ));
            }
        }

        let constellation = builder
            .build_constellation(name, epoch, origin.0, DynFrame::Icrf)
            .map_err(PyConstellationError)?;

        Ok(PyConstellation(
            constellation.with_propagator(prop).into_dyn(),
        ))
    }

    /// Return the constellation name.
    #[getter]
    fn name(&self) -> &str {
        self.0.name()
    }

    /// Return the list of satellites.
    #[getter]
    fn satellites(&self) -> Vec<PyConstellationSatellite> {
        self.0
            .satellites()
            .iter()
            .map(|s| PyConstellationSatellite(s.clone()))
            .collect()
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "Constellation(\"{}\", {} satellites)",
            self.0.name(),
            self.0.len()
        )
    }
}
