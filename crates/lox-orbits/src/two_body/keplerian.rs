use float_eq::float_eq;
use glam::{DMat3, DVec3};
use lox_utils::types::units::{Dimensionless, Kilometers, Radians, Seconds};
use std::f64::consts::TAU;

use lox_bodies::PointMass;
use lox_time::{base_time::BaseTime, time_scales::TimeScale, Time, TimeSystem};

use crate::{frames::InertialFrame, CoordinateSystem};

use super::cartesian::{BaseCartesian, Cartesian, CartesianState};

pub struct KeplerianElements {
    pub semi_major_axis: Kilometers,
    pub eccentricity: Dimensionless,
    pub inclination: Radians,
    pub longitude_of_ascending_node: Radians,
    pub argument_of_periapsis: Radians,
    pub true_anomaly: Radians,
}

pub fn is_equatorial(inclination: f64) -> bool {
    float_eq!(inclination.abs(), 0.0, abs <= 1e-8)
}

pub fn is_circular(eccentricity: f64) -> bool {
    float_eq!(eccentricity, 0.0, abs <= 1e-8)
}

pub trait KeplerianState {
    type Time;

    fn time(&self) -> Self::Time;
    fn semi_major_axis(&self) -> Kilometers;
    fn eccentricity(&self) -> Dimensionless;
    fn inclination(&self) -> Radians;
    fn longitude_of_ascending_node(&self) -> Radians;
    fn argument_of_periapsis(&self) -> Radians;
    fn true_anomaly(&self) -> Radians;
    fn gravitational_parameter(&self) -> f64;

    fn orbital_period(&self) -> Seconds {
        let mu = self.gravitational_parameter();
        let a = self.semi_major_axis();
        TAU * (a.powi(3) / mu).sqrt()
    }

    fn is_equatorial(&self) -> bool {
        is_equatorial(self.inclination())
    }

    fn is_circular(&self) -> bool {
        is_circular(self.eccentricity())
    }

    fn semiparameter(&self) -> f64 {
        if self.is_circular() {
            self.semi_major_axis()
        } else {
            self.semi_major_axis() * (1.0 - self.eccentricity().powi(2))
        }
    }

    fn perifocal(&self) -> (DVec3, DVec3) {
        let mu = self.gravitational_parameter();
        let semiparameter = self.semiparameter();
        let (sin_nu, cos_nu) = self.true_anomaly().sin_cos();
        let sqrt_mu_p = (mu / semiparameter).sqrt();

        let pos = DVec3::new(cos_nu, sin_nu, 0.0)
            * (semiparameter / (1.0 + self.eccentricity() * cos_nu));
        let vel = DVec3::new(-sin_nu, self.eccentricity() + cos_nu, 0.0) * sqrt_mu_p;

        (pos, vel)
    }

    fn cartesian(&self) -> (DVec3, DVec3) {
        let (pos, vel) = self.perifocal();
        let rot = DMat3::from_rotation_z(self.longitude_of_ascending_node())
            * DMat3::from_rotation_x(self.inclination())
            * DMat3::from_rotation_z(self.argument_of_periapsis());

        (rot * pos, rot * vel)
    }
}

pub struct BaseKeplerian {
    time: BaseTime,
    grav_param: f64,
    elements: KeplerianElements,
}

impl KeplerianState for BaseKeplerian {
    type Time = BaseTime;

    fn time(&self) -> BaseTime {
        self.time
    }

    fn semi_major_axis(&self) -> Kilometers {
        self.elements.semi_major_axis
    }

    fn eccentricity(&self) -> Dimensionless {
        self.elements.eccentricity
    }

    fn inclination(&self) -> Radians {
        self.elements.inclination
    }

    fn longitude_of_ascending_node(&self) -> Radians {
        self.elements.longitude_of_ascending_node
    }

    fn argument_of_periapsis(&self) -> Radians {
        self.elements.argument_of_periapsis
    }

    fn true_anomaly(&self) -> Radians {
        self.elements.true_anomaly
    }

    fn gravitational_parameter(&self) -> f64 {
        self.grav_param
    }
}

impl From<BaseCartesian> for BaseKeplerian {
    fn from(cartesian: BaseCartesian) -> Self {
        let elements = cartesian.keplerian();
        Self {
            time: cartesian.time(),
            grav_param: cartesian.gravitational_parameter(),
            elements,
        }
    }
}

pub struct Keplerian<T: TimeScale + Copy, O: PointMass + Copy, F: InertialFrame + Copy> {
    origin: O,
    frame: F,
    time: Time<T>,
    elements: KeplerianElements,
}

impl<T, O, F> KeplerianState for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    type Time = Time<T>;

    fn time(&self) -> Self::Time {
        self.time
    }

    fn semi_major_axis(&self) -> Kilometers {
        self.elements.semi_major_axis
    }

    fn eccentricity(&self) -> Dimensionless {
        self.elements.eccentricity
    }

    fn inclination(&self) -> Radians {
        self.elements.inclination
    }

    fn longitude_of_ascending_node(&self) -> Radians {
        self.elements.longitude_of_ascending_node
    }

    fn argument_of_periapsis(&self) -> Radians {
        self.elements.argument_of_periapsis
    }

    fn true_anomaly(&self) -> Radians {
        self.elements.true_anomaly
    }

    fn gravitational_parameter(&self) -> f64 {
        self.origin.gravitational_parameter()
    }
}

impl<T, O, F> TimeSystem for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    type Scale = T;

    fn scale(&self) -> Self::Scale {
        self.time.scale()
    }
}

impl<T, O, F> CoordinateSystem for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    type Origin = O;

    type Frame = F;

    fn origin(&self) -> Self::Origin {
        self.origin
    }

    fn reference_frame(&self) -> Self::Frame {
        self.frame
    }
}

impl<T, O, F> From<Cartesian<T, O, F>> for Keplerian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn from(cartesian: Cartesian<T, O, F>) -> Self {
        let elements = cartesian.keplerian();
        Self {
            origin: cartesian.origin(),
            frame: cartesian.reference_frame(),
            time: cartesian.time(),
            elements,
        }
    }
}
