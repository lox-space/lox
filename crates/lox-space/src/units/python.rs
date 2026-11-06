// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// PyO3 requires `&self` on #[pymethods], which conflicts with clippy's
// `wrong_self_convention` lint for `to_*` methods on Copy types.
#![allow(clippy::wrong_self_convention)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyFloat;
use std::format;
use std::string::{String, ToString};

use lox_core::elements::{
    GravitationalParameter, GravitationalParameterUnit, SpecificAngularMomentum,
    SpecificAngularMomentumUnit, SpecificEnergy, SpecificEnergyUnit,
};
use lox_core::units::{
    Angle, AngleUnit, AngularRate, AngularRateUnit, Area, AreaToMass, AreaToMassUnit, AreaUnit,
    DataRate, DataRateUnit, DataVolume, DataVolumeUnit, Decibel, DecibelUnit, DecibelWatts,
    DecibelWattsUnit, Distance, DistanceUnit, Frequency, FrequencyUnit, Mass, MassUnit, Power,
    PowerUnit, Pressure, PressureUnit, Temperature, TemperatureUnit, Velocity, VelocityUnit,
};

use crate::time::python::deltas::PyTimeDelta;

/// Formats an f64 as a valid Python float literal (always includes a decimal point).
pub(crate) fn repr_f64(v: f64) -> String {
    let s = v.to_string();
    if v.is_finite() && !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

/// Formats `value` with a Python format spec, followed by the unit symbol.
pub(crate) fn format_with_unit(
    py: Python<'_>,
    value: f64,
    spec: &str,
    symbol: &str,
) -> PyResult<String> {
    let number: String = PyFloat::new(py, value)
        .call_method1("__format__", (spec,))?
        .extract()?;
    Ok(format!("{number} {symbol}"))
}

/// Defines the Python classes for a quantity and its unit.
///
/// `ops` is either `linear`, which adds the arithmetic shared by all linear quantities, or
/// `none`. `mul` and `div` list products and quotients with other types, as
/// `Rhs => Output`. Products listed in `mul` are also available with the operands swapped.
/// `square` gives the type of the product of two values of the quantity.
macro_rules! py_quantity {
    (
        $quantity:ident, $py_quantity:ident, $name:literal,
        $unit:ident, $py_unit:ident, $unit_name:literal,
        si = ($si_variant:ident, $si_ctor:ident, $si_accessor:ident),
        units = [$(($ctor:ident, $accessor:ident)),+ $(,)?],
        ops = none
        $(, extra = { $($extra:tt)* })?
    ) => {
        py_quantity!(@define $quantity, $py_quantity, $name, $unit, $py_unit, $unit_name,
            ($si_variant, $si_ctor, $si_accessor), [$(($ctor, $accessor)),+],
            { $($($extra)*)? });
    };

    (
        $quantity:ident, $py_quantity:ident, $name:literal,
        $unit:ident, $py_unit:ident, $unit_name:literal,
        si = ($si_variant:ident, $si_ctor:ident, $si_accessor:ident),
        units = [$(($ctor:ident, $accessor:ident)),+ $(,)?],
        ops = linear,
        $(square = $square_out:ident,)?
        mul = [$($mul_rhs:ident => $mul_out:ident),* $(,)?],
        div = [$($div_rhs:ident => $div_out:ident),* $(,)?]
        $(, extra = { $($extra:tt)* })?
    ) => {
        py_quantity!(@define $quantity, $py_quantity, $name, $unit, $py_unit, $unit_name,
            ($si_variant, $si_ctor, $si_accessor), [$(($ctor, $accessor)),+],
            {
                fn __neg__(&self) -> Self {
                    Self(-self.0)
                }

                fn __pos__(&self) -> Self {
                    *self
                }

                fn __abs__(&self) -> Self {
                    Self(self.0.abs())
                }

                fn __add__(&self, other: &Self) -> Self {
                    Self(self.0 + other.0)
                }

                fn __sub__(&self, other: &Self) -> Self {
                    Self(self.0 - other.0)
                }

                fn __mul__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                    $(
                        if let Ok(rhs) = other.cast::<Self>() {
                            return Ok($square_out(self.0 * rhs.get().0).into_pyobject(py)?.into_any().unbind());
                        }
                    )?
                    $(
                        if let Ok(rhs) = other.cast::<$mul_rhs>() {
                            return Ok($mul_out(self.0 * rhs.get().0).into_pyobject(py)?.into_any().unbind());
                        }
                    )*
                    match other.extract::<f64>() {
                        Ok(factor) => Ok(Self(self.0 * factor).into_pyobject(py)?.into_any().unbind()),
                        Err(_) => Ok(py.NotImplemented()),
                    }
                }

                fn __rmul__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                    $(
                        if let Ok(lhs) = other.cast::<$mul_rhs>() {
                            return Ok($mul_out(lhs.get().0 * self.0).into_pyobject(py)?.into_any().unbind());
                        }
                    )*
                    match other.extract::<f64>() {
                        Ok(factor) => Ok(Self(factor * self.0).into_pyobject(py)?.into_any().unbind()),
                        Err(_) => Ok(py.NotImplemented()),
                    }
                }

                fn __truediv__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                    if let Ok(rhs) = other.cast::<Self>() {
                        return Ok((self.0 / rhs.get().0).into_pyobject(py)?.into_any().unbind());
                    }
                    $(
                        if let Ok(rhs) = other.cast::<$div_rhs>() {
                            return Ok($div_out(self.0 / rhs.get().0).into_pyobject(py)?.into_any().unbind());
                        }
                    )*
                    match other.extract::<f64>() {
                        Ok(divisor) => Ok(Self(self.0 / divisor).into_pyobject(py)?.into_any().unbind()),
                        Err(_) => Ok(py.NotImplemented()),
                    }
                }

                $($($extra)*)?
            });
    };

    (
        @define $quantity:ident, $py_quantity:ident, $name:literal,
        $unit:ident, $py_unit:ident, $unit_name:literal,
        ($si_variant:ident, $si_ctor:ident, $si_accessor:ident),
        [$(($ctor:ident, $accessor:ident)),+],
        { $($methods:tt)* }
    ) => {
        #[doc = concat!("A unit of [`", $name, "`].")]
        ///
        /// Multiply a number by a unit to create a quantity, e.g. `7000 * lox.km`.
        ///
        /// Args:
        ///     symbol: The unit symbol, e.g. "km".
        #[pyclass(name = $unit_name, module = "lox_space", frozen, eq, hash, from_py_object)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $py_unit(pub $unit);

        #[pymethods]
        impl $py_unit {
            #[new]
            fn new(symbol: &str) -> PyResult<Self> {
                symbol
                    .parse()
                    .map(Self)
                    .map_err(|_| PyValueError::new_err(format!("unknown unit `{symbol}`")))
            }

            /// The unit symbol.
            #[getter]
            fn symbol(&self) -> &'static str {
                self.0.symbol()
            }

            fn __mul__(&self, value: f64) -> $py_quantity {
                $py_quantity($quantity::new_in(value, self.0))
            }

            fn __rmul__(&self, value: f64) -> $py_quantity {
                $py_quantity($quantity::new_in(value, self.0))
            }

            fn __getnewargs__(&self) -> (&'static str,) {
                (self.0.symbol(),)
            }

            fn __repr__(&self) -> String {
                format!("{}(\"{}\")", $unit_name, self.0.symbol())
            }

            fn __str__(&self) -> &'static str {
                self.0.symbol()
            }
        }

        #[doc = concat!("A `", $name, "` value, stored in SI units.")]
        ///
        #[doc = concat!(
            "Create it by multiplying a number by a `", $unit_name,
            "`, or with one of the static constructors named after the units."
        )]
        #[pyclass(name = $name, module = "lox_space", frozen, from_py_object)]
        #[derive(Clone, Copy)]
        pub struct $py_quantity(pub $quantity);

        #[pymethods]
        impl $py_quantity {
            $(
                /// Creates the quantity from a value in the unit of the same name.
                #[staticmethod]
                pub fn $ctor(value: f64) -> Self {
                    Self($quantity::$ctor(value))
                }

                /// Returns the value in the unit of the same name.
                pub fn $accessor(&self) -> f64 {
                    self.0.$accessor()
                }
            )+

            /// Returns the value in the given unit.
            pub fn to(&self, unit: $py_unit) -> f64 {
                self.0.to(unit.0)
            }

            /// Returns the quantity formatted in the given unit.
            ///
            /// Args:
            ///     unit: The unit to display the value in.
            ///     format_spec: A format spec for the number, e.g. ".3f".
            #[pyo3(signature = (unit, format_spec = ""))]
            pub fn display_in(&self, py: Python<'_>, unit: $py_unit, format_spec: &str) -> PyResult<String> {
                format_with_unit(py, self.0.to(unit.0), format_spec, unit.0.symbol())
            }

            fn __format__(&self, py: Python<'_>, format_spec: &str) -> PyResult<String> {
                if format_spec.is_empty() {
                    return Ok(self.0.to_string());
                }
                format_with_unit(py, self.0.$si_accessor(), format_spec, $unit::$si_variant.symbol())
            }

            /// Returns the value in SI units with the unit symbol.
            pub fn __str__(&self) -> String {
                self.0.to_string()
            }

            /// Returns a representation that evaluates to an equal value.
            pub fn __repr__(&self) -> String {
                format!(
                    "{}.{}({})",
                    $name,
                    stringify!($si_ctor),
                    repr_f64(self.0.$si_accessor())
                )
            }

            fn __reduce__<'py>(
                slf: &Bound<'py, Self>,
            ) -> PyResult<(Bound<'py, PyAny>, (f64,))> {
                let ctor = slf.get_type().getattr(stringify!($si_ctor))?;
                Ok((ctor, (slf.get().0.$si_accessor(),)))
            }

            fn __eq__(&self, other: &Self) -> bool {
                self.0 == other.0
            }

            fn __lt__(&self, other: &Self) -> bool {
                self.0 < other.0
            }

            fn __le__(&self, other: &Self) -> bool {
                self.0 <= other.0
            }

            fn __gt__(&self, other: &Self) -> bool {
                self.0 > other.0
            }

            fn __ge__(&self, other: &Self) -> bool {
                self.0 >= other.0
            }

            $($methods)*
        }
    };
}

py_quantity!(
    Angle, PyAngle, "Angle", AngleUnit, PyAngleUnit, "AngleUnit",
    si = (Radians, radians, to_radians),
    units = [
        (radians, to_radians),
        (microradians, to_microradians),
        (degrees, to_degrees),
        (arcseconds, to_arcseconds),
        (milliarcseconds, to_milliarcseconds),
        (microarcseconds, to_microarcseconds),
    ],
    ops = linear,
    mul = [],
    div = [PyTimeDelta => PyAngularRate]
);

py_quantity!(
    AngularRate, PyAngularRate, "AngularRate", AngularRateUnit, PyAngularRateUnit,
    "AngularRateUnit",
    si = (RadiansPerSecond, radians_per_second, to_radians_per_second),
    units = [
        (radians_per_second, to_radians_per_second),
        (degrees_per_second, to_degrees_per_second),
    ],
    ops = linear,
    mul = [PyTimeDelta => PyAngle],
    div = []
);

py_quantity!(
    Area, PyArea, "Area", AreaUnit, PyAreaUnit, "AreaUnit",
    si = (SquareMeters, square_meters, to_square_meters),
    units = [
        (square_meters, to_square_meters),
        (square_kilometers, to_square_kilometers),
    ],
    ops = linear,
    mul = [],
    div = [PyMass => PyAreaToMass]
);

py_quantity!(
    AreaToMass,
    PyAreaToMass,
    "AreaToMass",
    AreaToMassUnit,
    PyAreaToMassUnit,
    "AreaToMassUnit",
    si = (
        SquareMetersPerKilogram,
        square_meters_per_kilogram,
        to_square_meters_per_kilogram
    ),
    units = [(square_meters_per_kilogram, to_square_meters_per_kilogram)],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    DataRate, PyDataRate, "DataRate", DataRateUnit, PyDataRateUnit, "DataRateUnit",
    si = (BitsPerSecond, bits_per_second, to_bits_per_second),
    units = [
        (bits_per_second, to_bits_per_second),
        (kilobits_per_second, to_kilobits_per_second),
        (megabits_per_second, to_megabits_per_second),
        (gigabits_per_second, to_gigabits_per_second),
    ],
    ops = linear,
    mul = [PyTimeDelta => PyDataVolume],
    div = []
);

py_quantity!(
    DataVolume, PyDataVolume, "DataVolume", DataVolumeUnit, PyDataVolumeUnit, "DataVolumeUnit",
    si = (Bits, bits, to_bits),
    units = [
        (bits, to_bits),
        (kilobits, to_kilobits),
        (megabits, to_megabits),
        (gigabits, to_gigabits),
        (terabits, to_terabits),
        (bytes, to_bytes),
        (kilobytes, to_kilobytes),
        (megabytes, to_megabytes),
        (gigabytes, to_gigabytes),
        (terabytes, to_terabytes),
        (kibibytes, to_kibibytes),
        (mebibytes, to_mebibytes),
        (gibibytes, to_gibibytes),
        (tebibytes, to_tebibytes),
    ],
    ops = linear,
    mul = [],
    div = [PyTimeDelta => PyDataRate]
);

py_quantity!(
    Decibel,
    PyDecibel,
    "Decibel",
    DecibelUnit,
    PyDecibelUnit,
    "DecibelUnit",
    si = (Decibels, decibels, to_decibels),
    units = [(decibels, to_decibels)],
    ops = linear,
    mul = [],
    div = [],
    extra = {
        /// Creates a value in decibels from a linear power ratio.
        #[staticmethod]
        fn from_linear(value: f64) -> Self {
            Self(Decibel::from_linear(value))
        }

        /// Returns the linear power ratio.
        fn to_linear(&self) -> f64 {
            self.0.to_linear()
        }
    }
);

py_quantity!(
    DecibelWatts,
    PyDecibelWatts,
    "DecibelWatts",
    DecibelWattsUnit,
    PyDecibelWattsUnit,
    "DecibelWattsUnit",
    si = (DecibelWatts, decibel_watts, to_decibel_watts),
    units = [
        (decibel_watts, to_decibel_watts),
        (decibel_milliwatts, to_decibel_milliwatts),
    ],
    ops = none,
    extra = {
        /// Returns the power level as a linear power.
        fn to_power(&self) -> PyPower {
            PyPower(self.0.to_power())
        }

        fn __add__(&self, other: &PyDecibel) -> Self {
            Self(self.0 + other.0)
        }

        fn __radd__(&self, other: &PyDecibel) -> Self {
            Self(other.0 + self.0)
        }

        fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
            if let Ok(rhs) = other.cast::<PyDecibel>() {
                return Ok(Self(self.0 - rhs.get().0)
                    .into_pyobject(py)?
                    .into_any()
                    .unbind());
            }
            if let Ok(rhs) = other.cast::<Self>() {
                return Ok(PyDecibel(self.0 - rhs.get().0)
                    .into_pyobject(py)?
                    .into_any()
                    .unbind());
            }
            Ok(py.NotImplemented())
        }
    }
);

py_quantity!(
    Distance, PyDistance, "Distance", DistanceUnit, PyDistanceUnit, "DistanceUnit",
    si = (Meters, meters, to_meters),
    units = [
        (meters, to_meters),
        (kilometers, to_kilometers),
        (astronomical_units, to_astronomical_units),
    ],
    ops = linear,
    square = PyArea,
    mul = [],
    div = [PyTimeDelta => PyVelocity]
);

py_quantity!(
    Frequency,
    PyFrequency,
    "Frequency",
    FrequencyUnit,
    PyFrequencyUnit,
    "FrequencyUnit",
    si = (Hertz, hertz, to_hertz),
    units = [
        (hertz, to_hertz),
        (kilohertz, to_kilohertz),
        (megahertz, to_megahertz),
        (gigahertz, to_gigahertz),
        (terahertz, to_terahertz),
    ],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    GravitationalParameter,
    PyGravitationalParameter,
    "GravitationalParameter",
    GravitationalParameterUnit,
    PyGravitationalParameterUnit,
    "GravitationalParameterUnit",
    si = (CubicMetersPerSecondSquared, m3_per_s2, to_m3_per_s2),
    units = [(m3_per_s2, to_m3_per_s2), (km3_per_s2, to_km3_per_s2)],
    ops = none
);

py_quantity!(
    Mass,
    PyMass,
    "Mass",
    MassUnit,
    PyMassUnit,
    "MassUnit",
    si = (Kilograms, kilograms, to_kilograms),
    units = [
        (kilograms, to_kilograms),
        (grams, to_grams),
        (metric_tons, to_metric_tons),
    ],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    Power,
    PyPower,
    "Power",
    PowerUnit,
    PyPowerUnit,
    "PowerUnit",
    si = (Watts, watts, to_watts),
    units = [(watts, to_watts), (kilowatts, to_kilowatts)],
    ops = linear,
    mul = [],
    div = [],
    extra = {
        /// Returns the power as a level in dBW.
        fn to_decibel_watts(&self) -> PyDecibelWatts {
            PyDecibelWatts(self.0.to_decibel_watts())
        }
    }
);

py_quantity!(
    Pressure,
    PyPressure,
    "Pressure",
    PressureUnit,
    PyPressureUnit,
    "PressureUnit",
    si = (Pascals, pascals, to_pascals),
    units = [(pascals, to_pascals), (hectopascals, to_hectopascals)],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    SpecificAngularMomentum,
    PySpecificAngularMomentum,
    "SpecificAngularMomentum",
    SpecificAngularMomentumUnit,
    PySpecificAngularMomentumUnit,
    "SpecificAngularMomentumUnit",
    si = (
        SquareMetersPerSecond,
        square_meters_per_second,
        to_square_meters_per_second
    ),
    units = [
        (square_meters_per_second, to_square_meters_per_second),
        (
            square_kilometers_per_second,
            to_square_kilometers_per_second
        ),
    ],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    SpecificEnergy,
    PySpecificEnergy,
    "SpecificEnergy",
    SpecificEnergyUnit,
    PySpecificEnergyUnit,
    "SpecificEnergyUnit",
    si = (
        SquareMetersPerSecondSquared,
        square_meters_per_second_squared,
        to_square_meters_per_second_squared
    ),
    units = [
        (
            square_meters_per_second_squared,
            to_square_meters_per_second_squared
        ),
        (
            square_kilometers_per_second_squared,
            to_square_kilometers_per_second_squared
        ),
    ],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    Temperature,
    PyTemperature,
    "Temperature",
    TemperatureUnit,
    PyTemperatureUnit,
    "TemperatureUnit",
    si = (Kelvin, kelvin, to_kelvin),
    units = [(kelvin, to_kelvin)],
    ops = linear,
    mul = [],
    div = []
);

py_quantity!(
    Velocity, PyVelocity, "Velocity", VelocityUnit, PyVelocityUnit, "VelocityUnit",
    si = (MetersPerSecond, meters_per_second, to_meters_per_second),
    units = [
        (meters_per_second, to_meters_per_second),
        (kilometers_per_second, to_kilometers_per_second),
        (astronomical_units_per_day, to_astronomical_units_per_day),
        (fraction_of_speed_of_light, to_fraction_of_speed_of_light),
    ],
    ops = linear,
    mul = [PyTimeDelta => PyDistance],
    div = []
);

/// The module constants for `DataVolumeUnit`, e.g. `lox.MB`.
const DATA_VOLUME_CONSTANTS: [(&str, DataVolumeUnit); 14] = [
    ("bit", DataVolumeUnit::Bits),
    ("kbit", DataVolumeUnit::Kilobits),
    ("Mbit", DataVolumeUnit::Megabits),
    ("Gbit", DataVolumeUnit::Gigabits),
    ("Tbit", DataVolumeUnit::Terabits),
    ("B", DataVolumeUnit::Bytes),
    ("kB", DataVolumeUnit::Kilobytes),
    ("MB", DataVolumeUnit::Megabytes),
    ("GB", DataVolumeUnit::Gigabytes),
    ("TB", DataVolumeUnit::Terabytes),
    ("KiB", DataVolumeUnit::Kibibytes),
    ("MiB", DataVolumeUnit::Mebibytes),
    ("GiB", DataVolumeUnit::Gibibytes),
    ("TiB", DataVolumeUnit::Tebibytes),
];

/// Registers the quantity and unit classes and the unit constants, e.g. `lox.km`.
pub fn register_units(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAngle>()?;
    m.add_class::<PyAngleUnit>()?;
    m.add("rad", PyAngleUnit(AngleUnit::Radians))?;
    m.add("urad", PyAngleUnit(AngleUnit::Microradians))?;
    m.add("deg", PyAngleUnit(AngleUnit::Degrees))?;
    m.add("arcsec", PyAngleUnit(AngleUnit::Arcseconds))?;
    m.add("mas", PyAngleUnit(AngleUnit::Milliarcseconds))?;
    m.add("uas", PyAngleUnit(AngleUnit::Microarcseconds))?;

    m.add_class::<PyAngularRate>()?;
    m.add_class::<PyAngularRateUnit>()?;
    m.add(
        "rad_per_s",
        PyAngularRateUnit(AngularRateUnit::RadiansPerSecond),
    )?;
    m.add(
        "deg_per_s",
        PyAngularRateUnit(AngularRateUnit::DegreesPerSecond),
    )?;

    m.add_class::<PyArea>()?;
    m.add_class::<PyAreaUnit>()?;
    m.add("m2", PyAreaUnit(AreaUnit::SquareMeters))?;
    m.add("km2", PyAreaUnit(AreaUnit::SquareKilometers))?;

    m.add_class::<PyAreaToMass>()?;
    m.add_class::<PyAreaToMassUnit>()?;
    m.add(
        "m2_per_kg",
        PyAreaToMassUnit(AreaToMassUnit::SquareMetersPerKilogram),
    )?;

    m.add_class::<PyDataRate>()?;
    m.add_class::<PyDataRateUnit>()?;
    m.add("bps", PyDataRateUnit(DataRateUnit::BitsPerSecond))?;
    m.add("kbps", PyDataRateUnit(DataRateUnit::KilobitsPerSecond))?;
    m.add("Mbps", PyDataRateUnit(DataRateUnit::MegabitsPerSecond))?;
    m.add("Gbps", PyDataRateUnit(DataRateUnit::GigabitsPerSecond))?;

    m.add_class::<PyDataVolume>()?;
    m.add_class::<PyDataVolumeUnit>()?;
    for (name, unit) in DATA_VOLUME_CONSTANTS {
        m.add(name, PyDataVolumeUnit(unit))?;
    }

    m.add_class::<PyDecibel>()?;
    m.add_class::<PyDecibelUnit>()?;
    m.add("dB", PyDecibelUnit(DecibelUnit::Decibels))?;

    m.add_class::<PyDecibelWatts>()?;
    m.add_class::<PyDecibelWattsUnit>()?;
    m.add("dBW", PyDecibelWattsUnit(DecibelWattsUnit::DecibelWatts))?;
    m.add(
        "dBm",
        PyDecibelWattsUnit(DecibelWattsUnit::DecibelMilliwatts),
    )?;

    m.add_class::<PyDistance>()?;
    m.add_class::<PyDistanceUnit>()?;
    m.add("m", PyDistanceUnit(DistanceUnit::Meters))?;
    m.add("km", PyDistanceUnit(DistanceUnit::Kilometers))?;
    m.add("au", PyDistanceUnit(DistanceUnit::AstronomicalUnits))?;

    m.add_class::<PyFrequency>()?;
    m.add_class::<PyFrequencyUnit>()?;
    m.add("Hz", PyFrequencyUnit(FrequencyUnit::Hertz))?;
    m.add("kHz", PyFrequencyUnit(FrequencyUnit::Kilohertz))?;
    m.add("MHz", PyFrequencyUnit(FrequencyUnit::Megahertz))?;
    m.add("GHz", PyFrequencyUnit(FrequencyUnit::Gigahertz))?;
    m.add("THz", PyFrequencyUnit(FrequencyUnit::Terahertz))?;

    m.add_class::<PyGravitationalParameter>()?;
    m.add_class::<PyGravitationalParameterUnit>()?;
    m.add(
        "m3_per_s2",
        PyGravitationalParameterUnit(GravitationalParameterUnit::CubicMetersPerSecondSquared),
    )?;
    m.add(
        "km3_per_s2",
        PyGravitationalParameterUnit(GravitationalParameterUnit::CubicKilometersPerSecondSquared),
    )?;

    m.add_class::<PyMass>()?;
    m.add_class::<PyMassUnit>()?;
    m.add("g", PyMassUnit(MassUnit::Grams))?;
    m.add("kg", PyMassUnit(MassUnit::Kilograms))?;
    m.add("t", PyMassUnit(MassUnit::MetricTons))?;

    m.add_class::<PyPower>()?;
    m.add_class::<PyPowerUnit>()?;
    m.add("W", PyPowerUnit(PowerUnit::Watts))?;
    m.add("kW", PyPowerUnit(PowerUnit::Kilowatts))?;

    m.add_class::<PyPressure>()?;
    m.add_class::<PyPressureUnit>()?;
    m.add("Pa", PyPressureUnit(PressureUnit::Pascals))?;
    m.add("hPa", PyPressureUnit(PressureUnit::Hectopascals))?;

    m.add_class::<PySpecificAngularMomentum>()?;
    m.add_class::<PySpecificAngularMomentumUnit>()?;
    m.add(
        "m2_per_s",
        PySpecificAngularMomentumUnit(SpecificAngularMomentumUnit::SquareMetersPerSecond),
    )?;
    m.add(
        "km2_per_s",
        PySpecificAngularMomentumUnit(SpecificAngularMomentumUnit::SquareKilometersPerSecond),
    )?;

    m.add_class::<PySpecificEnergy>()?;
    m.add_class::<PySpecificEnergyUnit>()?;
    m.add(
        "m2_per_s2",
        PySpecificEnergyUnit(SpecificEnergyUnit::SquareMetersPerSecondSquared),
    )?;
    m.add(
        "km2_per_s2",
        PySpecificEnergyUnit(SpecificEnergyUnit::SquareKilometersPerSecondSquared),
    )?;

    m.add_class::<PyTemperature>()?;
    m.add_class::<PyTemperatureUnit>()?;
    m.add("K", PyTemperatureUnit(TemperatureUnit::Kelvin))?;

    m.add_class::<PyVelocity>()?;
    m.add_class::<PyVelocityUnit>()?;
    m.add("m_per_s", PyVelocityUnit(VelocityUnit::MetersPerSecond))?;
    m.add(
        "km_per_s",
        PyVelocityUnit(VelocityUnit::KilometersPerSecond),
    )?;
    m.add(
        "au_per_d",
        PyVelocityUnit(VelocityUnit::AstronomicalUnitsPerDay),
    )?;
    m.add("c", PyVelocityUnit(VelocityUnit::FractionOfSpeedOfLight))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_data_volume_unit_has_a_constant() {
        for unit in DataVolumeUnit::ALL {
            assert!(
                DATA_VOLUME_CONSTANTS.contains(&(unit.symbol(), *unit)),
                "no module constant named `{}` for {unit:?}",
                unit.symbol()
            );
        }
    }
}
