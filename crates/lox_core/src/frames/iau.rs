/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

use glam::{DMat3, DVec3};

use crate::bodies::RotationalElements;
use crate::frames::{FromIcrf, Rotation};

struct BodyFixed<T: RotationalElements>(T);

impl<T: RotationalElements> FromIcrf for BodyFixed<T> {
    fn rotation_from_icrf(&self, t: f64) -> Rotation {
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

    use crate::bodies::planets::Jupiter;
    use crate::frames::iau::BodyFixed;
    use crate::frames::{FromIcrf, ToIcrf};

    #[test]
    fn test_bodyfixed() {
        let iau_jupiter = BodyFixed(Jupiter);
        let from_icrf = iau_jupiter.rotation_from_icrf(0.0);
        let to_icrf = iau_jupiter.rotation_to_icrf(0.0);

        let r0 = DVec3::new(6068279.27e-3, -1692843.94e-3, -2516619.18e-3);
        let v0 = DVec3::new(-660.415582e-3, 5495.938726e-3, -5303.093233e-3);
        let r1 = DVec3::new(3922.220687351738, 5289.381014412637, -1631.4837924820245);
        let v1 = DVec3::new(-1.852284168309543, -0.8227941105651749, -7.14175174489828);

        let rv0_exp = (r0, v0);
        let rv1_exp = (r1, v1);

        let rv1_act = from_icrf.rotate(rv0_exp);
        let rv0_act = to_icrf.rotate(rv1_exp);

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
