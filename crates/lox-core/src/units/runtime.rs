// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Units selected at runtime: per-dimension unit enums, formatting in a chosen unit, and parsing
//! of `"<value> <unit>"` strings.

use core::fmt::{Alignment, Display, Formatter, LowerExp, Result, UpperExp, Write};
use core::str::FromStr;

use super::{
    Angle, AngularRate, Area, AreaToMass, Decibel, Distance, Frequency, Mass, Power, Pressure,
    Temperature, Velocity,
};

/// Error returned when parsing a unit or a `"<value> <unit>"` string fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParseUnitError {
    /// The input is not a value followed by a unit.
    #[error("expected a value followed by a unit")]
    InvalidFormat,
    /// The value is not a valid number.
    #[error("invalid number")]
    InvalidNumber,
    /// The unit symbol is not known for the dimension.
    #[error("unknown unit")]
    UnknownUnit,
    /// The value is infinite or NaN.
    #[error("value must be finite")]
    NonFinite,
}

/// Displays a value in a chosen unit, see e.g. [`Distance::display_in`].
///
/// All formatting options, including `{:e}` and `{:E}`, apply to the number, which is followed
/// by the unit symbol.
#[derive(Clone, Copy, Debug)]
pub struct DisplayIn {
    value: f64,
    symbol: &'static str,
}

/// Writes content of `chars` characters, padded to the formatter's width with its fill and
/// alignment. The content is right-aligned by default, like numbers.
pub(crate) fn write_padded(
    f: &mut Formatter<'_>,
    chars: usize,
    write: impl FnOnce(&mut Formatter<'_>) -> Result,
) -> Result {
    let padding = f.width().unwrap_or(0).saturating_sub(chars);
    let (before, after) = match f.align() {
        Some(Alignment::Left) => (0, padding),
        Some(Alignment::Center) => (padding / 2, padding - padding / 2),
        Some(Alignment::Right) | None => (padding, 0),
    };
    let fill = f.fill();
    for _ in 0..before {
        f.write_char(fill)?;
    }
    write(f)?;
    for _ in 0..after {
        f.write_char(fill)?;
    }
    Ok(())
}

impl Display for DisplayIn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.value, f)?;
        write!(f, " {}", self.symbol)
    }
}

impl LowerExp for DisplayIn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        LowerExp::fmt(&self.value, f)?;
        write!(f, " {}", self.symbol)
    }
}

impl UpperExp for DisplayIn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        UpperExp::fmt(&self.value, f)?;
        write!(f, " {}", self.symbol)
    }
}

/// Parses a finite value followed by a unit, with or without whitespace in between.
fn parse_value_and_unit<U>(
    s: &str,
    from_symbol: fn(&str) -> Option<U>,
) -> core::result::Result<(f64, U), ParseUnitError> {
    let s = s.trim();
    let (value, unit) = if let Some((value, unit)) = s.split_once(char::is_whitespace) {
        let value: f64 = value.parse().map_err(|_| ParseUnitError::InvalidNumber)?;
        let unit = unit.trim();
        let unit = from_symbol(unit).ok_or(ParseUnitError::UnknownUnit)?;
        (value, unit)
    } else {
        // Without whitespace, split where the rest is a known unit, e.g. "7km" or "1e3km".
        s.char_indices()
            .skip(1)
            .find_map(|(i, _)| Some((s[..i].parse::<f64>().ok()?, from_symbol(&s[i..])?)))
            .ok_or(ParseUnitError::InvalidFormat)?
    };
    if !value.is_finite() {
        return Err(ParseUnitError::NonFinite);
    }
    Ok((value, unit))
}

macro_rules! runtime_units {
    ($(
        $quantity:ident, $unit:ident, si = $si:ident {
            $($variant:ident, $symbol:literal $(| $alias:literal)*, $ctor:ident, $accessor:ident;)+
        }
    )+) => {
        $(
            #[doc = concat!("Units of [`", stringify!($quantity), "`].")]
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            #[non_exhaustive]
            pub enum $unit {
                $(
                    #[doc = concat!("`", $symbol, "`")]
                    $variant,
                )+
            }

            impl $unit {
                /// All units of this dimension.
                pub const ALL: &'static [Self] = &[$(Self::$variant),+];

                /// Returns the unit symbol.
                pub const fn symbol(self) -> &'static str {
                    match self {
                        $(Self::$variant => $symbol,)+
                    }
                }

                /// Returns the unit with the given symbol or alias, if any.
                pub fn from_symbol(symbol: &str) -> Option<Self> {
                    match symbol {
                        $($symbol $(| $alias)* => Some(Self::$variant),)+
                        _ => None,
                    }
                }
            }

            impl Display for $unit {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    f.pad(self.symbol())
                }
            }

            impl FromStr for $unit {
                type Err = ParseUnitError;

                fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                    Self::from_symbol(s).ok_or(ParseUnitError::UnknownUnit)
                }
            }

            impl $quantity {
                /// Creates a new value from an `f64` value in the given unit.
                pub const fn new_in(value: f64, unit: $unit) -> Self {
                    match unit {
                        $($unit::$variant => Self::$ctor(value),)+
                    }
                }

                /// Returns the value in the given unit.
                pub const fn to(self, unit: $unit) -> f64 {
                    match unit {
                        $($unit::$variant => self.$accessor(),)+
                    }
                }

                /// Returns an adapter that displays the value in the given unit.
                pub const fn display_in(self, unit: $unit) -> DisplayIn {
                    DisplayIn {
                        value: self.to(unit),
                        symbol: unit.symbol(),
                    }
                }
            }

            /// Displays the value in SI units. Use `display_in` for other units.
            impl Display for $quantity {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    Display::fmt(&self.display_in($unit::$si), f)
                }
            }

            /// Displays the value in SI units in scientific notation.
            impl LowerExp for $quantity {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    LowerExp::fmt(&self.display_in($unit::$si), f)
                }
            }

            /// Displays the value in SI units in scientific notation.
            impl UpperExp for $quantity {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    UpperExp::fmt(&self.display_in($unit::$si), f)
                }
            }

            /// Parses a value followed by a unit symbol, e.g. `"500 km"` or `"500km"`.
            impl FromStr for $quantity {
                type Err = ParseUnitError;

                fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
                    let (value, unit) = parse_value_and_unit(s, $unit::from_symbol)?;
                    Ok(Self::new_in(value, unit))
                }
            }
        )+
    };
}

runtime_units! {
    Angle, AngleUnit, si = Radians {
        Radians, "rad", radians, to_radians;
        Microradians, "µrad" | "μrad" | "urad", microradians, to_microradians;
        Degrees, "deg" | "°", degrees, to_degrees;
        Arcseconds, "arcsec" | "″", arcseconds, to_arcseconds;
        Milliarcseconds, "mas", milliarcseconds, to_milliarcseconds;
        Microarcseconds, "µas" | "μas" | "uas", microarcseconds, to_microarcseconds;
    }
    AngularRate, AngularRateUnit, si = RadiansPerSecond {
        RadiansPerSecond, "rad/s", radians_per_second, to_radians_per_second;
        DegreesPerSecond, "deg/s" | "°/s", degrees_per_second, to_degrees_per_second;
    }
    Area, AreaUnit, si = SquareMeters {
        SquareMeters, "m²" | "m2" | "m^2", square_meters, to_square_meters;
        SquareKilometers, "km²" | "km2" | "km^2", square_kilometers, to_square_kilometers;
    }
    AreaToMass, AreaToMassUnit, si = SquareMetersPerKilogram {
        SquareMetersPerKilogram, "m²/kg" | "m2/kg" | "m^2/kg", square_meters_per_kilogram,
            to_square_meters_per_kilogram;
    }
    Decibel, DecibelUnit, si = Decibels {
        Decibels, "dB", decibels, to_decibels;
    }
    Distance, DistanceUnit, si = Meters {
        Meters, "m", meters, to_meters;
        Kilometers, "km", kilometers, to_kilometers;
        AstronomicalUnits, "au" | "AU", astronomical_units, to_astronomical_units;
    }
    Frequency, FrequencyUnit, si = Hertz {
        Hertz, "Hz", hertz, to_hertz;
        Kilohertz, "kHz", kilohertz, to_kilohertz;
        Megahertz, "MHz", megahertz, to_megahertz;
        Gigahertz, "GHz", gigahertz, to_gigahertz;
        Terahertz, "THz", terahertz, to_terahertz;
    }
    Mass, MassUnit, si = Kilograms {
        Kilograms, "kg", kilograms, to_kilograms;
        Grams, "g", grams, to_grams;
        MetricTons, "t", metric_tons, to_metric_tons;
    }
    Power, PowerUnit, si = Watts {
        Watts, "W", watts, to_watts;
        Kilowatts, "kW", kilowatts, to_kilowatts;
    }
    Pressure, PressureUnit, si = Pascals {
        Pascals, "Pa", pascals, to_pascals;
        Hectopascals, "hPa", hectopascals, to_hectopascals;
    }
    Temperature, TemperatureUnit, si = Kelvin {
        Kelvin, "K", kelvin, to_kelvin;
    }
    Velocity, VelocityUnit, si = MetersPerSecond {
        MetersPerSecond, "m/s", meters_per_second, to_meters_per_second;
        KilometersPerSecond, "km/s", kilometers_per_second, to_kilometers_per_second;
        AstronomicalUnitsPerDay, "au/d" | "AU/d", astronomical_units_per_day, to_astronomical_units_per_day;
        FractionOfSpeedOfLight, "c", fraction_of_speed_of_light, to_fraction_of_speed_of_light;
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;

    use lox_approx::assert_approx_eq;

    use super::*;

    #[test]
    fn test_new_in_and_to_match_named_units() {
        let d = Distance::new_in(7000.0, DistanceUnit::Kilometers);
        assert_eq!(d, Distance::kilometers(7000.0));
        assert_eq!(d.to(DistanceUnit::Kilometers), 7000.0);
        assert_eq!(d.to(DistanceUnit::Meters), 7e6);
        let a = Angle::new_in(90.0, AngleUnit::Degrees);
        assert_eq!(a, Angle::FRAC_PI_2);
    }

    #[test]
    fn test_display_is_si() {
        assert_eq!(Distance::kilometers(7.0).to_string(), "7000 m");
        assert_eq!(Angle::radians(0.5).to_string(), "0.5 rad");
        assert_eq!(Frequency::gigahertz(8.4).to_string(), "8400000000 Hz");
        assert_eq!(Decibel::decibels(3.0).to_string(), "3 dB");
    }

    #[test]
    fn test_display_in_respects_precision() {
        let d = Distance::meters(7_000_123.0);
        assert_eq!(
            format!("{:.2}", d.display_in(DistanceUnit::Kilometers)),
            "7000.12 km"
        );
        assert_eq!(
            format!("{:.1}", Angle::FRAC_PI_2.display_in(AngleUnit::Degrees)),
            "90.0 deg"
        );
        assert_eq!(
            format!("{:.3}", Velocity::meters_per_second(1.0)),
            "1.000 m/s"
        );
    }

    #[test]
    fn test_unit_symbols_round_trip() {
        for unit in DistanceUnit::ALL {
            assert_eq!(unit.symbol().parse::<DistanceUnit>(), Ok(*unit));
            assert_eq!(unit.to_string(), unit.symbol());
        }
        for unit in AngleUnit::ALL {
            assert_eq!(unit.symbol().parse::<AngleUnit>(), Ok(*unit));
        }
        assert_eq!("uas".parse::<AngleUnit>(), Ok(AngleUnit::Microarcseconds));
        assert_eq!("m2".parse::<AreaUnit>(), Ok(AreaUnit::SquareMeters));
        assert_eq!(
            "AU/d".parse::<VelocityUnit>(),
            Ok(VelocityUnit::AstronomicalUnitsPerDay)
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "500 km".parse::<Distance>(),
            Ok(Distance::kilometers(500.0))
        );
        assert_eq!(
            "  8.4\tGHz ".parse::<Frequency>(),
            Ok(Frequency::gigahertz(8.4))
        );
        assert_approx_eq!(
            "-12.5 deg".parse::<Angle>().unwrap().to_degrees(),
            -12.5,
            rtol <= 1e-15
        );
    }

    #[test]
    fn test_from_str_without_whitespace() {
        assert_eq!("7km".parse::<Distance>(), Ok(Distance::kilometers(7.0)));
        assert_eq!("1e3km".parse::<Distance>(), Ok(Distance::kilometers(1e3)));
        assert_eq!("-90°".parse::<Angle>(), Ok(Angle::degrees(-90.0)));
        assert_eq!("2.5dB".parse::<Decibel>(), Ok(Decibel::decibels(2.5)));
        assert_eq!(
            "7mm".parse::<Distance>(),
            Err(ParseUnitError::InvalidFormat)
        );
    }

    #[test]
    fn test_unit_aliases() {
        // Micro sign (U+00B5), Greek mu (U+03BC), and ASCII.
        for s in ["5 µrad", "5 μrad", "5 urad"] {
            assert_eq!(s.parse::<Angle>(), Ok(Angle::microradians(5.0)));
        }
        for s in ["5 µas", "5 μas", "5 uas"] {
            assert_eq!(s.parse::<Angle>(), Ok(Angle::microarcseconds(5.0)));
        }
        // Seconds of time have no runtime unit; a bare "s" would be ambiguous with arcseconds.
        assert_eq!("10 s".parse::<Angle>(), Err(ParseUnitError::UnknownUnit));
    }

    #[test]
    fn test_formatting_options_apply_to_number() {
        let d = Distance::meters(7000.0);
        assert_eq!(format!("{d:<6}|"), "7000   m|");
        assert_eq!(format!("{d:6}"), "  7000 m");
        assert_eq!(format!("{d:+}"), "+7000 m");
        assert_eq!(format!("{d:08.1}"), "007000.0 m");
        assert_eq!(format!("{:+04}", Distance::meters(-7.0)), "-007 m");
    }

    #[test]
    fn test_scientific_notation() {
        let f = Frequency::gigahertz(8.4);
        assert_eq!(format!("{f:e}"), "8.4e9 Hz");
        assert_eq!(format!("{f:.2E}"), "8.40E9 Hz");
        assert_eq!(
            format!(
                "{:e}",
                Angle::microarcseconds(1.0).display_in(AngleUnit::Arcseconds)
            ),
            "1e-6 arcsec"
        );
    }

    #[test]
    fn test_from_str_rejects_non_finite_values() {
        for s in ["nan km", "inf km", "-inf km", "NaN m"] {
            assert_eq!(s.parse::<Distance>(), Err(ParseUnitError::NonFinite));
        }
        assert_eq!("infkm".parse::<Distance>(), Err(ParseUnitError::NonFinite));
    }

    #[test]
    fn test_unit_from_symbol() {
        assert_eq!(
            DistanceUnit::from_symbol("km"),
            Some(DistanceUnit::Kilometers)
        );
        assert_eq!(DistanceUnit::from_symbol("GHz"), None);
    }

    #[test]
    fn test_from_str_errors() {
        assert_eq!(
            "500".parse::<Distance>(),
            Err(ParseUnitError::InvalidFormat)
        );
        assert_eq!(
            "five km".parse::<Distance>(),
            Err(ParseUnitError::InvalidNumber)
        );
        assert_eq!(
            "500 GHz".parse::<Distance>(),
            Err(ParseUnitError::UnknownUnit)
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_uses_bare_si_numbers() {
        let d = Distance::kilometers(7.0);
        assert_eq!(serde_json::to_string(&d).unwrap(), "7000.0");
        assert_eq!(serde_json::from_str::<Distance>("7000.0").unwrap(), d);
        assert!(serde_json::from_str::<Distance>("\"7 km\"").is_err());
    }
}
