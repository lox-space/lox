// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// PyO3 requires `&self` on #[pymethods], which conflicts with clippy's
// `wrong_self_convention` lint for `to_*` methods on Copy types.
#![allow(clippy::wrong_self_convention)]

use numpy::PyArray;
use numpy::ndarray::arr0;
use pyo3::exceptions::PyTypeError;
use pyo3::sync::PyOnceLock;
use pyo3::types::{
    PyAnyMethods, PyComplex, PyFloat, PyFloatMethods, PyInt, PyModule, PyModuleMethods, PyString,
    PyType, PyTypeMethods,
};
use pyo3::{
    Borrowed, Bound, FromPyObject, Py, PyAny, PyErr, PyResult, Python, intern, pyclass, pymethods,
};
use std::f64::consts::PI;
use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

use lox_units::ASTRONOMICAL_UNIT;
use lox_units::{
    Angle, AngularRate, Decibel, Distance, Frequency, Power, Pressure, Quantity, Temperature,
    Velocity,
};

/// Significant digits used when rendering a quantity for humans.
///
/// Shorter than the 17 digits needed to round-trip an `f64`, which is what
/// suppresses artefacts such as `29.000000000000004` from scaling a value by a
/// unit constant. `__repr__` keeps full precision so `eval(repr(x))` still
/// reproduces the value exactly.
const DISPLAY_PRECISION: &str = ".15g";

/// Formats an f64 as a valid Python float literal (always includes a decimal point).
pub(crate) fn repr_f64(v: f64) -> String {
    let s = v.to_string();
    if v.is_finite() && !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

/// Formats `value` with a Python format spec by deferring to `float.__format__`.
pub(crate) fn format_f64(py: Python<'_>, value: f64, spec: &str) -> PyResult<String> {
    PyFloat::new(py, value)
        .call_method1("__format__", (spec,))?
        .extract()
}

/// Splits a Python format spec into the part that pads the finished string and
/// the part that formats the number.
///
/// `f"{d:>12.1f}"` must right-align `'909.4 km'` within twelve columns, not the
/// digits alone, so `[[fill]align][width]` is peeled off and applied to the
/// result. A `0` flag asks for numeric zero-padding, so there the width stays
/// with the number and nothing is peeled off.
fn split_format_spec(spec: &str) -> (String, String) {
    let chars: Vec<char> = spec.chars().collect();
    let is_align = |c: char| matches!(c, '<' | '>' | '^' | '=');

    let mut i = 0;
    let mut fill = None;
    let mut align = None;
    if chars.len() >= 2 && is_align(chars[1]) {
        fill = Some(chars[0]);
        align = Some(chars[1]);
        i = 2;
    } else if !chars.is_empty() && is_align(chars[0]) {
        align = Some(chars[0]);
        i = 1;
    }

    let flags_start = i;
    if i < chars.len() && matches!(chars[i], '+' | '-' | ' ') {
        i += 1;
    }
    if i < chars.len() && chars[i] == 'z' {
        i += 1;
    }
    if i < chars.len() && chars[i] == '#' {
        i += 1;
    }

    if i < chars.len() && chars[i] == '0' {
        // Numeric zero-padding pads the digits, so hand the whole spec to the number.
        return (String::new(), chars[flags_start..].iter().collect());
    }
    let flags: String = chars[flags_start..i].iter().collect();

    let width_start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    let width: String = chars[width_start..i].iter().collect();
    let tail: String = chars[i..].iter().collect();

    let mut pad = String::new();
    if let Some(fill) = fill {
        pad.push(fill);
    }
    // Numbers right-align by default, strings left-align, so make it explicit.
    pad.push(align.unwrap_or('>'));
    pad.push_str(&width);
    if width.is_empty() && align.is_none() {
        pad.clear();
    }

    (pad, format!("{flags}{tail}"))
}

/// Splits a trailing unit name off a format spec.
///
/// The numeric mini-language never ends in a space followed by a word, so
/// `".1f m"` unambiguously means "one decimal place, in metres".
pub(crate) fn split_unit_token(spec: &str) -> (&str, Option<&str>) {
    match spec.rsplit_once(' ') {
        // The token must start with a letter, and something must precede the
        // space. That rules out the two places a space is already meaningful:
        // the `sign` option in `" .1f"`, and a space fill in `"> 12.1f"`.
        Some((head, unit))
            if !head.is_empty() && unit.starts_with(|c: char| c.is_ascii_alphabetic()) =>
        {
            (head, Some(unit))
        }
        _ => (spec, None),
    }
}

/// Renders `value` followed by `suffix`, honouring a Python format spec.
pub(crate) fn format_in_unit(
    py: Python<'_>,
    value: f64,
    suffix: &str,
    spec: &str,
) -> PyResult<String> {
    if spec.is_empty() {
        return Ok(format!(
            "{} {}",
            format_f64(py, value, DISPLAY_PRECISION)?,
            suffix
        ));
    }
    let (pad, number_spec) = split_format_spec(spec);
    let body = format!("{} {}", format_f64(py, value, &number_spec)?, suffix);
    if pad.is_empty() {
        return Ok(body);
    }
    PyString::new(py, &body)
        .call_method1("__format__", (pad,))?
        .extract()
}

/// Renders a quantity in its display unit, honouring a Python format spec.
fn format_quantity<Q: Quantity>(py: Python<'_>, value: Q, spec: &str) -> PyResult<String> {
    format_in_unit(py, value.to_display(), Q::SUFFIX, spec)
}

/// A real number that is not itself a `lox_space` quantity.
///
/// PyO3 extracts `f64` through `PyFloat_AsDouble`, which honours `__float__` —
/// so a plain `f64` parameter would silently accept another quantity and make
/// `distance * distance` return a `Distance`. Requiring `Scalar` instead makes
/// PyO3 return `NotImplemented`, which Python reports as a `TypeError`.
pub struct Scalar(pub f64);

static NUMBERS_REAL: PyOnceLock<Py<PyType>> = PyOnceLock::new();

impl<'py> FromPyObject<'_, 'py> for Scalar {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        // Exact `float`: the overwhelmingly common case, as cheap as extracting an f64.
        if let Ok(float) = obj.cast_exact::<PyFloat>() {
            return Ok(Self(float.value()));
        }
        // `int`, `bool` and `float` subclasses, which is where `numpy.float64` lands.
        // No quantity can reach here: they all derive from `object`.
        if obj.is_instance_of::<PyInt>() || obj.is_instance_of::<PyFloat>() {
            return Ok(Self(obj.extract::<f64>()?));
        }
        // Anything else must positively declare itself a real number. numpy registers
        // its scalar types in the `numbers` ABCs, so `numpy.float32` and friends pass
        // while quantities, `TimeDelta` and `str` do not.
        let py = obj.py();
        if obj.is_instance(NUMBERS_REAL.import(py, "numbers", "Real")?)? {
            return Ok(Self(obj.extract::<f64>()?));
        }
        Err(PyTypeError::new_err(format!(
            "expected a real number, got {}",
            obj.get_type().name()?
        )))
    }
}

macro_rules! py_unit {
    ($((
        $unit:ident, $name:literal, $pyunit:ident, $pyunitunit:ident, $unitname:literal,
        units = [$(($usym:literal, $usuffix:literal, $uscale:expr)),* $(,)?],
        conv = [$(($to:ident, $from:ident, $ctor:ident, $unitdoc:literal)),* $(,)?]
        $(, { $($extra:tt)* })?
    )),* $(,)?) => {
        $(
            #[doc = concat!("A unit a `", $name, "` can be expressed in.")]
            ///
            /// Multiplying by a number produces a quantity, and dividing a
            /// quantity by one converts it: `500 * lox.km` is a `Distance`,
            /// and `distance / lox.m` is that distance in metres.
            #[pyclass(name = $unitname, module = "lox_space", frozen, from_py_object)]
            #[derive(Clone, Copy)]
            pub struct $pyunitunit {
                symbol: &'static str,
                suffix: &'static str,
                scale: f64,
            }

            impl $pyunitunit {
                /// The units this quantity can be expressed in, as
                /// `(constant name, display suffix, value in base SI units)`.
                const UNITS: &'static [(&'static str, &'static str, f64)] =
                    &[$(($usym, $usuffix, $uscale)),*];

                /// Looks a unit up by its constant name or its display suffix.
                fn lookup(symbol: &str) -> Option<Self> {
                    Self::UNITS
                        .iter()
                        .find(|(name, suffix, _)| *name == symbol || *suffix == symbol)
                        .map(|&(symbol, suffix, scale)| Self { symbol, suffix, scale })
                }

                fn unknown(symbol: &str) -> PyErr {
                    let known: Vec<&str> = Self::UNITS.iter().map(|(name, ..)| *name).collect();
                    pyo3::exceptions::PyValueError::new_err(format!(
                        "'{}' is not a {}; expected one of {}",
                        symbol,
                        $unitname,
                        known.join(", ")
                    ))
                }

                /// Registers the class and its unit constants on the module.
                fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
                    m.add_class::<$pyunitunit>()?;
                    for &(symbol, suffix, scale) in Self::UNITS {
                        m.add(symbol, Self { symbol, suffix, scale })?;
                    }
                    Ok(())
                }
            }

            #[pymethods]
            impl $pyunitunit {
                #[new]
                #[doc = concat!("Looks up a `", $unitname, "` by name, e.g. `\"km\"`.")]
                fn new(symbol: &str) -> PyResult<Self> {
                    Self::lookup(symbol).ok_or_else(|| Self::unknown(symbol))
                }

                /// The name this unit is exported under, e.g. `"km_per_s"`.
                #[getter]
                fn symbol(&self) -> &'static str {
                    self.symbol
                }

                /// The suffix used when rendering, e.g. `"km/s"`.
                #[getter]
                fn suffix(&self) -> &'static str {
                    self.suffix
                }

                /// The value of one of this unit in base SI units.
                #[getter]
                fn scale(&self) -> f64 {
                    self.scale
                }

                #[doc = concat!("Scales this unit into a `", $name, "`.")]
                fn __mul__(&self, other: Scalar) -> $pyunit {
                    $pyunit($unit::from_base(other.0 * self.scale))
                }

                #[doc = concat!("Scales this unit into a `", $name, "` (right-hand side).")]
                fn __rmul__(&self, other: Scalar) -> $pyunit {
                    $pyunit($unit::from_base(other.0 * self.scale))
                }

                fn __eq__(&self, other: &$pyunitunit) -> bool {
                    self.symbol == other.symbol
                }

                fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
                    PyString::new(py, self.symbol).hash()
                }

                fn __getnewargs__(&self) -> (&'static str,) {
                    (self.symbol,)
                }

                fn __repr__(&self) -> String {
                    format!("{}(\"{}\")", $unitname, self.symbol)
                }

                fn __str__(&self) -> &'static str {
                    self.suffix
                }
            }

            #[pyclass(name = $name, module = "lox_space", frozen, from_py_object)]
            #[derive(Debug, Clone, Copy)]
            /// Python wrapper for a typed unit quantity.
            pub struct $pyunit(pub $unit);

            impl $pyunit {
                /// Constructs the quantity from a value in its base SI unit.
                ///
                /// The Rust-side counterpart of the Python constructor, which
                /// takes a [`Scalar`] so that another quantity cannot be
                /// coerced into it.
                pub fn new(value: f64) -> Self {
                    Self($unit::new(value))
                }
            }

            #[pymethods]
            impl $pyunit {
                #[new]
                /// Constructs the unit quantity from a value in its base SI unit.
                fn py_new(value: Scalar) -> Self {
                    Self::new(value.0)
                }

                $(
                    #[doc = concat!("Returns the value in ", $unitdoc, ".")]
                    fn $to(&self) -> f64 {
                        self.0.$to()
                    }

                    #[classmethod]
                    #[doc = concat!("Creates a `", $name, "` from a value in ", $unitdoc, ".")]
                    fn $from(_cls: &Bound<'_, PyType>, value: Scalar) -> Self {
                        Self($unit::$ctor(value.0))
                    }
                )*

                /// Adds two unit quantities.
                pub fn __add__(&self, other: &$pyunit) -> Self {
                    Self(self.0 + other.0)
                }

                /// Subtracts another unit quantity.
                pub fn __sub__(&self, other: &$pyunit) -> Self {
                    Self(self.0 - other.0)
                }

                /// Negates the unit quantity.
                pub fn __neg__(&self) -> Self {
                    Self(-self.0)
                }

                /// Returns the magnitude of the unit quantity.
                pub fn __abs__(&self) -> Self {
                    Self($unit::from_base(self.0.to_base().abs()))
                }

                /// Scales the unit quantity by a scalar.
                pub fn __mul__(&self, other: Scalar) -> Self {
                    Self(other.0 * self.0)
                }

                /// Scales the unit quantity by a scalar (right-hand side).
                pub fn __rmul__(&self, other: Scalar) -> Self {
                    Self(other.0 * self.0)
                }

                /// Divides by a scalar, or by a same-typed quantity or unit for a plain number.
                pub fn __truediv__<'py>(
                    &self,
                    py: Python<'py>,
                    other: &Bound<'py, PyAny>,
                ) -> PyResult<Bound<'py, PyAny>> {
                    use pyo3::IntoPyObject;
                    if let Ok(quantity) = other.extract::<$pyunit>() {
                        let ratio = self.0.to_base() / quantity.0.to_base();
                        return Ok(ratio.into_pyobject(py)?.into_any());
                    }
                    if let Ok(unit) = other.extract::<$pyunitunit>() {
                        let converted = self.0.to_base() / unit.scale;
                        return Ok(converted.into_pyobject(py)?.into_any());
                    }
                    match other.extract::<Scalar>() {
                        Ok(scalar) => {
                            let scaled = Self($unit::from_base(self.0.to_base() / scalar.0));
                            Ok(scaled.into_pyobject(py)?.into_any())
                        }
                        Err(_) => Ok(py.NotImplemented().into_bound(py)),
                    }
                }

                /// Rounds the value in its base SI unit.
                #[pyo3(signature = (ndigits=None))]
                pub fn __round__(&self, py: Python<'_>, ndigits: Option<i32>) -> PyResult<Self> {
                    let rounded: f64 = PyFloat::new(py, self.0.to_base())
                        .call_method1("__round__", (ndigits,))?
                        .extract()?;
                    Ok(Self($unit::from_base(rounded)))
                }

                /// Returns true if both quantities are numerically equal.
                pub fn __eq__(&self, other: &$pyunit) -> bool {
                    self.0.to_base() == other.0.to_base()
                }

                /// Returns true if this quantity is smaller than `other`.
                pub fn __lt__(&self, other: &$pyunit) -> bool {
                    self.0.to_base() < other.0.to_base()
                }

                /// Returns true if this quantity is smaller than or equal to `other`.
                pub fn __le__(&self, other: &$pyunit) -> bool {
                    self.0.to_base() <= other.0.to_base()
                }

                /// Returns true if this quantity is greater than `other`.
                pub fn __gt__(&self, other: &$pyunit) -> bool {
                    self.0.to_base() > other.0.to_base()
                }

                /// Returns true if this quantity is greater than or equal to `other`.
                pub fn __ge__(&self, other: &$pyunit) -> bool {
                    self.0.to_base() >= other.0.to_base()
                }

                /// Hashes the base SI value, consistently with `__eq__`.
                pub fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
                    PyFloat::new(py, self.0.to_base()).hash()
                }

                /// Returns the constructor arguments for pickling.
                pub fn __getnewargs__(&self) -> (f64,) {
                    (self.0.to_base(),)
                }

                /// Returns the developer-readable representation.
                pub fn __repr__(&self) -> String {
                    format!("{}({})", $name, repr_f64(self.0.to_base()))
                }

                /// Returns the human-readable string representation.
                pub fn __str__(&self, py: Python<'_>) -> PyResult<String> {
                    format_quantity(py, self.0, "")
                }

                /// Renders the value using a Python format spec.
                ///
                /// The default is the type's display unit; a trailing unit name
                /// selects another, as in `f"{distance:.1f m}"`.
                pub fn __format__(&self, py: Python<'_>, spec: &str) -> PyResult<String> {
                    let (spec, unit) = split_unit_token(spec);
                    match unit {
                        None => format_quantity(py, self.0, spec),
                        Some(symbol) => {
                            let unit = $pyunitunit::lookup(symbol)
                                .ok_or_else(|| $pyunitunit::unknown(symbol))?;
                            format_in_unit(
                                py,
                                self.0.to_base() / unit.scale,
                                unit.suffix,
                                spec,
                            )
                        }
                    }
                }

                /// Returns the value as a Python complex number.
                pub fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
                    PyComplex::from_doubles(py, self.0.to_base(), 0.0)
                }

                /// Returns the base SI value as a Python float.
                pub fn __float__(&self) -> f64 {
                    self.0.to_base()
                }

                /// Returns the base SI value as a 0-d float64 NumPy array.
                ///
                /// Makes `np.array([500 * lox.km, 100 * lox.km])` a float64 array of
                /// metres rather than an object array. Quantities are scalars, and
                /// units are not preserved across the NumPy boundary.
                #[pyo3(signature = (dtype=None, copy=None))]
                pub fn __array__<'py>(
                    &self,
                    py: Python<'py>,
                    dtype: Option<&Bound<'py, PyAny>>,
                    copy: Option<bool>,
                ) -> PyResult<Bound<'py, PyAny>> {
                    // A fresh array is always built, so `copy` is always satisfiable.
                    let _ = copy;
                    let array = PyArray::from_owned_array(py, arr0(self.0.to_base())).into_any();
                    match dtype {
                        None => Ok(array),
                        Some(dtype) => array.call_method1(intern!(py, "astype"), (dtype,)),
                    }
                }

                /// Opts out of NumPy's ufunc machinery.
                ///
                /// Without this `np.float64(2) * distance` would go through NumPy and
                /// return a bare `np.float64`, silently dropping the unit. Opting out
                /// makes NumPy defer to `__rmul__`, which preserves the type.
                #[allow(non_upper_case_globals)]
                #[allow(non_upper_case_globals)]
                #[classattr]
                const __array_ufunc__: Option<Py<PyAny>> = None;

                $($($extra)*)?
            }
        )*

        /// Registers every quantity class, unit class and unit constant.
        pub fn register_units(m: &Bound<'_, PyModule>) -> PyResult<()> {
            $(
                m.add_class::<$pyunit>()?;
                $pyunitunit::register(m)?;
            )*
            m.add_class::<PyGravitationalParameter>()?;
            Ok(())
        }
    };
}

py_unit!(
    (
        Angle,
        "Angle",
        PyAngle,
        PyAngleUnit,
        "AngleUnit",
        units = [("deg", "deg", PI / 180.0), ("rad", "rad", 1.0)],
        conv = [
            (to_radians, from_radians, radians, "radians"),
            (to_degrees, from_degrees, degrees, "degrees"),
            (to_arcseconds, from_arcseconds, arcseconds, "arcseconds"),
        ]
    ),
    (
        AngularRate,
        "AngularRate",
        PyAngularRate,
        PyAngularRateUnit,
        "AngularRateUnit",
        units = [
            ("deg_per_s", "deg/s", PI / 180.0),
            ("rad_per_s", "rad/s", 1.0)
        ],
        conv = [
            (
                to_radians_per_second,
                from_radians_per_second,
                radians_per_second,
                "radians per second"
            ),
            (
                to_degrees_per_second,
                from_degrees_per_second,
                degrees_per_second,
                "degrees per second"
            ),
        ]
    ),
    (
        Decibel,
        "Decibel",
        PyDecibel,
        PyDecibelUnit,
        "DecibelUnit",
        units = [("dB", "dB", 1.0)],
        conv = [],
        {
            /// Creates a `Decibel` from a linear power ratio.
            #[staticmethod]
            fn from_linear(value: Scalar) -> Self {
                Self(Decibel::from_linear(value.0))
            }

            /// Returns the linear power ratio.
            fn to_linear(&self) -> f64 {
                self.0.to_linear()
            }
        }
    ),
    (
        Distance,
        "Distance",
        PyDistance,
        PyDistanceUnit,
        "DistanceUnit",
        units = [
            ("km", "km", 1e3),
            ("m", "m", 1.0),
            ("au", "au", ASTRONOMICAL_UNIT)
        ],
        conv = [
            (to_meters, from_meters, meters, "meters"),
            (to_kilometers, from_kilometers, kilometers, "kilometers"),
            (
                to_astronomical_units,
                from_astronomical_units,
                astronomical_units,
                "astronomical units"
            ),
        ]
    ),
    (
        Frequency,
        "Frequency",
        PyFrequency,
        PyFrequencyUnit,
        "FrequencyUnit",
        units = [
            ("GHz", "GHz", 1e9),
            ("Hz", "Hz", 1.0),
            ("kHz", "kHz", 1e3),
            ("MHz", "MHz", 1e6),
            ("THz", "THz", 1e12)
        ],
        conv = [
            (to_hertz, from_hertz, hertz, "hertz"),
            (to_kilohertz, from_kilohertz, kilohertz, "kilohertz"),
            (to_megahertz, from_megahertz, megahertz, "megahertz"),
            (to_gigahertz, from_gigahertz, gigahertz, "gigahertz"),
            (to_terahertz, from_terahertz, terahertz, "terahertz"),
        ]
    ),
    (
        Power,
        "Power",
        PyPower,
        PyPowerUnit,
        "PowerUnit",
        units = [("W", "W", 1.0), ("kW", "kW", 1e3)],
        conv = [
            (to_watts, from_watts, watts, "Watts"),
            (to_kilowatts, from_kilowatts, kilowatts, "kilowatts"),
        ],
        {
            /// Returns the value in dBW.
            fn to_dbw(&self) -> f64 {
                self.0.to_dbw()
            }
        }
    ),
    (
        Pressure,
        "Pressure",
        PyPressure,
        PyPressureUnit,
        "PressureUnit",
        units = [("Pa", "Pa", 1.0), ("hPa", "hPa", 100.0)],
        conv = [
            (to_pa, from_pa, pa, "pascals"),
            (to_hpa, from_hpa, hpa, "hectopascals"),
        ]
    ),
    (
        Temperature,
        "Temperature",
        PyTemperature,
        PyTemperatureUnit,
        "TemperatureUnit",
        units = [("K", "K", 1.0)],
        conv = [(to_kelvin, from_kelvin, kelvin, "Kelvin"),]
    ),
    (
        Velocity,
        "Velocity",
        PyVelocity,
        PyVelocityUnit,
        "VelocityUnit",
        units = [("km_per_s", "km/s", 1e3), ("m_per_s", "m/s", 1.0)],
        conv = [
            (
                to_meters_per_second,
                from_meters_per_second,
                meters_per_second,
                "meters per second"
            ),
            (
                to_kilometers_per_second,
                from_kilometers_per_second,
                kilometers_per_second,
                "kilometers per second"
            ),
        ]
    ),
);

// --- GravitationalParameter ---
//
// Hand-written rather than macro-generated: unlike the quantities above it has
// no Rust `Add`/`Sub`/`Neg` impls, and gravitational parameters are not
// meaningfully negated or scaled. The shared behaviour is delegated to the same
// helpers the macro uses, so only thin forwarding lives here.

use lox_core::elements::GravitationalParameter;

/// A gravitational parameter (GM) value.
///
/// Args:
///     value: The value in m³/s².
#[pyclass(
    name = "GravitationalParameter",
    module = "lox_space",
    frozen,
    from_py_object
)]
#[derive(Clone, Copy)]
pub struct PyGravitationalParameter(pub GravitationalParameter);

impl PyGravitationalParameter {
    /// Constructs a gravitational parameter from a value in m³/s².
    ///
    /// The Rust-side counterpart of the Python constructor, which takes a
    /// [`Scalar`] so that another quantity cannot be coerced into it.
    pub fn new(value: f64) -> Self {
        Self(GravitationalParameter::m3_per_s2(value))
    }
}

#[pymethods]
impl PyGravitationalParameter {
    #[new]
    /// Constructs a gravitational parameter from a value in m³/s².
    fn py_new(value: Scalar) -> Self {
        Self::new(value.0)
    }

    /// Creates a `GravitationalParameter` from a value in m³/s².
    #[classmethod]
    fn from_m3_per_s2(_cls: &Bound<'_, PyType>, value: Scalar) -> Self {
        Self::new(value.0)
    }

    /// Creates a `GravitationalParameter` from a value in km³/s².
    #[staticmethod]
    fn from_km3_per_s2(value: Scalar) -> Self {
        Self(GravitationalParameter::km3_per_s2(value.0))
    }

    /// Returns the value in m³/s².
    fn to_m3_per_s2(&self) -> f64 {
        self.0.to_m3_per_s2()
    }

    /// Returns the value in km³/s².
    fn to_km3_per_s2(&self) -> f64 {
        self.0.to_km3_per_s2()
    }

    /// Returns the magnitude of the gravitational parameter.
    fn __abs__(&self) -> Self {
        Self(GravitationalParameter::m3_per_s2(self.0.to_base().abs()))
    }

    /// Rounds the value in its base SI unit.
    #[pyo3(signature = (ndigits=None))]
    fn __round__(&self, py: Python<'_>, ndigits: Option<i32>) -> PyResult<Self> {
        let rounded: f64 = PyFloat::new(py, self.0.to_base())
            .call_method1(intern!(py, "__round__"), (ndigits,))?
            .extract()?;
        Ok(Self(GravitationalParameter::m3_per_s2(rounded)))
    }

    fn __float__(&self) -> f64 {
        self.0.to_base()
    }

    fn __eq__(&self, other: &PyGravitationalParameter) -> bool {
        self.0.to_base() == other.0.to_base()
    }

    fn __lt__(&self, other: &PyGravitationalParameter) -> bool {
        self.0.to_base() < other.0.to_base()
    }

    fn __le__(&self, other: &PyGravitationalParameter) -> bool {
        self.0.to_base() <= other.0.to_base()
    }

    fn __gt__(&self, other: &PyGravitationalParameter) -> bool {
        self.0.to_base() > other.0.to_base()
    }

    fn __ge__(&self, other: &PyGravitationalParameter) -> bool {
        self.0.to_base() >= other.0.to_base()
    }

    /// Hashes the base SI value, consistently with `__eq__`.
    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        PyFloat::new(py, self.0.to_base()).hash()
    }

    fn __getnewargs__(&self) -> (f64,) {
        (self.0.to_base(),)
    }

    fn __repr__(&self) -> String {
        format!("GravitationalParameter({})", repr_f64(self.0.to_base()))
    }

    fn __str__(&self, py: Python<'_>) -> PyResult<String> {
        format_quantity(py, self.0, "")
    }

    /// Renders the value in km³/s² using a Python format spec.
    fn __format__(&self, py: Python<'_>, spec: &str) -> PyResult<String> {
        format_quantity(py, self.0, spec)
    }

    /// Returns the base SI value as a 0-d float64 NumPy array.
    #[pyo3(signature = (dtype=None, copy=None))]
    fn __array__<'py>(
        &self,
        py: Python<'py>,
        dtype: Option<&Bound<'py, PyAny>>,
        copy: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let _ = copy;
        let array = PyArray::from_owned_array(py, arr0(self.0.to_base())).into_any();
        match dtype {
            None => Ok(array),
            Some(dtype) => array.call_method1(intern!(py, "astype"), (dtype,)),
        }
    }

    /// Opts out of NumPy's ufunc machinery so scalar arithmetic keeps the type.
    #[allow(non_upper_case_globals)]
    #[allow(non_upper_case_globals)]
    #[classattr]
    const __array_ufunc__: Option<Py<PyAny>> = None;
}
