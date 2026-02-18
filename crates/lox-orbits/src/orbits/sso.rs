// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use lox_bodies::{Earth, PointMass, Spheroid};
use lox_core::anomalies::TrueAnomaly;
use lox_core::elements::{
    ArgumentOfPeriapsis, Eccentricity, GravitationalParameter, Inclination, InclinationError,
    Keplerian, LongitudeOfAscendingNode, NegativeEccentricityError, SemiMajorAxis,
};
use lox_core::glam::Azimuth;
use lox_core::time::julian_dates::JulianDate;
use lox_core::time::time_of_day::TimeOfDay;
use lox_core::units::DistanceUnits;
use lox_core::units::{Angle, AngleUnits, Distance};
use lox_earth::ephemeris::apparent_sun_position;
use lox_frames::Icrf;
use lox_frames::iers::fundamental::iers03::lp_iers03;
use lox_time::Time;
use lox_time::offsets::{DefaultOffsetProvider, TryOffset};
use lox_time::time_of_day::TimeOfDayError;
use lox_time::time_scales::{Tai, Tdb, TimeScale, Ut1};
use thiserror::Error;

use crate::orbits::{KeplerianOrbit, Orbit};

const OMEGA_SUN_SYNC: f64 = 1.99651502e-7; // rad/sec
// FIXME: Implement trait in lox-bodies
const J2_EARTH: f64 = 0.001_082_626_174;

#[derive(Debug, Error)]
pub enum SsoError {
    #[error("either altitude, semi-major axis, or inclination need to be provided")]
    InvalidParameters,
    #[error("semi-major axis is out-of-range: {0}")]
    SemiMajorAxisOutOfRange(Distance),
    #[error("invalid local time of ascending/descending node: {0}")]
    InvalidLtan(String),
    #[error("offset provider error: {0}")]
    OffsetProvider(String),
    #[error(transparent)]
    Inclination(#[from] InclinationError),
    #[error(transparent)]
    Eccentricity(#[from] NegativeEccentricityError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalTimeOfNode {
    LTAN(TimeOfDay),
    LTDN(TimeOfDay),
}

impl LocalTimeOfNode {
    pub fn local_time_of_ascending_node(&self) -> Angle {
        let ltan = match self {
            LocalTimeOfNode::LTAN(time_of_day) => time_of_day.to_angle(),
            LocalTimeOfNode::LTDN(time_of_day) => {
                time_of_day.to_angle() + TimeOfDay::NOON.to_angle()
            }
        };
        ltan.mod_two_pi()
    }
}

impl Default for LocalTimeOfNode {
    fn default() -> Self {
        Self::LTAN(TimeOfDay::default())
    }
}

fn inclination_sso(
    semi_major_axis: Distance,
    eccentricity: Eccentricity,
) -> Result<Inclination, SsoError> {
    let r_eq = Earth.equatorial_radius().km().as_f64();
    let mu = GravitationalParameter::km3_per_s2(Earth.gravitational_parameter()).as_f64();
    let num = -semi_major_axis.as_f64().powf(7.0 / 2.0)
        * 2.0
        * OMEGA_SUN_SYNC
        * (1.0 - eccentricity.as_f64().powi(2)).powi(2);
    let den = 3.0 * r_eq.powi(2) * J2_EARTH * mu.sqrt();
    let inc = Angle::from_acos(num / den);
    if !inc.as_f64().is_finite() {
        return Err(SsoError::SemiMajorAxisOutOfRange(semi_major_axis));
    }
    Ok(Inclination::try_new(Angle::from_acos(num / den))?)
}

fn semi_major_axis_sso(inclination: Inclination, eccentricity: Eccentricity) -> Distance {
    let r_eq = Earth.equatorial_radius().km().as_f64();
    let mu = GravitationalParameter::km3_per_s2(Earth.gravitational_parameter()).as_f64();
    let num = -3.0 * r_eq.powi(2) * J2_EARTH * mu.sqrt();
    let den = 2.0 * OMEGA_SUN_SYNC * (1.0 - eccentricity.as_f64().powi(2)).powi(2);
    (num * inclination.as_f64().cos() / den).powf(2.0 / 7.0).m()
}

fn right_ascension_sun(time: Time<Tdb>) -> Angle {
    let sun = apparent_sun_position(time);
    sun.azimuth().mod_two_pi()
}

fn longitude_of_ascending_node_sso<T, P>(
    time: Time<T>,
    ltan: LocalTimeOfNode,
    provider: &P,
) -> Result<Angle, SsoError>
where
    T: TimeScale + Copy,
    P: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
{
    let tdb = time
        .try_to_scale(Tdb, provider)
        .map_err(|err| SsoError::OffsetProvider(err.to_string()))?;

    let t_ut1 = time
        .try_to_scale(Ut1, provider)
        .map_err(|err| SsoError::OffsetProvider(err.to_string()))?
        .centuries_since_j2000();

    let ra_sun = right_ascension_sun(tdb);

    // Apparent solar local time = RA + 12h
    let salt = ra_sun + TimeOfDay::NOON.to_angle();

    // G: Mean anomaly of the Sun
    let m_sun = lp_iers03(tdb.centuries_since_j2000());
    // L: Mean longitude of the Sun
    let l_sun = t_ut1.mul_add(36000.77, 280.460).deg();
    // λ: Longitude of the ecliptic
    let l_ecliptic_part2 =
        (1.914666471 * m_sun.sin() + 0.019994643 * (2.0 * m_sun.as_f64()).sin()).deg();
    let l_ecliptic = l_sun + l_ecliptic_part2;

    // Equation of time
    let eq_time = (-l_ecliptic_part2.to_degrees() + 2.466 * (2.0 * l_ecliptic.as_f64()).sin()
        - 0.0053 * (4.0 * l_ecliptic.as_f64()).sin())
    .deg();

    // Mean solar local time
    let smlt = salt + eq_time;

    // RAAN = smlt + LTAN
    Ok((smlt + ltan.local_time_of_ascending_node()).mod_two_pi())
}

#[derive(Debug)]
enum SemiMajorAxisOrInclination {
    SemiMajorAxis(SemiMajorAxis),
    Inclination(Inclination),
}

impl SemiMajorAxisOrInclination {
    fn semi_major_axis(&self, eccentricity: Eccentricity) -> SemiMajorAxis {
        match self {
            SemiMajorAxisOrInclination::SemiMajorAxis(semi_major_axis) => *semi_major_axis,
            SemiMajorAxisOrInclination::Inclination(inclination) => {
                semi_major_axis_sso(*inclination, eccentricity)
            }
        }
    }

    fn inclination(&self, eccentricity: Eccentricity) -> Result<Inclination, SsoError> {
        match self {
            SemiMajorAxisOrInclination::SemiMajorAxis(semi_major_axis) => {
                inclination_sso(*semi_major_axis, eccentricity)
            }
            SemiMajorAxisOrInclination::Inclination(inclination) => Ok(*inclination),
        }
    }
}

fn keplerian_from_sso<T, P>(
    time: Time<T>,
    semi_major_axis_or_inclination: SemiMajorAxisOrInclination,
    eccentricity: Eccentricity,
    ltan: LocalTimeOfNode,
    true_anomaly: TrueAnomaly,
    provider: &P,
) -> Result<Keplerian, SsoError>
where
    T: TimeScale + Copy,
    P: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
{
    let semi_major_axis = semi_major_axis_or_inclination.semi_major_axis(eccentricity);
    let inclination = semi_major_axis_or_inclination.inclination(eccentricity)?;
    let longitude_of_ascending_node =
        LongitudeOfAscendingNode::try_new(longitude_of_ascending_node_sso(time, ltan, provider)?)
            .expect("SSO RAAN should be valid");
    let argument_of_periapsis = ArgumentOfPeriapsis::default();

    Ok(Keplerian::new(
        semi_major_axis,
        eccentricity,
        inclination,
        longitude_of_ascending_node,
        argument_of_periapsis,
        true_anomaly,
    ))
}

impl<T> KeplerianOrbit<T, Earth, Icrf>
where
    T: TimeScale + Copy,
{
    fn from_sso<P>(
        time: Time<T>,
        semi_major_axis_or_inclination: SemiMajorAxisOrInclination,
        eccentricity: Eccentricity,
        ltan: LocalTimeOfNode,
        true_anomaly: TrueAnomaly,
        provider: &P,
    ) -> Result<Self, SsoError>
    where
        P: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
    {
        let state = keplerian_from_sso(
            time,
            semi_major_axis_or_inclination,
            eccentricity,
            ltan,
            true_anomaly,
            provider,
        )?;
        Ok(Orbit::from_state(state, time, Earth, Icrf))
    }
}

#[derive(Debug, Clone)]
pub struct SsoBuilder<'a, T: TimeScale + Copy, P: TryOffset<T, Ut1> + TryOffset<T, Tdb>> {
    time: Time<T>,
    semi_major_axis: Option<SemiMajorAxis>,
    eccentricity: Result<Eccentricity, NegativeEccentricityError>,
    inclination: Option<Result<Inclination, InclinationError>>,
    ltan: Result<LocalTimeOfNode, TimeOfDayError>,
    true_anomaly: TrueAnomaly,
    provider: Option<&'a P>,
}

impl<'a> SsoBuilder<'a, Tai, DefaultOffsetProvider> {
    pub fn new() -> Self {
        Self {
            time: Time::default(),
            semi_major_axis: None,
            eccentricity: Ok(Eccentricity::default()),
            inclination: None,
            ltan: Ok(LocalTimeOfNode::default()),
            true_anomaly: TrueAnomaly::default(),
            provider: None,
        }
    }
}

impl<'a> Default for SsoBuilder<'a, Tai, DefaultOffsetProvider> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T, U> SsoBuilder<'a, T, U>
where
    T: TimeScale + Copy,
    U: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
{
    pub fn with_provider<P>(self, provider: &'a P) -> SsoBuilder<'a, T, P>
    where
        P: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
    {
        SsoBuilder {
            time: self.time,
            semi_major_axis: self.semi_major_axis,
            eccentricity: self.eccentricity,
            inclination: self.inclination,
            ltan: self.ltan,
            true_anomaly: self.true_anomaly,
            provider: Some(provider),
        }
    }
}

impl<'a, S, P> SsoBuilder<'a, S, P>
where
    S: TimeScale + Copy,
    P: TryOffset<S, Ut1> + TryOffset<S, Tdb>,
{
    pub fn with_time<T: TimeScale + Copy>(self, time: Time<T>) -> SsoBuilder<'a, T, P>
    where
        P: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
    {
        SsoBuilder {
            time,
            semi_major_axis: self.semi_major_axis,
            eccentricity: self.eccentricity,
            inclination: self.inclination,
            ltan: self.ltan,
            true_anomaly: self.true_anomaly,
            provider: self.provider,
        }
    }
}

impl<'a, T, P> SsoBuilder<'a, T, P>
where
    T: TimeScale + Copy,
    P: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
{
    pub fn with_semi_major_axis(mut self, semi_major_axis: SemiMajorAxis) -> Self {
        self.semi_major_axis = Some(semi_major_axis);
        self
    }

    pub fn with_eccentricity(mut self, eccentricity: f64) -> Self {
        self.eccentricity = Eccentricity::try_new(eccentricity);
        self
    }

    pub fn with_altitude(mut self, altitude: Distance) -> Self {
        self.semi_major_axis = Some(altitude + Earth.equatorial_radius().km());
        self.eccentricity = Ok(Eccentricity::default());
        self
    }

    pub fn with_inclination(mut self, inclination: Angle) -> Self {
        self.inclination = Some(Inclination::try_new(inclination));
        self
    }

    pub fn with_ltan(mut self, hours: u8, minutes: u8) -> Self {
        let time = TimeOfDay::from_hour_and_minute(hours, minutes);
        self.ltan = time.map(LocalTimeOfNode::LTAN);
        self
    }

    pub fn with_ltdn(mut self, hours: u8, minutes: u8) -> Self {
        let time = TimeOfDay::from_hour_and_minute(hours, minutes);
        self.ltan = time.map(LocalTimeOfNode::LTDN);
        self
    }

    pub fn with_true_anomaly(mut self, true_anomaly: Angle) -> Self {
        self.true_anomaly = TrueAnomaly::new(true_anomaly);
        self
    }

    pub fn build(self) -> Result<KeplerianOrbit<T, Earth, Icrf>, SsoError>
    where
        DefaultOffsetProvider: TryOffset<T, Ut1> + TryOffset<T, Tdb>,
    {
        let semi_major_axis_or_inclination = match (self.semi_major_axis, self.inclination) {
            (None, Some(inclination)) => SemiMajorAxisOrInclination::Inclination(inclination?),
            (Some(semi_major_axis), None) => {
                SemiMajorAxisOrInclination::SemiMajorAxis(semi_major_axis)
            }
            (_, _) => return Err(SsoError::InvalidParameters),
        };
        let eccentricity = self.eccentricity?;
        let ltan = self
            .ltan
            .map_err(|err| SsoError::InvalidLtan(err.to_string()))?;
        match self.provider {
            Some(provider) => KeplerianOrbit::from_sso(
                self.time,
                semi_major_axis_or_inclination,
                eccentricity,
                ltan,
                self.true_anomaly,
                provider,
            ),
            None => KeplerianOrbit::from_sso(
                self.time,
                semi_major_axis_or_inclination,
                eccentricity,
                ltan,
                self.true_anomaly,
                &DefaultOffsetProvider,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use lox_core::units::DistanceUnits;
    use lox_earth::eop::{EopParser, EopProvider};
    use lox_test_utils::{assert_approx_eq, data_file};
    use lox_time::{deltas::TimeDelta, utc::Utc};

    use super::*;

    #[test]
    fn test_sso_inclination() {
        let exp = Inclination::try_new(98.627.deg()).unwrap();

        let act = inclination_sso(7178.1363.km(), Eccentricity::default()).unwrap();
        assert_approx_eq!(exp, act, rtol <= 1e-5);

        let act = inclination_sso(7179.821.km(), Eccentricity::try_new(0.02).unwrap()).unwrap();
        assert_approx_eq!(exp, act, rtol <= 1e-5);
    }

    #[test]
    #[should_panic]
    fn test_sso_invalid_sma() {
        inclination_sso(36000.0.km(), Eccentricity::default()).unwrap();
    }

    #[test]
    fn test_sso_semi_major_axis() {
        let exp = 7179.821.km();
        let act = semi_major_axis_sso(
            Inclination::try_new(98.627.deg()).unwrap(),
            Eccentricity::try_new(0.02).unwrap(),
        );
        assert_approx_eq!(exp, act, rtol <= 1e-5);
    }

    #[test]
    fn test_sso_longitude_of_ascending_node() {
        let exp = 350.5997.deg();
        let jd1 = 2458849.5;
        let jd2 = 49.78099017 - 1.0;
        let epoch = Utc::from_delta(TimeDelta::from_two_part_julian_date(jd1, jd2))
            .unwrap()
            .to_time()
            .to_scale(Tdb);
        let ltan = LocalTimeOfNode::LTAN(TimeOfDay::from_hour_and_minute(13, 30).unwrap());
        let act = longitude_of_ascending_node_sso(epoch, ltan, eop_provider()).unwrap();
        assert_approx_eq!(exp, act, atol <= 4e-3);
    }

    #[test]
    fn test_sso_builder() {
        let semi_major_axis = 7178.1363.km();
        let jd1 = 2458849.5;
        let jd2 = 49.78099017 - 1.0;
        let epoch = Utc::from_delta(TimeDelta::from_two_part_julian_date(jd1, jd2))
            .unwrap()
            .to_time()
            .to_scale(Tdb);

        let exp_node = LongitudeOfAscendingNode::try_new(350.5997.deg()).unwrap();
        let exp_inc = Inclination::try_new(98.627.deg()).unwrap();

        let sso = SsoBuilder::default()
            .with_provider(eop_provider())
            .with_semi_major_axis(semi_major_axis)
            .with_eccentricity(Eccentricity::default().as_f64())
            .with_true_anomaly(Angle::ZERO)
            .with_time(epoch)
            .with_ltan(13, 30)
            .build()
            .unwrap();
        let act_inc = sso.inclination();
        let act_node = sso.longitude_of_ascending_node();
        assert_approx_eq!(act_inc, exp_inc, rtol <= 1e-5);
        assert_approx_eq!(act_node, exp_node, rtol <= 4e-3);

        let altitude = semi_major_axis - Earth.equatorial_radius().km();
        let sso = SsoBuilder::default()
            .with_provider(eop_provider())
            .with_altitude(altitude)
            .with_time(epoch)
            .with_ltdn(1, 30)
            .build()
            .unwrap();
        let act_inc = sso.inclination();
        let act_node = sso.longitude_of_ascending_node();
        assert_approx_eq!(act_inc, exp_inc, rtol <= 1e-5);
        assert_approx_eq!(act_node, exp_node, rtol <= 4e-3);
    }

    fn eop_provider() -> &'static EopProvider {
        static EOP: OnceLock<EopProvider> = OnceLock::new();
        EOP.get_or_init(|| {
            EopParser::new()
                .from_path(data_file("iers/finals2000A.all.csv"))
                .parse()
                .unwrap()
        })
    }
}
