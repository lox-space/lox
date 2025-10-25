// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use core::f64::consts::TAU;

use glam::DVec3;
use thiserror::Error;

use crate::{Angle, Distance, Velocity};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct AzEl(Angle, Angle);

impl AzEl {
    pub fn builder() -> AzElBuilder {
        AzElBuilder::default()
    }

    pub fn azimuth(&self) -> Angle {
        self.0
    }

    pub fn elevation(&self) -> Angle {
        self.1
    }
}

#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum AzElError {
    #[error("azimuth must be between 0 deg and 360 deg but was {0}")]
    InvalidAzimuth(Angle),
    #[error("elevation must be between 0 deg and 360 deg but was {0}")]
    InvalidElevation(Angle),
}

#[derive(Copy, Clone, Debug)]
pub struct AzElBuilder {
    azimuth: Result<Angle, AzElError>,
    elevation: Result<Angle, AzElError>,
}

impl Default for AzElBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AzElBuilder {
    pub fn new() -> Self {
        Self {
            azimuth: Ok(Angle::default()),
            elevation: Ok(Angle::default()),
        }
    }

    pub fn azimuth(&mut self, azimuth: Angle) -> &mut Self {
        self.azimuth = match azimuth.to_radians() {
            lon if lon < 0.0 => Err(AzElError::InvalidAzimuth(azimuth)),
            lon if lon > TAU => Err(AzElError::InvalidAzimuth(azimuth)),
            _ => Ok(azimuth),
        };
        self
    }

    pub fn elevation(&mut self, elevation: Angle) -> &mut Self {
        self.elevation = match elevation.to_radians() {
            lat if lat < 0.0 => Err(AzElError::InvalidElevation(elevation)),
            lat if lat > TAU => Err(AzElError::InvalidElevation(elevation)),
            _ => Ok(elevation),
        };
        self
    }

    pub fn build(&self) -> Result<AzEl, AzElError> {
        Ok(AzEl(self.azimuth?, self.elevation?))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct LonLatAlt(Angle, Angle, Distance);

impl LonLatAlt {
    pub fn builder() -> LonLatAltBuilder {
        LonLatAltBuilder::default()
    }

    pub fn lon(&self) -> Angle {
        self.0
    }

    pub fn lat(&self) -> Angle {
        self.1
    }

    pub fn alt(&self) -> Distance {
        self.2
    }
}

#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum LonLatAltError {
    #[error("longitude must be between -180 deg and 180 deg but was {0}")]
    InvalidLongitude(Angle),
    #[error("latitude must between -90 deg and 90 deg but was {0}")]
    InvalidLatitude(Angle),
    #[error("invalid altitude {0}")]
    InvalidAltitude(Distance),
}

#[derive(Copy, Clone, Debug)]
pub struct LonLatAltBuilder {
    longitude: Result<Angle, LonLatAltError>,
    latitude: Result<Angle, LonLatAltError>,
    altitude: Result<Distance, LonLatAltError>,
}

impl Default for LonLatAltBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LonLatAltBuilder {
    pub fn new() -> Self {
        Self {
            longitude: Ok(Angle::default()),
            latitude: Ok(Angle::default()),
            altitude: Ok(Distance::default()),
        }
    }

    pub fn longitude(&mut self, longitude: Angle) -> &mut Self {
        self.longitude = match longitude.to_degrees() {
            lon if lon < -180.0 => Err(LonLatAltError::InvalidLongitude(longitude)),
            lon if lon > 180.0 => Err(LonLatAltError::InvalidLongitude(longitude)),
            _ => Ok(longitude),
        };
        self
    }

    pub fn latitude(&mut self, latitude: Angle) -> &mut Self {
        self.latitude = match latitude.to_degrees() {
            lat if lat < -90.0 => Err(LonLatAltError::InvalidLatitude(latitude)),
            lat if lat > 90.0 => Err(LonLatAltError::InvalidLatitude(latitude)),
            _ => Ok(latitude),
        };
        self
    }

    pub fn altitude(&mut self, altitude: Distance) -> &mut Self {
        self.altitude = if !altitude.to_meters().is_finite() {
            Err(LonLatAltError::InvalidAltitude(altitude))
        } else {
            Ok(altitude)
        };
        self
    }

    pub fn build(&self) -> Result<LonLatAlt, LonLatAltError> {
        Ok(LonLatAlt(self.longitude?, self.latitude?, self.altitude?))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Cartesian {
    pos: DVec3,
    vel: DVec3,
}

impl Cartesian {
    pub const fn new(
        x: Distance,
        y: Distance,
        z: Distance,
        vx: Velocity,
        vy: Velocity,
        vz: Velocity,
    ) -> Self {
        Self {
            pos: DVec3::new(x.to_meters(), y.to_meters(), z.to_meters()),
            vel: DVec3::new(
                vx.to_meters_per_second(),
                vy.to_meters_per_second(),
                vz.to_meters_per_second(),
            ),
        }
    }

    pub const fn from_vecs(pos: DVec3, vel: DVec3) -> Self {
        Self { pos, vel }
    }

    pub const fn builder() -> CartesianBuilder {
        CartesianBuilder::new()
    }

    pub fn position(&self) -> DVec3 {
        self.pos
    }

    pub fn velocity(&self) -> DVec3 {
        self.vel
    }

    pub fn x(&self) -> Distance {
        Distance::meters(self.pos.x)
    }

    pub fn y(&self) -> Distance {
        Distance::meters(self.pos.y)
    }

    pub fn z(&self) -> Distance {
        Distance::meters(self.pos.z)
    }

    pub fn vx(&self) -> Velocity {
        Velocity::meters_per_second(self.vel.x)
    }

    pub fn vy(&self) -> Velocity {
        Velocity::meters_per_second(self.vel.y)
    }

    pub fn vz(&self) -> Velocity {
        Velocity::meters_per_second(self.vel.z)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CartesianBuilder {
    pos: Option<DVec3>,
    vel: Option<DVec3>,
}

impl CartesianBuilder {
    pub const fn new() -> Self {
        Self {
            pos: None,
            vel: None,
        }
    }

    pub const fn position(&mut self, x: Distance, y: Distance, z: Distance) -> &mut Self {
        self.pos = Some(DVec3::new(x.to_meters(), y.to_meters(), z.to_meters()));
        self
    }

    pub const fn velocity(&mut self, vx: Velocity, vy: Velocity, vz: Velocity) -> &mut Self {
        self.vel = Some(DVec3::new(
            vx.to_meters_per_second(),
            vy.to_meters_per_second(),
            vz.to_meters_per_second(),
        ));
        self
    }

    pub const fn build(&self) -> Cartesian {
        Cartesian {
            pos: match self.pos {
                Some(pos) => pos,
                None => DVec3::ZERO,
            },
            vel: match self.vel {
                Some(vel) => vel,
                None => DVec3::ZERO,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{AngleUnits, DistanceUnits, VelocityUnits};

    use super::*;

    #[test]
    fn test_azel_api() {
        let azel = AzEl::builder()
            .azimuth(45.0.deg())
            .elevation(45.0.deg())
            .build()
            .unwrap();
        assert_eq!(azel.azimuth(), 45.0.deg());
        assert_eq!(azel.elevation(), 45.0.deg());
    }

    #[rstest]
    #[case(0.0.deg(), 0.0.deg(), Ok(AzEl(0.0.deg(), 0.0.deg())))]
    #[case(-1.0.deg(), 0.0.deg(), Err(AzElError::InvalidAzimuth(-1.0.deg())))]
    #[case(361.0.deg(), 0.0.deg(), Err(AzElError::InvalidAzimuth(361.0.deg())))]
    #[case(0.0.deg(), -1.0.deg(), Err(AzElError::InvalidElevation(-1.0.deg())))]
    #[case(0.0.deg(), 361.0.deg(), Err(AzElError::InvalidElevation(361.0.deg())))]
    fn test_azel(#[case] az: Angle, #[case] el: Angle, #[case] exp: Result<AzEl, AzElError>) {
        let act = AzEl::builder().azimuth(az).elevation(el).build();
        assert_eq!(act, exp)
    }

    #[test]
    fn test_lla_api() {
        let lla = LonLatAlt::builder()
            .longitude(45.0.deg())
            .latitude(45.0.deg())
            .altitude(100.0.m())
            .build()
            .unwrap();
        assert_eq!(lla.lon(), 45.0.deg());
        assert_eq!(lla.lat(), 45.0.deg());
        assert_eq!(lla.alt(), 100.0.m());
    }

    #[rstest]
    #[case(0.0.deg(), 0.0.deg(), 0.0.m(), Ok(LonLatAlt(0.0.deg(), 0.0.deg(), 0.0.m())))]
    #[case(-181.0.deg(), 0.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLongitude(-181.0.deg())))]
    #[case(181.0.deg(), 0.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLongitude(181.0.deg())))]
    #[case(0.0.deg(), -91.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLatitude(-91.0.deg())))]
    #[case(0.0.deg(), 91.0.deg(), 0.0.m(), Err(LonLatAltError::InvalidLatitude(91.0.deg())))]
    #[case(0.0.deg(), 0.0.deg(), f64::INFINITY.m(), Err(LonLatAltError::InvalidAltitude(f64::INFINITY.m())))]
    fn test_lla(
        #[case] lon: Angle,
        #[case] lat: Angle,
        #[case] alt: Distance,
        #[case] exp: Result<LonLatAlt, LonLatAltError>,
    ) {
        let act = LonLatAlt::builder()
            .longitude(lon)
            .latitude(lat)
            .altitude(alt)
            .build();
        assert_eq!(act, exp)
    }

    #[test]
    fn test_cartesian() {
        let c = Cartesian::builder()
            .position(1000.0.km(), 1000.0.km(), 1000.0.km())
            .velocity(1.0.kps(), 1.0.kps(), 1.0.kps())
            .build();
        assert_eq!(c.position(), DVec3::new(1e6, 1e6, 1e6));
        assert_eq!(c.velocity(), DVec3::new(1e3, 1e3, 1e3));
        assert_eq!(c.x(), 1e6.m());
        assert_eq!(c.y(), 1e6.m());
        assert_eq!(c.z(), 1e6.m());
        assert_eq!(c.vx(), 1e3.mps());
        assert_eq!(c.vy(), 1e3.mps());
        assert_eq!(c.vz(), 1e3.mps());
    }
}
