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

use crate::frames::{Epoch, FromFrame, Icrf, ReferenceFrame, RotatingFrame, Rotation};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BodyFixed<T: RotationalElements>(pub T);

impl<T: RotationalElements> Display for BodyFixed<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IAU_{}", &self.0.name().to_uppercase())
    }
}

impl<T: RotationalElements> ReferenceFrame for BodyFixed<T> {}
impl<T: RotationalElements> RotatingFrame for BodyFixed<T> {}

impl<T: RotationalElements> FromFrame<Icrf> for BodyFixed<T> {
    fn rotation_from(&self, _: Icrf, t: Epoch) -> Rotation {
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

    use crate::frames::{Icrf, IntoFrame};

    use super::*;

    #[test]
    fn test_bodyfixed() {
        let iau_jupiter = BodyFixed(Jupiter);
        assert_eq!(format!("{}", iau_jupiter), "IAU_JUPITER");
        assert!(!iau_jupiter.is_inertial());
        assert!(iau_jupiter.is_rotating());

        let from_icrf = iau_jupiter.rotation_from(Icrf, 0.0);
        let to_icrf = iau_jupiter.rotation_into(Icrf, 0.0);

        let r0 = DVec3::new(6068279.27e-3, -1692843.94e-3, -2516619.18e-3);
        let v0 = DVec3::new(-660.415582e-3, 5495.938726e-3, -5303.093233e-3);
        let r1 = DVec3::new(3922.220687351738, 5289.381014412637, -1631.4837924820245);
        let v1 = DVec3::new(-1.852284168309543, -0.8227941105651749, -7.14175174489828);

        let rv0_exp = (r0, v0);
        let rv1_exp = (r1, v1);

        let rv1_act = from_icrf.apply(rv0_exp);
        let rv0_act = to_icrf.apply(rv1_exp);

        assert_float_eq!(rv1_act.0.x, rv1_exp.0.x, rel <= 1e-8);
        assert_float_eq!(rv1_act.0.y, rv1_exp.0.y, rel <= 1e-8);
        assert_float_eq!(rv1_act.0.z, rv1_exp.0.z, rel <= 1e-8);
        assert_float_eq!(rv1_act.1.x, rv1_exp.1.x, rel <= 1e-8);
        assert_float_eq!(rv1_act.1.y, rv1_exp.1.y, rel <= 1e-8);
        assert_float_eq!(rv1_act.1.z, rv1_exp.1.z, rel <= 1e-8);

        assert_float_eq!(rv0_act.0.x, rv0_exp.0.x, rel <= 1e-8);
        assert_float_eq!(rv0_act.0.y, rv0_exp.0.y, rel <= 1e-8);
        assert_float_eq!(rv0_act.0.z, rv0_exp.0.z, rel <= 1e-8);
        assert_float_eq!(rv0_act.1.x, rv0_exp.1.x, rel <= 1e-8);
        assert_float_eq!(rv0_act.1.y, rv0_exp.1.y, rel <= 1e-8);
        assert_float_eq!(rv0_act.1.z, rv0_exp.1.z, rel <= 1e-8);

        let rv1_act = iau_jupiter.transform_from(Icrf, 0.0, rv0_exp);
        let rv0_act = iau_jupiter.transform_into(Icrf, 0.0, rv1_exp);

        assert_float_eq!(rv1_act.0.x, rv1_exp.0.x, rel <= 1e-8);
        assert_float_eq!(rv1_act.0.y, rv1_exp.0.y, rel <= 1e-8);
        assert_float_eq!(rv1_act.0.z, rv1_exp.0.z, rel <= 1e-8);
        assert_float_eq!(rv1_act.1.x, rv1_exp.1.x, rel <= 1e-8);
        assert_float_eq!(rv1_act.1.y, rv1_exp.1.y, rel <= 1e-8);
        assert_float_eq!(rv1_act.1.z, rv1_exp.1.z, rel <= 1e-8);

        assert_float_eq!(rv0_act.0.x, rv0_exp.0.x, rel <= 1e-8);
        assert_float_eq!(rv0_act.0.y, rv0_exp.0.y, rel <= 1e-8);
        assert_float_eq!(rv0_act.0.z, rv0_exp.0.z, rel <= 1e-8);
        assert_float_eq!(rv0_act.1.x, rv0_exp.1.x, rel <= 1e-8);
        assert_float_eq!(rv0_act.1.y, rv0_exp.1.y, rel <= 1e-8);
        assert_float_eq!(rv0_act.1.z, rv0_exp.1.z, rel <= 1e-8);
    }
}
