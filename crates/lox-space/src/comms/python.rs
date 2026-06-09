// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::string::String;

use lox_core::glam::DVec3;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use lox_comms::antenna::{Antenna, AntennaGain, ConstantAntenna, PatternedAntenna};
use lox_comms::channel::{Channel, LinkDirection, Modulation};
use lox_comms::link_budget::{
    InterferenceStats, LinkStats, ModulatedLinkStats, frequency_overlap_factor,
};
use lox_itur::EnvironmentalLosses;

use crate::itur::python::PyEnvironmentalLosses;
use lox_comms::pattern::{AntennaPattern, DipolePattern, GaussianPattern, ParabolicPattern};
use lox_comms::pfd;
use lox_comms::receiver::{CascadeReceiver, GtReceiver, NoiseStage, NoiseTempReceiver, Receiver};
use lox_comms::system::CommunicationSystem;
use lox_comms::transmitter::{AmplifierTransmitter, EirpTransmitter, Transmitter};
use lox_comms::utils::{free_space_path_loss, slant_range as comms_slant_range};
use lox_core::units::{Angle, Decibel};

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
    fn new(diameter: PyDistance, efficiency: f64) -> Self {
        Self(ParabolicPattern::new(diameter.0, efficiency))
    }

    /// Creates a parabolic pattern from a desired beamwidth.
    ///
    /// Args:
    ///     beamwidth: Half-power beamwidth as Angle.
    ///     frequency: Frequency.
    ///     efficiency: Aperture efficiency (0, 1].
    #[staticmethod]
    fn from_beamwidth(beamwidth: PyAngle, frequency: PyFrequency, efficiency: f64) -> Self {
        Self(ParabolicPattern::from_beamwidth(
            beamwidth.0,
            frequency.0,
            efficiency,
        ))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(self.0.gain(
            frequency.0,
            theta.0,
            phi.map_or(Angle::radians(0.0), |p| p.0),
        ))
    }

    /// Returns the half-power beamwidth, or ``None`` when the
    /// antenna diameter is smaller than ~1.22 wavelengths at this frequency.
    fn beamwidth(&self, frequency: PyFrequency) -> Option<PyAngle> {
        self.0.beamwidth(frequency.0).map(PyAngle)
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    fn __eq__(&self, other: &PyParabolicPattern) -> bool {
        self.0.diameter.to_meters() == other.0.diameter.to_meters()
            && self.0.efficiency == other.0.efficiency
    }

    fn __getnewargs__(&self) -> (PyDistance, f64) {
        (PyDistance(self.0.diameter), self.0.efficiency)
    }

    fn __repr__(&self) -> String {
        format!(
            "ParabolicPattern(diameter={}, efficiency={})",
            PyDistance(self.0.diameter).__repr__(),
            repr_f64(self.0.efficiency),
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
    fn new(diameter: PyDistance, efficiency: f64) -> Self {
        Self(GaussianPattern::new(diameter.0, efficiency))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(self.0.gain(
            frequency.0,
            theta.0,
            phi.map_or(Angle::radians(0.0), |p| p.0),
        ))
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
        self.0.diameter.to_meters() == other.0.diameter.to_meters()
            && self.0.efficiency == other.0.efficiency
    }

    fn __getnewargs__(&self) -> (PyDistance, f64) {
        (PyDistance(self.0.diameter), self.0.efficiency)
    }

    fn __repr__(&self) -> String {
        format!(
            "GaussianPattern(diameter={}, efficiency={})",
            PyDistance(self.0.diameter).__repr__(),
            repr_f64(self.0.efficiency),
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
    fn new(length: PyDistance) -> Self {
        Self(DipolePattern::new(length.0))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(self.0.gain(
            frequency.0,
            theta.0,
            phi.map_or(Angle::radians(0.0), |p| p.0),
        ))
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    fn __eq__(&self, other: &PyDipolePattern) -> bool {
        self.0.length.to_meters() == other.0.length.to_meters()
    }

    fn __getnewargs__(&self) -> (PyDistance,) {
        (PyDistance(self.0.length),)
    }

    fn __repr__(&self) -> String {
        format!(
            "DipolePattern(length={})",
            PyDistance(self.0.length).__repr__(),
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
    fn new(gain: PyDecibel) -> Self {
        Self {
            inner: ConstantAntenna { gain: gain.0 },
        }
    }

    fn __eq__(&self, other: &PyConstantAntenna) -> bool {
        self.inner.gain.as_f64() == other.inner.gain.as_f64()
    }

    fn __getnewargs__(&self) -> (PyDecibel,) {
        (PyDecibel(self.inner.gain),)
    }

    fn __repr__(&self) -> String {
        format!(
            "ConstantAntenna(gain={})",
            PyDecibel(self.inner.gain).__repr__(),
        )
    }
}

/// An antenna with a physics-based gain pattern and boresight vector.
///
/// Args:
///     pattern: An antenna pattern (ParabolicPattern, GaussianPattern, or DipolePattern).
///     boresight: Boresight direction as [x, y, z].
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
    fn new(pattern: &Bound<'_, PyAny>, boresight: [f64; 3]) -> PyResult<Self> {
        let pattern = extract_antenna_pattern(pattern)?;
        Ok(Self(PatternedAntenna {
            pattern,
            boresight: DVec3::from_array(boresight),
        }))
    }

    /// Returns the gain in dBi at the given frequency and pattern angles.
    #[pyo3(signature = (frequency, theta, phi=None))]
    fn gain(&self, frequency: PyFrequency, theta: PyAngle, phi: Option<PyAngle>) -> PyDecibel {
        PyDecibel(self.0.gain(
            frequency.0,
            theta.0,
            phi.map_or(Angle::radians(0.0), |p| p.0),
        ))
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency: PyFrequency) -> PyDecibel {
        PyDecibel(self.0.peak_gain(frequency.0))
    }

    /// Returns the half-power beamwidth, or ``None`` when the underlying
    /// pattern does not define one (e.g. ``DipolePattern``, or a
    /// ``ParabolicPattern`` whose diameter is below ~0.51 wavelengths).
    fn beamwidth(&self, frequency: PyFrequency) -> Option<PyAngle> {
        self.0.pattern.beamwidth(frequency.0).map(PyAngle)
    }

    fn __getnewargs__<'py>(&self, py: Python<'py>) -> (Bound<'py, PyAny>, [f64; 3]) {
        let pattern = pattern_to_py(py, &self.0.pattern);
        let b = self.0.boresight;
        (pattern, [b.x, b.y, b.z])
    }

    fn __repr__(&self) -> String {
        let pattern_repr = match &self.0.pattern {
            AntennaPattern::Parabolic(p) => format!(
                "ParabolicPattern(diameter={}, efficiency={})",
                PyDistance(p.diameter).__repr__(),
                repr_f64(p.efficiency),
            ),
            AntennaPattern::Gaussian(p) => format!(
                "GaussianPattern(diameter={}, efficiency={})",
                PyDistance(p.diameter).__repr__(),
                repr_f64(p.efficiency),
            ),
            AntennaPattern::Dipole(p) => {
                format!("DipolePattern(length={})", PyDistance(p.length).__repr__())
            }
            &_ => unreachable!("unknown antenna variant"),
        };
        let b = self.0.boresight;
        format!(
            "PatternedAntenna(pattern={pattern_repr}, boresight=[{}, {}, {}])",
            repr_f64(b.x),
            repr_f64(b.y),
            repr_f64(b.z),
        )
    }
}

fn extract_antenna_pattern(obj: &Bound<'_, PyAny>) -> PyResult<AntennaPattern> {
    if let Ok(p) = obj.extract::<PyRef<'_, PyParabolicPattern>>() {
        Ok(AntennaPattern::Parabolic(ParabolicPattern::new(
            p.0.diameter,
            p.0.efficiency,
        )))
    } else if let Ok(p) = obj.extract::<PyRef<'_, PyGaussianPattern>>() {
        Ok(AntennaPattern::Gaussian(GaussianPattern::new(
            p.0.diameter,
            p.0.efficiency,
        )))
    } else if let Ok(p) = obj.extract::<PyRef<'_, PyDipolePattern>>() {
        Ok(AntennaPattern::Dipole(DipolePattern::new(p.0.length)))
    } else {
        Err(PyValueError::new_err(
            "expected a ParabolicPattern, GaussianPattern, or DipolePattern",
        ))
    }
}

fn pattern_to_py<'py>(py: Python<'py>, pattern: &AntennaPattern) -> Bound<'py, PyAny> {
    match pattern {
        AntennaPattern::Parabolic(p) => Bound::new(
            py,
            PyParabolicPattern(ParabolicPattern::new(p.diameter, p.efficiency)),
        )
        .unwrap()
        .into_any(),
        AntennaPattern::Gaussian(p) => Bound::new(
            py,
            PyGaussianPattern(GaussianPattern::new(p.diameter, p.efficiency)),
        )
        .unwrap()
        .into_any(),
        AntennaPattern::Dipole(p) => Bound::new(py, PyDipolePattern(DipolePattern::new(p.length)))
            .unwrap()
            .into_any(),
        _ => unreachable!("unknown antenna variant"),
    }
}

fn antenna_to_py<'py>(py: Python<'py>, antenna: &Antenna) -> Bound<'py, PyAny> {
    match antenna {
        Antenna::Constant(a) => Bound::new(
            py,
            PyConstantAntenna {
                inner: ConstantAntenna { gain: a.gain },
            },
        )
        .unwrap()
        .into_any(),
        Antenna::Patterned(a) => {
            let pattern = match &a.pattern {
                AntennaPattern::Parabolic(p) => {
                    AntennaPattern::Parabolic(ParabolicPattern::new(p.diameter, p.efficiency))
                }
                AntennaPattern::Gaussian(p) => {
                    AntennaPattern::Gaussian(GaussianPattern::new(p.diameter, p.efficiency))
                }
                AntennaPattern::Dipole(p) => AntennaPattern::Dipole(DipolePattern::new(p.length)),
                _ => unreachable!("unknown antenna variant"),
            };
            Bound::new(
                py,
                PyPatternedAntenna(PatternedAntenna {
                    pattern,
                    boresight: a.boresight,
                }),
            )
            .unwrap()
            .into_any()
        }
        _ => unreachable!("unknown antenna variant"),
    }
}

fn receiver_to_py<'py>(py: Python<'py>, receiver: &Receiver) -> Bound<'py, PyAny> {
    match receiver {
        Receiver::NoiseTemperature(r) => Bound::new(
            py,
            PyNoiseTempReceiver(NoiseTempReceiver {
                frequency: r.frequency,
                system_noise_temperature: r.system_noise_temperature,
            }),
        )
        .unwrap()
        .into_any(),
        Receiver::Cascade(r) => Bound::new(py, PyCascadeReceiver(r.clone()))
            .unwrap()
            .into_any(),
        Receiver::Gt(r) => Bound::new(
            py,
            PyGtReceiver(GtReceiver {
                frequency: r.frequency,
                gt: r.gt,
            }),
        )
        .unwrap()
        .into_any(),
        _ => unreachable!("unknown receiver variant"),
    }
}

fn build_antenna(obj: &Bound<'_, PyAny>) -> PyResult<Antenna> {
    if let Ok(a) = obj.extract::<PyRef<'_, PyConstantAntenna>>() {
        Ok(Antenna::Constant(ConstantAntenna { gain: a.inner.gain }))
    } else if let Ok(a) = obj.extract::<PyRef<'_, PyPatternedAntenna>>() {
        let pattern = match &a.0.pattern {
            AntennaPattern::Parabolic(p) => {
                AntennaPattern::Parabolic(ParabolicPattern::new(p.diameter, p.efficiency))
            }
            AntennaPattern::Gaussian(p) => {
                AntennaPattern::Gaussian(GaussianPattern::new(p.diameter, p.efficiency))
            }
            AntennaPattern::Dipole(p) => AntennaPattern::Dipole(DipolePattern::new(p.length)),
            _ => unreachable!("unknown antenna variant"),
        };
        Ok(Antenna::Patterned(PatternedAntenna {
            pattern,
            boresight: a.0.boresight,
        }))
    } else {
        Err(PyValueError::new_err(
            "expected a ConstantAntenna or PatternedAntenna",
        ))
    }
}

fn build_receiver(obj: &Bound<'_, PyAny>) -> PyResult<Receiver> {
    if let Ok(r) = obj.extract::<PyRef<'_, PyNoiseTempReceiver>>() {
        Ok(Receiver::NoiseTemperature(NoiseTempReceiver {
            frequency: r.0.frequency,
            system_noise_temperature: r.0.system_noise_temperature,
        }))
    } else if let Ok(r) = obj.extract::<PyRef<'_, PyCascadeReceiver>>() {
        Ok(Receiver::Cascade(r.0.clone()))
    } else if let Ok(r) = obj.extract::<PyRef<'_, PyGtReceiver>>() {
        Ok(Receiver::Gt(GtReceiver {
            frequency: r.0.frequency,
            gt: r.0.gt,
        }))
    } else {
        Err(PyValueError::new_err(
            "expected NoiseTempReceiver, CascadeReceiver, or GtReceiver",
        ))
    }
}

fn build_transmitter_any(obj: &Bound<'_, PyAny>) -> PyResult<Transmitter> {
    if let Ok(eirp) = obj.extract::<PyRef<'_, PyEirpTransmitter>>() {
        return Ok(Transmitter::Eirp(eirp.0.clone()));
    }
    if let Ok(amp) = obj.extract::<PyRef<'_, PyAmplifierTransmitter>>() {
        return Ok(amp.0.clone());
    }
    Err(PyValueError::new_err(
        "expected EirpTransmitter or AmplifierTransmitter",
    ))
}

fn build_receiver_any(obj: &Bound<'_, PyAny>) -> PyResult<Receiver> {
    build_receiver(obj)
}

fn transmitter_to_py<'py>(py: Python<'py>, tx: &Transmitter) -> Bound<'py, PyAny> {
    match tx {
        Transmitter::Eirp(t) => Bound::new(py, PyEirpTransmitter(t.clone()))
            .unwrap()
            .into_any(),
        Transmitter::Amplifier(_) => Bound::new(py, PyAmplifierTransmitter(tx.clone()))
            .unwrap()
            .into_any(),
        _ => unreachable!("unknown transmitter variant"),
    }
}

// --- AmplifierTransmitter ---

/// A radio transmitter with an RF power amplifier.
///
/// Args:
///     frequency: Transmit frequency.
///     power: Transmit power.
///     line_loss: Feed/line loss as Decibel.
///     output_back_off: Output back-off as Decibel (default Decibel(0)).
#[pyclass(
    name = "AmplifierTransmitter",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyAmplifierTransmitter(pub Transmitter);

#[pymethods]
impl PyAmplifierTransmitter {
    #[new]
    #[pyo3(signature = (frequency, power, line_loss, output_back_off=None))]
    fn new(
        frequency: PyFrequency,
        power: PyPower,
        line_loss: PyDecibel,
        output_back_off: Option<PyDecibel>,
    ) -> Self {
        Self(Transmitter::Amplifier(AmplifierTransmitter::new(
            frequency.0,
            f64::from(power.0),
            line_loss.0,
            output_back_off.map_or(Decibel::new(0.0), |d| d.0),
        )))
    }

    /// Returns the EIRP in dBW for the given antenna and pattern angles.
    #[pyo3(signature = (antenna, theta, phi=None))]
    fn eirp(
        &self,
        antenna: &Bound<'_, PyAny>,
        theta: PyAngle,
        phi: Option<PyAngle>,
    ) -> PyResult<PyDecibel> {
        let ant = build_antenna(antenna)?;
        Ok(PyDecibel(self.0.eirp(
            &ant,
            theta.0,
            phi.map_or(Angle::radians(0.0), |p| p.0),
        )))
    }

    fn __eq__(&self, other: &PyAmplifierTransmitter) -> bool {
        let freq_eq = f64::from(self.0.frequency()) == f64::from(other.0.frequency());
        match (&self.0, &other.0) {
            (Transmitter::Amplifier(a), Transmitter::Amplifier(b)) => {
                freq_eq
                    && a.power_w == b.power_w
                    && a.line_loss.as_f64() == b.line_loss.as_f64()
                    && a.output_back_off.as_f64() == b.output_back_off.as_f64()
            }
            _ => unreachable!("unknown transmitter variant"),
        }
    }

    fn __getnewargs__(&self) -> (PyFrequency, PyPower, PyDecibel, Option<PyDecibel>) {
        match &self.0 {
            Transmitter::Amplifier(tx) => (
                PyFrequency(tx.frequency),
                PyPower::new(tx.power_w),
                PyDecibel(tx.line_loss),
                Some(PyDecibel(tx.output_back_off)),
            ),
            _ => unreachable!("unknown transmitter variant"),
        }
    }

    fn __repr__(&self) -> String {
        match &self.0 {
            Transmitter::Amplifier(tx) => format!(
                "AmplifierTransmitter(frequency={}, power={}, line_loss={}, output_back_off={})",
                PyFrequency(tx.frequency).__repr__(),
                PyPower::new(tx.power_w).__repr__(),
                PyDecibel(tx.line_loss).__repr__(),
                PyDecibel(tx.output_back_off).__repr__(),
            ),
            _ => unreachable!("unknown transmitter variant"),
        }
    }
}

// --- EirpTransmitter ---

/// A lumped transmitter defined by a frequency and aggregate EIRP.
///
/// Args:
///     frequency: Transmit frequency.
///     eirp: Effective isotropic radiated power in dBW.
#[pyclass(name = "EirpTransmitter", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyEirpTransmitter(pub EirpTransmitter);

#[pymethods]
impl PyEirpTransmitter {
    #[new]
    fn new(frequency: PyFrequency, eirp: PyDecibel) -> Self {
        Self(EirpTransmitter {
            frequency: frequency.0,
            eirp: eirp.0,
        })
    }

    #[getter]
    fn frequency(&self) -> PyFrequency {
        PyFrequency(self.0.frequency)
    }

    #[getter]
    fn eirp(&self) -> PyDecibel {
        PyDecibel(self.0.eirp)
    }

    fn __repr__(&self) -> String {
        format!(
            "EirpTransmitter(frequency={}, eirp={})",
            repr_f64(self.0.frequency.to_hertz()),
            repr_f64(self.0.eirp.as_f64())
        )
    }

    fn __getnewargs__(&self) -> (PyFrequency, PyDecibel) {
        (PyFrequency(self.0.frequency), PyDecibel(self.0.eirp))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0.frequency == other.0.frequency && self.0.eirp == other.0.eirp
    }
}

// --- GtReceiver ---

/// A lumped receiver defined by a frequency and aggregate G/T.
///
/// Args:
///     frequency: Receive frequency.
///     gt: Gain-to-noise-temperature ratio in dB/K.
#[pyclass(name = "GtReceiver", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyGtReceiver(pub GtReceiver);

#[pymethods]
impl PyGtReceiver {
    #[new]
    fn new(frequency: PyFrequency, gt: PyDecibel) -> Self {
        Self(GtReceiver {
            frequency: frequency.0,
            gt: gt.0,
        })
    }

    #[getter]
    fn frequency(&self) -> PyFrequency {
        PyFrequency(self.0.frequency)
    }

    #[getter]
    fn gt(&self) -> PyDecibel {
        PyDecibel(self.0.gt)
    }

    fn __repr__(&self) -> String {
        format!(
            "GtReceiver(frequency={}, gt={})",
            repr_f64(self.0.frequency.to_hertz()),
            repr_f64(self.0.gt.as_f64())
        )
    }

    fn __getnewargs__(&self) -> (PyFrequency, PyDecibel) {
        (PyFrequency(self.0.frequency), PyDecibel(self.0.gt))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0.frequency == other.0.frequency && self.0.gt == other.0.gt
    }
}

// --- Receivers ---

/// A receiver with a known system noise temperature.
///
/// Args:
///     frequency: Receive frequency.
///     system_noise_temperature: System noise temperature.
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
    fn new(frequency: PyFrequency, system_noise_temperature: PyTemperature) -> Self {
        Self(NoiseTempReceiver {
            frequency: frequency.0,
            system_noise_temperature: f64::from(system_noise_temperature.0),
        })
    }

    fn __eq__(&self, other: &PyNoiseTempReceiver) -> bool {
        f64::from(self.0.frequency) == f64::from(other.0.frequency)
            && self.0.system_noise_temperature == other.0.system_noise_temperature
    }

    fn __getnewargs__(&self) -> (PyFrequency, PyTemperature) {
        (
            PyFrequency(self.0.frequency),
            PyTemperature::new(self.0.system_noise_temperature),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "NoiseTempReceiver(frequency={}, system_noise_temperature={})",
            PyFrequency(self.0.frequency).__repr__(),
            PyTemperature::new(self.0.system_noise_temperature).__repr__(),
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
    fn new(gain: PyDecibel, noise_temperature: PyTemperature) -> Self {
        Self(NoiseStage {
            gain: gain.0,
            noise_temperature: f64::from(noise_temperature.0),
        })
    }

    fn __getnewargs__(&self) -> (PyDecibel, PyTemperature) {
        (
            PyDecibel(self.0.gain),
            PyTemperature::new(self.0.noise_temperature),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "NoiseStage(gain={}, noise_temperature={})",
            PyDecibel(self.0.gain).__repr__(),
            PyTemperature::new(self.0.noise_temperature).__repr__(),
        )
    }
}

// --- Cascade Receiver ---

/// An N-stage cascade receiver using the Friis noise formula.
///
/// Args:
///     frequency: Receive frequency.
///     antenna_noise_temperature: Antenna noise temperature.
///     stages: List of NoiseStage (ordered: LNA first, then downstream).
///     demodulator_loss: Demodulator loss as Decibel (default Decibel(0)).
///     implementation_loss: Other implementation losses as Decibel (default Decibel(0)).
#[pyclass(name = "CascadeReceiver", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyCascadeReceiver(pub CascadeReceiver);

#[pymethods]
impl PyCascadeReceiver {
    #[new]
    #[pyo3(signature = (frequency, antenna_noise_temperature, stages, demodulator_loss=None, implementation_loss=None))]
    fn new(
        frequency: PyFrequency,
        antenna_noise_temperature: PyTemperature,
        stages: Vec<PyNoiseStage>,
        demodulator_loss: Option<PyDecibel>,
        implementation_loss: Option<PyDecibel>,
    ) -> Self {
        Self(CascadeReceiver {
            frequency: frequency.0,
            antenna_noise_temperature: f64::from(antenna_noise_temperature.0),
            stages: stages.into_iter().map(|s| s.0).collect(),
            demodulator_loss: demodulator_loss.map_or(Decibel::new(0.0), |d| d.0),
            implementation_loss: implementation_loss.map_or(Decibel::new(0.0), |d| d.0),
        })
    }

    /// Creates a two-stage model: lossy feed line at room temperature → receiver.
    #[staticmethod]
    #[pyo3(signature = (frequency, antenna_noise_temperature, feed_loss, receiver_noise_figure, receiver_gain, demodulator_loss=None, implementation_loss=None))]
    #[allow(clippy::too_many_arguments)]
    fn from_feed_loss_and_noise_figure(
        frequency: PyFrequency,
        antenna_noise_temperature: PyTemperature,
        feed_loss: PyDecibel,
        receiver_noise_figure: PyDecibel,
        receiver_gain: PyDecibel,
        demodulator_loss: Option<PyDecibel>,
        implementation_loss: Option<PyDecibel>,
    ) -> Self {
        Self(CascadeReceiver::from_feed_loss_and_noise_figure(
            frequency.0,
            f64::from(antenna_noise_temperature.0),
            feed_loss.0,
            receiver_noise_figure.0,
            receiver_gain.0,
            demodulator_loss.map_or(Decibel::new(0.0), |d| d.0),
            implementation_loss.map_or(Decibel::new(0.0), |d| d.0),
        ))
    }

    /// Creates a two-stage model: LNA → receiver (from noise figure).
    #[staticmethod]
    #[pyo3(signature = (frequency, antenna_noise_temperature, lna_gain, lna_noise_temperature, receiver_noise_figure, demodulator_loss=None, implementation_loss=None))]
    #[allow(clippy::too_many_arguments)]
    fn from_lna_and_noise_figure(
        frequency: PyFrequency,
        antenna_noise_temperature: PyTemperature,
        lna_gain: PyDecibel,
        lna_noise_temperature: PyTemperature,
        receiver_noise_figure: PyDecibel,
        demodulator_loss: Option<PyDecibel>,
        implementation_loss: Option<PyDecibel>,
    ) -> Self {
        Self(CascadeReceiver::from_lna_and_noise_figure(
            frequency.0,
            f64::from(antenna_noise_temperature.0),
            lna_gain.0,
            f64::from(lna_noise_temperature.0),
            receiver_noise_figure.0,
            demodulator_loss.map_or(Decibel::new(0.0), |d| d.0),
            implementation_loss.map_or(Decibel::new(0.0), |d| d.0),
        ))
    }

    /// Returns the system noise temperature via the Friis formula.
    fn system_noise_temperature(&self) -> PyTemperature {
        PyTemperature::new(self.0.system_noise_temperature())
    }

    /// Returns the total RF chain gain in dB.
    fn chain_gain(&self) -> PyDecibel {
        PyDecibel(self.0.chain_gain())
    }

    fn __eq__(&self, other: &PyCascadeReceiver) -> bool {
        f64::from(self.0.frequency) == f64::from(other.0.frequency)
            && self.0.antenna_noise_temperature == other.0.antenna_noise_temperature
            && self.0.stages.len() == other.0.stages.len()
            && self
                .0
                .stages
                .iter()
                .zip(other.0.stages.iter())
                .all(|(a, b)| {
                    a.gain.as_f64() == b.gain.as_f64() && a.noise_temperature == b.noise_temperature
                })
            && self.0.demodulator_loss.as_f64() == other.0.demodulator_loss.as_f64()
            && self.0.implementation_loss.as_f64() == other.0.implementation_loss.as_f64()
    }

    fn __getnewargs__(
        &self,
    ) -> (
        PyFrequency,
        PyTemperature,
        Vec<PyNoiseStage>,
        Option<PyDecibel>,
        Option<PyDecibel>,
    ) {
        (
            PyFrequency(self.0.frequency),
            PyTemperature::new(self.0.antenna_noise_temperature),
            self.0
                .stages
                .iter()
                .map(|s| PyNoiseStage(s.clone()))
                .collect(),
            Some(PyDecibel(self.0.demodulator_loss)),
            Some(PyDecibel(self.0.implementation_loss)),
        )
    }

    fn __repr__(&self) -> String {
        let stages_repr: Vec<String> = self
            .0
            .stages
            .iter()
            .map(|s| {
                format!(
                    "NoiseStage(gain={}, noise_temperature={})",
                    PyDecibel(s.gain).__repr__(),
                    PyTemperature::new(s.noise_temperature).__repr__(),
                )
            })
            .collect();
        format!(
            "CascadeReceiver(frequency={}, antenna_noise_temperature={}, stages=[{}], demodulator_loss={}, implementation_loss={})",
            PyFrequency(self.0.frequency).__repr__(),
            PyTemperature::new(self.0.antenna_noise_temperature).__repr__(),
            stages_repr.join(", "),
            PyDecibel(self.0.demodulator_loss).__repr__(),
            PyDecibel(self.0.implementation_loss).__repr__(),
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

// --- Communication System ---

/// A communication system combining an antenna with optional transmitter and receiver.
///
/// Args:
///     antenna: A ConstantAntenna or PatternedAntenna.
///     receiver: A NoiseTempReceiver, CascadeReceiver, or GtReceiver (optional).
///     transmitter: An AmplifierTransmitter or EirpTransmitter (optional).
#[pyclass(
    name = "CommunicationSystem",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyCommunicationSystem(pub CommunicationSystem);

#[pymethods]
impl PyCommunicationSystem {
    #[new]
    #[pyo3(signature = (antenna=None, receiver=None, transmitter=None))]
    fn new(
        antenna: Option<&Bound<'_, PyAny>>,
        receiver: Option<&Bound<'_, PyAny>>,
        transmitter: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let ant = antenna.map(build_antenna).transpose()?;
        let rx = receiver.map(build_receiver_any).transpose()?;
        let tx = transmitter.map(build_transmitter_any).transpose()?;

        if let Some(Transmitter::Eirp(_)) = &tx
            && ant.is_some()
        {
            return Err(PyValueError::new_err(
                "EirpTransmitter must not be paired with an antenna; \
                     build the system via CommunicationSystem(transmitter=...) \
                     with antenna=None, or use CommunicationSystem.eirp_only(...)",
            ));
        }
        if let Some(Transmitter::Amplifier(_)) = &tx
            && ant.is_none()
        {
            return Err(PyValueError::new_err(
                "AmplifierTransmitter requires an antenna; provide one as the \
                     antenna= argument or use CommunicationSystem.amplifier_with(...)",
            ));
        }
        if let Some(Receiver::Gt(_)) = &rx
            && ant.is_some()
        {
            return Err(PyValueError::new_err(
                "GtReceiver must not be paired with an antenna; \
                     build the system via CommunicationSystem(receiver=...) \
                     with antenna=None, or use CommunicationSystem.gt_only(...)",
            ));
        }
        if let Some(Receiver::NoiseTemperature(_)) | Some(Receiver::Cascade(_)) = &rx
            && ant.is_none()
        {
            return Err(PyValueError::new_err(
                "component-tier receiver requires an antenna; provide one as the \
                     antenna= argument",
            ));
        }

        Ok(Self(CommunicationSystem {
            antenna: ant,
            receiver: rx,
            transmitter: tx,
        }))
    }

    /// Creates a lumped transmit-only system from a known EIRP.
    #[classmethod]
    fn eirp_only(_cls: &Bound<'_, pyo3::types::PyType>, tx: PyEirpTransmitter) -> Self {
        Self(CommunicationSystem::eirp_only(tx.0))
    }

    /// Creates a lumped receive-only system from a known G/T.
    #[classmethod]
    fn gt_only(_cls: &Bound<'_, pyo3::types::PyType>, rx: PyGtReceiver) -> Self {
        Self(CommunicationSystem::gt_only(rx.0))
    }

    /// Computes the carrier-to-noise density ratio (C/N0).
    ///
    /// Args:
    ///     rx_system: The receiving CommunicationSystem.
    ///     losses: Additional losses as Decibel.
    ///     range: Slant range as Distance.
    ///     tx_angle: Off-boresight angle at transmitter as Angle.
    ///     rx_angle: Off-boresight angle at receiver as Angle.
    fn carrier_to_noise_density(
        &self,
        rx_system: &PyCommunicationSystem,
        losses: PyDecibel,
        range: PyDistance,
        tx_angle: PyAngle,
        rx_angle: PyAngle,
    ) -> PyResult<PyDecibel> {
        Ok(PyDecibel(
            self.0
                .carrier_to_noise_density(&rx_system.0, losses.0, range.0, tx_angle.0, rx_angle.0)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }

    /// Computes the received carrier power in dBW.
    ///
    /// Returns ``None`` for lumped G/T receivers.
    fn carrier_power(
        &self,
        rx_system: &PyCommunicationSystem,
        losses: PyDecibel,
        range: PyDistance,
        tx_angle: PyAngle,
        rx_angle: PyAngle,
    ) -> PyResult<Option<PyDecibel>> {
        self.0
            .carrier_power(&rx_system.0, losses.0, range.0, tx_angle.0, rx_angle.0)
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(|opt| opt.map(PyDecibel))
    }

    /// Computes the noise power in dBW for a given bandwidth.
    ///
    /// Returns ``None`` for lumped G/T receivers.
    fn noise_power(&self, bandwidth: PyFrequency) -> PyResult<Option<PyDecibel>> {
        self.0
            .noise_power(f64::from(bandwidth.0))
            .map_err(|e| PyValueError::new_err(e.to_string()))
            .map(|opt| opt.map(PyDecibel))
    }

    #[allow(clippy::type_complexity)]
    fn __getnewargs__<'py>(
        &self,
        py: Python<'py>,
    ) -> (
        Option<Bound<'py, PyAny>>,
        Option<Bound<'py, PyAny>>,
        Option<Bound<'py, PyAny>>,
    ) {
        let antenna = self.0.antenna.as_ref().map(|a| antenna_to_py(py, a));
        let receiver = self.0.receiver.as_ref().map(|r| receiver_to_py(py, r));
        let transmitter = self
            .0
            .transmitter
            .as_ref()
            .map(|t| transmitter_to_py(py, t));
        (antenna, receiver, transmitter)
    }

    fn __eq__(&self, other: &PyCommunicationSystem) -> bool {
        self.__repr__() == other.__repr__()
    }

    fn __repr__(&self) -> String {
        let antenna_repr = match self.0.antenna.as_ref() {
            Some(Antenna::Constant(a)) => {
                format!("ConstantAntenna(gain={})", PyDecibel(a.gain).__repr__())
            }
            Some(Antenna::Patterned(a)) => {
                let pattern_repr = match &a.pattern {
                    AntennaPattern::Parabolic(p) => format!(
                        "ParabolicPattern(diameter={}, efficiency={})",
                        PyDistance(p.diameter).__repr__(),
                        repr_f64(p.efficiency),
                    ),
                    AntennaPattern::Gaussian(p) => format!(
                        "GaussianPattern(diameter={}, efficiency={})",
                        PyDistance(p.diameter).__repr__(),
                        repr_f64(p.efficiency),
                    ),
                    AntennaPattern::Dipole(p) => {
                        format!("DipolePattern(length={})", PyDistance(p.length).__repr__())
                    }
                    _ => unreachable!("unknown antenna variant"),
                };
                let b = a.boresight;
                format!(
                    "PatternedAntenna(pattern={pattern_repr}, boresight=[{}, {}, {}])",
                    repr_f64(b.x),
                    repr_f64(b.y),
                    repr_f64(b.z),
                )
            }
            Some(_) => unreachable!("unknown antenna variant"),
            None => String::from("None"),
        };
        let rx_repr = match &self.0.receiver {
            Some(Receiver::NoiseTemperature(r)) => format!(
                ", receiver=NoiseTempReceiver(frequency={}, system_noise_temperature={})",
                PyFrequency(r.frequency).__repr__(),
                PyTemperature::new(r.system_noise_temperature).__repr__(),
            ),
            Some(Receiver::Cascade(r)) => {
                format!(", receiver={}", PyCascadeReceiver(r.clone()).__repr__())
            }
            Some(Receiver::Gt(r)) => format!(
                ", receiver=GtReceiver(frequency={}, gt={})",
                repr_f64(r.frequency.to_hertz()),
                repr_f64(r.gt.as_f64()),
            ),
            Some(_) => unreachable!("unknown receiver variant"),
            None => String::new(),
        };
        let tx_repr = match &self.0.transmitter {
            Some(t) => match t {
                Transmitter::Amplifier(a) => format!(
                    ", transmitter=AmplifierTransmitter(frequency={}, power={}, line_loss={}, output_back_off={})",
                    PyFrequency(a.frequency).__repr__(),
                    PyPower::new(a.power_w).__repr__(),
                    PyDecibel(a.line_loss).__repr__(),
                    PyDecibel(a.output_back_off).__repr__(),
                ),
                Transmitter::Eirp(e) => format!(
                    ", transmitter=EirpTransmitter(frequency={}, eirp={})",
                    repr_f64(e.frequency.to_hertz()),
                    repr_f64(e.eirp.as_f64()),
                ),
                _ => unreachable!("unknown transmitter variant"),
            },
            None => String::new(),
        };
        format!("CommunicationSystem(antenna={antenna_repr}{rx_repr}{tx_repr})")
    }
}

// --- Link Stats ---

/// Modulation-agnostic link budget statistics.
#[pyclass(name = "LinkStats", module = "lox_space", frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct PyLinkStats(pub LinkStats);

#[pymethods]
impl PyLinkStats {
    /// Computes a modulation-agnostic link budget.
    ///
    /// Args:
    ///     tx_system: The transmitting CommunicationSystem.
    ///     rx_system: The receiving CommunicationSystem.
    ///     range: Slant range as Distance.
    ///     bandwidth: Noise bandwidth as Frequency.
    ///     tx_angle: Off-boresight angle at transmitter as Angle.
    ///     rx_angle: Off-boresight angle at receiver as Angle.
    ///     losses: EnvironmentalLosses (optional, defaults to none).
    #[staticmethod]
    #[pyo3(signature = (tx_system, rx_system, range, bandwidth, tx_angle, rx_angle, losses=None))]
    fn calculate(
        tx_system: &PyCommunicationSystem,
        rx_system: &PyCommunicationSystem,
        range: PyDistance,
        bandwidth: PyFrequency,
        tx_angle: PyAngle,
        rx_angle: PyAngle,
        losses: Option<&PyEnvironmentalLosses>,
    ) -> PyResult<Self> {
        let env_losses = losses
            .map(|l| EnvironmentalLosses {
                rain: l.0.rain,
                gaseous: l.0.gaseous,
                scintillation: l.0.scintillation,
                atmospheric: l.0.atmospheric,
                cloud: l.0.cloud,
                depolarization: l.0.depolarization,
            })
            .unwrap_or_else(EnvironmentalLosses::none);

        LinkStats::calculate(
            &tx_system.0,
            &rx_system.0,
            range.0,
            bandwidth.0,
            env_losses,
            tx_angle.0,
            rx_angle.0,
        )
        .map(Self)
        .map_err(|e| PyValueError::new_err(e.to_string()))
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

    /// Off-boresight angle at the transmitter.
    #[getter]
    fn tx_angle(&self) -> PyAngle {
        PyAngle(self.0.tx_angle)
    }

    /// Off-boresight angle at the receiver.
    #[getter]
    fn rx_angle(&self) -> PyAngle {
        PyAngle(self.0.rx_angle)
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
    /// Interference power in watts.
    #[getter]
    fn interference_power_w(&self) -> f64 {
        self.0.interference_power_w
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
            "InterferenceStats(interference_power_w={}, c_n0i0={:.2} dB·Hz, eb_n0i0={:.2} dB, margin_with_interference={:.2} dB)",
            repr_f64(self.0.interference_power_w),
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

    /// Interference statistics (if computed).
    #[getter]
    fn interference(&self) -> Option<PyInterferenceStats> {
        self.0
            .interference
            .as_ref()
            .map(|i| PyInterferenceStats(i.clone()))
    }

    /// Computes interference statistics for a given interferer power.
    fn with_interference(&self, interference_power_w: f64) -> PyResult<PyInterferenceStats> {
        self.0
            .with_interference(interference_power_w)
            .map(PyInterferenceStats)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "ModulatedLinkStats(eb_n0={:.2} dB, margin={:.2} dB, interference={})",
            self.0.eb_n0.as_f64(),
            self.0.margin.as_f64(),
            if self.0.interference.is_some() {
                "present"
            } else {
                "None"
            },
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
    frequency_overlap_factor(
        f64::from(rx_freq.0),
        f64::from(rx_bw.0),
        f64::from(tx_freq.0),
        f64::from(tx_bw.0),
    )
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

/// Computes the ITU RR Article 21.16 PFD mask value.
///
/// Args:
///     elevation: Elevation angle.
///     start_val: PFD limit at low elevation as Decibel.
///     end_val: PFD limit at high elevation as Decibel.
///
/// Returns:
///     PFD mask value as Decibel.
#[pyfunction]
pub fn pfd_mask(elevation: PyAngle, start_val: PyDecibel, end_val: PyDecibel) -> PyDecibel {
    PyDecibel(pfd::pfd_mask(elevation.0, start_val.0, end_val.0))
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
