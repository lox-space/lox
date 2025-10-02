use std::f64::consts::{FRAC_PI_2, PI, TAU};

use thiserror::Error;

use crate::{Angle, Distance};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct AzEl(Angle, Angle);

impl AzEl {
    pub fn build() -> AzElBuilder {
        AzElBuilder::default()
    }

    pub fn az(&self) -> Angle {
        self.0
    }

    pub fn el(&self) -> Angle {
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

    pub fn with_azimuth(&mut self, azimuth: Angle) -> &mut Self {
        self.azimuth = match azimuth.0 {
            lon if lon < 0.0 => Err(AzElError::InvalidAzimuth(azimuth)),
            lon if lon > TAU => Err(AzElError::InvalidAzimuth(azimuth)),
            _ => Ok(azimuth),
        };
        self
    }

    pub fn with_elevation(&mut self, elevation: Angle) -> &mut Self {
        self.elevation = match elevation.0 {
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
    pub fn build() -> LonLatAltBuilder {
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

    pub fn with_longitude(&mut self, longitude: Angle) -> &mut Self {
        self.longitude = match longitude.0 {
            lon if lon < -PI => Err(LonLatAltError::InvalidLongitude(longitude)),
            lon if lon > PI => Err(LonLatAltError::InvalidLongitude(longitude)),
            _ => Ok(longitude),
        };
        self
    }

    pub fn with_latitude(&mut self, latitude: Angle) -> &mut Self {
        self.latitude = match latitude.0 {
            lat if lat < -FRAC_PI_2 => Err(LonLatAltError::InvalidLatitude(latitude)),
            lat if lat > FRAC_PI_2 => Err(LonLatAltError::InvalidLatitude(latitude)),
            _ => Ok(latitude),
        };
        self
    }

    pub fn with_altitude(&mut self, altitude: Distance) -> &mut Self {
        self.altitude = if !altitude.0.is_finite() {
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{AngleUnits, DistanceUnits};

    use super::*;

    #[test]
    fn test_azel_api() {
        let azel = AzEl::build()
            .with_azimuth(45.0.deg())
            .with_elevation(45.0.deg())
            .build()
            .unwrap();
        assert_eq!(azel.az(), 45.0.deg());
        assert_eq!(azel.el(), 45.0.deg());
    }

    #[rstest]
    #[case(0.0.deg(), 0.0.deg(), Ok(AzEl(0.0.deg(), 0.0.deg())))]
    #[case(-1.0.deg(), 0.0.deg(), Err(AzElError::InvalidAzimuth(-1.0.deg())))]
    #[case(361.0.deg(), 0.0.deg(), Err(AzElError::InvalidAzimuth(361.0.deg())))]
    #[case(0.0.deg(), -1.0.deg(), Err(AzElError::InvalidElevation(-1.0.deg())))]
    #[case(0.0.deg(), 361.0.deg(), Err(AzElError::InvalidElevation(361.0.deg())))]
    fn test_azel(#[case] az: Angle, #[case] el: Angle, #[case] exp: Result<AzEl, AzElError>) {
        let act = AzEl::build().with_azimuth(az).with_elevation(el).build();
        assert_eq!(act, exp)
    }

    #[test]
    fn test_lla_api() {
        let lla = LonLatAlt::build()
            .with_longitude(45.0.deg())
            .with_latitude(45.0.deg())
            .with_altitude(100.0.m())
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
        let act = LonLatAlt::build()
            .with_longitude(lon)
            .with_latitude(lat)
            .with_altitude(alt)
            .build();
        assert_eq!(act, exp)
    }
}
