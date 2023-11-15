/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Auto-generated by `lox_gen`. Do not edit!

use crate::bodies::{
    EarthBarycenter, JupiterBarycenter, MarsBarycenter, MercuryBarycenter, NeptuneBarycenter,
    PlutoBarycenter, PointMass, SaturnBarycenter, UranusBarycenter, VenusBarycenter,
};
impl PointMass for MercuryBarycenter {
    fn gravitational_parameter() -> f64 {
        22031.868551400003f64
    }
}
impl PointMass for VenusBarycenter {
    fn gravitational_parameter() -> f64 {
        324858.592f64
    }
}
impl PointMass for EarthBarycenter {
    fn gravitational_parameter() -> f64 {
        403503.2356254802f64
    }
}
impl PointMass for MarsBarycenter {
    fn gravitational_parameter() -> f64 {
        42828.3758157561f64
    }
}
impl PointMass for JupiterBarycenter {
    fn gravitational_parameter() -> f64 {
        126712764.09999998f64
    }
}
impl PointMass for SaturnBarycenter {
    fn gravitational_parameter() -> f64 {
        37940584.8418f64
    }
}
impl PointMass for UranusBarycenter {
    fn gravitational_parameter() -> f64 {
        5794556.3999999985f64
    }
}
impl PointMass for NeptuneBarycenter {
    fn gravitational_parameter() -> f64 {
        6836527.100580399f64
    }
}
impl PointMass for PlutoBarycenter {
    fn gravitational_parameter() -> f64 {
        975.5f64
    }
}
#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use crate::bodies::*;
    #[test]
    fn test_point_mass_1() {
        assert_eq!(
            MercuryBarycenter::gravitational_parameter(),
            22031.868551400003f64
        );
    }
    #[test]
    fn test_point_mass_2() {
        assert_eq!(VenusBarycenter::gravitational_parameter(), 324858.592f64);
    }
    #[test]
    fn test_point_mass_3() {
        assert_eq!(
            EarthBarycenter::gravitational_parameter(),
            403503.2356254802f64
        );
    }
    #[test]
    fn test_point_mass_4() {
        assert_eq!(
            MarsBarycenter::gravitational_parameter(),
            42828.3758157561f64
        );
    }
    #[test]
    fn test_point_mass_5() {
        assert_eq!(
            JupiterBarycenter::gravitational_parameter(),
            126712764.09999998f64
        );
    }
    #[test]
    fn test_point_mass_6() {
        assert_eq!(
            SaturnBarycenter::gravitational_parameter(),
            37940584.8418f64
        );
    }
    #[test]
    fn test_point_mass_7() {
        assert_eq!(
            UranusBarycenter::gravitational_parameter(),
            5794556.3999999985f64
        );
    }
    #[test]
    fn test_point_mass_8() {
        assert_eq!(
            NeptuneBarycenter::gravitational_parameter(),
            6836527.100580399f64
        );
    }
    #[test]
    fn test_point_mass_9() {
        assert_eq!(PlutoBarycenter::gravitational_parameter(), 975.5f64);
    }
}
