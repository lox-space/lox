// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::string::String;

use lox_core::glam::DVec3;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use lox_comms::antenna::{Antenna, AntennaFrame, AntennaGain, ConstantAntenna, PatternedAntenna};
use lox_comms::band::FrequencyRange;
use lox_comms::channel::{Channel, LinkDirection, Modulation};
use lox_comms::link_budget::{
    InterferenceStats, LinkStats, ModulatedLinkStats, frequency_overlap_factor,
};
use lox_itur::EnvironmentalLosses;

use crate::itur::python::PyEnvironmentalLosses;
use lox_comms::pattern::{AntennaPattern, DipolePattern, GaussianPattern, ParabolicPattern};
use lox_comms::payload::{
    CommsPayload, EirpModel, GtModel, RxChain, RxPort, Terminal, TerminalRole, TxChain, TxPort,
};
use lox_comms::pfd;
use lox_comms::pointing::Pointing;
use lox_comms::receiver::{CascadeReceiver, NoiseStage, NoiseTempReceiver, Receiver};
use lox_comms::transmitter::AmplifierTransmitter;
use lox_comms::utils::{free_space_path_loss, slant_range as comms_slant_range};
use lox_core::units::{Angle, Decibel, Temperature};

use crate::units::python::{PyAngle, PyDistance, PyFrequency, PyPower, PyTemperature};

/// Formats an f64 as a valid Python float literal (always includes a decimal point).
fn repr_f64(v: f64) -> String {
    let s = v.to_string();
    if v.is_finite() && !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

fn modulation_name(m: Modulation) -> &'static str {
    match m {
        Modulation::Bpsk => "BPSK",
        Modulation::Qpsk => "QPSK",
        Modulation::Psk8 => "8PSK",
        Modulation::Qam16 => "16QAM",
        Modulation::Qam32 => "32QAM",
        Modulation::Qam64 => "64QAM",
        Modulation::Qam128 => "128QAM",
        Modulation::Qam256 => "256QAM",
        _ => unreachable!("unknown modulation variant"),
    }
}

// --- Decibel ---

/// A value in decibels.
///
/// Args:
///     value: The value in dB.
#[pyclass(name = "Decibel", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyDecibel(pub Decibel);

#[pymethods]
impl PyDecibel {
    #[new]
    /// Constructs a Decibel value from a raw dB number.
    pub fn new(value: f64) -> Self {
        Self(Decibel::new(value))
    }

    /// Creates a Decibel value from a linear power ratio.
    #[staticmethod]
    fn from_linear(value: f64) -> Self {
        Self(Decibel::from_linear(value))
    }

    /// Returns the linear power ratio.
    #[allow(clippy::wrong_self_convention)]
    fn to_linear(&self) -> f64 {
        self.0.to_linear()
    }

    fn __float__(&self) -> f64 {
        self.0.as_f64()
    }

    fn __add__(&self, other: &PyDecibel) -> Self {
        Self(self.0 + other.0)
    }

    fn __sub__(&self, other: &PyDecibel) -> Self {
        Self(self.0 - other.0)
    }

    fn __mul__(&self, other: f64) -> Self {
        Self(Decibel::new(other * self.0.as_f64()))
    }

    fn __rmul__(&self, other: f64) -> Self {
        Self(Decibel::new(other * self.0.as_f64()))
    }

    fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    fn __eq__(&self, other: &PyDecibel) -> bool {
        self.0.as_f64() == other.0.as_f64()
    }

    fn __getnewargs__(&self) -> (f64,) {
        (self.0.as_f64(),)
    }

    /// Returns the developer-readable representation.
    pub fn __repr__(&self) -> String {
        format!("Decibel({})", repr_f64(self.0.as_f64()))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

// --- Modulation ---

/// Digital modulation scheme.
///
/// Args:
///     name: One of "BPSK", "QPSK", "8PSK", "16QAM", "32QAM", "64QAM", "128QAM", "256QAM".
#[pyclass(name = "Modulation", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyModulation(pub Modulation);

#[pymethods]
impl PyModulation {
    #[new]
    fn new(name: &str) -> PyResult<Self> {
        let m = match name {
            "BPSK" => Modulation::Bpsk,
            "QPSK" => Modulation::Qpsk,
            "8PSK" => Modulation::Psk8,
            "16QAM" => Modulation::Qam16,
            "32QAM" => Modulation::Qam32,
            "64QAM" => Modulation::Qam64,
            "128QAM" => Modulation::Qam128,
            "256QAM" => Modulation::Qam256,
            _ => return Err(PyValueError::new_err(format!("unknown modulation: {name}"))),
        };
        Ok(Self(m))
    }

    /// Returns the number of bits per symbol.
    fn bits_per_symbol(&self) -> u8 {
        self.0.bits_per_symbol()
    }

    fn __eq__(&self, other: &PyModulation) -> bool {
        self.0 == other.0
    }

    fn __getnewargs__(&self) -> (&str,) {
        (modulation_name(self.0),)
    }

    fn __repr__(&self) -> String {
        format!("Modulation('{}')", modulation_name(self.0))
    }
}

// --- Antenna Patterns ---

/// Parabolic antenna gain pattern.
///
/// Args:
///     diameter: Antenna diameter as Distance.
///     efficiency: Aperture efficiency (0, 1].
#[pyclass(
    name = "ParabolicPattern",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyParabolicPattern(pub ParabolicPattern);

#[pymethods]
impl PyParabolicPattern {
    #[new]
    fn new(diameter: PyDistance, efficiency: f64) -> PyResult<Self> {
        ParabolicPattern::new(diameter.0, efficiency)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Creates a parabolic pattern from a desired beamwidth.
    ///
    /// Args:
    ///     beamwidth: Half-power beamwidth as Angle.
    ///     frequency: Frequency.
    ///     efficiency: Aperture efficiency (0, 1].
    #[staticmethod]
    fn from_beamwidth(
        beamwidth: PyAngle,
        frequency: PyFrequency,
        efficiency: f64,
    ) -> PyResult<Self> {
        ParabolicPattern::from_beamwidth(beamwidth.0, frequency.0, efficiency)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(frequency.0, theta.0, phi.map_or(Angle::ZERO, |p| p.0)),
        )
    }

    /// Returns the half-power beamwidth, or ``None`` when the
    /// antenna diameter is smaller than ~0.51 wavelengths at this frequency.
    fn beamwidth(&self, frequency: PyFrequency) -> Option<PyAngle> {
        self.0.beamwidth(frequency.0).map(PyAngle)
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    fn __eq__(&self, other: &PyParabolicPattern) -> bool {
        self.0.diameter().to_meters() == other.0.diameter().to_meters()
            && self.0.efficiency() == other.0.efficiency()
    }

    fn __getnewargs__(&self) -> (PyDistance, f64) {
        (PyDistance(self.0.diameter()), self.0.efficiency())
    }

    fn __repr__(&self) -> String {
        format!(
            "ParabolicPattern(diameter={}, efficiency={})",
            PyDistance(self.0.diameter()).__repr__(),
            repr_f64(self.0.efficiency()),
        )
    }
}

/// Gaussian antenna gain pattern.
///
/// Args:
///     diameter: Antenna diameter as Distance.
///     efficiency: Aperture efficiency (0, 1].
#[pyclass(name = "GaussianPattern", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyGaussianPattern(pub GaussianPattern);

#[pymethods]
impl PyGaussianPattern {
    #[new]
    fn new(diameter: PyDistance, efficiency: f64) -> PyResult<Self> {
        GaussianPattern::new(diameter.0, efficiency)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(frequency.0, theta.0, phi.map_or(Angle::ZERO, |p| p.0)),
        )
    }

    /// Returns the half-power beamwidth.
    fn beamwidth(&self, frequency: PyFrequency) -> PyAngle {
        PyAngle(self.0.beamwidth(frequency.0))
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    fn __eq__(&self, other: &PyGaussianPattern) -> bool {
        self.0.diameter().to_meters() == other.0.diameter().to_meters()
            && self.0.efficiency() == other.0.efficiency()
    }

    fn __getnewargs__(&self) -> (PyDistance, f64) {
        (PyDistance(self.0.diameter()), self.0.efficiency())
    }

    fn __repr__(&self) -> String {
        format!(
            "GaussianPattern(diameter={}, efficiency={})",
            PyDistance(self.0.diameter()).__repr__(),
            repr_f64(self.0.efficiency()),
        )
    }
}

/// Dipole antenna gain pattern.
///
/// Args:
///     length: Dipole length as Distance.
#[pyclass(name = "DipolePattern", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyDipolePattern(pub DipolePattern);

#[pymethods]
impl PyDipolePattern {
    #[new]
    fn new(length: PyDistance) -> PyResult<Self> {
        DipolePattern::new(length.0)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(frequency.0, theta.0, phi.map_or(Angle::ZERO, |p| p.0)),
        )
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    fn __eq__(&self, other: &PyDipolePattern) -> bool {
        self.0.length().to_meters() == other.0.length().to_meters()
    }

    fn __getnewargs__(&self) -> (PyDistance,) {
        (PyDistance(self.0.length()),)
    }

    fn __repr__(&self) -> String {
        format!(
            "DipolePattern(length={})",
            PyDistance(self.0.length()).__repr__(),
        )
    }
}

// --- Antennas ---

/// An antenna with constant gain.
///
/// Args:
///     gain: Peak gain as Decibel.
#[pyclass(name = "ConstantAntenna", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyConstantAntenna {
    /// The wrapped [`ConstantAntenna`] value.
    pub inner: ConstantAntenna,
}

#[pymethods]
impl PyConstantAntenna {
    #[new]
    fn new(gain: PyDecibel) -> PyResult<Self> {
        Ok(Self {
            inner: ConstantAntenna::new(gain.0)
                .map_err(|err| PyValueError::new_err(err.to_string()))?,
        })
    }

    fn __eq__(&self, other: &PyConstantAntenna) -> bool {
        self.inner.peak_gain().as_f64() == other.inner.peak_gain().as_f64()
    }

    fn __getnewargs__(&self) -> (PyDecibel,) {
        (PyDecibel(self.inner.peak_gain()),)
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstantAntenna(gain={})",
            PyDecibel(self.inner.peak_gain()).__repr__(),
        )
    }
}

/// Right-handed antenna coordinate frame expressed in a parent frame.
///
/// Args:
///     boresight: Antenna +Z axis as [x, y, z].
///     reference: Direction used to define the antenna +X axis after projection
///         into the plane perpendicular to boresight.
#[pyclass(name = "AntennaFrame", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyAntennaFrame(pub AntennaFrame);

#[pymethods]
impl PyAntennaFrame {
    #[new]
    fn new(boresight: [f64; 3], reference: [f64; 3]) -> PyResult<Self> {
        Self::from_boresight_and_reference(boresight, reference)
    }

    /// Creates an antenna frame aligned with the parent frame.
    #[staticmethod]
    fn identity() -> Self {
        Self(AntennaFrame::identity())
    }

    /// Creates an antenna frame from boresight and reference directions.
    #[staticmethod]
    fn from_boresight_and_reference(boresight: [f64; 3], reference: [f64; 3]) -> PyResult<Self> {
        AntennaFrame::from_boresight_and_reference(
            DVec3::from_array(boresight),
            DVec3::from_array(reference),
        )
        .map(Self)
        .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the antenna-frame +X axis in the parent frame.
    fn x(&self) -> [f64; 3] {
        self.0.x().to_array()
    }

    /// Returns the antenna-frame +Y axis in the parent frame.
    fn y(&self) -> [f64; 3] {
        self.0.y().to_array()
    }

    /// Returns the antenna-frame +Z axis in the parent frame.
    fn z(&self) -> [f64; 3] {
        self.0.z().to_array()
    }

    /// Returns the pattern angles for a parent-frame direction vector.
    fn angles_for(&self, direction: [f64; 3]) -> PyResult<(PyAngle, PyAngle)> {
        self.0
            .angles_for(DVec3::from_array(direction))
            .map(|(theta, phi)| (PyAngle(theta), PyAngle(phi)))
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    fn __eq__(&self, other: &PyAntennaFrame) -> bool {
        self.0 == other.0
    }

    fn __getnewargs__(&self) -> ([f64; 3], [f64; 3]) {
        (self.0.z().to_array(), self.0.x().to_array())
    }

    fn __repr__(&self) -> String {
        let z = self.0.z();
        let x = self.0.x();
        format!(
            "AntennaFrame(boresight=[{}, {}, {}], reference=[{}, {}, {}])",
            repr_f64(z.x),
            repr_f64(z.y),
            repr_f64(z.z),
            repr_f64(x.x),
            repr_f64(x.y),
            repr_f64(x.z),
        )
    }
}

/// An antenna with a physics-based gain pattern and antenna frame.
///
/// Args:
///     pattern: An antenna pattern (ParabolicPattern, GaussianPattern, or DipolePattern).
///     frame: Antenna frame defining the pattern orientation. Defaults to identity.
#[pyclass(
    name = "PatternedAntenna",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyPatternedAntenna(pub PatternedAntenna);

#[pymethods]
impl PyPatternedAntenna {
    #[new]
    #[pyo3(signature = (pattern, frame=None))]
    fn new(pattern: &Bound<'_, PyAny>, frame: Option<&PyAntennaFrame>) -> PyResult<Self> {
        let pattern = extract_antenna_pattern(pattern)?;
        Ok(Self(PatternedAntenna {
            pattern,
            frame: frame.map_or_else(AntennaFrame::identity, |f| f.0),
        }))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(frequency.0, theta.0, phi.map_or(Angle::ZERO, |p| p.0)),
        )
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    /// Returns the gain in dBi toward a parent-frame direction vector.
    fn gain_toward(&self, frequency: PyFrequency, direction: [f64; 3]) -> PyResult<PyDecibel> {
        self.0
            .gain_toward(frequency.0, DVec3::from_array(direction))
            .map(PyDecibel)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the half-power beamwidth, or ``None`` when the underlying
    /// pattern does not define one (e.g. ``DipolePattern``, or a
    /// ``ParabolicPattern`` whose diameter is below ~0.51 wavelengths).
    fn beamwidth(&self, frequency: PyFrequency) -> Option<PyAngle> {
        self.0.pattern.beamwidth(frequency.0).map(PyAngle)
    }

    fn __getnewargs__<'py>(&self, py: Python<'py>) -> (Bound<'py, PyAny>, PyAntennaFrame) {
        let pattern = pattern_to_py(py, &self.0.pattern);
        (pattern, PyAntennaFrame(self.0.frame))
    }

    fn __repr__(&self) -> String {
        let pattern_repr = match &self.0.pattern {
            AntennaPattern::Parabolic(p) => format!(
                "ParabolicPattern(diameter={}, efficiency={})",
                PyDistance(p.diameter()).__repr__(),
                repr_f64(p.efficiency()),
            ),
            AntennaPattern::Gaussian(p) => format!(
                "GaussianPattern(diameter={}, efficiency={})",
                PyDistance(p.diameter()).__repr__(),
                repr_f64(p.efficiency()),
            ),
            AntennaPattern::Dipole(p) => {
                format!(
                    "DipolePattern(length={})",
                    PyDistance(p.length()).__repr__()
                )
            }
            &_ => unreachable!("unknown antenna variant"),
        };
        format!(
            "PatternedAntenna(pattern={pattern_repr}, frame={})",
            PyAntennaFrame(self.0.frame).__repr__(),
        )
    }
}

fn extract_antenna_pattern(obj: &Bound<'_, PyAny>) -> PyResult<AntennaPattern> {
    if let Ok(p) = obj.extract::<PyRef<'_, PyParabolicPattern>>() {
        Ok(AntennaPattern::Parabolic(p.0.clone()))
    } else if let Ok(p) = obj.extract::<PyRef<'_, PyGaussianPattern>>() {
        Ok(AntennaPattern::Gaussian(p.0.clone()))
    } else if let Ok(p) = obj.extract::<PyRef<'_, PyDipolePattern>>() {
        Ok(AntennaPattern::Dipole(p.0.clone()))
    } else {
        Err(PyValueError::new_err(
            "expected a ParabolicPattern, GaussianPattern, or DipolePattern",
        ))
    }
}

fn pattern_to_py<'py>(py: Python<'py>, pattern: &AntennaPattern) -> Bound<'py, PyAny> {
    match pattern {
        AntennaPattern::Parabolic(p) => Bound::new(py, PyParabolicPattern(p.clone()))
            .unwrap()
            .into_any(),
        AntennaPattern::Gaussian(p) => Bound::new(py, PyGaussianPattern(p.clone()))
            .unwrap()
            .into_any(),
        AntennaPattern::Dipole(p) => Bound::new(py, PyDipolePattern(p.clone()))
            .unwrap()
            .into_any(),
        _ => unreachable!("unknown antenna variant"),
    }
}

fn build_antenna(obj: &Bound<'_, PyAny>) -> PyResult<Antenna> {
    if let Ok(a) = obj.extract::<PyRef<'_, PyConstantAntenna>>() {
        Ok(Antenna::Constant(a.inner.clone()))
    } else if let Ok(a) = obj.extract::<PyRef<'_, PyPatternedAntenna>>() {
        Ok(Antenna::Patterned(a.0.clone()))
    } else {
        Err(PyValueError::new_err(
            "expected a ConstantAntenna or PatternedAntenna",
        ))
    }
}

fn build_receiver(obj: &Bound<'_, PyAny>) -> PyResult<Receiver> {
    if let Ok(r) = obj.extract::<PyRef<'_, PyNoiseTempReceiver>>() {
        Ok(Receiver::NoiseTemperature(r.0.clone()))
    } else if let Ok(r) = obj.extract::<PyRef<'_, PyCascadeReceiver>>() {
        Ok(Receiver::Cascade(r.0.clone()))
    } else {
        Err(PyValueError::new_err(
            "expected NoiseTempReceiver or CascadeReceiver",
        ))
    }
}

fn build_receiver_any(obj: &Bound<'_, PyAny>) -> PyResult<Receiver> {
    build_receiver(obj)
}

// --- AmplifierTransmitter ---

/// A radio transmitter with an RF power amplifier.
///
/// Args:
///     band: Supported frequency range.
///     power: Transmit power.
///     output_back_off: Output back-off as Decibel (default Decibel(0)).
#[pyclass(
    name = "AmplifierTransmitter",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyAmplifierTransmitter(pub AmplifierTransmitter);

#[pymethods]
impl PyAmplifierTransmitter {
    #[new]
    #[pyo3(signature = (band, power, output_back_off=None))]
    fn new(
        band: PyFrequencyRange,
        power: PyPower,
        output_back_off: Option<PyDecibel>,
    ) -> PyResult<Self> {
        AmplifierTransmitter::new(
            band.0,
            power.0,
            output_back_off.map_or(Decibel::new(0.0), |d| d.0),
        )
        .map(Self)
        .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Supported frequency range.
    #[getter]
    fn band(&self) -> PyFrequencyRange {
        PyFrequencyRange(self.0.band())
    }

    /// Transmit power.
    #[getter]
    fn power(&self) -> PyPower {
        PyPower(self.0.power())
    }

    /// Output back-off.
    #[getter]
    fn output_back_off(&self) -> PyDecibel {
        PyDecibel(self.0.output_back_off())
    }

    fn __eq__(&self, other: &PyAmplifierTransmitter) -> bool {
        self.0.band() == other.0.band()
            && self.0.power().to_watts() == other.0.power().to_watts()
            && self.0.output_back_off().as_f64() == other.0.output_back_off().as_f64()
    }

    fn __getnewargs__(&self) -> (PyFrequencyRange, PyPower, Option<PyDecibel>) {
        (
            PyFrequencyRange(self.0.band()),
            PyPower(self.0.power()),
            Some(PyDecibel(self.0.output_back_off())),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "AmplifierTransmitter(band={}, power={}, output_back_off={})",
            PyFrequencyRange(self.0.band()).__repr__(),
            PyPower(self.0.power()).__repr__(),
            PyDecibel(self.0.output_back_off()).__repr__(),
        )
    }
}

// --- Receivers ---

/// A receiver with a known system noise temperature.
///
/// Args:
///     band: Supported frequency range.
///     noise_temperature: System noise temperature.
#[pyclass(
    name = "NoiseTempReceiver",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyNoiseTempReceiver(pub NoiseTempReceiver);

#[pymethods]
impl PyNoiseTempReceiver {
    #[new]
    fn new(band: PyFrequencyRange, noise_temperature: PyTemperature) -> PyResult<Self> {
        NoiseTempReceiver::new(band.0, noise_temperature.0)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Supported frequency range.
    #[getter]
    fn band(&self) -> PyFrequencyRange {
        PyFrequencyRange(self.0.band())
    }

    /// System noise temperature.
    #[getter]
    fn noise_temperature(&self) -> PyTemperature {
        PyTemperature(self.0.noise_temperature())
    }

    fn __eq__(&self, other: &PyNoiseTempReceiver) -> bool {
        self.0.band() == other.0.band()
            && self.0.noise_temperature().to_kelvin() == other.0.noise_temperature().to_kelvin()
    }

    fn __getnewargs__(&self) -> (PyFrequencyRange, PyTemperature) {
        (
            PyFrequencyRange(self.0.band()),
            PyTemperature(self.0.noise_temperature()),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "NoiseTempReceiver(band={}, noise_temperature={})",
            PyFrequencyRange(self.0.band()).__repr__(),
            PyTemperature(self.0.noise_temperature()).__repr__(),
        )
    }
}

// --- Noise Stage ---

/// A single stage in an RF receiver chain.
///
/// Args:
///     gain: Stage gain as Decibel.
///     noise_temperature: Stage equivalent noise temperature.
#[pyclass(name = "NoiseStage", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyNoiseStage(pub NoiseStage);

#[pymethods]
impl PyNoiseStage {
    #[new]
    fn new(gain: PyDecibel, noise_temperature: PyTemperature) -> PyResult<Self> {
        NoiseStage::new(gain.0, noise_temperature.0)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    fn __getnewargs__(&self) -> (PyDecibel, PyTemperature) {
        (
            PyDecibel(self.0.gain()),
            PyTemperature(self.0.noise_temperature()),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "NoiseStage(gain={}, noise_temperature={})",
            PyDecibel(self.0.gain()).__repr__(),
            PyTemperature(self.0.noise_temperature()).__repr__(),
        )
    }
}

// --- Cascade Receiver ---

/// An N-stage cascade receiver using the Friis noise formula.
///
/// Args:
///     band: Supported frequency range.
///     stages: List of NoiseStage (ordered: LNA first, then downstream).
///     demodulator_loss: Demodulator loss as Decibel (default Decibel(0)).
///     implementation_loss: Other implementation losses as Decibel (default Decibel(0)).
#[pyclass(name = "CascadeReceiver", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyCascadeReceiver(pub CascadeReceiver);

#[pymethods]
impl PyCascadeReceiver {
    #[new]
    #[pyo3(signature = (band, stages, demodulator_loss=None, implementation_loss=None))]
    fn new(
        band: PyFrequencyRange,
        stages: Vec<PyNoiseStage>,
        demodulator_loss: Option<PyDecibel>,
        implementation_loss: Option<PyDecibel>,
    ) -> PyResult<Self> {
        CascadeReceiver::new(
            band.0,
            stages.into_iter().map(|s| s.0).collect(),
            demodulator_loss.map_or(Decibel::new(0.0), |d| d.0),
            implementation_loss.map_or(Decibel::new(0.0), |d| d.0),
        )
        .map(Self)
        .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Creates a two-stage model: LNA → receiver (from noise figure).
    #[staticmethod]
    #[pyo3(signature = (band, lna_gain, lna_noise_temperature, receiver_noise_figure, demodulator_loss=None, implementation_loss=None))]
    fn from_lna_and_noise_figure(
        band: PyFrequencyRange,
        lna_gain: PyDecibel,
        lna_noise_temperature: PyTemperature,
        receiver_noise_figure: PyDecibel,
        demodulator_loss: Option<PyDecibel>,
        implementation_loss: Option<PyDecibel>,
    ) -> PyResult<Self> {
        CascadeReceiver::from_lna_and_noise_figure(
            band.0,
            lna_gain.0,
            lna_noise_temperature.0,
            receiver_noise_figure.0,
            demodulator_loss.map_or(Decibel::new(0.0), |d| d.0),
            implementation_loss.map_or(Decibel::new(0.0), |d| d.0),
        )
        .map(Self)
        .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the chain's equivalent noise temperature referred to its
    /// input connector, via the Friis formula.
    fn chain_noise_temperature(&self) -> PyTemperature {
        PyTemperature(self.0.chain_noise_temperature())
    }

    /// Returns the total RF chain gain in dB.
    fn chain_gain(&self) -> PyDecibel {
        PyDecibel(self.0.chain_gain())
    }

    /// Supported frequency range.
    #[getter]
    fn band(&self) -> PyFrequencyRange {
        PyFrequencyRange(self.0.band())
    }

    fn __eq__(&self, other: &PyCascadeReceiver) -> bool {
        self.0.band() == other.0.band()
            && self.0.stages().len() == other.0.stages().len()
            && self
                .0
                .stages()
                .iter()
                .zip(other.0.stages().iter())
                .all(|(a, b)| {
                    a.gain().as_f64() == b.gain().as_f64()
                        && a.noise_temperature() == b.noise_temperature()
                })
            && self.0.demodulator_loss().as_f64() == other.0.demodulator_loss().as_f64()
            && self.0.implementation_loss().as_f64() == other.0.implementation_loss().as_f64()
    }

    fn __getnewargs__(
        &self,
    ) -> (
        PyFrequencyRange,
        Vec<PyNoiseStage>,
        Option<PyDecibel>,
        Option<PyDecibel>,
    ) {
        (
            PyFrequencyRange(self.0.band()),
            self.0
                .stages()
                .iter()
                .map(|s| PyNoiseStage(s.clone()))
                .collect(),
            Some(PyDecibel(self.0.demodulator_loss())),
            Some(PyDecibel(self.0.implementation_loss())),
        )
    }

    fn __repr__(&self) -> String {
        let stages_repr: Vec<String> = self
            .0
            .stages()
            .iter()
            .map(|s| {
                format!(
                    "NoiseStage(gain={}, noise_temperature={})",
                    PyDecibel(s.gain()).__repr__(),
                    PyTemperature(s.noise_temperature()).__repr__(),
                )
            })
            .collect();
        format!(
            "CascadeReceiver(band={}, stages=[{}], demodulator_loss={}, implementation_loss={})",
            PyFrequencyRange(self.0.band()).__repr__(),
            stages_repr.join(", "),
            PyDecibel(self.0.demodulator_loss()).__repr__(),
            PyDecibel(self.0.implementation_loss()).__repr__(),
        )
    }
}

// --- Channel ---

/// A communication channel.
///
/// Args:
///     link_type: "uplink", "downlink", or "crosslink".
///     symbol_rate: Symbol rate in symbols per second.
///     required_eb_n0: Required Eb/N0 as Decibel.
///     margin: Required link margin as Decibel.
///     modulation: Modulation scheme.
///     roll_off: Roll-off factor (default 0.35).
///     fec: Forward error correction code rate (default 0.5).
///     chip_rate: Chip rate for DSSS in chips per second (optional).
#[pyclass(name = "Channel", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyChannel(pub Channel);

fn parse_link_direction(s: &str) -> PyResult<LinkDirection> {
    s.parse().map_err(|e: String| PyValueError::new_err(e))
}

#[pymethods]
impl PyChannel {
    #[new]
    #[pyo3(signature = (link_type, symbol_rate, required_eb_n0, margin, modulation, roll_off=0.35, fec=0.5, chip_rate=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        link_type: &str,
        symbol_rate: PyFrequency,
        required_eb_n0: PyDecibel,
        margin: PyDecibel,
        modulation: &PyModulation,
        roll_off: f64,
        fec: f64,
        chip_rate: Option<PyFrequency>,
    ) -> PyResult<Self> {
        let lt = parse_link_direction(link_type)?;
        Ok(Self(Channel {
            link_type: lt,
            symbol_rate: symbol_rate.0,
            required_eb_n0: required_eb_n0.0,
            margin: margin.0,
            modulation: modulation.0,
            roll_off,
            fec,
            chip_rate: chip_rate.map(|cr| cr.0),
        }))
    }

    /// Returns the raw bit rate.
    fn data_rate(&self) -> PyFrequency {
        PyFrequency(self.0.data_rate())
    }

    /// Returns the information (post-FEC) bit rate.
    fn information_rate(&self) -> PyFrequency {
        PyFrequency(self.0.information_rate())
    }

    /// Returns the occupied channel bandwidth.
    fn bandwidth(&self) -> PyFrequency {
        PyFrequency(self.0.bandwidth())
    }

    /// Computes Es/N0 from a given C/N0.
    fn es_n0(&self, c_n0: &PyDecibel) -> PyDecibel {
        PyDecibel(self.0.es_n0(c_n0.0))
    }

    /// Computes Eb/N0 from a given C/N0.
    fn eb_n0(&self, c_n0: &PyDecibel) -> PyDecibel {
        PyDecibel(self.0.eb_n0(c_n0.0))
    }

    /// Computes C/N from a given C/N0.
    fn c_n(&self, c_n0: &PyDecibel) -> PyDecibel {
        PyDecibel(self.0.c_n(c_n0.0))
    }

    /// Computes the link margin from a given Eb/N0.
    fn link_margin(&self, eb_n0: &PyDecibel) -> PyDecibel {
        PyDecibel(self.0.link_margin(eb_n0.0))
    }

    /// Returns the DSSS spreading factor, or None for narrowband.
    fn spreading_factor(&self) -> Option<f64> {
        self.0.spreading_factor()
    }

    /// Returns the DSSS processing gain in dB, or None for narrowband.
    fn processing_gain(&self) -> Option<PyDecibel> {
        self.0.processing_gain().map(PyDecibel)
    }

    /// Layers modulation/FEC figures onto a modulation-agnostic link budget.
    fn apply(&self, link: PyLinkStats) -> PyModulatedLinkStats {
        PyModulatedLinkStats(self.0.apply(link.0))
    }

    #[allow(clippy::type_complexity)]
    fn __getnewargs__<'py>(
        &self,
        py: Python<'py>,
    ) -> (
        &str,
        PyFrequency,
        PyDecibel,
        PyDecibel,
        Bound<'py, PyAny>,
        f64,
        f64,
        Option<PyFrequency>,
    ) {
        let lt = match self.0.link_type {
            LinkDirection::Uplink => "uplink",
            LinkDirection::Downlink => "downlink",
            LinkDirection::Crosslink => "crosslink",
            _ => unreachable!("unknown link direction variant"),
        };
        let modulation = Bound::new(py, PyModulation(self.0.modulation))
            .unwrap()
            .into_any();
        (
            lt,
            PyFrequency(self.0.symbol_rate),
            PyDecibel(self.0.required_eb_n0),
            PyDecibel(self.0.margin),
            modulation,
            self.0.roll_off,
            self.0.fec,
            self.0.chip_rate.map(PyFrequency),
        )
    }

    fn __repr__(&self) -> String {
        let chip = self.0.chip_rate.map_or(String::new(), |cr| {
            format!(", chip_rate={}", PyFrequency(cr).__repr__())
        });
        format!(
            "Channel(link_type='{}', symbol_rate={}, required_eb_n0={}, margin={}, modulation=Modulation('{}'), roll_off={}, fec={}{})",
            self.0.link_type,
            PyFrequency(self.0.symbol_rate).__repr__(),
            PyDecibel(self.0.required_eb_n0).__repr__(),
            PyDecibel(self.0.margin).__repr__(),
            modulation_name(self.0.modulation),
            repr_f64(self.0.roll_off),
            repr_f64(self.0.fec),
            chip,
        )
    }
}

/// Builds a [`Pointing`] from mutually exclusive angle/direction arguments.
fn build_pointing(
    angle: Option<PyAngle>,
    direction: Option<[f64; 3]>,
    endpoint: &str,
) -> PyResult<Pointing> {
    match (angle, direction) {
        (Some(_), Some(_)) => {
            let (angle_kwarg, direction_kwarg) = if endpoint.is_empty() {
                ("angle".to_owned(), "direction".to_owned())
            } else {
                (format!("{endpoint}_angle"), format!("{endpoint}_direction"))
            };
            Err(PyValueError::new_err(format!(
                "specify either {angle_kwarg} or {direction_kwarg}, not both"
            )))
        }
        (Some(angle), None) => Ok(Pointing::off_boresight(angle.0)),
        (None, Some(direction)) => Ok(Pointing::Direction(DVec3::from_array(direction))),
        (None, None) => Ok(Pointing::Boresight),
    }
}

// --- Link Stats ---

/// Modulation-agnostic link budget statistics.
#[pyclass(name = "LinkStats", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyLinkStats(pub LinkStats);

#[pymethods]
impl PyLinkStats {
    /// Computes a modulation-agnostic link budget between payload terminals.
    ///
    /// Resolves the TX and RX terminals and evaluates the
    /// link at the given carrier. The carrier must lie inside both
    /// terminals' effective frequency ranges. Each endpoint's pointing is
    /// given either as an off-boresight angle or as a line-of-sight
    /// direction vector in the antenna's parent frame; omitting both
    /// assumes ideal (boresight) pointing.
    ///
    /// Args:
    ///     tx_payload: The transmitting CommsPayload.
    ///     tx_terminal: Terminal of the transmitting payload.
    ///     rx_payload: The receiving CommsPayload.
    ///     rx_terminal: Terminal of the receiving payload.
    ///     carrier: Carrier frequency.
    ///     bandwidth: Noise bandwidth as Frequency.
    ///     range: Slant range as Distance.
    ///     direction: Link direction ("uplink", "downlink", or "crosslink").
    ///     tx_angle: Off-boresight angle at transmitter as Angle (optional).
    ///     rx_angle: Off-boresight angle at receiver as Angle (optional).
    ///     tx_direction: Line-of-sight direction at transmitter as [x, y, z] (optional).
    ///     rx_direction: Line-of-sight direction at receiver as [x, y, z] (optional).
    ///     losses: EnvironmentalLosses (optional, defaults to none).
    #[staticmethod]
    #[pyo3(signature = (tx_payload, tx_terminal, rx_payload, rx_terminal, carrier, bandwidth, range, direction, tx_angle=None, rx_angle=None, tx_direction=None, rx_direction=None, losses=None))]
    #[allow(clippy::too_many_arguments)]
    fn for_link(
        tx_payload: &PyCommsPayload,
        tx_terminal: PyTerminalId,
        rx_payload: &PyCommsPayload,
        rx_terminal: PyTerminalId,
        carrier: PyFrequency,
        bandwidth: PyFrequency,
        range: PyDistance,
        direction: &str,
        tx_angle: Option<PyAngle>,
        rx_angle: Option<PyAngle>,
        tx_direction: Option<[f64; 3]>,
        rx_direction: Option<[f64; 3]>,
        losses: Option<&PyEnvironmentalLosses>,
    ) -> PyResult<Self> {
        let direction: LinkDirection = direction
            .parse()
            .map_err(|err: String| PyValueError::new_err(err))?;
        let tx_pointing = build_pointing(tx_angle, tx_direction, "tx")?;
        let rx_pointing = build_pointing(rx_angle, rx_direction, "rx")?;
        let env_losses = losses
            .map(|l| l.0.clone())
            .unwrap_or_else(EnvironmentalLosses::none);

        let resolve_tx = tx_payload
            .0
            .resolve_tx(tx_terminal.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let resolve_rx = rx_payload
            .0
            .resolve_rx(rx_terminal.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        LinkStats::for_link(
            &resolve_tx,
            &resolve_rx,
            carrier.0,
            bandwidth.0,
            range.0,
            env_losses,
            tx_pointing,
            rx_pointing,
            direction,
        )
        .map(Self)
        .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Slant range.
    #[getter]
    fn slant_range(&self) -> PyDistance {
        PyDistance(self.0.slant_range)
    }

    /// Free-space path loss in dB.
    #[getter]
    fn fspl(&self) -> PyDecibel {
        PyDecibel(self.0.fspl)
    }

    /// EIRP in dBW.
    #[getter]
    fn eirp(&self) -> PyDecibel {
        PyDecibel(self.0.eirp)
    }

    /// Receiver G/T in dB/K.
    #[getter]
    fn gt(&self) -> PyDecibel {
        PyDecibel(self.0.gt)
    }

    /// Carrier-to-noise density ratio in dB·Hz.
    #[getter]
    fn c_n0(&self) -> PyDecibel {
        PyDecibel(self.0.c_n0)
    }

    /// C/N in dB.
    #[getter]
    fn c_n(&self) -> PyDecibel {
        PyDecibel(self.0.c_n)
    }

    /// Received carrier power in dBW. ``None`` for lumped G/T receivers.
    #[getter]
    fn carrier_rx_power(&self) -> Option<PyDecibel> {
        self.0.carrier_rx_power.map(PyDecibel)
    }

    /// Noise power in dBW. ``None`` for lumped G/T receivers.
    #[getter]
    fn noise_power(&self) -> Option<PyDecibel> {
        self.0.noise_power.map(PyDecibel)
    }

    /// Channel noise bandwidth.
    #[getter]
    fn bandwidth(&self) -> PyFrequency {
        PyFrequency(self.0.bandwidth)
    }

    /// Link frequency.
    #[getter]
    fn frequency(&self) -> PyFrequency {
        PyFrequency(self.0.frequency)
    }

    /// Derived TX pattern polar angle from boresight.
    #[getter]
    fn tx_theta(&self) -> PyAngle {
        PyAngle(self.0.tx_theta)
    }

    /// Derived TX pattern azimuth about boresight.
    #[getter]
    fn tx_phi(&self) -> PyAngle {
        PyAngle(self.0.tx_phi)
    }

    /// Derived RX pattern polar angle from boresight.
    #[getter]
    fn rx_theta(&self) -> PyAngle {
        PyAngle(self.0.rx_theta)
    }

    /// Derived RX pattern azimuth about boresight.
    #[getter]
    fn rx_phi(&self) -> PyAngle {
        PyAngle(self.0.rx_phi)
    }

    /// Link direction ("uplink", "downlink", or "crosslink").
    #[getter]
    fn direction(&self) -> String {
        self.0.direction.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "LinkStats(c_n0={:.2} dB·Hz, c_n={:.2} dB, eirp={:.2} dBW, gt={:.2} dB/K)",
            self.0.c_n0.as_f64(),
            self.0.c_n.as_f64(),
            self.0.eirp.as_f64(),
            self.0.gt.as_f64(),
        )
    }
}

// --- Interference Stats ---

/// Interference statistics for a link with a given interferer power.
#[pyclass(
    name = "InterferenceStats",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyInterferenceStats(pub InterferenceStats);

#[pymethods]
impl PyInterferenceStats {
    /// Interference power.
    #[getter]
    fn interference_power(&self) -> PyPower {
        PyPower(self.0.interference_power)
    }

    /// Carrier-to-noise-plus-interference density ratio in dB·Hz.
    #[getter]
    fn c_n0i0(&self) -> PyDecibel {
        PyDecibel(self.0.c_n0i0)
    }

    /// Eb/(N0+I0) in dB.
    #[getter]
    fn eb_n0i0(&self) -> PyDecibel {
        PyDecibel(self.0.eb_n0i0)
    }

    /// Link margin with interference in dB.
    #[getter]
    fn margin_with_interference(&self) -> PyDecibel {
        PyDecibel(self.0.margin_with_interference)
    }

    fn __repr__(&self) -> String {
        format!(
            "InterferenceStats(interference_power={} W, c_n0i0={:.2} dB·Hz, eb_n0i0={:.2} dB, margin_with_interference={:.2} dB)",
            repr_f64(self.0.interference_power.to_watts()),
            self.0.c_n0i0.as_f64(),
            self.0.eb_n0i0.as_f64(),
            self.0.margin_with_interference.as_f64(),
        )
    }
}

// --- Modulated Link Stats ---

/// Link-budget output with modulation/coding figures applied.
#[pyclass(
    name = "ModulatedLinkStats",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyModulatedLinkStats(pub ModulatedLinkStats);

#[pymethods]
impl PyModulatedLinkStats {
    /// The underlying modulation-agnostic link budget.
    #[getter]
    fn link(&self) -> PyLinkStats {
        PyLinkStats(self.0.link.clone())
    }

    /// The channel (modulation, FEC, required Eb/N0, margin) applied.
    #[getter]
    fn channel(&self) -> PyChannel {
        PyChannel(self.0.channel.clone())
    }

    /// Symbol rate from the channel.
    #[getter]
    fn symbol_rate(&self) -> PyFrequency {
        PyFrequency(self.0.symbol_rate)
    }

    /// Es/N0 (energy per symbol to noise spectral density) in dB.
    #[getter]
    fn es_n0(&self) -> PyDecibel {
        PyDecibel(self.0.es_n0)
    }

    /// Eb/N0 (energy per information bit to noise spectral density) in dB.
    #[getter]
    fn eb_n0(&self) -> PyDecibel {
        PyDecibel(self.0.eb_n0)
    }

    /// Link margin in dB.
    #[getter]
    fn margin(&self) -> PyDecibel {
        PyDecibel(self.0.margin)
    }

    /// Computes interference statistics for a given interferer power.
    fn with_interference(&self, interference_power: PyPower) -> PyResult<PyInterferenceStats> {
        self.0
            .with_interference(interference_power.0)
            .map(PyInterferenceStats)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "ModulatedLinkStats(eb_n0={:.2} dB, margin={:.2} dB)",
            self.0.eb_n0.as_f64(),
            self.0.margin.as_f64(),
        )
    }
}

// --- Free functions ---

/// Computes the free-space path loss in dB.
///
/// Args:
///     distance: Distance.
///     frequency: Frequency.
///
/// Returns:
///     Free-space path loss as a Decibel value.
#[pyfunction]
pub fn fspl(distance: PyDistance, frequency: PyFrequency) -> PyDecibel {
    PyDecibel(free_space_path_loss(distance.0, frequency.0))
}

/// Computes the frequency overlap factor between a receiver and an interferer.
///
/// Args:
///     rx_freq: Receiver center frequency.
///     rx_bw: Receiver bandwidth.
///     tx_freq: Interferer center frequency.
///     tx_bw: Interferer bandwidth.
///
/// Returns:
///     Overlap factor in [0, 1].
#[pyfunction]
pub fn freq_overlap(
    rx_freq: PyFrequency,
    rx_bw: PyFrequency,
    tx_freq: PyFrequency,
    tx_bw: PyFrequency,
) -> f64 {
    frequency_overlap_factor(rx_freq.0, rx_bw.0, tx_freq.0, tx_bw.0)
}

/// Computes the power flux density in dBW/m²/ref_bw.
///
/// Args:
///     eirp: EIRP as Decibel.
///     distance: Distance.
///     occupied_bw: Occupied bandwidth as Frequency.
///     reference_bw: ITU reference bandwidth as Frequency.
///
/// Returns:
///     PFD as Decibel.
#[pyfunction]
pub fn power_flux_density(
    eirp: PyDecibel,
    distance: PyDistance,
    occupied_bw: PyFrequency,
    reference_bw: PyFrequency,
) -> PyDecibel {
    PyDecibel(pfd::power_flux_density(
        eirp.0,
        distance.0,
        occupied_bw.0,
        reference_bw.0,
    ))
}

/// A piecewise-linear PFD mask over elevation in dBW/m²/ref_bw.
///
/// The mask is linear in elevation between consecutive breakpoints and constant
/// below the first and above the last.
///
/// Args:
///     nodes: (elevation, value) breakpoints with strictly ascending elevations.
#[pyclass(name = "PfdMask", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyPfdMask(pub pfd::PfdMask);

#[pymethods]
impl PyPfdMask {
    #[new]
    fn new(nodes: Vec<(PyAngle, PyDecibel)>) -> PyResult<Self> {
        pfd::PfdMask::new(
            nodes
                .into_iter()
                .map(|(elevation, value)| (elevation.0, value.0))
                .collect(),
        )
        .map(Self)
        .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// The ITU RR Article 21.16 mask shape for a given low-elevation limit.
    ///
    /// Rises from `start` at 5° elevation by 0.5 dB per degree to `start + 10 dB`
    /// at 25° and is constant outside that range.
    #[staticmethod]
    fn art_21_16(start: PyDecibel) -> Self {
        Self(pfd::PfdMask::art_21_16(start.0))
    }

    /// Returns the mask value at the given elevation angle.
    fn value_at(&self, elevation: PyAngle) -> PyDecibel {
        PyDecibel(self.0.value_at(elevation.0))
    }

    /// Returns the mask breakpoints as (elevation, value) tuples.
    fn nodes(&self) -> Vec<(PyAngle, PyDecibel)> {
        self.0
            .nodes()
            .iter()
            .map(|&(elevation, value)| (PyAngle(elevation), PyDecibel(value)))
            .collect()
    }

    fn __eq__(&self, other: &PyPfdMask) -> bool {
        self.0 == other.0
    }

    fn __getnewargs__(&self) -> (Vec<(PyAngle, PyDecibel)>,) {
        (self.nodes(),)
    }

    fn __repr__(&self) -> String {
        let nodes = self
            .0
            .nodes()
            .iter()
            .map(|(elevation, value)| {
                format!(
                    "(Angle({}), Decibel({}))",
                    repr_f64(elevation.to_radians()),
                    repr_f64(value.as_f64())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("PfdMask(nodes=[{nodes}])")
    }
}

/// Computes the slant range from a ground station to a satellite.
///
/// Args:
///     elevation: Elevation angle.
///     earth_radius: Earth radius as Distance.
///     altitude: Satellite altitude as Distance.
///
/// Returns:
///     Slant range as Distance.
#[pyfunction]
pub fn slant_range(
    elevation: PyAngle,
    earth_radius: PyDistance,
    altitude: PyDistance,
) -> PyDistance {
    PyDistance(comms_slant_range(elevation.0, earth_radius.0, altitude.0))
}

// --- Frequency ranges ---

/// A contiguous frequency range with inclusive bounds.
///
/// Args:
///     min: Lower frequency bound.
///     max: Upper frequency bound.
#[pyclass(
    name = "FrequencyRange",
    module = "lox_space",
    frozen,
    eq,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PyFrequencyRange(pub FrequencyRange);

#[pymethods]
impl PyFrequencyRange {
    #[new]
    fn new(min: PyFrequency, max: PyFrequency) -> PyResult<Self> {
        FrequencyRange::new(min.0, max.0)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Creates a frequency range from wavelength bounds (e.g. optical bands).
    #[staticmethod]
    fn from_wavelengths(min_wavelength: PyDistance, max_wavelength: PyDistance) -> PyResult<Self> {
        FrequencyRange::from_wavelengths(min_wavelength.0, max_wavelength.0)
            .map(Self)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the lower frequency bound.
    fn min(&self) -> PyFrequency {
        PyFrequency(self.0.min())
    }

    /// Returns the upper frequency bound.
    fn max(&self) -> PyFrequency {
        PyFrequency(self.0.max())
    }

    /// Returns whether the frequency lies within the range (bounds inclusive).
    fn contains(&self, frequency: PyFrequency) -> bool {
        self.0.contains(frequency.0)
    }

    fn __getnewargs__(&self) -> (PyFrequency, PyFrequency) {
        (self.min(), self.max())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "FrequencyRange(min=Frequency({}), max=Frequency({}))",
            repr_f64(self.0.min().to_hertz()),
            repr_f64(self.0.max().to_hertz()),
        )
    }
}

// --- Comms payload (inventory + wiring) ---

macro_rules! py_payload_id {
    ($(#[$doc:meta])* $pyname:literal, $id:ty, $wrapper:ident) => {
        $(#[$doc])*
        #[pyclass(name = $pyname, module = "lox_space", frozen, eq, from_py_object)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $wrapper(pub $id);

        #[pymethods]
        impl $wrapper {
            fn __repr__(&self) -> String {
                format!("<{} {:?}>", $pyname, self.0)
            }
        }
    };
}

py_payload_id!(
    /// Identifier of an antenna in a CommsPayload. Only valid for the payload
    /// that minted it.
    "AntennaId", lox_comms::payload::AntennaId, PyAntennaId
);
py_payload_id!(
    /// Identifier of a transmitter in a CommsPayload. Only valid for the
    /// payload that minted it.
    "TransmitterId", lox_comms::payload::TransmitterId, PyTransmitterId
);
py_payload_id!(
    /// Identifier of a receiver in a CommsPayload. Only valid for the payload
    /// that minted it.
    "ReceiverId", lox_comms::payload::ReceiverId, PyReceiverId
);
py_payload_id!(
    /// Identifier of a lumped EIRP model in a CommsPayload. Only valid for
    /// the payload that minted it.
    "EirpModelId", lox_comms::payload::EirpModelId, PyEirpModelId
);
py_payload_id!(
    /// Identifier of a lumped G/T model in a CommsPayload. Only valid for the
    /// payload that minted it.
    "GtModelId", lox_comms::payload::GtModelId, PyGtModelId
);
py_payload_id!(
    /// Identifier of a transmit port in a CommsPayload. Only valid for the
    /// payload that minted it.
    "TxPortId", lox_comms::payload::TxPortId, PyTxPortId
);
py_payload_id!(
    /// Identifier of a receive port in a CommsPayload. Only valid for the
    /// payload that minted it.
    "RxPortId", lox_comms::payload::RxPortId, PyRxPortId
);
py_payload_id!(
    /// Identifier of a terminal in a CommsPayload. Only valid for the payload
    /// that minted it.
    "TerminalId", lox_comms::payload::TerminalId, PyTerminalId
);

fn build_tx_chain(
    port: Option<PyTxPortId>,
    eirp_model: Option<PyEirpModelId>,
) -> PyResult<TxChain> {
    match (port, eirp_model) {
        (Some(port), None) => Ok(TxChain::Component(port.0)),
        (None, Some(model)) => Ok(TxChain::Lumped(model.0)),
        _ => Err(PyValueError::new_err(
            "specify exactly one of port or eirp_model",
        )),
    }
}

fn build_rx_chain(port: Option<PyRxPortId>, gt_model: Option<PyGtModelId>) -> PyResult<RxChain> {
    match (port, gt_model) {
        (Some(port), None) => Ok(RxChain::Component(port.0)),
        (None, Some(model)) => Ok(RxChain::Lumped(model.0)),
        _ => Err(PyValueError::new_err(
            "specify exactly one of port or gt_model",
        )),
    }
}

/// Communications hardware inventory and wiring for one platform.
///
/// Owns antennas, radios, lumped models, ports, and terminals. Wiring is by
/// ID and validated at insertion; names are display-only. The payload holds
/// no operational state: carrier, bandwidth, modulation, and pointing are
/// link-level inputs.
#[pyclass(name = "CommsPayload", module = "lox_space", from_py_object)]
#[derive(Debug, Clone, Default)]
pub struct PyCommsPayload(pub CommsPayload);

#[pymethods]
impl PyCommsPayload {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    /// Adds an antenna (ConstantAntenna or PatternedAntenna) to the inventory.
    fn add_antenna(&mut self, name: String, antenna: &Bound<'_, PyAny>) -> PyResult<PyAntennaId> {
        Ok(PyAntennaId(
            self.0.add_antenna(name, build_antenna(antenna)?),
        ))
    }

    /// Adds a component-tier transmitter to the inventory.
    fn add_transmitter(
        &mut self,
        name: String,
        transmitter: PyAmplifierTransmitter,
    ) -> PyResult<PyTransmitterId> {
        Ok(PyTransmitterId(
            self.0.add_transmitter(name, transmitter.0.clone()),
        ))
    }

    /// Adds a component-tier receiver (NoiseTempReceiver or CascadeReceiver)
    /// to the inventory.
    fn add_receiver(
        &mut self,
        name: String,
        receiver: &Bound<'_, PyAny>,
    ) -> PyResult<PyReceiverId> {
        Ok(PyReceiverId(
            self.0.add_receiver(name, build_receiver_any(receiver)?),
        ))
    }

    /// Adds a lumped EIRP model to the inventory.
    fn add_eirp_model(
        &mut self,
        name: String,
        band: PyFrequencyRange,
        eirp: PyDecibel,
    ) -> PyResult<PyEirpModelId> {
        let model = EirpModel::new(name, band.0, eirp.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(PyEirpModelId(self.0.add_eirp_model(model)))
    }

    /// Adds a lumped G/T model to the inventory.
    fn add_gt_model(
        &mut self,
        name: String,
        band: PyFrequencyRange,
        gt: PyDecibel,
    ) -> PyResult<PyGtModelId> {
        let model = GtModel::new(name, band.0, gt.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(PyGtModelId(self.0.add_gt_model(model)))
    }

    /// Adds a transmit port wiring an antenna to a transmitter.
    ///
    /// ``feed_loss`` defaults to a lossless feed (0 dB).
    #[pyo3(signature = (name, antenna, transmitter, feed_loss=None, band=None))]
    fn add_tx_port(
        &mut self,
        name: String,
        antenna: PyAntennaId,
        transmitter: PyTransmitterId,
        feed_loss: Option<PyDecibel>,
        band: Option<PyFrequencyRange>,
    ) -> PyResult<PyTxPortId> {
        let port = TxPort::new(
            name,
            antenna.0,
            transmitter.0,
            feed_loss.map_or(Decibel::new(0.0), |l| l.0),
            band.map(|b| b.0),
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        self.0
            .add_tx_port(port)
            .map(PyTxPortId)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Adds a receive port wiring an antenna to a receiver.
    ///
    /// ``feed_loss`` defaults to a lossless feed (0 dB) and
    /// ``antenna_noise_temperature`` to 0 K.
    #[pyo3(signature = (name, antenna, receiver, feed_loss=None, antenna_noise_temperature=None, band=None))]
    fn add_rx_port(
        &mut self,
        name: String,
        antenna: PyAntennaId,
        receiver: PyReceiverId,
        feed_loss: Option<PyDecibel>,
        antenna_noise_temperature: Option<PyTemperature>,
        band: Option<PyFrequencyRange>,
    ) -> PyResult<PyRxPortId> {
        let port = RxPort::new(
            name,
            antenna.0,
            receiver.0,
            feed_loss.map_or(Decibel::new(0.0), |l| l.0),
            antenna_noise_temperature.map_or(Temperature::kelvin(0.0), |t| t.0),
            band.map(|b| b.0),
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        self.0
            .add_rx_port(port)
            .map(PyRxPortId)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Adds a transmit-only terminal from a port or a lumped EIRP model.
    #[pyo3(signature = (name, port=None, eirp_model=None))]
    fn add_tx_terminal(
        &mut self,
        name: String,
        port: Option<PyTxPortId>,
        eirp_model: Option<PyEirpModelId>,
    ) -> PyResult<PyTerminalId> {
        self.0
            .add_terminal(Terminal {
                name,
                role: TerminalRole::Tx(build_tx_chain(port, eirp_model)?),
            })
            .map(PyTerminalId)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Adds a receive-only terminal from a port or a lumped G/T model.
    #[pyo3(signature = (name, port=None, gt_model=None))]
    fn add_rx_terminal(
        &mut self,
        name: String,
        port: Option<PyRxPortId>,
        gt_model: Option<PyGtModelId>,
    ) -> PyResult<PyTerminalId> {
        self.0
            .add_terminal(Terminal {
                name,
                role: TerminalRole::Rx(build_rx_chain(port, gt_model)?),
            })
            .map(PyTerminalId)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Adds a transceiver terminal with one chain per direction.
    #[pyo3(signature = (name, tx_port=None, rx_port=None, eirp_model=None, gt_model=None))]
    fn add_transceiver_terminal(
        &mut self,
        name: String,
        tx_port: Option<PyTxPortId>,
        rx_port: Option<PyRxPortId>,
        eirp_model: Option<PyEirpModelId>,
        gt_model: Option<PyGtModelId>,
    ) -> PyResult<PyTerminalId> {
        self.0
            .add_terminal(Terminal {
                name,
                role: TerminalRole::Transceiver {
                    tx: build_tx_chain(tx_port, eirp_model)?,
                    rx: build_rx_chain(rx_port, gt_model)?,
                },
            })
            .map(PyTerminalId)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the first terminal with the given name, if any.
    fn find_terminal(&self, name: &str) -> Option<PyTerminalId> {
        self.0.find_terminal(name).map(PyTerminalId)
    }

    /// Lists all terminals as (id, name, kind) with kind one of
    /// "tx", "rx", or "transceiver".
    fn terminals(&self) -> Vec<(PyTerminalId, String, &'static str)> {
        self.0
            .terminals()
            .map(|(id, terminal)| {
                let kind = match terminal.role {
                    TerminalRole::Tx(_) => "tx",
                    TerminalRole::Rx(_) => "rx",
                    TerminalRole::Transceiver { .. } => "transceiver",
                };
                (PyTerminalId(id), terminal.name.clone(), kind)
            })
            .collect()
    }

    /// Returns a multi-line wiring summary for inspection.
    fn describe(&self) -> String {
        self.0.to_string()
    }

    /// Returns the effective transmit frequency range of a terminal.
    fn tx_band(&self, terminal: PyTerminalId) -> PyResult<PyFrequencyRange> {
        self.0
            .resolve_tx(terminal.0)
            .map(|endpoint| PyFrequencyRange(endpoint.band()))
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the effective receive frequency range of a terminal.
    fn rx_band(&self, terminal: PyTerminalId) -> PyResult<PyFrequencyRange> {
        self.0
            .resolve_rx(terminal.0)
            .map(|endpoint| PyFrequencyRange(endpoint.band()))
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the EIRP in dBW of a terminal at the given carrier and pointing.
    ///
    /// Pointing is given as an off-boresight angle or a line-of-sight
    /// direction vector; omitting both assumes boresight.
    #[pyo3(signature = (terminal, carrier, angle=None, direction=None))]
    fn eirp_at(
        &self,
        terminal: PyTerminalId,
        carrier: PyFrequency,
        angle: Option<PyAngle>,
        direction: Option<[f64; 3]>,
    ) -> PyResult<PyDecibel> {
        let pointing = build_pointing(angle, direction, "")?;
        let endpoint = self
            .0
            .resolve_tx(terminal.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        endpoint
            .eirp_at(carrier.0, pointing)
            .map(PyDecibel)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// Returns the G/T in dB/K of a terminal at the given carrier and pointing.
    ///
    /// Pointing is given as an off-boresight angle or a line-of-sight
    /// direction vector; omitting both assumes boresight.
    #[pyo3(signature = (terminal, carrier, angle=None, direction=None))]
    fn gt_at(
        &self,
        terminal: PyTerminalId,
        carrier: PyFrequency,
        angle: Option<PyAngle>,
        direction: Option<[f64; 3]>,
    ) -> PyResult<PyDecibel> {
        let pointing = build_pointing(angle, direction, "")?;
        let endpoint = self
            .0
            .resolve_rx(terminal.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        endpoint
            .gt_at(carrier.0, pointing)
            .map(PyDecibel)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    /// Creates a single-terminal transmit-only payload.
    #[staticmethod]
    #[pyo3(signature = (name, antenna, transmitter, feed_loss, band=None))]
    fn transmitter_only(
        name: String,
        antenna: &Bound<'_, PyAny>,
        transmitter: PyAmplifierTransmitter,
        feed_loss: PyDecibel,
        band: Option<PyFrequencyRange>,
    ) -> PyResult<(Self, PyTerminalId)> {
        let (payload, terminal) = CommsPayload::transmitter_only(
            name,
            build_antenna(antenna)?,
            transmitter.0.clone(),
            feed_loss.0,
            band.map(|b| b.0),
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok((Self(payload), PyTerminalId(terminal)))
    }

    /// Creates a single-terminal receive-only payload.
    #[staticmethod]
    #[pyo3(signature = (name, antenna, receiver, feed_loss, antenna_noise_temperature, band=None))]
    fn receiver_only(
        name: String,
        antenna: &Bound<'_, PyAny>,
        receiver: &Bound<'_, PyAny>,
        feed_loss: PyDecibel,
        antenna_noise_temperature: PyTemperature,
        band: Option<PyFrequencyRange>,
    ) -> PyResult<(Self, PyTerminalId)> {
        let (payload, terminal) = CommsPayload::receiver_only(
            name,
            build_antenna(antenna)?,
            build_receiver_any(receiver)?,
            feed_loss.0,
            antenna_noise_temperature.0,
            band.map(|b| b.0),
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok((Self(payload), PyTerminalId(terminal)))
    }

    /// Creates a single-terminal transceiver payload sharing one antenna.
    #[staticmethod]
    #[pyo3(signature = (name, antenna, transmitter, receiver, tx_feed_loss, rx_feed_loss, antenna_noise_temperature, band=None))]
    #[allow(clippy::too_many_arguments)]
    fn transceiver(
        name: String,
        antenna: &Bound<'_, PyAny>,
        transmitter: PyAmplifierTransmitter,
        receiver: &Bound<'_, PyAny>,
        tx_feed_loss: PyDecibel,
        rx_feed_loss: PyDecibel,
        antenna_noise_temperature: PyTemperature,
        band: Option<PyFrequencyRange>,
    ) -> PyResult<(Self, PyTerminalId)> {
        let (payload, terminal) = CommsPayload::transceiver(
            name,
            build_antenna(antenna)?,
            transmitter.0.clone(),
            build_receiver_any(receiver)?,
            tx_feed_loss.0,
            rx_feed_loss.0,
            antenna_noise_temperature.0,
            band.map(|b| b.0),
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok((Self(payload), PyTerminalId(terminal)))
    }

    /// Creates a single-terminal payload from a lumped EIRP model.
    #[staticmethod]
    fn eirp_only(
        name: String,
        band: PyFrequencyRange,
        eirp: PyDecibel,
    ) -> PyResult<(Self, PyTerminalId)> {
        let model = EirpModel::new(name, band.0, eirp.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let (payload, terminal) = CommsPayload::eirp_only(model);
        Ok((Self(payload), PyTerminalId(terminal)))
    }

    /// Creates a single-terminal payload from a lumped G/T model.
    #[staticmethod]
    fn gt_only(
        name: String,
        band: PyFrequencyRange,
        gt: PyDecibel,
    ) -> PyResult<(Self, PyTerminalId)> {
        let model = GtModel::new(name, band.0, gt.0)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let (payload, terminal) = CommsPayload::gt_only(model);
        Ok((Self(payload), PyTerminalId(terminal)))
    }

    fn __repr__(&self) -> String {
        format!(
            "<CommsPayload terminals=[{}]>",
            self.0
                .terminals()
                .map(|(_, t)| t.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}
