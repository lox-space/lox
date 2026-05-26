// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// PyO3 requires `&self` on #[pymethods], which conflicts with clippy's
// `wrong_self_convention` lint for `to_*` methods on Copy types.
#![allow(clippy::wrong_self_convention)]

use pyo3::{Bound, Python, pyclass, pymethods, types::PyComplex};
use std::format;
use std::string::{String, ToString};

use lox_units::{Angle, AngularRate, Distance, Frequency, Power, Pressure, Temperature, Velocity};

/// Formats an f64 as a valid Python float literal (always includes a decimal point).
fn repr_f64(v: f64) -> String {
    let s = v.to_string();
    if v.is_finite() && !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

macro_rules! py_unit {
    ($(($unit:ident, $name:literal, $pyunit:ident $(, { $($extra:tt)* })?)),*) => {
        $(
            #[pyclass(name = $name, module = "lox_space", frozen, from_py_object)]
            #[derive(Clone, Copy)]
            /// Python wrapper for a typed unit quantity.
            pub struct $pyunit(pub $unit);

            #[pymethods]
            impl $pyunit {
                #[new]
                /// Constructs the unit quantity from a raw numeric value.
                pub fn new(value: f64) -> Self {
                    Self($unit::new(value))
                }

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

                /// Scales the unit quantity by a scalar.
                pub fn __mul__(&self, other: f64) -> Self {
                    Self(other * self.0)
                }

                /// Scales the unit quantity by a scalar (right-hand side).
                pub fn __rmul__(&self, other: f64) -> Self {
                    Self(other * self.0)
                }

                /// Returns true if both quantities are numerically equal.
                pub fn __eq__(&self, other: &$pyunit) -> bool {
                    f64::from(self.0) == f64::from(other.0)
                }

                /// Returns the constructor arguments for pickling.
                pub fn __getnewargs__(&self) -> (f64,) {
                    (f64::from(self.0),)
                }

                /// Returns the developer-readable representation.
                pub fn __repr__(&self) -> String {
                    format!("{}({})", $name, repr_f64(f64::from(self.0)))
                }

                /// Returns the human-readable string representation.
                pub fn __str__(&self) -> String {
                    self.0.to_string()
                }

                /// Returns the value as a Python complex number.
                pub fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
                    PyComplex::from_doubles(py, self.0.into(), 0.0)
                }

                /// Returns the raw numeric value as a Python float.
                pub fn __float__(&self) -> f64 {
                    self.0.into()
                }

                /// Returns the value rounded to the nearest integer.
                pub fn __int__(&self) -> i64 {
                   let val: f64 = self.0.into();
                    val.round_ties_even() as i64
                }

                $($($extra)*)?
            }
        )*
    };
}

py_unit!(
    (Angle, "Angle", PyAngle, {
        /// Returns the value in radians.
        fn to_radians(&self) -> f64 {
            self.0.to_radians()
        }

        /// Returns the value in degrees.
        fn to_degrees(&self) -> f64 {
            self.0.to_degrees()
        }

        /// Returns the value in arcseconds.
        fn to_arcseconds(&self) -> f64 {
            self.0.to_arcseconds()
        }
    }),
    (AngularRate, "AngularRate", PyAngularRate, {
        /// Returns the value in radians per second.
        fn to_radians_per_second(&self) -> f64 {
            self.0.to_radians_per_second()
        }

        /// Returns the value in degrees per second.
        fn to_degrees_per_second(&self) -> f64 {
            self.0.to_degrees_per_second()
        }
    }),
    (Distance, "Distance", PyDistance, {
        /// Returns the value in meters.
        fn to_meters(&self) -> f64 {
            self.0.to_meters()
        }

        /// Returns the value in kilometers.
        fn to_kilometers(&self) -> f64 {
            self.0.to_kilometers()
        }

        /// Returns the value in astronomical units.
        fn to_astronomical_units(&self) -> f64 {
            self.0.to_astronomical_units()
        }
    }),
    (Frequency, "Frequency", PyFrequency, {
        /// Returns the value in hertz.
        fn to_hertz(&self) -> f64 {
            self.0.to_hertz()
        }

        /// Returns the value in kilohertz.
        fn to_kilohertz(&self) -> f64 {
            self.0.to_kilohertz()
        }

        /// Returns the value in megahertz.
        fn to_megahertz(&self) -> f64 {
            self.0.to_megahertz()
        }

        /// Returns the value in gigahertz.
        fn to_gigahertz(&self) -> f64 {
            self.0.to_gigahertz()
        }

        /// Returns the value in terahertz.
        fn to_terahertz(&self) -> f64 {
            self.0.to_terahertz()
        }
    }),
    (Power, "Power", PyPower, {
        /// Returns the value in Watts.
        fn to_watts(&self) -> f64 {
            self.0.to_watts()
        }

        /// Returns the value in kilowatts.
        fn to_kilowatts(&self) -> f64 {
            self.0.to_kilowatts()
        }

        /// Returns the value in dBW.
        fn to_dbw(&self) -> f64 {
            self.0.to_dbw()
        }
    }),
    (Pressure, "Pressure", PyPressure, {
        /// Returns the value in hectopascals.
        fn to_hpa(&self) -> f64 {
            self.0.to_hpa()
        }

        /// Returns the value in pascals.
        fn to_pa(&self) -> f64 {
            self.0.to_pa()
        }
    }),
    (Temperature, "Temperature", PyTemperature, {
        /// Returns the value in Kelvin.
        fn to_kelvin(&self) -> f64 {
            self.0.to_kelvin()
        }
    }),
    (Velocity, "Velocity", PyVelocity, {
        /// Returns the value in meters per second.
        fn to_meters_per_second(&self) -> f64 {
            self.0.to_meters_per_second()
        }

        /// Returns the value in kilometers per second.
        fn to_kilometers_per_second(&self) -> f64 {
            self.0.to_kilometers_per_second()
        }
    })
);

// --- GravitationalParameter (manual impl since it's in elements.rs) ---

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

#[pymethods]
impl PyGravitationalParameter {
    #[new]
    /// Constructs a gravitational parameter from a value in m³/s².
    pub fn new(value: f64) -> Self {
        Self(GravitationalParameter::m3_per_s2(value))
    }

    /// Creates a GravitationalParameter from a value in km³/s².
    #[staticmethod]
    fn from_km3_per_s2(value: f64) -> Self {
        Self(GravitationalParameter::km3_per_s2(value))
    }

    /// Returns the value in m³/s².
    fn to_m3_per_s2(&self) -> f64 {
        self.0.as_f64()
    }

    /// Returns the value in km³/s².
    fn to_km3_per_s2(&self) -> f64 {
        self.0.as_f64() * 1e-9
    }

    fn __float__(&self) -> f64 {
        self.0.as_f64()
    }

    fn __eq__(&self, other: &PyGravitationalParameter) -> bool {
        self.0.as_f64() == other.0.as_f64()
    }

    fn __getnewargs__(&self) -> (f64,) {
        (self.0.as_f64(),)
    }

    fn __repr__(&self) -> String {
        format!("GravitationalParameter({})", repr_f64(self.0.as_f64()))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
