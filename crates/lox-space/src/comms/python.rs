// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use std::string::String;

use glam::DVec3;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use lox_comms::antenna::{Antenna, AntennaGain, ComplexAntenna, SimpleAntenna};
use lox_comms::channel::{Channel, LinkDirection, Modulation};
use lox_comms::link_budget::{EnvironmentalLosses, LinkStats, frequency_overlap_factor};
use lox_comms::pattern::{AntennaPattern, DipolePattern, GaussianPattern, ParabolicPattern};
use lox_comms::receiver::{ComplexReceiver, Receiver, SimpleReceiver};
use lox_comms::system::CommunicationSystem;
use lox_comms::transmitter::Transmitter;
use lox_comms::utils::free_space_path_loss;
use lox_core::units::{Angle, Decibel, Distance, Frequency};

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
    }
}

// --- Decibel ---

/// A value in decibels.
///
/// Args:
///     value: The value in dB.
#[pyclass(name = "Decibel", module = "lox_space", frozen)]
#[derive(Clone, Copy)]
pub struct PyDecibel(pub Decibel);

#[pymethods]
impl PyDecibel {
    #[new]
    fn new(value: f64) -> Self {
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

    fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    fn __eq__(&self, other: &PyDecibel) -> bool {
        self.0.as_f64() == other.0.as_f64()
    }

    fn __getnewargs__(&self) -> (f64,) {
        (self.0.as_f64(),)
    }

    fn __repr__(&self) -> String {
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
#[pyclass(name = "Modulation", module = "lox_space", frozen)]
#[derive(Clone, Copy)]
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
///     diameter_m: Antenna diameter in meters.
///     efficiency: Aperture efficiency (0, 1].
#[pyclass(name = "ParabolicPattern", module = "lox_space", frozen)]
pub struct PyParabolicPattern(pub ParabolicPattern);

#[pymethods]
impl PyParabolicPattern {
    #[new]
    fn new(diameter_m: f64, efficiency: f64) -> Self {
        Self(ParabolicPattern::new(
            Distance::meters(diameter_m),
            efficiency,
        ))
    }

    /// Creates a parabolic pattern from a desired beamwidth.
    ///
    /// Args:
    ///     beamwidth_deg: Half-power beamwidth in degrees.
    ///     frequency_hz: Frequency in Hz.
    ///     efficiency: Aperture efficiency (0, 1].
    #[staticmethod]
    fn from_beamwidth(beamwidth_deg: f64, frequency_hz: f64, efficiency: f64) -> Self {
        Self(ParabolicPattern::from_beamwidth(
            Angle::degrees(beamwidth_deg),
            Frequency::new(frequency_hz),
            efficiency,
        ))
    }

    /// Returns the gain in dBi at the given frequency and off-boresight angle.
    fn gain(&self, frequency_hz: f64, angle_deg: f64) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(Frequency::new(frequency_hz), Angle::degrees(angle_deg)),
        )
    }

    /// Returns the half-power beamwidth in degrees.
    fn beamwidth(&self, frequency_hz: f64) -> f64 {
        self.0.beamwidth(Frequency::new(frequency_hz)).to_degrees()
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency_hz: f64) -> PyDecibel {
        PyDecibel(self.0.peak_gain(Frequency::new(frequency_hz)))
    }

    fn __eq__(&self, other: &PyParabolicPattern) -> bool {
        self.0.diameter.to_meters() == other.0.diameter.to_meters()
            && self.0.efficiency == other.0.efficiency
    }

    fn __getnewargs__(&self) -> (f64, f64) {
        (self.0.diameter.to_meters(), self.0.efficiency)
    }

    fn __repr__(&self) -> String {
        format!(
            "ParabolicPattern(diameter_m={}, efficiency={})",
            repr_f64(self.0.diameter.to_meters()),
            repr_f64(self.0.efficiency),
        )
    }
}

/// Gaussian antenna gain pattern.
///
/// Args:
///     diameter_m: Antenna diameter in meters.
///     efficiency: Aperture efficiency (0, 1].
#[pyclass(name = "GaussianPattern", module = "lox_space", frozen)]
pub struct PyGaussianPattern(pub GaussianPattern);

#[pymethods]
impl PyGaussianPattern {
    #[new]
    fn new(diameter_m: f64, efficiency: f64) -> Self {
        Self(GaussianPattern::new(
            Distance::meters(diameter_m),
            efficiency,
        ))
    }

    /// Returns the gain in dBi at the given frequency and off-boresight angle.
    fn gain(&self, frequency_hz: f64, angle_deg: f64) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(Frequency::new(frequency_hz), Angle::degrees(angle_deg)),
        )
    }

    /// Returns the half-power beamwidth in degrees.
    fn beamwidth(&self, frequency_hz: f64) -> f64 {
        self.0.beamwidth(Frequency::new(frequency_hz)).to_degrees()
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency_hz: f64) -> PyDecibel {
        PyDecibel(self.0.peak_gain(Frequency::new(frequency_hz)))
    }

    fn __eq__(&self, other: &PyGaussianPattern) -> bool {
        self.0.diameter.to_meters() == other.0.diameter.to_meters()
            && self.0.efficiency == other.0.efficiency
    }

    fn __getnewargs__(&self) -> (f64, f64) {
        (self.0.diameter.to_meters(), self.0.efficiency)
    }

    fn __repr__(&self) -> String {
        format!(
            "GaussianPattern(diameter_m={}, efficiency={})",
            repr_f64(self.0.diameter.to_meters()),
            repr_f64(self.0.efficiency),
        )
    }
}

/// Dipole antenna gain pattern.
///
/// Args:
///     length_m: Dipole length in meters.
#[pyclass(name = "DipolePattern", module = "lox_space", frozen)]
pub struct PyDipolePattern(pub DipolePattern);

#[pymethods]
impl PyDipolePattern {
    #[new]
    fn new(length_m: f64) -> Self {
        Self(DipolePattern::new(Distance::meters(length_m)))
    }

    /// Returns the gain in dBi at the given frequency and off-boresight angle.
    fn gain(&self, frequency_hz: f64, angle_deg: f64) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(Frequency::new(frequency_hz), Angle::degrees(angle_deg)),
        )
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency_hz: f64) -> PyDecibel {
        PyDecibel(self.0.peak_gain(Frequency::new(frequency_hz)))
    }

    fn __eq__(&self, other: &PyDipolePattern) -> bool {
        self.0.length.to_meters() == other.0.length.to_meters()
    }

    fn __getnewargs__(&self) -> (f64,) {
        (self.0.length.to_meters(),)
    }

    fn __repr__(&self) -> String {
        format!(
            "DipolePattern(length_m={})",
            repr_f64(self.0.length.to_meters()),
        )
    }
}

// --- Antennas ---

/// A simple antenna with constant gain and beamwidth.
///
/// Args:
///     gain_db: Peak gain in dBi.
///     beamwidth_deg: Half-power beamwidth in degrees.
#[pyclass(name = "SimpleAntenna", module = "lox_space", frozen)]
pub struct PySimpleAntenna {
    pub inner: SimpleAntenna,
    gain_db: f64,
    beamwidth_deg: f64,
}

#[pymethods]
impl PySimpleAntenna {
    #[new]
    fn new(gain_db: f64, beamwidth_deg: f64) -> Self {
        Self {
            inner: SimpleAntenna {
                gain: Decibel::new(gain_db),
                beamwidth: Angle::degrees(beamwidth_deg),
            },
            gain_db,
            beamwidth_deg,
        }
    }

    fn __eq__(&self, other: &PySimpleAntenna) -> bool {
        self.gain_db == other.gain_db && self.beamwidth_deg == other.beamwidth_deg
    }

    fn __getnewargs__(&self) -> (f64, f64) {
        (self.gain_db, self.beamwidth_deg)
    }

    fn __repr__(&self) -> String {
        format!(
            "SimpleAntenna(gain_db={}, beamwidth_deg={})",
            repr_f64(self.gain_db),
            repr_f64(self.beamwidth_deg),
        )
    }
}

/// An antenna with a physics-based gain pattern and boresight vector.
///
/// Args:
///     pattern: An antenna pattern (ParabolicPattern, GaussianPattern, or DipolePattern).
///     boresight: Boresight direction as [x, y, z].
#[pyclass(name = "ComplexAntenna", module = "lox_space", frozen)]
pub struct PyComplexAntenna(pub ComplexAntenna);

#[pymethods]
impl PyComplexAntenna {
    #[new]
    fn new(pattern: &Bound<'_, PyAny>, boresight: [f64; 3]) -> PyResult<Self> {
        let pattern = extract_antenna_pattern(pattern)?;
        Ok(Self(ComplexAntenna {
            pattern,
            boresight: DVec3::from_array(boresight),
        }))
    }

    /// Returns the gain in dBi at the given frequency and off-boresight angle.
    fn gain(&self, frequency_hz: f64, angle_deg: f64) -> PyDecibel {
        PyDecibel(
            self.0
                .gain(Frequency::new(frequency_hz), Angle::degrees(angle_deg)),
        )
    }

    /// Returns the half-power beamwidth in degrees.
    fn beamwidth(&self, frequency_hz: f64) -> f64 {
        self.0.beamwidth(Frequency::new(frequency_hz)).to_degrees()
    }

    /// Returns the peak gain in dBi.
    fn peak_gain(&self, frequency_hz: f64) -> PyDecibel {
        PyDecibel(self.0.peak_gain(Frequency::new(frequency_hz)))
    }

    fn __getnewargs__<'py>(&self, py: Python<'py>) -> (Bound<'py, PyAny>, [f64; 3]) {
        let pattern = pattern_to_py(py, &self.0.pattern);
        let b = self.0.boresight;
        (pattern, [b.x, b.y, b.z])
    }

    fn __repr__(&self) -> String {
        let pattern_repr = match &self.0.pattern {
            AntennaPattern::Parabolic(p) => format!(
                "ParabolicPattern(diameter_m={}, efficiency={})",
                repr_f64(p.diameter.to_meters()),
                repr_f64(p.efficiency),
            ),
            AntennaPattern::Gaussian(p) => format!(
                "GaussianPattern(diameter_m={}, efficiency={})",
                repr_f64(p.diameter.to_meters()),
                repr_f64(p.efficiency),
            ),
            AntennaPattern::Dipole(p) => {
                format!("DipolePattern(length_m={})", repr_f64(p.length.to_meters()))
            }
        };
        let b = self.0.boresight;
        format!(
            "ComplexAntenna(pattern={pattern_repr}, boresight=[{}, {}, {}])",
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
    }
}

fn antenna_to_py<'py>(py: Python<'py>, antenna: &Antenna) -> Bound<'py, PyAny> {
    match antenna {
        Antenna::Simple(a) => {
            let gain_db = a.gain.as_f64();
            let beamwidth_deg = a.beamwidth.to_degrees();
            Bound::new(
                py,
                PySimpleAntenna {
                    inner: SimpleAntenna {
                        gain: a.gain,
                        beamwidth: a.beamwidth,
                    },
                    gain_db,
                    beamwidth_deg,
                },
            )
            .unwrap()
            .into_any()
        }
        Antenna::Complex(a) => {
            let pattern = match &a.pattern {
                AntennaPattern::Parabolic(p) => {
                    AntennaPattern::Parabolic(ParabolicPattern::new(p.diameter, p.efficiency))
                }
                AntennaPattern::Gaussian(p) => {
                    AntennaPattern::Gaussian(GaussianPattern::new(p.diameter, p.efficiency))
                }
                AntennaPattern::Dipole(p) => AntennaPattern::Dipole(DipolePattern::new(p.length)),
            };
            Bound::new(
                py,
                PyComplexAntenna(ComplexAntenna {
                    pattern,
                    boresight: a.boresight,
                }),
            )
            .unwrap()
            .into_any()
        }
    }
}

fn receiver_to_py<'py>(py: Python<'py>, receiver: &Receiver) -> Bound<'py, PyAny> {
    match receiver {
        Receiver::Simple(r) => Bound::new(
            py,
            PySimpleReceiver(SimpleReceiver {
                frequency: r.frequency,
                system_noise_temperature: r.system_noise_temperature,
            }),
        )
        .unwrap()
        .into_any(),
        Receiver::Complex(r) => Bound::new(
            py,
            PyComplexReceiver(ComplexReceiver {
                frequency: r.frequency,
                antenna_noise_temperature: r.antenna_noise_temperature,
                lna_gain: r.lna_gain,
                lna_noise_figure: r.lna_noise_figure,
                noise_figure: r.noise_figure,
                loss: r.loss,
                demodulator_loss: r.demodulator_loss,
                implementation_loss: r.implementation_loss,
            }),
        )
        .unwrap()
        .into_any(),
    }
}

fn build_antenna(obj: &Bound<'_, PyAny>) -> PyResult<Antenna> {
    if let Ok(a) = obj.extract::<PyRef<'_, PySimpleAntenna>>() {
        Ok(Antenna::Simple(SimpleAntenna {
            gain: a.inner.gain,
            beamwidth: a.inner.beamwidth,
        }))
    } else if let Ok(a) = obj.extract::<PyRef<'_, PyComplexAntenna>>() {
        let pattern = match &a.0.pattern {
            AntennaPattern::Parabolic(p) => {
                AntennaPattern::Parabolic(ParabolicPattern::new(p.diameter, p.efficiency))
            }
            AntennaPattern::Gaussian(p) => {
                AntennaPattern::Gaussian(GaussianPattern::new(p.diameter, p.efficiency))
            }
            AntennaPattern::Dipole(p) => AntennaPattern::Dipole(DipolePattern::new(p.length)),
        };
        Ok(Antenna::Complex(ComplexAntenna {
            pattern,
            boresight: a.0.boresight,
        }))
    } else {
        Err(PyValueError::new_err(
            "expected a SimpleAntenna or ComplexAntenna",
        ))
    }
}

fn build_receiver(obj: &Bound<'_, PyAny>) -> PyResult<Receiver> {
    if let Ok(r) = obj.extract::<PyRef<'_, PySimpleReceiver>>() {
        Ok(Receiver::Simple(SimpleReceiver {
            frequency: r.0.frequency,
            system_noise_temperature: r.0.system_noise_temperature,
        }))
    } else if let Ok(r) = obj.extract::<PyRef<'_, PyComplexReceiver>>() {
        Ok(Receiver::Complex(ComplexReceiver {
            frequency: r.0.frequency,
            antenna_noise_temperature: r.0.antenna_noise_temperature,
            lna_gain: r.0.lna_gain,
            lna_noise_figure: r.0.lna_noise_figure,
            noise_figure: r.0.noise_figure,
            loss: r.0.loss,
            demodulator_loss: r.0.demodulator_loss,
            implementation_loss: r.0.implementation_loss,
        }))
    } else {
        Err(PyValueError::new_err(
            "expected a SimpleReceiver or ComplexReceiver",
        ))
    }
}

// --- Transmitter ---

/// A radio transmitter.
///
/// Args:
///     frequency_hz: Transmit frequency in Hz.
///     power_w: Transmit power in watts.
///     line_loss_db: Feed/line loss in dB.
///     output_back_off_db: Output back-off in dB (default 0).
#[pyclass(name = "Transmitter", module = "lox_space", frozen)]
pub struct PyTransmitter(pub Transmitter);

#[pymethods]
impl PyTransmitter {
    #[new]
    #[pyo3(signature = (frequency_hz, power_w, line_loss_db, output_back_off_db=0.0))]
    fn new(frequency_hz: f64, power_w: f64, line_loss_db: f64, output_back_off_db: f64) -> Self {
        Self(Transmitter::new(
            Frequency::new(frequency_hz),
            power_w,
            Decibel::new(line_loss_db),
            Decibel::new(output_back_off_db),
        ))
    }

    /// Returns the EIRP in dBW for the given antenna and off-boresight angle.
    fn eirp(&self, antenna: &Bound<'_, PyAny>, angle_deg: f64) -> PyResult<PyDecibel> {
        let ant = build_antenna(antenna)?;
        Ok(PyDecibel(self.0.eirp(&ant, Angle::degrees(angle_deg))))
    }

    fn __eq__(&self, other: &PyTransmitter) -> bool {
        f64::from(self.0.frequency) == f64::from(other.0.frequency)
            && self.0.power_w == other.0.power_w
            && self.0.line_loss.as_f64() == other.0.line_loss.as_f64()
            && self.0.output_back_off.as_f64() == other.0.output_back_off.as_f64()
    }

    fn __getnewargs__(&self) -> (f64, f64, f64, f64) {
        (
            f64::from(self.0.frequency),
            self.0.power_w,
            self.0.line_loss.as_f64(),
            self.0.output_back_off.as_f64(),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "Transmitter(frequency_hz={}, power_w={}, line_loss_db={}, output_back_off_db={})",
            repr_f64(f64::from(self.0.frequency)),
            repr_f64(self.0.power_w),
            repr_f64(self.0.line_loss.as_f64()),
            repr_f64(self.0.output_back_off.as_f64()),
        )
    }
}

// --- Receivers ---

/// A simple receiver with a known system noise temperature.
///
/// Args:
///     frequency_hz: Receive frequency in Hz.
///     system_noise_temperature_k: System noise temperature in Kelvin.
#[pyclass(name = "SimpleReceiver", module = "lox_space", frozen)]
pub struct PySimpleReceiver(pub SimpleReceiver);

#[pymethods]
impl PySimpleReceiver {
    #[new]
    fn new(frequency_hz: f64, system_noise_temperature_k: f64) -> Self {
        Self(SimpleReceiver {
            frequency: Frequency::new(frequency_hz),
            system_noise_temperature: system_noise_temperature_k,
        })
    }

    fn __eq__(&self, other: &PySimpleReceiver) -> bool {
        f64::from(self.0.frequency) == f64::from(other.0.frequency)
            && self.0.system_noise_temperature == other.0.system_noise_temperature
    }

    fn __getnewargs__(&self) -> (f64, f64) {
        (f64::from(self.0.frequency), self.0.system_noise_temperature)
    }

    fn __repr__(&self) -> String {
        format!(
            "SimpleReceiver(frequency_hz={}, system_noise_temperature_k={})",
            repr_f64(f64::from(self.0.frequency)),
            repr_f64(self.0.system_noise_temperature),
        )
    }
}

/// A complex receiver with detailed noise and gain parameters.
///
/// Args:
///     frequency_hz: Receive frequency in Hz.
///     antenna_noise_temperature_k: Antenna noise temperature in Kelvin.
///     lna_gain_db: LNA gain in dB.
///     lna_noise_figure_db: LNA noise figure in dB.
///     noise_figure_db: Receiver noise figure in dB.
///     loss_db: Receiver chain loss in dB.
///     demodulator_loss_db: Demodulator loss in dB (default 0).
///     implementation_loss_db: Other implementation losses in dB (default 0).
#[pyclass(name = "ComplexReceiver", module = "lox_space", frozen)]
pub struct PyComplexReceiver(pub ComplexReceiver);

#[pymethods]
impl PyComplexReceiver {
    #[new]
    #[pyo3(signature = (frequency_hz, antenna_noise_temperature_k, lna_gain_db, lna_noise_figure_db, noise_figure_db, loss_db, demodulator_loss_db=0.0, implementation_loss_db=0.0))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        frequency_hz: f64,
        antenna_noise_temperature_k: f64,
        lna_gain_db: f64,
        lna_noise_figure_db: f64,
        noise_figure_db: f64,
        loss_db: f64,
        demodulator_loss_db: f64,
        implementation_loss_db: f64,
    ) -> Self {
        Self(ComplexReceiver {
            frequency: Frequency::new(frequency_hz),
            antenna_noise_temperature: antenna_noise_temperature_k,
            lna_gain: Decibel::new(lna_gain_db),
            lna_noise_figure: Decibel::new(lna_noise_figure_db),
            noise_figure: Decibel::new(noise_figure_db),
            loss: Decibel::new(loss_db),
            demodulator_loss: Decibel::new(demodulator_loss_db),
            implementation_loss: Decibel::new(implementation_loss_db),
        })
    }

    /// Returns the receiver noise temperature in Kelvin.
    fn noise_temperature(&self) -> f64 {
        self.0.noise_temperature()
    }

    /// Returns the system noise temperature in Kelvin.
    fn system_noise_temperature(&self) -> f64 {
        self.0.system_noise_temperature()
    }

    fn __eq__(&self, other: &PyComplexReceiver) -> bool {
        f64::from(self.0.frequency) == f64::from(other.0.frequency)
            && self.0.antenna_noise_temperature == other.0.antenna_noise_temperature
            && self.0.lna_gain.as_f64() == other.0.lna_gain.as_f64()
            && self.0.lna_noise_figure.as_f64() == other.0.lna_noise_figure.as_f64()
            && self.0.noise_figure.as_f64() == other.0.noise_figure.as_f64()
            && self.0.loss.as_f64() == other.0.loss.as_f64()
            && self.0.demodulator_loss.as_f64() == other.0.demodulator_loss.as_f64()
            && self.0.implementation_loss.as_f64() == other.0.implementation_loss.as_f64()
    }

    #[allow(clippy::type_complexity)]
    fn __getnewargs__(&self) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
        (
            f64::from(self.0.frequency),
            self.0.antenna_noise_temperature,
            self.0.lna_gain.as_f64(),
            self.0.lna_noise_figure.as_f64(),
            self.0.noise_figure.as_f64(),
            self.0.loss.as_f64(),
            self.0.demodulator_loss.as_f64(),
            self.0.implementation_loss.as_f64(),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "ComplexReceiver(frequency_hz={}, antenna_noise_temperature_k={}, lna_gain_db={}, lna_noise_figure_db={}, noise_figure_db={}, loss_db={}, demodulator_loss_db={}, implementation_loss_db={})",
            repr_f64(f64::from(self.0.frequency)),
            repr_f64(self.0.antenna_noise_temperature),
            repr_f64(self.0.lna_gain.as_f64()),
            repr_f64(self.0.lna_noise_figure.as_f64()),
            repr_f64(self.0.noise_figure.as_f64()),
            repr_f64(self.0.loss.as_f64()),
            repr_f64(self.0.demodulator_loss.as_f64()),
            repr_f64(self.0.implementation_loss.as_f64()),
        )
    }
}

// --- Channel ---

/// A communication channel.
///
/// Args:
///     link_type: "uplink" or "downlink".
///     data_rate: Data rate in bits per second.
///     required_eb_n0_db: Required Eb/N0 in dB.
///     margin_db: Required link margin in dB.
///     modulation: Modulation scheme.
///     roll_off: Roll-off factor (default 1.5).
///     fec: Forward error correction code rate (default 0.5).
#[pyclass(name = "Channel", module = "lox_space", frozen)]
pub struct PyChannel(pub Channel);

#[pymethods]
impl PyChannel {
    #[new]
    #[pyo3(signature = (link_type, data_rate, required_eb_n0_db, margin_db, modulation, roll_off=1.5, fec=0.5))]
    fn new(
        link_type: &str,
        data_rate: f64,
        required_eb_n0_db: f64,
        margin_db: f64,
        modulation: &PyModulation,
        roll_off: f64,
        fec: f64,
    ) -> PyResult<Self> {
        let lt = match link_type {
            "uplink" => LinkDirection::Uplink,
            "downlink" => LinkDirection::Downlink,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "unknown link type: {link_type}, expected 'uplink' or 'downlink'"
                )));
            }
        };
        Ok(Self(Channel {
            link_type: lt,
            data_rate,
            required_eb_n0: Decibel::new(required_eb_n0_db),
            margin: Decibel::new(margin_db),
            modulation: modulation.0,
            roll_off,
            fec,
        }))
    }

    /// Returns the channel bandwidth in Hz.
    fn bandwidth(&self) -> f64 {
        self.0.bandwidth()
    }

    /// Computes Eb/N0 from a given C/N0.
    fn eb_n0(&self, c_n0: &PyDecibel) -> PyDecibel {
        PyDecibel(self.0.eb_n0(c_n0.0))
    }

    /// Computes the link margin from a given Eb/N0.
    fn link_margin(&self, eb_n0: &PyDecibel) -> PyDecibel {
        PyDecibel(self.0.link_margin(eb_n0.0))
    }

    fn __getnewargs__<'py>(
        &self,
        py: Python<'py>,
    ) -> (&str, f64, f64, f64, Bound<'py, PyAny>, f64, f64) {
        let lt = match self.0.link_type {
            LinkDirection::Uplink => "uplink",
            LinkDirection::Downlink => "downlink",
        };
        let modulation = Bound::new(py, PyModulation(self.0.modulation))
            .unwrap()
            .into_any();
        (
            lt,
            self.0.data_rate,
            self.0.required_eb_n0.as_f64(),
            self.0.margin.as_f64(),
            modulation,
            self.0.roll_off,
            self.0.fec,
        )
    }

    fn __repr__(&self) -> String {
        let lt = match self.0.link_type {
            LinkDirection::Uplink => "uplink",
            LinkDirection::Downlink => "downlink",
        };
        format!(
            "Channel(link_type='{}', data_rate={}, required_eb_n0_db={}, margin_db={}, modulation=Modulation('{}'), roll_off={}, fec={})",
            lt,
            repr_f64(self.0.data_rate),
            repr_f64(self.0.required_eb_n0.as_f64()),
            repr_f64(self.0.margin.as_f64()),
            modulation_name(self.0.modulation),
            repr_f64(self.0.roll_off),
            repr_f64(self.0.fec),
        )
    }
}

// --- Environmental Losses ---

/// Environmental losses for a link.
///
/// Args:
///     rain_db: Rain attenuation in dB (default 0).
///     gaseous_db: Gaseous absorption in dB (default 0).
///     scintillation_db: Scintillation loss in dB (default 0).
///     atmospheric_db: Atmospheric loss in dB (default 0).
///     cloud_db: Cloud attenuation in dB (default 0).
///     depolarization_db: Depolarization loss in dB (default 0).
#[pyclass(name = "EnvironmentalLosses", module = "lox_space", frozen)]
pub struct PyEnvironmentalLosses(pub EnvironmentalLosses);

#[pymethods]
impl PyEnvironmentalLosses {
    #[new]
    #[pyo3(signature = (rain_db=0.0, gaseous_db=0.0, scintillation_db=0.0, atmospheric_db=0.0, cloud_db=0.0, depolarization_db=0.0))]
    fn new(
        rain_db: f64,
        gaseous_db: f64,
        scintillation_db: f64,
        atmospheric_db: f64,
        cloud_db: f64,
        depolarization_db: f64,
    ) -> Self {
        Self(EnvironmentalLosses {
            rain: Decibel::new(rain_db),
            gaseous: Decibel::new(gaseous_db),
            scintillation: Decibel::new(scintillation_db),
            atmospheric: Decibel::new(atmospheric_db),
            cloud: Decibel::new(cloud_db),
            depolarization: Decibel::new(depolarization_db),
        })
    }

    /// Returns the total environmental loss in dB.
    fn total(&self) -> PyDecibel {
        PyDecibel(self.0.total())
    }

    fn __eq__(&self, other: &PyEnvironmentalLosses) -> bool {
        self.0.rain.as_f64() == other.0.rain.as_f64()
            && self.0.gaseous.as_f64() == other.0.gaseous.as_f64()
            && self.0.scintillation.as_f64() == other.0.scintillation.as_f64()
            && self.0.atmospheric.as_f64() == other.0.atmospheric.as_f64()
            && self.0.cloud.as_f64() == other.0.cloud.as_f64()
            && self.0.depolarization.as_f64() == other.0.depolarization.as_f64()
    }

    fn __getnewargs__(&self) -> (f64, f64, f64, f64, f64, f64) {
        (
            self.0.rain.as_f64(),
            self.0.gaseous.as_f64(),
            self.0.scintillation.as_f64(),
            self.0.atmospheric.as_f64(),
            self.0.cloud.as_f64(),
            self.0.depolarization.as_f64(),
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "EnvironmentalLosses(rain_db={}, gaseous_db={}, scintillation_db={}, atmospheric_db={}, cloud_db={}, depolarization_db={})",
            repr_f64(self.0.rain.as_f64()),
            repr_f64(self.0.gaseous.as_f64()),
            repr_f64(self.0.scintillation.as_f64()),
            repr_f64(self.0.atmospheric.as_f64()),
            repr_f64(self.0.cloud.as_f64()),
            repr_f64(self.0.depolarization.as_f64()),
        )
    }
}

// --- Communication System ---

/// A communication system combining an antenna with optional transmitter and receiver.
///
/// Args:
///     antenna: A SimpleAntenna or ComplexAntenna.
///     receiver: A SimpleReceiver or ComplexReceiver (optional).
///     transmitter: A Transmitter (optional).
#[pyclass(name = "CommunicationSystem", module = "lox_space", frozen)]
pub struct PyCommunicationSystem(pub CommunicationSystem);

#[pymethods]
impl PyCommunicationSystem {
    #[new]
    #[pyo3(signature = (antenna, receiver=None, transmitter=None))]
    fn new(
        antenna: &Bound<'_, PyAny>,
        receiver: Option<&Bound<'_, PyAny>>,
        transmitter: Option<&PyTransmitter>,
    ) -> PyResult<Self> {
        let ant = build_antenna(antenna)?;
        let rx = receiver.map(build_receiver).transpose()?;
        let tx = transmitter.map(|t| {
            Transmitter::new(
                t.0.frequency,
                t.0.power_w,
                t.0.line_loss,
                t.0.output_back_off,
            )
        });
        Ok(Self(CommunicationSystem {
            antenna: ant,
            receiver: rx,
            transmitter: tx,
        }))
    }

    /// Computes the carrier-to-noise density ratio (C/N0) in dB·Hz.
    ///
    /// Args:
    ///     rx_system: The receiving CommunicationSystem.
    ///     losses_db: Additional losses in dB.
    ///     range_km: Slant range in kilometers.
    ///     tx_angle_deg: Off-boresight angle at transmitter in degrees.
    ///     rx_angle_deg: Off-boresight angle at receiver in degrees.
    fn carrier_to_noise_density(
        &self,
        rx_system: &PyCommunicationSystem,
        losses_db: f64,
        range_km: f64,
        tx_angle_deg: f64,
        rx_angle_deg: f64,
    ) -> PyDecibel {
        PyDecibel(self.0.carrier_to_noise_density(
            &rx_system.0,
            Decibel::new(losses_db),
            Distance::kilometers(range_km),
            Angle::degrees(tx_angle_deg),
            Angle::degrees(rx_angle_deg),
        ))
    }

    /// Computes the received carrier power in dBW.
    fn carrier_power(
        &self,
        rx_system: &PyCommunicationSystem,
        losses_db: f64,
        range_km: f64,
        tx_angle_deg: f64,
        rx_angle_deg: f64,
    ) -> PyDecibel {
        PyDecibel(self.0.carrier_power(
            &rx_system.0,
            Decibel::new(losses_db),
            Distance::kilometers(range_km),
            Angle::degrees(tx_angle_deg),
            Angle::degrees(rx_angle_deg),
        ))
    }

    /// Computes the noise power in dBW for a given bandwidth.
    fn noise_power(&self, bandwidth_hz: f64) -> PyDecibel {
        PyDecibel(self.0.noise_power(bandwidth_hz))
    }

    #[allow(clippy::type_complexity)]
    fn __getnewargs__<'py>(
        &self,
        py: Python<'py>,
    ) -> (
        Bound<'py, PyAny>,
        Option<Bound<'py, PyAny>>,
        Option<PyTransmitter>,
    ) {
        let antenna = antenna_to_py(py, &self.0.antenna);
        let receiver = self.0.receiver.as_ref().map(|r| receiver_to_py(py, r));
        let transmitter = self.0.transmitter.as_ref().map(|t| {
            PyTransmitter(Transmitter::new(
                t.frequency,
                t.power_w,
                t.line_loss,
                t.output_back_off,
            ))
        });
        (antenna, receiver, transmitter)
    }

    fn __repr__(&self) -> String {
        let has_tx = self.0.transmitter.is_some();
        let has_rx = self.0.receiver.is_some();
        format!("CommunicationSystem(has_tx={has_tx}, has_rx={has_rx})")
    }
}

// --- Link Stats ---

/// Complete link budget statistics.
#[pyclass(name = "LinkStats", module = "lox_space", frozen)]
pub struct PyLinkStats(pub LinkStats);

#[pymethods]
impl PyLinkStats {
    /// Computes a full link budget.
    ///
    /// Args:
    ///     tx_system: The transmitting CommunicationSystem.
    ///     rx_system: The receiving CommunicationSystem.
    ///     channel: The Channel.
    ///     range_km: Slant range in kilometers.
    ///     tx_angle_deg: Off-boresight angle at transmitter in degrees.
    ///     rx_angle_deg: Off-boresight angle at receiver in degrees.
    ///     losses: EnvironmentalLosses (optional, defaults to none).
    #[staticmethod]
    #[pyo3(signature = (tx_system, rx_system, channel, range_km, tx_angle_deg, rx_angle_deg, losses=None))]
    fn calculate(
        tx_system: &PyCommunicationSystem,
        rx_system: &PyCommunicationSystem,
        channel: &PyChannel,
        range_km: f64,
        tx_angle_deg: f64,
        rx_angle_deg: f64,
        losses: Option<&PyEnvironmentalLosses>,
    ) -> Self {
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

        Self(LinkStats::calculate(
            &tx_system.0,
            &rx_system.0,
            &channel.0,
            Distance::kilometers(range_km),
            Angle::degrees(tx_angle_deg),
            Angle::degrees(rx_angle_deg),
            env_losses,
        ))
    }

    /// Slant range in kilometers.
    #[getter]
    fn slant_range_km(&self) -> f64 {
        self.0.slant_range.to_kilometers()
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

    /// Eb/N0 in dB.
    #[getter]
    fn eb_n0(&self) -> PyDecibel {
        PyDecibel(self.0.eb_n0)
    }

    /// Link margin in dB.
    #[getter]
    fn margin(&self) -> PyDecibel {
        PyDecibel(self.0.margin)
    }

    /// Received carrier power in dBW.
    #[getter]
    fn carrier_rx_power(&self) -> PyDecibel {
        PyDecibel(self.0.carrier_rx_power)
    }

    /// Noise power in dBW.
    #[getter]
    fn noise_power(&self) -> PyDecibel {
        PyDecibel(self.0.noise_power)
    }

    /// Data rate in bits per second.
    #[getter]
    fn data_rate(&self) -> f64 {
        self.0.data_rate
    }

    /// Channel bandwidth in Hz.
    #[getter]
    fn bandwidth_hz(&self) -> f64 {
        self.0.bandwidth_hz
    }

    /// Link frequency in Hz.
    #[getter]
    fn frequency_hz(&self) -> f64 {
        f64::from(self.0.frequency)
    }

    fn __repr__(&self) -> String {
        format!(
            "LinkStats(c_n0={:.2} dB·Hz, eb_n0={:.2} dB, margin={:.2} dB)",
            self.0.c_n0.as_f64(),
            self.0.eb_n0.as_f64(),
            self.0.margin.as_f64(),
        )
    }
}

// --- Free functions ---

/// Computes the free-space path loss in dB.
///
/// Args:
///     distance_km: Distance in kilometers.
///     frequency_hz: Frequency in Hz.
///
/// Returns:
///     Free-space path loss as a Decibel value.
#[pyfunction]
pub fn fspl(distance_km: f64, frequency_hz: f64) -> PyDecibel {
    PyDecibel(free_space_path_loss(
        Distance::kilometers(distance_km),
        Frequency::new(frequency_hz),
    ))
}

/// Computes the frequency overlap factor between a receiver and an interferer.
///
/// Args:
///     rx_freq_hz: Receiver center frequency in Hz.
///     rx_bw_hz: Receiver bandwidth in Hz.
///     tx_freq_hz: Interferer center frequency in Hz.
///     tx_bw_hz: Interferer bandwidth in Hz.
///
/// Returns:
///     Overlap factor in [0, 1].
#[pyfunction]
pub fn freq_overlap(rx_freq_hz: f64, rx_bw_hz: f64, tx_freq_hz: f64, tx_bw_hz: f64) -> f64 {
    frequency_overlap_factor(rx_freq_hz, rx_bw_hz, tx_freq_hz, tx_bw_hz)
}
