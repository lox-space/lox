// SPDX-FileCopyrightText: 2023 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::analysis::python::{
    PyAccessResults, PyAccessWindow, PyAoi, PyCarrierPlan, PyEnsemble, PyEvent, PyGroundStation,
    PyHorizonMask, PyLinkBudgetAnalysis, PyLinkBudgetResults, PyLinkWindow, PyLookSide,
    PyObservables, PyOpticalAccessAnalysis, PyOpticalPayload, PyPass, PyPassDirection,
    PyPowerBudgetAnalysis, PyPowerBudgetResults, PySarAccessAnalysis, PySarPayload, PyScenario,
    PySpacecraft, PyTerminalId, PyVisibilityAnalysis, PyVisibilityResults,
};
use crate::bodies::python::{PyOrigin, UndefinedOriginPropertyError};
use crate::comms::python::{
    PyAmplifierTransmitter, PyAntennaFrame, PyCascadeReceiver, PyChannel, PyConstantAntenna,
    PyDipolePattern, PyEirpModel, PyFrequencyRange, PyGaussianPattern, PyGtModel,
    PyInterferenceStats, PyLinkBudget, PyModCod, PyModulatedLinkBudget, PyModulation, PyNoiseStage,
    PyNoiseTempReceiver, PyParabolicPattern, PyPatternedAntenna, PyPfdMask, PyPropagationLosses,
    PyRxChain, PyTxChain, combine_carrier_to_noise, freq_overlap, fspl, power_flux_density,
    slant_range,
};
use crate::constellations::python::{PyConstellation, PyConstellationSatellite};
use crate::earth::python::ut1::{EopParserError, EopProviderError, PyEopProvider};
use crate::ephem::python::PySpk;
use crate::frames::python::{FrameTransformationError, PyFrame};
use crate::itur::python::PyItuProvider;
use crate::itur::python::register_itur_functions;
use crate::math::python::PySeries;
use crate::orbits::python::{
    PyCartesian, PyEllipsoid, PyEllipsoidLocation, PyEquinoctial, PyGroundPropagator,
    PyJ2Propagator, PyJ4Propagator, PyKeplerian, PyModifiedEquinoctial, PyNumericalPropagator,
    PySgp4, PyTle, PyTrajectory, PyVallado,
};
use crate::time::python::{
    deltas::{PyTimeDelta, PyTimeDeltaUnit},
    intervals::{PyInterval, py_complement_intervals, py_intersect_intervals, py_union_intervals},
    time::PyTime,
    time_scales::PyTimeScale,
    time_series::PyTimeSeries,
    utc::PyUtc,
};
use crate::units::python::register_units;

use lox_core::time::deltas::TimeDeltaUnit;
use pyo3::prelude::*;

/// Register all lox-space Python types, functions, and constants into the given module.
///
/// This can be used to embed all Lox types into a downstream Python extension module.
pub fn register_types(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // bodies
    m.add_class::<PyOrigin>()?;
    m.add(
        "UndefinedOriginPropertyError",
        m.py().get_type::<UndefinedOriginPropertyError>(),
    )?;

    // comms
    m.add_class::<PyModulation>()?;
    m.add_class::<PyParabolicPattern>()?;
    m.add_class::<PyGaussianPattern>()?;
    m.add_class::<PyDipolePattern>()?;
    m.add_class::<PyAntennaFrame>()?;
    m.add_class::<PyConstantAntenna>()?;
    m.add_class::<PyPatternedAntenna>()?;
    m.add_class::<PyAmplifierTransmitter>()?;
    m.add_class::<PyNoiseTempReceiver>()?;
    m.add_class::<PyCascadeReceiver>()?;
    m.add_class::<PyNoiseStage>()?;
    m.add_class::<PyChannel>()?;
    m.add_class::<PyModCod>()?;
    m.add_class::<PyItuProvider>()?;
    m.add_class::<PyPropagationLosses>()?;
    m.add_class::<PyLinkBudget>()?;
    m.add_class::<PyInterferenceStats>()?;
    m.add_class::<PyModulatedLinkBudget>()?;
    m.add_class::<PyPfdMask>()?;
    m.add_class::<PyFrequencyRange>()?;
    m.add_class::<PyTxChain>()?;
    m.add_class::<PyRxChain>()?;
    m.add_class::<PyEirpModel>()?;
    m.add_class::<PyGtModel>()?;
    m.add_function(wrap_pyfunction!(fspl, m)?)?;
    m.add_function(wrap_pyfunction!(combine_carrier_to_noise, m)?)?;
    m.add_function(wrap_pyfunction!(freq_overlap, m)?)?;
    m.add_function(wrap_pyfunction!(power_flux_density, m)?)?;
    m.add_function(wrap_pyfunction!(slant_range, m)?)?;

    // itur
    register_itur_functions(m)?;

    // earth
    m.add_class::<PyEopProvider>()?;
    m.add("EopParserError", m.py().get_type::<EopParserError>())?;
    m.add("EopProviderError", m.py().get_type::<EopProviderError>())?;

    // ephem
    m.add_class::<PySpk>()?;

    // frames
    m.add_class::<PyFrame>()?;
    m.add(
        "FrameTransformationError",
        m.py().get_type::<FrameTransformationError>(),
    )?;

    // math
    m.add_class::<PySeries>()?;

    // analysis
    m.add_class::<PyGroundStation>()?;
    m.add_class::<PyHorizonMask>()?;
    m.add_class::<PyObservables>()?;
    m.add_class::<PyPass>()?;
    m.add_class::<PyScenario>()?;
    m.add_class::<PySpacecraft>()?;
    m.add_class::<PyEnsemble>()?;
    m.add_class::<PyVisibilityAnalysis>()?;
    m.add_class::<PyVisibilityResults>()?;
    m.add_class::<PyPowerBudgetAnalysis>()?;
    m.add_class::<PyPowerBudgetResults>()?;
    m.add_class::<PyTerminalId>()?;
    m.add_class::<PyCarrierPlan>()?;
    m.add_class::<PyLinkWindow>()?;
    m.add_class::<PyLinkBudgetAnalysis>()?;
    m.add_class::<PyLinkBudgetResults>()?;
    m.add_class::<PyAoi>()?;
    m.add_class::<PyPassDirection>()?;
    m.add_class::<PyAccessWindow>()?;
    m.add_class::<PyAccessResults>()?;
    m.add_class::<PyOpticalPayload>()?;
    m.add_class::<PyOpticalAccessAnalysis>()?;
    m.add_class::<PyLookSide>()?;
    m.add_class::<PySarPayload>()?;
    m.add_class::<PySarAccessAnalysis>()?;

    // constellations
    m.add_class::<PyConstellation>()?;
    m.add_class::<PyConstellationSatellite>()?;

    // orbits
    m.add_class::<PyCartesian>()?;
    m.add_class::<PyEllipsoid>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyEllipsoidLocation>()?;
    m.add_class::<PyGroundPropagator>()?;
    m.add_class::<PyInterval>()?;
    m.add_class::<PyJ2Propagator>()?;
    m.add_class::<PyJ4Propagator>()?;
    m.add_class::<PyNumericalPropagator>()?;
    m.add_class::<PyKeplerian>()?;
    m.add_class::<PyEquinoctial>()?;
    m.add_class::<PyModifiedEquinoctial>()?;
    m.add_class::<PySgp4>()?;
    m.add_class::<PyTle>()?;
    m.add_class::<PyTrajectory>()?;
    m.add_class::<PyVallado>()?;
    m.add_function(wrap_pyfunction!(py_intersect_intervals, m)?)?;
    m.add_function(wrap_pyfunction!(py_union_intervals, m)?)?;
    m.add_function(wrap_pyfunction!(py_complement_intervals, m)?)?;

    // time
    m.add_class::<PyTime>()?;
    m.add_class::<PyTimeDelta>()?;
    m.add_class::<PyTimeDeltaUnit>()?;
    for (name, unit) in [
        ("attoseconds", TimeDeltaUnit::Attoseconds),
        ("femtoseconds", TimeDeltaUnit::Femtoseconds),
        ("picoseconds", TimeDeltaUnit::Picoseconds),
        ("nanoseconds", TimeDeltaUnit::Nanoseconds),
        ("microseconds", TimeDeltaUnit::Microseconds),
        ("milliseconds", TimeDeltaUnit::Milliseconds),
        ("seconds", TimeDeltaUnit::Seconds),
        ("minutes", TimeDeltaUnit::Minutes),
        ("hours", TimeDeltaUnit::Hours),
        ("days", TimeDeltaUnit::Days),
        ("julian_years", TimeDeltaUnit::JulianYears),
        ("julian_centuries", TimeDeltaUnit::JulianCenturies),
    ] {
        m.add(name, PyTimeDeltaUnit(unit))?;
    }
    m.add_class::<PyTimeScale>()?;
    m.add_class::<PyTimeSeries>()?;
    m.add_class::<PyUtc>()?;

    // units
    register_units(m)?;

    Ok(())
}

#[pymodule]
fn lox_space(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_types(m)
}
