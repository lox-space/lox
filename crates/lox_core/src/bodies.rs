/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::f64::consts::PI;

use crate::time::constants::f64::{SECONDS_PER_DAY, SECONDS_PER_JULIAN_CENTURY};

pub mod barycenters;
pub mod minor;
pub mod planets;
pub mod satellites;
pub mod sun;

/// NaifId is implemented for all bodies.
pub trait NaifId: Copy {
    const ID: i32;
}

/// Expands to derivations of the fundamental traits every body must implement.
macro_rules! body {
    ($i:ident, $naif_id:literal) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl NaifId for $i {
            const ID: i32 = $naif_id;
        }
    };
}

// The Sun.
body! { Sun, 10 }

// Planets.
body! { Mercury, 199 }
body! { Venus, 299 }
body! { Earth, 399 }
body! { Mars, 499 }
body! { Jupiter, 599 }
body! { Saturn, 699 }
body! { Uranus, 799 }
body! { Neptune, 899 }
body! { Pluto, 999 }

// Barycenters.
body! { SolarSystemBarycenter, 0 }
body! { MercuryBarycenter, 1 }
body! { VenusBarycenter, 2 }
body! { EarthBarycenter, 3 }
body! { MarsBarycenter, 4 }
body! { JupiterBarycenter, 5 }
body! { SaturnBarycenter, 6 }
body! { UranusBarycenter, 7 }
body! { NeptuneBarycenter, 8 }
body! { PlutoBarycenter, 9 }

// Satellites.
body! { Moon, 301 }
body! { Phobos, 401 }
body! { Deimos, 402 }
body! { Io, 501 }
body! { Europa, 502 }
body! { Ganymede, 503 }
body! { Callisto, 504 }
body! { Amalthea, 505 }
body! { Himalia, 506 }
body! { Elara, 507 }
body! { Pasiphae, 508 }
body! { Sinope, 509 }
body! { Lysithea, 510 }
body! { Carme, 511 }
body! { Ananke, 512 }
body! { Leda, 513 }
body! { Thebe, 514 }
body! { Adrastea, 515 }
body! { Metis, 516 }
body! { Callirrhoe, 517 }
body! { Themisto, 518 }
body! { Magaclite, 519 }
body! { Taygete, 520 }
body! { Chaldene, 521 }
body! { Harpalyke, 522 }
body! { Kalyke, 523 }
body! { Iocaste, 524 }
body! { Erinome, 525 }
body! { Isonoe, 526 }
body! { Praxidike, 527 }
body! { Autonoe, 528 }
body! { Thyone, 529 }
body! { Hermippe, 530 }
body! { Aitne, 531 }
body! { Eurydome, 532 }
body! { Euanthe, 533 }
body! { Euporie, 534 }
body! { Orthosie, 535 }
body! { Sponde, 536 }
body! { Kale, 537 }
body! { Pasithee, 538 }
body! { Hegemone, 539 }
body! { Mneme, 540 }
body! { Aoede, 541 }
body! { Thelxinoe, 542 }
body! { Arche, 543 }
body! { Kallichore, 544 }
body! { Helike, 545 }
body! { Carpo, 546 }
body! { Eukelade, 547 }
body! { Cyllene, 548 }
body! { Kore, 549 }
body! { Herse, 550 }
body! { Dia, 553 }
body! { Mimas, 601 }
body! { Enceladus, 602 }
body! { Tethys, 603 }
body! { Dione, 604 }
body! { Rhea, 605 }
body! { Titan, 606 }
body! { Hyperion, 607 }
body! { Iapetus, 608 }
body! { Phoebe, 609 }
body! { Janus, 610 }
body! { Epimetheus, 611 }
body! { Helene, 612 }
body! { Telesto, 613 }
body! { Calypso, 614 }
body! { Atlas, 615 }
body! { Prometheus, 616 }
body! { Pandora, 617 }
body! { Pan, 618 }
body! { Ymir, 619 }
body! { Paaliaq, 620 }
body! { Tarvos, 621 }
body! { Ijiraq, 622 }
body! { Suttungr, 623 }
body! { Kiviuq, 624 }
body! { Mundilfari, 625 }
body! { Albiorix, 626 }
body! { Skathi, 627 }
body! { Erriapus, 628 }
body! { Siarnaq, 629 }
body! { Thrymr, 630 }
body! { Narvi, 631 }
body! { Methone, 632 }
body! { Pallene, 633 }
body! { Polydeuces, 634 }
body! { Daphnis, 635 }
body! { Aegir, 636 }
body! { Bebhionn, 637 }
body! { Bergelmir, 638 }
body! { Bestla, 639 }
body! { Farbauti, 640 }
body! { Fenrir, 641 }
body! { Fornjot, 642 }
body! { Hati, 643 }
body! { Hyrrokkin, 644 }
body! { Kari, 645 }
body! { Loge, 646 }
body! { Skoll, 647 }
body! { Surtur, 648 }
body! { Anthe, 649 }
body! { Jarnsaxa, 650 }
body! { Greip, 651 }
body! { Tarqeq, 652 }
body! { Aegaeon, 653 }
body! { Ariel, 701 }
body! { Umbriel, 702 }
body! { Titania, 703 }
body! { Oberon, 704 }
body! { Miranda, 705 }
body! { Cordelia, 706 }
body! { Ophelia, 707 }
body! { Bianca, 708 }
body! { Cressida, 709 }
body! { Desdemona, 710 }
body! { Juliet, 711 }
body! { Portia, 712 }
body! { Rosalind, 713 }
body! { Belinda, 714 }
body! { Puck, 715 }
body! { Caliban, 716 }
body! { Sycorax, 717 }
body! { Prospero, 718 }
body! { Setebos, 719 }
body! { Stephano, 720 }
body! { Trinculo, 721 }
body! { Francisco, 722 }
body! { Margaret, 723 }
body! { Ferdinand, 724 }
body! { Perdita, 725 }
body! { Mab, 726 }
body! { Cupid, 727 }
body! { Triton, 801 }
body! { Nereid, 802 }
body! { Naiad, 803 }
body! { Thalassa, 804 }
body! { Despina, 805 }
body! { Galatea, 806 }
body! { Larissa, 807 }
body! { Proteus, 808 }
body! { Halimede, 809 }
body! { Psamathe, 810 }
body! { Sao, 811 }
body! { Laomedeia, 812 }
body! { Neso, 813 }
body! { Charon, 901 }
body! { Nix, 902 }
body! { Hydra, 903 }
body! { Kerberos, 904 }
body! { Styx, 905 }

// Minor bodies.
body! {Gaspra, 9511010 }
body! {Ida, 2431010 }
body! {Dactyl, 2431011 }
body! {Ceres, 2000001 }
body! {Pallas, 2000002 }
body! {Vesta, 2000004 }
body! {Psyche, 2000016 }
body! {Lutetia, 2000021 }
body! {Kleopatra, 2000216 }
body! {Eros, 2000433 }
body! {Davida, 2000511 }
body! {Mathilde, 2000253 }
body! {Steins, 2002867 }
body! {Braille, 2009969 }
body! {WilsonHarrington, 2004015 }
body! {Toutatis, 2004179 }
body! {Itokawa, 2025143 }
body! {Bennu, 2101955 }

pub trait Ellipsoid: NaifId {
    fn polar_radius() -> f64;

    fn mean_radius() -> f64;
}

pub fn polar_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::polar_radius()
}

pub fn mean_radius<T: Ellipsoid>(_: T) -> f64 {
    <T as Ellipsoid>::mean_radius()
}

pub trait Spheroid: Ellipsoid {
    fn equatorial_radius() -> f64;
}

pub fn equatorial_radius<T: Spheroid>(_: T) -> f64 {
    <T as Spheroid>::equatorial_radius()
}

pub trait TriAxial: Ellipsoid {
    fn subplanetary_radius() -> f64;

    fn along_orbit_radius() -> f64;
}

pub fn subplanetary_radius<T: TriAxial>(_: T) -> f64 {
    <T as TriAxial>::subplanetary_radius()
}

pub fn along_orbit_radius<T: TriAxial>(_: T) -> f64 {
    <T as TriAxial>::along_orbit_radius()
}

pub trait PointMass: NaifId {
    fn gravitational_parameter() -> f64;
}

pub fn gravitational_parameter<T: PointMass>(_: T) -> f64 {
    <T as PointMass>::gravitational_parameter()
}

pub type PolynomialCoefficients = (f64, f64, f64, &'static [f64]);

pub type NutationPrecessionCoefficients = (&'static [f64], &'static [f64]);

type Elements = (f64, f64, f64);

pub trait RotationalElements: NaifId {
    const NUTATION_PRECESSION_COEFFICIENTS: NutationPrecessionCoefficients;
    const RIGHT_ASCENSION_COEFFICIENTS: PolynomialCoefficients;
    const DECLINATION_COEFFICIENTS: PolynomialCoefficients;
    const PRIME_MERIDIAN_COEFFICIENTS: PolynomialCoefficients;

    fn theta(t: f64) -> Vec<f64> {
        let t = t / SECONDS_PER_JULIAN_CENTURY;
        let (theta0, theta1) = Self::NUTATION_PRECESSION_COEFFICIENTS;
        let mut theta = vec![0.0; theta0.len()];
        if theta0.is_empty() {
            return theta;
        }

        for i in 0..theta.len() {
            theta[i] = theta0[i] + theta1[i] * t;
        }
        theta
    }

    fn right_ascension(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (c0, c1, c2, c) = Self::RIGHT_ASCENSION_COEFFICIENTS;
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta[i].sin();
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
    }

    fn right_ascension_dot(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (_, c1, c2, c) = Self::RIGHT_ASCENSION_COEFFICIENTS;
        let (_, theta1) = Self::NUTATION_PRECESSION_COEFFICIENTS;
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta1[i] / SECONDS_PER_JULIAN_CENTURY * theta[i].cos()
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c1 / dt + 2.0 * c2 * t / dt.powi(2) + c_trig
    }

    fn declination(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (c0, c1, c2, c) = Self::DECLINATION_COEFFICIENTS;
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta[i].cos();
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
    }

    fn declination_dot(t: f64) -> f64 {
        let dt = SECONDS_PER_JULIAN_CENTURY;
        let (_, c1, c2, c) = Self::DECLINATION_COEFFICIENTS;
        let (_, theta1) = Self::NUTATION_PRECESSION_COEFFICIENTS;
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta1[i] / SECONDS_PER_JULIAN_CENTURY * theta[i].sin()
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c1 / dt + 2.0 * c2 * t / dt.powi(2) - c_trig
    }

    fn prime_meridian(t: f64) -> f64 {
        let dt = SECONDS_PER_DAY;
        let (c0, c1, c2, c) = Self::PRIME_MERIDIAN_COEFFICIENTS;
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta[i].sin();
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c0 + c1 * t / dt + c2 * t.powi(2) / dt.powi(2) + c_trig
    }

    fn prime_meridian_dot(t: f64) -> f64 {
        let dt = SECONDS_PER_DAY;
        let (_, c1, c2, c) = Self::PRIME_MERIDIAN_COEFFICIENTS;
        let (_, theta1) = Self::NUTATION_PRECESSION_COEFFICIENTS;
        let theta = Self::theta(t);
        let mut c_trig = vec![0.0; c.len()];
        if !c.is_empty() {
            for i in 0..c.len() {
                c_trig[i] = c[i] * theta1[i] / SECONDS_PER_JULIAN_CENTURY * theta[i].cos()
            }
        }
        let c_trig: f64 = c_trig.iter().sum();
        c1 / dt + 2.0 * c2 * t / dt.powi(2) + c_trig
    }

    fn rotational_elements(t: f64) -> Elements {
        (
            Self::right_ascension(t) + PI / 2.0,
            PI / 2.0 - Self::declination(t),
            Self::prime_meridian(t) % (2.0 * PI),
        )
    }

    fn rotational_element_rates(t: f64) -> Elements {
        (
            Self::right_ascension_dot(t),
            -Self::declination_dot(t),
            Self::prime_meridian_dot(t),
        )
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use super::*;

    // Jupiter is manually redefined here using known data. This avoids a dependecy on the
    // correctness of the PCK parser to test RotationalElements, and prevents compiler errors
    // when generated files are malformed or deleted in preparation for regeneration.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Jupiter;

    impl NaifId for Jupiter {
        const ID: i32 = 599;
    }

    impl PointMass for Jupiter {
        fn gravitational_parameter() -> f64 {
            126686531.9003704f64
        }
    }
    impl Ellipsoid for Jupiter {
        fn polar_radius() -> f64 {
            66854f64
        }
        fn mean_radius() -> f64 {
            69946f64
        }
    }
    impl Spheroid for Jupiter {
        fn equatorial_radius() -> f64 {
            71492f64
        }
    }

    impl RotationalElements for Jupiter {
        const NUTATION_PRECESSION_COEFFICIENTS: NutationPrecessionCoefficients = (
            &[
                1.2796754075622423f64,
                0.42970006184100396f64,
                4.9549897464119015f64,
                6.2098814785958245f64,
                2.092649773141201f64,
                4.010766621082969f64,
                6.147922290150026f64,
                1.9783307071355725f64,
                2.5593508151244846f64,
                0.8594001236820079f64,
                1.734171606432425f64,
                3.0699533280603655f64,
                5.241627996900319f64,
                1.9898901100379935f64,
                0.864134346731335f64,
            ],
            &[
                1596.503281347521f64,
                787.7927551311844f64,
                84.66068602648895f64,
                20.792107379008446f64,
                4.574507969477138f64,
                1.1222467090323538f64,
                41.58421475801689f64,
                105.9414855960558f64,
                3193.006562695042f64,
                1575.5855102623689f64,
                84.65553032387855f64,
                20.80363527871787f64,
                4.582318317879813f64,
                105.94580703128374f64,
                1.1222467090323538f64,
            ],
        );

        const RIGHT_ASCENSION_COEFFICIENTS: PolynomialCoefficients = (
            4.6784701644349695f64,
            -0.00011342894808711148f64,
            0f64,
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.0000020420352248333656f64,
                0.000016371188383706813f64,
                0.000024993114888558796f64,
                0.0000005235987755982989f64,
                0.00003752457891787809f64,
            ],
        );

        const DECLINATION_COEFFICIENTS: PolynomialCoefficients = (
            1.1256553894213766f64,
            0.00004211479485062318f64,
            0f64,
            &[
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0f64,
                0.0000008726646259971648f64,
                0.000007051130178057092f64,
                0.000010768681484805013f64,
                -0.00000022689280275926283f64,
                0.00001616174887346749f64,
            ],
        );

        const PRIME_MERIDIAN_COEFFICIENTS: PolynomialCoefficients = (
            4.973315703557842f64,
            15.193719457141356f64,
            0f64,
            &[
                0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64, 0f64,
                0f64,
            ],
        );
    }

    #[test]
    fn test_grav_param() {
        assert_eq!(
            gravitational_parameter(Jupiter),
            Jupiter::gravitational_parameter()
        );
    }

    #[test]
    fn test_mean_radius() {
        assert_eq!(mean_radius(Jupiter), Jupiter::mean_radius());
    }

    #[test]
    fn test_polar_radius() {
        assert_eq!(polar_radius(Jupiter), Jupiter::polar_radius());
    }

    #[test]
    fn test_equatorial_radius() {
        assert_eq!(equatorial_radius(Jupiter), Jupiter::equatorial_radius());
    }

    #[test]
    fn test_subplanetary_radius() {
        assert_eq!(subplanetary_radius(Moon), Moon::subplanetary_radius());
    }

    #[test]
    fn test_along_orbit_radius() {
        assert_eq!(along_orbit_radius(Moon), Moon::along_orbit_radius());
    }

    #[test]
    fn test_rotational_elements_right_ascension() {
        assert_float_eq!(
            Jupiter::right_ascension(0.0),
            4.678480799964803,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_right_ascension_dot() {
        assert_float_eq!(
            Jupiter::right_ascension_dot(0.0),
            -1.3266588500099516e-13,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_declination() {
        assert_float_eq!(Jupiter::declination(0.0), 1.1256642372977634, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_declination_dot() {
        assert_float_eq!(
            Jupiter::declination_dot(0.0),
            3.004482367136341e-15,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_prime_meridian() {
        assert_float_eq!(Jupiter::prime_meridian(0.0), 4.973315703557842, rel <= 1e-8);
    }

    #[test]
    fn test_rotational_elements_prime_meridian_dot() {
        assert_float_eq!(
            Jupiter::prime_meridian_dot(0.0),
            0.00017585323445765458,
            rel <= 1e-8
        );
    }

    #[test]
    fn test_rotational_elements_rotational_elements() {
        let (ra, dec, pm) = Jupiter::rotational_elements(0.0);
        let (expected_ra, expected_dec, expected_pm) =
            (6.249277121030398, 0.44513208936761073, 4.973315703557842);

        assert_float_eq!(
            ra,
            expected_ra,
            rel <= 1e-8,
            "Expected right ascension {}, got {}",
            expected_ra,
            ra
        );
        assert_float_eq!(
            dec,
            expected_dec,
            rel <= 1e-8,
            "Expected declination {}, got {}",
            expected_dec,
            dec
        );
        assert_float_eq!(
            pm,
            expected_pm,
            rel <= 1e-8,
            "Expected prime meridian {}, got {}",
            expected_pm,
            pm
        );
    }

    #[test]
    fn test_rotational_elements_rotational_element_rates() {
        let (ra_dot, dec_dot, pm_dot) = Jupiter::rotational_element_rates(0.0);
        let (expected_ra_dot, expected_dec_dot, expected_pm_dot) = (
            -1.3266588500099516e-13,
            -3.004482367136341e-15,
            0.00017585323445765458,
        );

        assert_float_eq!(
            ra_dot,
            expected_ra_dot,
            rel <= 1e-8,
            "Expected right ascension rate {}, got {}",
            expected_ra_dot,
            ra_dot
        );
        assert_float_eq!(
            dec_dot,
            expected_dec_dot,
            rel <= 1e-8,
            "Expected declination rate {}, got {}",
            expected_dec_dot,
            dec_dot
        );
        assert_float_eq!(
            pm_dot,
            expected_pm_dot,
            rel <= 1e-8,
            "Expected prime meridian rate {}, got {}",
            expected_pm_dot,
            pm_dot
        );
    }
}
