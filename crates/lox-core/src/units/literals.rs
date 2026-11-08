// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Extension traits for writing quantities as literals, e.g. `7.0.km()` or `30.deg()`.
//!
//! The literals for [`TimeDelta`](crate::time::TimeDelta) are
//! [`TimeDeltaLiterals`](crate::time::TimeDeltaLiterals).

use super::{
    Angle, Area, AreaToMass, DataRate, DataVolume, Decibel, DecibelWatts, Distance, Frequency,
    Mass, Power, Velocity,
};

/// A trait for creating [`Angle`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::AngleLiterals;
///
/// let angle = 360.deg();
/// assert_eq!(angle.to_radians(), std::f64::consts::TAU);
/// ```
pub trait AngleLiterals {
    /// Creates an angle from a value in radians.
    fn rad(&self) -> Angle;
    /// Creates an angle from a value in degrees.
    fn deg(&self) -> Angle;
    /// Creates an angle from a value in arcseconds.
    fn arcsec(&self) -> Angle;
    /// Creates an angle from a value in milliarcseconds.
    fn mas(&self) -> Angle;
    /// Creates an angle from a value in microarcseconds.
    fn uas(&self) -> Angle;
}

impl AngleLiterals for f64 {
    fn rad(&self) -> Angle {
        Angle::radians(*self)
    }

    fn deg(&self) -> Angle {
        Angle::degrees(*self)
    }

    fn arcsec(&self) -> Angle {
        Angle::arcseconds(*self)
    }

    fn mas(&self) -> Angle {
        Angle::milliarcseconds(*self)
    }

    fn uas(&self) -> Angle {
        Angle::microarcseconds(*self)
    }
}

impl AngleLiterals for i64 {
    fn rad(&self) -> Angle {
        Angle::radians(*self as f64)
    }

    fn deg(&self) -> Angle {
        Angle::degrees(*self as f64)
    }

    fn arcsec(&self) -> Angle {
        Angle::arcseconds(*self as f64)
    }

    fn mas(&self) -> Angle {
        Angle::milliarcseconds(*self as f64)
    }

    fn uas(&self) -> Angle {
        Angle::microarcseconds(*self as f64)
    }
}

/// A trait for creating [`Distance`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::DistanceLiterals;
///
/// let d = 1.km();
/// assert_eq!(d.to_meters(), 1e3);
/// ```
pub trait DistanceLiterals {
    /// Creates a distance from a value in meters.
    fn m(&self) -> Distance;
    /// Creates a distance from a value in kilometers.
    fn km(&self) -> Distance;
    /// Creates a distance from a value in astronomical units.
    fn au(&self) -> Distance;
}

impl DistanceLiterals for f64 {
    fn m(&self) -> Distance {
        Distance::meters(*self)
    }

    fn km(&self) -> Distance {
        Distance::kilometers(*self)
    }

    fn au(&self) -> Distance {
        Distance::astronomical_units(*self)
    }
}

impl DistanceLiterals for i64 {
    fn m(&self) -> Distance {
        Distance::meters(*self as f64)
    }

    fn km(&self) -> Distance {
        Distance::kilometers(*self as f64)
    }

    fn au(&self) -> Distance {
        Distance::astronomical_units(*self as f64)
    }
}

/// A trait for creating [`Velocity`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::VelocityLiterals;
///
/// let v = 1.kps();
/// assert_eq!(v.to_meters_per_second(), 1e3);
/// ```
pub trait VelocityLiterals {
    /// Creates a velocity from a value in m/s.
    fn mps(&self) -> Velocity;
    /// Creates a velocity from a value in km/s.
    fn kps(&self) -> Velocity;
    /// Creates a velocity from a value in au/d.
    fn aud(&self) -> Velocity;
}

impl VelocityLiterals for f64 {
    fn mps(&self) -> Velocity {
        Velocity::meters_per_second(*self)
    }

    fn kps(&self) -> Velocity {
        Velocity::kilometers_per_second(*self)
    }

    fn aud(&self) -> Velocity {
        Velocity::astronomical_units_per_day(*self)
    }
}

impl VelocityLiterals for i64 {
    fn mps(&self) -> Velocity {
        Velocity::meters_per_second(*self as f64)
    }

    fn kps(&self) -> Velocity {
        Velocity::kilometers_per_second(*self as f64)
    }

    fn aud(&self) -> Velocity {
        Velocity::astronomical_units_per_day(*self as f64)
    }
}

/// A trait for creating [`Frequency`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::FrequencyLiterals;
///
/// let f = 1.ghz();
/// assert_eq!(f.to_hertz(), 1e9);
/// ```
pub trait FrequencyLiterals {
    /// Creates a frequency from a value in Hz.
    fn hz(&self) -> Frequency;
    /// Creates a frequency from a value in KHz.
    fn khz(&self) -> Frequency;
    /// Creates a frequency from a value in MHz.
    fn mhz(&self) -> Frequency;
    /// Creates a frequency from a value in GHz.
    fn ghz(&self) -> Frequency;
    /// Creates a frequency from a value in THz.
    fn thz(&self) -> Frequency;
}

impl FrequencyLiterals for f64 {
    fn hz(&self) -> Frequency {
        Frequency::hertz(*self)
    }

    fn khz(&self) -> Frequency {
        Frequency::kilohertz(*self)
    }

    fn mhz(&self) -> Frequency {
        Frequency::megahertz(*self)
    }

    fn ghz(&self) -> Frequency {
        Frequency::gigahertz(*self)
    }

    fn thz(&self) -> Frequency {
        Frequency::terahertz(*self)
    }
}

impl FrequencyLiterals for i64 {
    fn hz(&self) -> Frequency {
        Frequency::hertz(*self as f64)
    }

    fn khz(&self) -> Frequency {
        Frequency::kilohertz(*self as f64)
    }

    fn mhz(&self) -> Frequency {
        Frequency::megahertz(*self as f64)
    }

    fn ghz(&self) -> Frequency {
        Frequency::gigahertz(*self as f64)
    }

    fn thz(&self) -> Frequency {
        Frequency::terahertz(*self as f64)
    }
}

/// A trait for creating [`Mass`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::MassLiterals;
///
/// let m = 1.kg();
/// assert_eq!(m.to_kilograms(), 1.0);
/// ```
pub trait MassLiterals {
    /// Creates a mass from a value in kilograms.
    fn kg(&self) -> Mass;
}

impl MassLiterals for f64 {
    fn kg(&self) -> Mass {
        Mass::kilograms(*self)
    }
}

impl MassLiterals for i64 {
    fn kg(&self) -> Mass {
        Mass::kilograms(*self as f64)
    }
}

/// A trait for creating [`Area`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::AreaLiterals;
///
/// let a = 4.m2();
/// assert_eq!(a.to_square_meters(), 4.0);
/// ```
pub trait AreaLiterals {
    /// Creates an area from a value in square meters.
    fn m2(&self) -> Area;
    /// Creates an area from a value in square kilometers.
    fn km2(&self) -> Area;
}

impl AreaLiterals for f64 {
    fn m2(&self) -> Area {
        Area::square_meters(*self)
    }

    fn km2(&self) -> Area {
        Area::square_kilometers(*self)
    }
}

impl AreaLiterals for i64 {
    fn m2(&self) -> Area {
        Area::square_meters(*self as f64)
    }

    fn km2(&self) -> Area {
        Area::square_kilometers(*self as f64)
    }
}

/// A trait for creating [`AreaToMass`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::AreaToMassLiterals;
///
/// let r = 0.05.m2_per_kg();
/// assert_eq!(r.to_square_meters_per_kilogram(), 0.05);
/// ```
pub trait AreaToMassLiterals {
    /// Creates an area-to-mass ratio from a value in m²/kg.
    fn m2_per_kg(&self) -> AreaToMass;
}

impl AreaToMassLiterals for f64 {
    fn m2_per_kg(&self) -> AreaToMass {
        AreaToMass::square_meters_per_kilogram(*self)
    }
}

impl AreaToMassLiterals for i64 {
    fn m2_per_kg(&self) -> AreaToMass {
        AreaToMass::square_meters_per_kilogram(*self as f64)
    }
}

/// Extension trait for ergonomic power construction.
///
/// # Example
///
/// ```
/// use lox_core::units::literals::PowerLiterals;
///
/// let p = 2.w();
/// assert_eq!(p.to_watts(), 2.0);
/// ```
pub trait PowerLiterals {
    /// Creates a power from a value in watts.
    fn w(&self) -> Power;
    /// Creates a power from a value in kilowatts.
    fn kw(&self) -> Power;
}

impl PowerLiterals for f64 {
    fn w(&self) -> Power {
        Power::watts(*self)
    }

    fn kw(&self) -> Power {
        Power::kilowatts(*self)
    }
}

impl PowerLiterals for i64 {
    fn w(&self) -> Power {
        Power::watts(*self as f64)
    }

    fn kw(&self) -> Power {
        Power::kilowatts(*self as f64)
    }
}

/// A trait for creating [`Decibel`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::DecibelLiterals;
///
/// let d = 3.0.db();
/// assert_eq!(d.to_decibels(), 3.0);
/// ```
pub trait DecibelLiterals {
    /// Creates a decibel value.
    fn db(&self) -> Decibel;
}

impl DecibelLiterals for f64 {
    fn db(&self) -> Decibel {
        Decibel::decibels(*self)
    }
}

impl DecibelLiterals for i64 {
    fn db(&self) -> Decibel {
        Decibel::decibels(*self as f64)
    }
}

/// A trait for creating [`DataRate`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::DataRateLiterals;
///
/// let r = 150.mbps();
/// assert_eq!(r.to_bits_per_second(), 150e6);
/// ```
pub trait DataRateLiterals {
    /// Creates a data rate from a value in bit/s.
    fn bps(&self) -> DataRate;
    /// Creates a data rate from a value in kbit/s.
    fn kbps(&self) -> DataRate;
    /// Creates a data rate from a value in Mbit/s.
    fn mbps(&self) -> DataRate;
    /// Creates a data rate from a value in Gbit/s.
    fn gbps(&self) -> DataRate;
}

impl DataRateLiterals for f64 {
    fn bps(&self) -> DataRate {
        DataRate::bits_per_second(*self)
    }

    fn kbps(&self) -> DataRate {
        DataRate::kilobits_per_second(*self)
    }

    fn mbps(&self) -> DataRate {
        DataRate::megabits_per_second(*self)
    }

    fn gbps(&self) -> DataRate {
        DataRate::gigabits_per_second(*self)
    }
}

impl DataRateLiterals for i64 {
    fn bps(&self) -> DataRate {
        DataRate::bits_per_second(*self as f64)
    }

    fn kbps(&self) -> DataRate {
        DataRate::kilobits_per_second(*self as f64)
    }

    fn mbps(&self) -> DataRate {
        DataRate::megabits_per_second(*self as f64)
    }

    fn gbps(&self) -> DataRate {
        DataRate::gigabits_per_second(*self as f64)
    }
}

/// A trait for creating [`DataVolume`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::DataVolumeLiterals;
///
/// let v = 2.bytes();
/// assert_eq!(v.to_bits(), 16.0);
/// ```
pub trait DataVolumeLiterals {
    /// Creates a data volume from a value in bits.
    fn bits(&self) -> DataVolume;
    /// Creates a data volume from a value in bytes.
    fn bytes(&self) -> DataVolume;
}

impl DataVolumeLiterals for f64 {
    fn bits(&self) -> DataVolume {
        DataVolume::bits(*self)
    }

    fn bytes(&self) -> DataVolume {
        DataVolume::bytes(*self)
    }
}

impl DataVolumeLiterals for i64 {
    fn bits(&self) -> DataVolume {
        DataVolume::bits(*self as f64)
    }

    fn bytes(&self) -> DataVolume {
        DataVolume::bytes(*self as f64)
    }
}

/// A trait for creating [`DecibelWatts`] instances from primitives.
///
/// By default it is implemented for [`f64`] and [`i64`].
///
/// # Examples
///
/// ```
/// use lox_core::units::literals::DecibelWattsLiterals;
///
/// let p = 30.dbm();
/// assert_eq!(p.to_decibel_watts(), 0.0);
/// ```
pub trait DecibelWattsLiterals {
    /// Creates a power level from a value in dBW.
    fn dbw(&self) -> DecibelWatts;
    /// Creates a power level from a value in dBm.
    fn dbm(&self) -> DecibelWatts;
}

impl DecibelWattsLiterals for f64 {
    fn dbw(&self) -> DecibelWatts {
        DecibelWatts::decibel_watts(*self)
    }

    fn dbm(&self) -> DecibelWatts {
        DecibelWatts::decibel_milliwatts(*self)
    }
}

impl DecibelWattsLiterals for i64 {
    fn dbw(&self) -> DecibelWatts {
        DecibelWatts::decibel_watts(*self as f64)
    }

    fn dbm(&self) -> DecibelWatts {
        DecibelWatts::decibel_milliwatts(*self as f64)
    }
}
