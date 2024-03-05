/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Debug, Display, Formatter};

use glam::{DMat3, DVec3};

use lox_bodies::RotationalElements;
use lox_time::continuous::{TDB, Time};

use crate::frames::{RotationFrom, Icrf, ReferenceFrame, RotatingFrame, Rotation};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BodyFixed<T: RotationalElements>(pub T);

impl<T: RotationalElements> Display for BodyFixed<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IAU_{}", &self.0.name().to_uppercase())
    }
}

impl<T: RotationalElements> ReferenceFrame for BodyFixed<T> {}
impl<T: RotationalElements> RotatingFrame for BodyFixed<T> {}

impl<T: RotationalElements> RotationFrom<Icrf, TDB> for BodyFixed<T> {
    fn rotation_from(&self, _: Icrf, t: Time<TDB>) -> Rotation {
        let (right_ascension, declination, prime_meridian) = T::rotational_elements(t);
        let (right_ascension_rate, declination_rate, prime_meridian_rate) =
            T::rotational_element_rates(t);
        let m1 = DMat3::from_rotation_z(-right_ascension);
        let m2 = DMat3::from_rotation_x(-declination);
        let m3 = DMat3::from_rotation_z(-prime_meridian);
        let m = m3 * m2 * m1;
        let v = DVec3::new(right_ascension_rate, declination_rate, prime_meridian_rate);
        Rotation::new(m).with_velocity(v)
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use glam::DVec3;

    use lox_bodies::Jupiter;
    use lox_time::Subsecond;
    use crate::base::BaseCartesian;

    use crate::frames::{Icrf, RotationInto};

    use super::*;

    #[test]
    fn test_bodyfixed() {
        let iau_jupiter = BodyFixed(Jupiter);
        assert_eq!(format!("{}", iau_jupiter), "IAU_JUPITER");
        assert!(!iau_jupiter.is_inertial());
        assert!(iau_jupiter.is_rotating());

        let t = Time::new(TDB, 0, Subsecond::default());
        let from_icrf = iau_jupiter.rotation_from(Icrf, t);
        let to_icrf = iau_jupiter.rotation_into(Icrf, t);

        let r0 = DVec3::new(6068279.27e-3, -1692843.94e-3, -2516619.18e-3);
        let v0 = DVec3::new(-660.415582e-3, 5495.938726e-3, -5303.093233e-3);
        let r1 = DVec3::new(3922.220687351738, 5289.381014412637, -1631.4837924820245);
        let v1 = DVec3::new(-1.852284168309543, -0.8227941105651749, -7.14175174489828);

        let rv0_exp = BaseCartesian::new(r0, v0);
        let rv1_exp = BaseCartesian::new(r1, v1);

        let rv1_act = from_icrf.apply(rv0_exp);
        let rv0_act = to_icrf.apply(rv1_exp);

        assert_float_eq!(rv1_act.position().x, rv1_exp.position().x, rel <= 1e-8);
        assert_float_eq!(rv1_act.position().y, rv1_exp.position().y, rel <= 1e-8);
        assert_float_eq!(rv1_act.position().z, rv1_exp.position().z, rel <= 1e-8);
        assert_float_eq!(rv1_act.velocity().x, rv1_exp.velocity().x, rel <= 1e-8);
        assert_float_eq!(rv1_act.velocity().y, rv1_exp.velocity().y, rel <= 1e-8);
        assert_float_eq!(rv1_act.velocity().z, rv1_exp.velocity().z, rel <= 1e-8);

        assert_float_eq!(rv0_act.position().x, rv0_exp.position().x, rel <= 1e-8);
        assert_float_eq!(rv0_act.position().y, rv0_exp.position().y, rel <= 1e-8);
        assert_float_eq!(rv0_act.position().z, rv0_exp.position().z, rel <= 1e-8);
        assert_float_eq!(rv0_act.velocity().x, rv0_exp.velocity().x, rel <= 1e-8);
        assert_float_eq!(rv0_act.velocity().y, rv0_exp.velocity().y, rel <= 1e-8);
        assert_float_eq!(rv0_act.velocity().z, rv0_exp.velocity().z, rel <= 1e-8);
    }
}
