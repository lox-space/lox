use glam::DVec3;
use lox_bodies::PointMass;
use lox_time::{base_time::BaseTime, time_scales::TimeScale, Time, TimeSystem};
use lox_utils::{
    glam::Azimuth,
    math::{mod_two_pi, normalize_two_pi},
};

use crate::{
    anomalies::{eccentric_to_true, hyperbolic_to_true},
    frames::{InertialFrame, ReferenceFrame},
    CoordinateSystem,
};

use super::keplerian::{
    is_circular, is_equatorial, BaseKeplerian, Keplerian, KeplerianElements, KeplerianState,
};

pub trait CartesianState {
    type Time;

    fn time(&self) -> Self::Time;
    fn position(&self) -> DVec3;
    fn velocity(&self) -> DVec3;
    fn gravitational_parameter(&self) -> f64;

    fn eccentricity_vector(&self) -> DVec3 {
        let r = self.position();
        let v = self.velocity();
        let mu = self.gravitational_parameter();

        let v2 = v.dot(v);
        let r2 = r.dot(r);
        let rv = r.dot(v);

        (v2 / mu - 1.0 / r2) * r - (rv / mu) * v
    }

    fn keplerian(&self) -> KeplerianElements {
        let mu = self.gravitational_parameter();
        let r = self.position();
        let v = self.velocity();
        let rm = r.length();
        let vm = v.length();
        let h = r.cross(v);
        let hm = h.length();
        let node = DVec3::Z.cross(h);
        let e = self.eccentricity_vector();
        let eccentricity = e.length();
        let inclination = h.angle_between(DVec3::Z);

        let equatorial = is_equatorial(inclination);
        let circular = is_circular(eccentricity);

        let semi_major_axis = if circular {
            hm.powi(2) / mu
        } else {
            -mu / (2.0 * (vm.powi(2) / 2.0 - mu / rm))
        };

        let ascending_node;
        let periapsis_arg;
        let true_anomaly;
        if equatorial && !circular {
            ascending_node = 0.0;
            periapsis_arg = e.azimuth();
            true_anomaly = (h.dot(e.cross(r)) / hm).atan2(r.dot(e));
        } else if !equatorial && circular {
            ascending_node = node.azimuth();
            periapsis_arg = 0.0;
            true_anomaly = (r.dot(h.cross(node)) / hm).atan2(r.dot(node));
        } else if equatorial && circular {
            ascending_node = 0.0;
            periapsis_arg = 0.0;
            true_anomaly = r.azimuth();
        } else {
            if semi_major_axis > 0.0 {
                let e_se = r.dot(v) / (mu * semi_major_axis).sqrt();
                let e_ce = (rm * vm.powi(2)) / mu - 1.0;
                true_anomaly = eccentric_to_true(e_se.atan2(e_ce), eccentricity);
            } else {
                let e_sh = r.dot(v) / (-mu * semi_major_axis).sqrt();
                let e_ch = (rm * vm.powi(2)) / mu - 1.0;
                true_anomaly =
                    hyperbolic_to_true(((e_ch + e_sh) / (e_ch - e_sh)).ln() / 2.0, eccentricity);
            }
            ascending_node = node.azimuth();
            let px = r.dot(node);
            let py = r.dot(h.cross(node)) / hm;
            periapsis_arg = py.atan2(px) - true_anomaly;
        }

        KeplerianElements {
            semi_major_axis,
            eccentricity,
            inclination,
            longitude_of_ascending_node: mod_two_pi(ascending_node),
            argument_of_periapsis: mod_two_pi(periapsis_arg),
            true_anomaly: normalize_two_pi(true_anomaly, 0.0),
        }
    }
}

pub struct BaseCartesian {
    time: BaseTime,
    position: DVec3,
    velocity: DVec3,
    grav_param: f64,
}

impl CartesianState for BaseCartesian {
    type Time = BaseTime;

    fn time(&self) -> BaseTime {
        self.time
    }

    fn position(&self) -> DVec3 {
        self.position
    }

    fn velocity(&self) -> DVec3 {
        self.velocity
    }

    fn gravitational_parameter(&self) -> f64 {
        self.grav_param
    }
}

impl From<BaseKeplerian> for BaseCartesian {
    fn from(keplerian: BaseKeplerian) -> Self {
        let (position, velocity) = keplerian.cartesian();
        BaseCartesian {
            time: keplerian.time(),
            position,
            velocity,
            grav_param: keplerian.gravitational_parameter(),
        }
    }
}

pub struct Cartesian<T: TimeScale + Copy, O: PointMass + Copy, F: ReferenceFrame + Copy> {
    origin: O,
    frame: F,
    time: Time<T>,
    position: DVec3,
    velocity: DVec3,
}

impl<T, O, F> CartesianState for Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Time = Time<T>;

    fn time(&self) -> Self::Time {
        self.time
    }

    fn position(&self) -> DVec3 {
        self.position
    }

    fn velocity(&self) -> DVec3 {
        self.velocity
    }

    fn gravitational_parameter(&self) -> f64 {
        self.origin.gravitational_parameter()
    }
}

impl<T, O, F> TimeSystem for Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
{
    type Scale = T;

    fn scale(&self) -> Self::Scale {
        self.time.scale()
    }
}

impl<T, O, F> CoordinateSystem for Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: ReferenceFrame + Copy,
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

impl<T, O, F> From<Keplerian<T, O, F>> for Cartesian<T, O, F>
where
    T: TimeScale + Copy,
    O: PointMass + Copy,
    F: InertialFrame + Copy,
{
    fn from(keplerian: Keplerian<T, O, F>) -> Self {
        let (position, velocity) = keplerian.cartesian();
        Cartesian {
            origin: keplerian.origin(),
            frame: keplerian.reference_frame(),
            time: keplerian.time(),
            position,
            velocity,
        }
    }
}
