/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use dyn_clone::{clone_trait_object, DynClone};
use std::f64::consts::PI;
use std::fmt::{Display, Formatter};

pub use generated::*;

use crate::time::constants::f64::{SECONDS_PER_DAY, SECONDS_PER_JULIAN_CENTURY};

mod generated;

pub mod fundamental;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct NaifId(pub i32);

impl Display for NaifId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// NaifId is implemented for all bodies.
pub trait Body {
    fn id(&self) -> NaifId;
    fn name(&self) -> &'static str;
}

/// Expands to derivations of the fundamental traits every body must implement.
macro_rules! body {
    ($i:ident, $naif_id:literal) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl Body for $i {
            fn id(&self) -> NaifId {
                NaifId($naif_id)
            }

            fn name(&self) -> &'static str {
                stringify!($i)
            }
        }

        impl Display for $i {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }
    };
    ($i:ident, $t:ident, $naif_id:literal) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl $t for $i {}

        impl Body for $i {
            fn id(&self) -> NaifId {
                NaifId($naif_id)
            }

            fn name(&self) -> &'static str {
                stringify!($i)
            }
        }

        impl Display for $i {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }
    };
    ($i:ident, $name:literal, $naif_id:literal) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl Body for $i {
            fn id(&self) -> NaifId {
                NaifId($naif_id)
            }

            fn name(&self) -> &'static str {
                $name
            }
        }

        impl Display for $i {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }
    };
    ($i:ident, $t:ident, $name:literal, $naif_id:literal) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl $t for $i {}

        impl Body for $i {
            fn id(&self) -> NaifId {
                NaifId($naif_id)
            }

            fn name(&self) -> &'static str {
                $name
            }
        }

        impl Display for $i {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }
    };
}

// The Sun.
body! { Sun, 10 }

// Planets.
pub trait Planet: PointMass + Spheroid + DynClone {}
clone_trait_object!(Planet);

body! { Mercury, Planet, 199 }
body! { Venus, Planet, 299 }
body! { Earth, Planet, 399 }
body! { Mars, Planet, 499 }
body! { Jupiter, Planet, 599 }
body! { Saturn, Planet, 699 }
body! { Uranus, Planet, 799 }
body! { Neptune, Planet, 899 }
body! { Pluto, Planet, 999 }

// Barycenters.
pub trait Barycenter: PointMass + DynClone {}
clone_trait_object!(Barycenter);

body! { SolarSystemBarycenter, Barycenter, "Solar System Barycenter", 0 }
body! { MercuryBarycenter, Barycenter, "Mercury Barycenter", 1 }
body! { VenusBarycenter, Barycenter, "Venus Barycenter", 2 }
body! { EarthBarycenter, Barycenter, "Earth Barycenter", 3 }
body! { MarsBarycenter, Barycenter, "Mars Barycenter", 4 }
body! { JupiterBarycenter, Barycenter, "Jupiter Barycenter", 5 }
body! { SaturnBarycenter, Barycenter, "Saturn Barycenter", 6 }
body! { UranusBarycenter, Barycenter, "Uranus Barycenter", 7 }
body! { NeptuneBarycenter, Barycenter, "Neptune Barycenter", 8 }
body! { PlutoBarycenter, Barycenter, "Pluto Barycenter", 9 }

impl PointMass for SolarSystemBarycenter {
    fn gravitational_parameter(&self) -> f64 {
        Sun.gravitational_parameter()
    }
}

// Satellites.
pub trait Satellite: PointMass + TriAxial + DynClone {}
clone_trait_object!(Satellite);

body! { Moon, Satellite, 301 }
body! { Phobos, Satellite, 401 }
body! { Deimos, Satellite, 402 }
body! { Io, Satellite, 501 }
body! { Europa, Satellite, 502 }
body! { Ganymede, Satellite, 503 }
body! { Callisto, Satellite, 504 }
body! { Amalthea, Satellite, 505 }
body! { Himalia, Satellite, 506 }
body! { Elara,  507 }
body! { Pasiphae,  508 }
body! { Sinope,  509 }
body! { Lysithea,  510 }
body! { Carme,  511 }
body! { Ananke,  512 }
body! { Leda,  513 }
body! { Thebe, Satellite, 514 }
body! { Adrastea, Satellite, 515 }
body! { Metis, Satellite, 516 }
body! { Callirrhoe,  517 }
body! { Themisto,  518 }
body! { Magaclite,  519 }
body! { Taygete,  520 }
body! { Chaldene,  521 }
body! { Harpalyke,  522 }
body! { Kalyke,  523 }
body! { Iocaste,  524 }
body! { Erinome,  525 }
body! { Isonoe,  526 }
body! { Praxidike,  527 }
body! { Autonoe,  528 }
body! { Thyone,  529 }
body! { Hermippe,  530 }
body! { Aitne,  531 }
body! { Eurydome,  532 }
body! { Euanthe,  533 }
body! { Euporie,  534 }
body! { Orthosie,  535 }
body! { Sponde,  536 }
body! { Kale,  537 }
body! { Pasithee,  538 }
body! { Hegemone,  539 }
body! { Mneme,  540 }
body! { Aoede,  541 }
body! { Thelxinoe,  542 }
body! { Arche,  543 }
body! { Kallichore,  544 }
body! { Helike,  545 }
body! { Carpo,  546 }
body! { Eukelade,  547 }
body! { Cyllene,  548 }
body! { Kore,  549 }
body! { Herse,  550 }
body! { Dia,  553 }
body! { Mimas, Satellite, 601 }
body! { Enceladus, Satellite, 602 }
body! { Tethys, Satellite, 603 }
body! { Dione, Satellite, 604 }
body! { Rhea, Satellite, 605 }
body! { Titan, Satellite, 606 }
body! { Hyperion, Satellite, 607 }
body! { Iapetus, Satellite, 608 }
body! { Phoebe, Satellite, 609 }
body! { Janus, Satellite, 610 }
body! { Epimetheus, Satellite, 611 }
body! { Helene, Satellite, 612 }
body! { Telesto,  613 }
body! { Calypso,  614 }
body! { Atlas, Satellite, 615 }
body! { Prometheus, Satellite, 616 }
body! { Pandora, Satellite, 617 }
body! { Pan,  618 }
body! { Ymir,  619 }
body! { Paaliaq,  620 }
body! { Tarvos,  621 }
body! { Ijiraq,  622 }
body! { Suttungr,  623 }
body! { Kiviuq,  624 }
body! { Mundilfari,  625 }
body! { Albiorix,  626 }
body! { Skathi,  627 }
body! { Erriapus,  628 }
body! { Siarnaq,  629 }
body! { Thrymr,  630 }
body! { Narvi,  631 }
body! { Methone,  632 }
body! { Pallene,  633 }
body! { Polydeuces,  634 }
body! { Daphnis,  635 }
body! { Aegir,  636 }
body! { Bebhionn,  637 }
body! { Bergelmir,  638 }
body! { Bestla,  639 }
body! { Farbauti,  640 }
body! { Fenrir,  641 }
body! { Fornjot,  642 }
body! { Hati,  643 }
body! { Hyrrokkin,  644 }
body! { Kari,  645 }
body! { Loge,  646 }
body! { Skoll,  647 }
body! { Surtur,  648 }
body! { Anthe,  649 }
body! { Jarnsaxa,  650 }
body! { Greip,  651 }
body! { Tarqeq,  652 }
body! { Aegaeon,  653 }
body! { Ariel, Satellite, 701 }
body! { Umbriel, Satellite, 702 }
body! { Titania, Satellite, 703 }
body! { Oberon, Satellite, 704 }
body! { Miranda, Satellite, 705 }
body! { Cordelia,  706 }
body! { Ophelia,  707 }
body! { Bianca,  708 }
body! { Cressida,  709 }
body! { Desdemona,  710 }
body! { Juliet,  711 }
body! { Portia,  712 }
body! { Rosalind,  713 }
body! { Belinda,  714 }
body! { Puck,  715 }
body! { Caliban,  716 }
body! { Sycorax,  717 }
body! { Prospero,  718 }
body! { Setebos,  719 }
body! { Stephano,  720 }
body! { Trinculo,  721 }
body! { Francisco,  722 }
body! { Margaret,  723 }
body! { Ferdinand,  724 }
body! { Perdita,  725 }
body! { Mab,  726 }
body! { Cupid,  727 }
body! { Triton, Satellite, 801 }
body! { Nereid,  802 }
body! { Naiad, Satellite, 803 }
body! { Thalassa, Satellite, 804 }
body! { Despina, Satellite, 805 }
body! { Galatea, Satellite, 806 }
body! { Larissa, Satellite, 807 }
body! { Proteus, Satellite, 808 }
body! { Halimede,  809 }
body! { Psamathe,  810 }
body! { Sao,  811 }
body! { Laomedeia,  812 }
body! { Neso,  813 }
body! { Charon, Satellite, 901 }
body! { Nix,  902 }
body! { Hydra,  903 }
body! { Kerberos,  904 }
body! { Styx,  905 }

// Minor bodies.
pub trait MinorBody: PointMass + TriAxial + DynClone {}
clone_trait_object!(MinorBody);

body! {Gaspra, 9511010 }
body! {Ida, 2431010 }
body! {Dactyl, 2431011 }
body! {Ceres, MinorBody, 2000001 }
body! {Pallas, 2000002 }
body! {Vesta, MinorBody, 2000004 }
body! {Psyche, MinorBody, 2000016 }
body! {Lutetia, 2000021 }
body! {Kleopatra, 2000216 }
body! {Eros, MinorBody, 2000433 }
body! {Davida, MinorBody, 2000511 }
body! {Mathilde, 2000253 }
body! {Steins, 2002867 }
body! {Braille, 2009969 }
body! {WilsonHarrington, "Wilson-Harrington", 2004015 }
body! {Toutatis, 2004179 }
body! {Itokawa, 2025143 }
body! {Bennu, 2101955 }

impl NaifId {
    pub fn body(&self) -> Option<Box<dyn Body>> {
        match self.0 {
            10 => Some(Box::new(Sun)),

            // Planets.
            199 => Some(Box::new(Mercury)),
            299 => Some(Box::new(Venus)),
            399 => Some(Box::new(Earth)),
            499 => Some(Box::new(Mars)),
            599 => Some(Box::new(Jupiter)),
            699 => Some(Box::new(Saturn)),
            799 => Some(Box::new(Uranus)),
            899 => Some(Box::new(Neptune)),
            999 => Some(Box::new(Pluto)),

            // Barycenters.
            0 => Some(Box::new(SolarSystemBarycenter)),
            1 => Some(Box::new(MercuryBarycenter)),
            2 => Some(Box::new(VenusBarycenter)),
            3 => Some(Box::new(EarthBarycenter)),
            4 => Some(Box::new(MarsBarycenter)),
            5 => Some(Box::new(JupiterBarycenter)),
            6 => Some(Box::new(SaturnBarycenter)),
            7 => Some(Box::new(UranusBarycenter)),
            8 => Some(Box::new(NeptuneBarycenter)),
            9 => Some(Box::new(PlutoBarycenter)),

            // Satellites.
            301 => Some(Box::new(Moon)),
            401 => Some(Box::new(Phobos)),
            402 => Some(Box::new(Deimos)),
            501 => Some(Box::new(Io)),
            502 => Some(Box::new(Europa)),
            503 => Some(Box::new(Ganymede)),
            504 => Some(Box::new(Callisto)),
            505 => Some(Box::new(Amalthea)),
            506 => Some(Box::new(Himalia)),
            507 => Some(Box::new(Elara)),
            508 => Some(Box::new(Pasiphae)),
            509 => Some(Box::new(Sinope)),
            510 => Some(Box::new(Lysithea)),
            511 => Some(Box::new(Carme)),
            512 => Some(Box::new(Ananke)),
            513 => Some(Box::new(Leda)),
            514 => Some(Box::new(Thebe)),
            515 => Some(Box::new(Adrastea)),
            516 => Some(Box::new(Metis)),
            517 => Some(Box::new(Callirrhoe)),
            518 => Some(Box::new(Themisto)),
            519 => Some(Box::new(Magaclite)),
            520 => Some(Box::new(Taygete)),
            521 => Some(Box::new(Chaldene)),
            522 => Some(Box::new(Harpalyke)),
            523 => Some(Box::new(Kalyke)),
            524 => Some(Box::new(Iocaste)),
            525 => Some(Box::new(Erinome)),
            526 => Some(Box::new(Isonoe)),
            527 => Some(Box::new(Praxidike)),
            528 => Some(Box::new(Autonoe)),
            529 => Some(Box::new(Thyone)),
            530 => Some(Box::new(Hermippe)),
            531 => Some(Box::new(Aitne)),
            532 => Some(Box::new(Eurydome)),
            533 => Some(Box::new(Euanthe)),
            534 => Some(Box::new(Euporie)),
            535 => Some(Box::new(Orthosie)),
            536 => Some(Box::new(Sponde)),
            537 => Some(Box::new(Kale)),
            538 => Some(Box::new(Pasithee)),
            539 => Some(Box::new(Hegemone)),
            540 => Some(Box::new(Mneme)),
            541 => Some(Box::new(Aoede)),
            542 => Some(Box::new(Thelxinoe)),
            543 => Some(Box::new(Arche)),
            544 => Some(Box::new(Kallichore)),
            545 => Some(Box::new(Helike)),
            546 => Some(Box::new(Carpo)),
            547 => Some(Box::new(Eukelade)),
            548 => Some(Box::new(Cyllene)),
            549 => Some(Box::new(Kore)),
            550 => Some(Box::new(Herse)),
            553 => Some(Box::new(Dia)),
            601 => Some(Box::new(Mimas)),
            602 => Some(Box::new(Enceladus)),
            603 => Some(Box::new(Tethys)),
            604 => Some(Box::new(Dione)),
            605 => Some(Box::new(Rhea)),
            606 => Some(Box::new(Titan)),
            607 => Some(Box::new(Hyperion)),
            608 => Some(Box::new(Iapetus)),
            609 => Some(Box::new(Phoebe)),
            610 => Some(Box::new(Janus)),
            611 => Some(Box::new(Epimetheus)),
            612 => Some(Box::new(Helene)),
            613 => Some(Box::new(Telesto)),
            614 => Some(Box::new(Calypso)),
            615 => Some(Box::new(Atlas)),
            616 => Some(Box::new(Prometheus)),
            617 => Some(Box::new(Pandora)),
            618 => Some(Box::new(Pan)),
            619 => Some(Box::new(Ymir)),
            620 => Some(Box::new(Paaliaq)),
            621 => Some(Box::new(Tarvos)),
            622 => Some(Box::new(Ijiraq)),
            623 => Some(Box::new(Suttungr)),
            624 => Some(Box::new(Kiviuq)),
            625 => Some(Box::new(Mundilfari)),
            626 => Some(Box::new(Albiorix)),
            627 => Some(Box::new(Skathi)),
            628 => Some(Box::new(Erriapus)),
            629 => Some(Box::new(Siarnaq)),
            630 => Some(Box::new(Thrymr)),
            631 => Some(Box::new(Narvi)),
            632 => Some(Box::new(Methone)),
            633 => Some(Box::new(Pallene)),
            634 => Some(Box::new(Polydeuces)),
            635 => Some(Box::new(Daphnis)),
            636 => Some(Box::new(Aegir)),
            637 => Some(Box::new(Bebhionn)),
            638 => Some(Box::new(Bergelmir)),
            639 => Some(Box::new(Bestla)),
            640 => Some(Box::new(Farbauti)),
            641 => Some(Box::new(Fenrir)),
            642 => Some(Box::new(Fornjot)),
            643 => Some(Box::new(Hati)),
            644 => Some(Box::new(Hyrrokkin)),
            645 => Some(Box::new(Kari)),
            646 => Some(Box::new(Loge)),
            647 => Some(Box::new(Skoll)),
            648 => Some(Box::new(Surtur)),
            649 => Some(Box::new(Anthe)),
            650 => Some(Box::new(Jarnsaxa)),
            651 => Some(Box::new(Greip)),
            652 => Some(Box::new(Tarqeq)),
            653 => Some(Box::new(Aegaeon)),
            701 => Some(Box::new(Ariel)),
            702 => Some(Box::new(Umbriel)),
            703 => Some(Box::new(Titania)),
            704 => Some(Box::new(Oberon)),
            705 => Some(Box::new(Miranda)),
            706 => Some(Box::new(Cordelia)),
            707 => Some(Box::new(Ophelia)),
            708 => Some(Box::new(Bianca)),
            709 => Some(Box::new(Cressida)),
            710 => Some(Box::new(Desdemona)),
            711 => Some(Box::new(Juliet)),
            712 => Some(Box::new(Portia)),
            713 => Some(Box::new(Rosalind)),
            714 => Some(Box::new(Belinda)),
            715 => Some(Box::new(Puck)),
            716 => Some(Box::new(Caliban)),
            717 => Some(Box::new(Sycorax)),
            718 => Some(Box::new(Prospero)),
            719 => Some(Box::new(Setebos)),
            720 => Some(Box::new(Stephano)),
            721 => Some(Box::new(Trinculo)),
            722 => Some(Box::new(Francisco)),
            723 => Some(Box::new(Margaret)),
            724 => Some(Box::new(Ferdinand)),
            725 => Some(Box::new(Perdita)),
            726 => Some(Box::new(Mab)),
            727 => Some(Box::new(Cupid)),
            801 => Some(Box::new(Triton)),
            802 => Some(Box::new(Nereid)),
            803 => Some(Box::new(Naiad)),
            804 => Some(Box::new(Thalassa)),
            805 => Some(Box::new(Despina)),
            806 => Some(Box::new(Galatea)),
            807 => Some(Box::new(Larissa)),
            808 => Some(Box::new(Proteus)),
            809 => Some(Box::new(Halimede)),
            810 => Some(Box::new(Psamathe)),
            811 => Some(Box::new(Sao)),
            812 => Some(Box::new(Laomedeia)),
            813 => Some(Box::new(Neso)),
            901 => Some(Box::new(Charon)),
            902 => Some(Box::new(Nix)),
            903 => Some(Box::new(Hydra)),
            904 => Some(Box::new(Kerberos)),
            905 => Some(Box::new(Styx)),

            // Minor bodies.
            9511010 => Some(Box::new(Gaspra)),
            2431010 => Some(Box::new(Ida)),
            2431011 => Some(Box::new(Dactyl)),
            2000001 => Some(Box::new(Ceres)),
            2000002 => Some(Box::new(Pallas)),
            2000004 => Some(Box::new(Vesta)),
            2000016 => Some(Box::new(Psyche)),
            2000021 => Some(Box::new(Lutetia)),
            2000216 => Some(Box::new(Kleopatra)),
            2000433 => Some(Box::new(Eros)),
            2000511 => Some(Box::new(Davida)),
            2000253 => Some(Box::new(Mathilde)),
            2002867 => Some(Box::new(Steins)),
            2009969 => Some(Box::new(Braille)),
            2004015 => Some(Box::new(WilsonHarrington)),
            2004179 => Some(Box::new(Toutatis)),
            2025143 => Some(Box::new(Itokawa)),
            2101955 => Some(Box::new(Bennu)),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        if let Some(body) = self.body() {
            body.name().to_string()
        } else {
            format!("Body {}", self.0)
        }
    }
}

pub trait Ellipsoid: Body {
    fn polar_radius(&self) -> f64;

    fn mean_radius(&self) -> f64;
}

pub trait Spheroid: Ellipsoid {
    fn equatorial_radius(&self) -> f64;
}

pub trait TriAxial: Ellipsoid {
    fn subplanetary_radius(&self) -> f64;

    fn along_orbit_radius(&self) -> f64;
}

pub trait PointMass: Body {
    fn gravitational_parameter(&self) -> f64;
}

pub type PolynomialCoefficients = (f64, f64, f64, &'static [f64]);

pub type NutationPrecessionCoefficients = (&'static [f64], &'static [f64]);

type Elements = (f64, f64, f64);

pub trait RotationalElements: Body {
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

    // Jupiter is manually redefined here using known data. This avoids a dependency on the
    // correctness of the PCK parser to test RotationalElements, and prevents compiler errors
    // when generated files are malformed or deleted in preparation for regeneration.
    body! { Jupiter, 599 }

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

    body! { Rupert, "Persephone/Rupert", 1099 }

    #[test]
    fn test_body() {
        let body = Jupiter;
        let id = body.id().0;
        let name = body.name();
        assert_eq!(id, 599);
        assert_eq!(name, "Jupiter");

        let body = Rupert;
        let id = body.id().0;
        let name = body.name();
        assert_eq!(id, 1099);
        assert_eq!(name, "Persephone/Rupert");
    }

    #[test]
    fn test_naif_id() {
        let id = NaifId(0);
        let name = id.name();
        assert_eq!(name, "Solar System Barycenter");

        let id = NaifId(-42);
        let name = id.name();
        assert_eq!(name, "Body -42");
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
