pub use glam::DVec3;

use crate::bodies::{gravitational_parameter, PointMass};
use crate::time::epochs::Epoch;
use crate::two_body::elements::{cartesian_to_keplerian, keplerian_to_cartesian};

pub mod elements;

type Elements = (f64, f64, f64, f64, f64, f64);

pub trait TwoBody {
    type Center;
    fn epoch(&self) -> Epoch;
    fn center(&self) -> Self::Center;
    fn position(&self) -> DVec3;
    fn velocity(&self) -> DVec3;
    fn cartesian(&self) -> (DVec3, DVec3);
    fn keplerian(&self) -> Elements;
    fn semi_major(&self) -> f64;
    fn eccentricity(&self) -> f64;
    fn inclination(&self) -> f64;
    fn ascending_node(&self) -> f64;
    fn periapsis_arg(&self) -> f64;
    fn true_anomaly(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct Cartesian<T: PointMass> {
    epoch: Epoch,
    center: T,
    position: DVec3,
    velocity: DVec3,
}

impl<T: PointMass> Cartesian<T> {
    pub fn new(epoch: Epoch, center: T, position: DVec3, velocity: DVec3) -> Self {
        Self {
            epoch,
            center,
            position,
            velocity,
        }
    }
}

impl<T: PointMass> TwoBody for Cartesian<T> {
    type Center = T;

    fn epoch(&self) -> Epoch {
        self.epoch
    }

    fn center(&self) -> Self::Center {
        self.center
    }

    fn position(&self) -> DVec3 {
        self.position
    }

    fn velocity(&self) -> DVec3 {
        self.velocity
    }

    fn cartesian(&self) -> (DVec3, DVec3) {
        (self.position, self.velocity)
    }

    fn keplerian(&self) -> Elements {
        let mu = gravitational_parameter(self.center);
        cartesian_to_keplerian(mu, self.position, self.velocity)
    }

    fn semi_major(&self) -> f64 {
        self.keplerian().0
    }

    fn eccentricity(&self) -> f64 {
        self.keplerian().1
    }

    fn inclination(&self) -> f64 {
        self.keplerian().2
    }

    fn ascending_node(&self) -> f64 {
        self.keplerian().3
    }

    fn periapsis_arg(&self) -> f64 {
        self.keplerian().4
    }

    fn true_anomaly(&self) -> f64 {
        self.keplerian().5
    }
}

impl<T: PointMass> From<Keplerian<T>> for Cartesian<T> {
    fn from(value: Keplerian<T>) -> Self {
        let epoch = value.epoch;
        let center = value.center;
        let (pos, vel) = value.cartesian();
        Cartesian::new(epoch, center, pos, vel)
    }
}

#[derive(Debug, Clone)]
pub struct Keplerian<T: PointMass> {
    epoch: Epoch,
    center: T,
    semi_major: f64,
    eccentricity: f64,
    inclination: f64,
    ascending_node: f64,
    periapsis_arg: f64,
    true_anomaly: f64,
}

impl<T: PointMass> Keplerian<T> {
    pub fn new(epoch: Epoch, center: T, elements: Elements) -> Self {
        let (semi_major, eccentricity, inclination, ascending_node, periapsis_arg, true_anomaly) =
            elements;
        Self {
            epoch,
            center,
            semi_major,
            eccentricity,
            inclination,
            ascending_node,
            periapsis_arg,
            true_anomaly,
        }
    }
}

impl<T: PointMass> TwoBody for Keplerian<T> {
    type Center = T;

    fn epoch(&self) -> Epoch {
        self.epoch
    }

    fn center(&self) -> Self::Center {
        self.center
    }

    fn position(&self) -> DVec3 {
        self.cartesian().0
    }

    fn velocity(&self) -> DVec3 {
        self.cartesian().1
    }

    fn cartesian(&self) -> (DVec3, DVec3) {
        let mu = gravitational_parameter(self.center);
        keplerian_to_cartesian(
            mu,
            self.semi_major,
            self.eccentricity,
            self.inclination,
            self.ascending_node,
            self.periapsis_arg,
            self.true_anomaly,
        )
    }

    fn keplerian(&self) -> Elements {
        (
            self.semi_major,
            self.eccentricity,
            self.inclination,
            self.ascending_node,
            self.periapsis_arg,
            self.true_anomaly,
        )
    }

    fn semi_major(&self) -> f64 {
        self.semi_major
    }

    fn eccentricity(&self) -> f64 {
        self.eccentricity
    }

    fn inclination(&self) -> f64 {
        self.inclination
    }

    fn ascending_node(&self) -> f64 {
        self.ascending_node
    }

    fn periapsis_arg(&self) -> f64 {
        self.periapsis_arg
    }

    fn true_anomaly(&self) -> f64 {
        self.true_anomaly
    }
}

impl<T: PointMass> From<Cartesian<T>> for Keplerian<T> {
    fn from(value: Cartesian<T>) -> Self {
        let epoch = value.epoch;
        let center = value.center;
        let elements = value.keplerian();
        Self::new(epoch, center, elements)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use std::ops::Mul;

    use crate::bodies::planets::Earth;
    use crate::time::dates::{Date, Time};
    use crate::time::epochs::TimeScale;

    use super::*;

    #[test]
    fn test_two_body() {
        let date = Date::new(2023, 3, 25).expect("Date should be valid");
        let time = Time::new(21, 8, 0).expect("Time should be valid");
        let epoch = Epoch::from_date_and_time(TimeScale::TDB, date, time);
        let semi_major = 24464560.0e-3;
        let eccentricity = 0.7311;
        let inclination = 0.122138;
        let ascending_node = 1.00681;
        let periapsis_arg = 3.10686;
        let true_anomaly = 0.44369564302687126;
        let pos = DVec3::new(
            -0.107622532467967e7,
            -0.676589636432773e7,
            -0.332308783350379e6,
        )
        .mul(1e-3);
        let vel = DVec3::new(
            0.935685775154103e4,
            -0.331234775037644e4,
            -0.118801577532701e4,
        )
        .mul(1e-3);

        let cartesian = Cartesian::new(epoch, Earth, pos, vel);

        assert_eq!(cartesian.epoch(), epoch);
        assert_eq!(cartesian.center(), Earth);
        assert_eq!(cartesian.position(), pos);
        assert_eq!(cartesian.velocity(), vel);
        assert_float_eq!(cartesian.semi_major(), semi_major, rel <= 1e-6);
        assert_float_eq!(cartesian.eccentricity(), eccentricity, rel <= 1e-6);
        assert_float_eq!(cartesian.inclination(), inclination, rel <= 1e-6);
        assert_float_eq!(cartesian.ascending_node(), ascending_node, rel <= 1e-6);
        assert_float_eq!(cartesian.periapsis_arg(), periapsis_arg, rel <= 1e-6);
        assert_float_eq!(cartesian.true_anomaly(), true_anomaly, rel <= 1e-6);

        let keplerian = Keplerian::new(
            epoch,
            Earth,
            (
                semi_major,
                eccentricity,
                inclination,
                ascending_node,
                periapsis_arg,
                true_anomaly,
            ),
        );

        assert_eq!(keplerian.epoch(), epoch);
        assert_eq!(keplerian.center(), Earth);
        assert_float_eq!(keplerian.position().x, pos.x, rel <= 1e-8);
        assert_float_eq!(keplerian.position().y, pos.y, rel <= 1e-8);
        assert_float_eq!(keplerian.position().z, pos.z, rel <= 1e-8);
        assert_float_eq!(keplerian.velocity().x, vel.x, rel <= 1e-6);
        assert_float_eq!(keplerian.velocity().y, vel.y, rel <= 1e-6);
        assert_float_eq!(keplerian.velocity().z, vel.z, rel <= 1e-6);
        assert_float_eq!(keplerian.semi_major(), semi_major, rel <= 1e-6);
        assert_float_eq!(keplerian.eccentricity(), eccentricity, rel <= 1e-6);
        assert_float_eq!(keplerian.inclination(), inclination, rel <= 1e-6);
        assert_float_eq!(keplerian.ascending_node(), ascending_node, rel <= 1e-6);
        assert_float_eq!(keplerian.periapsis_arg(), periapsis_arg, rel <= 1e-6);
        assert_float_eq!(keplerian.true_anomaly(), true_anomaly, rel <= 1e-6);

        let cartesian1 = Cartesian::from(keplerian.clone());
        let keplerian1 = Keplerian::from(cartesian.clone());

        assert_float_eq!(cartesian.position.x, cartesian1.position.x, rel <= 1e-8);
        assert_float_eq!(cartesian.position.y, cartesian1.position.y, rel <= 1e-8);
        assert_float_eq!(cartesian.position.z, cartesian1.position.z, rel <= 1e-8);
        assert_float_eq!(cartesian.velocity.x, cartesian1.velocity.x, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity.y, cartesian1.velocity.y, rel <= 1e-6);
        assert_float_eq!(cartesian.velocity.z, cartesian1.velocity.z, rel <= 1e-6);

        assert_float_eq!(keplerian.semi_major, keplerian1.semi_major, rel <= 1e-2);
        assert_float_eq!(keplerian.eccentricity, keplerian1.eccentricity, abs <= 1e-6);
        assert_float_eq!(keplerian.inclination, keplerian1.inclination, rel <= 1e-6);
        assert_float_eq!(
            keplerian.ascending_node,
            keplerian1.ascending_node,
            rel <= 1e-6
        );
        assert_float_eq!(
            keplerian.periapsis_arg,
            keplerian1.periapsis_arg,
            rel <= 1e-6
        );
        assert_float_eq!(keplerian.true_anomaly, keplerian1.true_anomaly, rel <= 1e-6);
    }
}
