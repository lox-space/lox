// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::common::write_file;
use lox_io::spice::Kernel;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use std::path::Path;

struct OriginData {
    name: &'static str,
    id: i32,
    mean_radius: Option<f64>,
}

const ORIGINS: [OriginData; 190] = [
    OriginData {
        name: "Sun",
        id: 10,
        mean_radius: None,
    },
    // Planets.
    OriginData {
        name: "Mercury",
        id: 199,
        mean_radius: Some(2439.4),
    },
    OriginData {
        name: "Venus",
        id: 299,
        mean_radius: Some(6051.8),
    },
    OriginData {
        name: "Earth",
        id: 399,
        mean_radius: Some(6371.0084),
    },
    OriginData {
        name: "Mars",
        id: 499,
        mean_radius: Some(3389.5),
    },
    OriginData {
        name: "Jupiter",
        id: 599,
        mean_radius: Some(69911.0),
    },
    OriginData {
        name: "Saturn",
        id: 699,
        mean_radius: Some(58232.0),
    },
    OriginData {
        name: "Uranus",
        id: 799,
        mean_radius: Some(25362.0),
    },
    OriginData {
        name: "Neptune",
        id: 899,
        mean_radius: Some(24622.0),
    },
    OriginData {
        name: "Pluto",
        id: 999,
        mean_radius: Some(1188.3),
    },
    // Barycenters.
    OriginData {
        name: "Solar System Barycenter",
        id: 0,
        mean_radius: None,
    },
    OriginData {
        name: "Mercury Barycenter",
        id: 1,
        mean_radius: None,
    },
    OriginData {
        name: "Venus Barycenter",
        id: 2,
        mean_radius: None,
    },
    OriginData {
        name: "Earth Barycenter",
        id: 3,
        mean_radius: None,
    },
    OriginData {
        name: "Mars Barycenter",
        id: 4,
        mean_radius: None,
    },
    OriginData {
        name: "Jupiter Barycenter",
        id: 5,
        mean_radius: None,
    },
    OriginData {
        name: "Saturn Barycenter",
        id: 6,
        mean_radius: None,
    },
    OriginData {
        name: "Uranus Barycenter",
        id: 7,
        mean_radius: None,
    },
    OriginData {
        name: "Neptune Barycenter",
        id: 8,
        mean_radius: None,
    },
    OriginData {
        name: "Pluto Barycenter",
        id: 9,
        mean_radius: None,
    },
    // Satellites.
    OriginData {
        name: "Moon",
        id: 301,
        mean_radius: Some(1737.4),
    },
    OriginData {
        name: "Phobos",
        id: 401,
        mean_radius: Some(11.08),
    },
    OriginData {
        name: "Deimos",
        id: 402,
        mean_radius: Some(6.2),
    },
    OriginData {
        name: "Io",
        id: 501,
        mean_radius: Some(1821.49),
    },
    OriginData {
        name: "Europa",
        id: 502,
        mean_radius: Some(1560.8),
    },
    OriginData {
        name: "Ganymede",
        id: 503,
        mean_radius: Some(2631.2),
    },
    OriginData {
        name: "Callisto",
        id: 504,
        mean_radius: Some(2410.3),
    },
    OriginData {
        name: "Amalthea",
        id: 505,
        mean_radius: Some(83.5),
    },
    OriginData {
        name: "Himalia",
        id: 506,
        mean_radius: Some(85.0),
    },
    OriginData {
        name: "Elara",
        id: 507,
        mean_radius: Some(40.0),
    },
    OriginData {
        name: "Pasiphae",
        id: 508,
        mean_radius: Some(18.0),
    },
    OriginData {
        name: "Sinope",
        id: 509,
        mean_radius: Some(14.0),
    },
    OriginData {
        name: "Lysithea",
        id: 510,
        mean_radius: Some(12.0),
    },
    OriginData {
        name: "Carme",
        id: 511,
        mean_radius: Some(15.0),
    },
    OriginData {
        name: "Ananke",
        id: 512,
        mean_radius: Some(10.0),
    },
    OriginData {
        name: "Leda",
        id: 513,
        mean_radius: Some(5.0),
    },
    OriginData {
        name: "Thebe",
        id: 514,
        mean_radius: Some(49.3),
    },
    OriginData {
        name: "Adrastea",
        id: 515,
        mean_radius: Some(8.2),
    },
    OriginData {
        name: "Metis",
        id: 516,
        mean_radius: Some(21.5),
    },
    OriginData {
        name: "Callirrhoe",
        id: 517,
        mean_radius: None,
    },
    OriginData {
        name: "Themisto",
        id: 518,
        mean_radius: None,
    },
    OriginData {
        name: "Magaclite",
        id: 519,
        mean_radius: None,
    },
    OriginData {
        name: "Taygete",
        id: 520,
        mean_radius: None,
    },
    OriginData {
        name: "Chaldene",
        id: 521,
        mean_radius: None,
    },
    OriginData {
        name: "Harpalyke",
        id: 522,
        mean_radius: None,
    },
    OriginData {
        name: "Kalyke",
        id: 523,
        mean_radius: None,
    },
    OriginData {
        name: "Iocaste",
        id: 524,
        mean_radius: None,
    },
    OriginData {
        name: "Erinome",
        id: 525,
        mean_radius: None,
    },
    OriginData {
        name: "Isonoe",
        id: 526,
        mean_radius: None,
    },
    OriginData {
        name: "Praxidike",
        id: 527,
        mean_radius: None,
    },
    OriginData {
        name: "Autonoe",
        id: 528,
        mean_radius: None,
    },
    OriginData {
        name: "Thyone",
        id: 529,
        mean_radius: None,
    },
    OriginData {
        name: "Hermippe",
        id: 530,
        mean_radius: None,
    },
    OriginData {
        name: "Aitne",
        id: 531,
        mean_radius: None,
    },
    OriginData {
        name: "Eurydome",
        id: 532,
        mean_radius: None,
    },
    OriginData {
        name: "Euanthe",
        id: 533,
        mean_radius: None,
    },
    OriginData {
        name: "Euporie",
        id: 534,
        mean_radius: None,
    },
    OriginData {
        name: "Orthosie",
        id: 535,
        mean_radius: None,
    },
    OriginData {
        name: "Sponde",
        id: 536,
        mean_radius: None,
    },
    OriginData {
        name: "Kale",
        id: 537,
        mean_radius: None,
    },
    OriginData {
        name: "Pasithee",
        id: 538,
        mean_radius: None,
    },
    OriginData {
        name: "Hegemone",
        id: 539,
        mean_radius: None,
    },
    OriginData {
        name: "Mneme",
        id: 540,
        mean_radius: None,
    },
    OriginData {
        name: "Aoede",
        id: 541,
        mean_radius: None,
    },
    OriginData {
        name: "Thelxinoe",
        id: 542,
        mean_radius: None,
    },
    OriginData {
        name: "Arche",
        id: 543,
        mean_radius: None,
    },
    OriginData {
        name: "Kallichore",
        id: 544,
        mean_radius: None,
    },
    OriginData {
        name: "Helike",
        id: 545,
        mean_radius: None,
    },
    OriginData {
        name: "Carpo",
        id: 546,
        mean_radius: None,
    },
    OriginData {
        name: "Eukelade",
        id: 547,
        mean_radius: None,
    },
    OriginData {
        name: "Cyllene",
        id: 548,
        mean_radius: None,
    },
    OriginData {
        name: "Kore",
        id: 549,
        mean_radius: None,
    },
    OriginData {
        name: "Herse",
        id: 550,
        mean_radius: None,
    },
    OriginData {
        name: "Dia",
        id: 553,
        mean_radius: None,
    },
    OriginData {
        name: "Mimas",
        id: 601,
        mean_radius: Some(198.2),
    },
    OriginData {
        name: "Enceladus",
        id: 602,
        mean_radius: Some(252.1),
    },
    OriginData {
        name: "Tethys",
        id: 603,
        mean_radius: Some(531.0),
    },
    OriginData {
        name: "Dione",
        id: 604,
        mean_radius: Some(561.4),
    },
    OriginData {
        name: "Rhea",
        id: 605,
        mean_radius: Some(763.5),
    },
    OriginData {
        name: "Titan",
        id: 606,
        mean_radius: Some(2575.0),
    },
    OriginData {
        name: "Hyperion",
        id: 607,
        mean_radius: Some(135.0),
    },
    OriginData {
        name: "Iapetus",
        id: 608,
        mean_radius: Some(734.3),
    },
    OriginData {
        name: "Phoebe",
        id: 609,
        mean_radius: Some(106.5),
    },
    OriginData {
        name: "Janus",
        id: 610,
        mean_radius: Some(89.2),
    },
    OriginData {
        name: "Epimetheus",
        id: 611,
        mean_radius: Some(58.2),
    },
    OriginData {
        name: "Helene",
        id: 612,
        mean_radius: Some(18.0),
    },
    OriginData {
        name: "Telesto",
        id: 613,
        mean_radius: Some(12.4),
    },
    OriginData {
        name: "Calypso",
        id: 614,
        mean_radius: Some(9.6),
    },
    OriginData {
        name: "Atlas",
        id: 615,
        mean_radius: Some(15.1),
    },
    OriginData {
        name: "Prometheus",
        id: 616,
        mean_radius: Some(43.1),
    },
    OriginData {
        name: "Pandora",
        id: 617,
        mean_radius: Some(40.6),
    },
    OriginData {
        name: "Pan",
        id: 618,
        mean_radius: Some(14.0),
    },
    OriginData {
        name: "Ymir",
        id: 619,
        mean_radius: None,
    },
    OriginData {
        name: "Paaliaq",
        id: 620,
        mean_radius: None,
    },
    OriginData {
        name: "Tarvos",
        id: 621,
        mean_radius: None,
    },
    OriginData {
        name: "Ijiraq",
        id: 622,
        mean_radius: None,
    },
    OriginData {
        name: "Suttungr",
        id: 623,
        mean_radius: None,
    },
    OriginData {
        name: "Kiviuq",
        id: 624,
        mean_radius: None,
    },
    OriginData {
        name: "Mundilfari",
        id: 625,
        mean_radius: None,
    },
    OriginData {
        name: "Albiorix",
        id: 626,
        mean_radius: None,
    },
    OriginData {
        name: "Skathi",
        id: 627,
        mean_radius: None,
    },
    OriginData {
        name: "Erriapus",
        id: 628,
        mean_radius: None,
    },
    OriginData {
        name: "Siarnaq",
        id: 629,
        mean_radius: None,
    },
    OriginData {
        name: "Thrymr",
        id: 630,
        mean_radius: None,
    },
    OriginData {
        name: "Narvi",
        id: 631,
        mean_radius: None,
    },
    OriginData {
        name: "Methone",
        id: 632,
        mean_radius: Some(1.45),
    },
    OriginData {
        name: "Pallene",
        id: 633,
        mean_radius: Some(2.23),
    },
    OriginData {
        name: "Polydeuces",
        id: 634,
        mean_radius: Some(1.3),
    },
    OriginData {
        name: "Daphnis",
        id: 635,
        mean_radius: Some(3.8),
    },
    OriginData {
        name: "Aegir",
        id: 636,
        mean_radius: None,
    },
    OriginData {
        name: "Bebhionn",
        id: 637,
        mean_radius: None,
    },
    OriginData {
        name: "Bergelmir",
        id: 638,
        mean_radius: None,
    },
    OriginData {
        name: "Bestla",
        id: 639,
        mean_radius: None,
    },
    OriginData {
        name: "Farbauti",
        id: 640,
        mean_radius: None,
    },
    OriginData {
        name: "Fenrir",
        id: 641,
        mean_radius: None,
    },
    OriginData {
        name: "Fornjot",
        id: 642,
        mean_radius: None,
    },
    OriginData {
        name: "Hati",
        id: 643,
        mean_radius: None,
    },
    OriginData {
        name: "Hyrrokkin",
        id: 644,
        mean_radius: None,
    },
    OriginData {
        name: "Kari",
        id: 645,
        mean_radius: None,
    },
    OriginData {
        name: "Loge",
        id: 646,
        mean_radius: None,
    },
    OriginData {
        name: "Skoll",
        id: 647,
        mean_radius: None,
    },
    OriginData {
        name: "Surtur",
        id: 648,
        mean_radius: None,
    },
    OriginData {
        name: "Anthe",
        id: 649,
        mean_radius: Some(0.5),
    },
    OriginData {
        name: "Jarnsaxa",
        id: 650,
        mean_radius: None,
    },
    OriginData {
        name: "Greip",
        id: 651,
        mean_radius: None,
    },
    OriginData {
        name: "Tarqeq",
        id: 652,
        mean_radius: None,
    },
    OriginData {
        name: "Aegaeon",
        id: 653,
        mean_radius: Some(0.33),
    },
    OriginData {
        name: "Ariel",
        id: 701,
        mean_radius: Some(578.9),
    },
    OriginData {
        name: "Umbriel",
        id: 702,
        mean_radius: Some(584.7),
    },
    OriginData {
        name: "Titania",
        id: 703,
        mean_radius: Some(788.9),
    },
    OriginData {
        name: "Oberon",
        id: 704,
        mean_radius: Some(761.4),
    },
    OriginData {
        name: "Miranda",
        id: 705,
        mean_radius: Some(235.8),
    },
    OriginData {
        name: "Cordelia",
        id: 706,
        mean_radius: Some(13.0),
    },
    OriginData {
        name: "Ophelia",
        id: 707,
        mean_radius: Some(15.0),
    },
    OriginData {
        name: "Bianca",
        id: 708,
        mean_radius: Some(21.0),
    },
    OriginData {
        name: "Cressida",
        id: 709,
        mean_radius: Some(31.0),
    },
    OriginData {
        name: "Desdemona",
        id: 710,
        mean_radius: Some(27.0),
    },
    OriginData {
        name: "Juliet",
        id: 711,
        mean_radius: Some(42.0),
    },
    OriginData {
        name: "Portia",
        id: 712,
        mean_radius: Some(54.0),
    },
    OriginData {
        name: "Rosalind",
        id: 713,
        mean_radius: Some(27.0),
    },
    OriginData {
        name: "Belinda",
        id: 714,
        mean_radius: Some(33.0),
    },
    OriginData {
        name: "Puck",
        id: 715,
        mean_radius: Some(77.0),
    },
    OriginData {
        name: "Caliban",
        id: 716,
        mean_radius: None,
    },
    OriginData {
        name: "Sycorax",
        id: 717,
        mean_radius: None,
    },
    OriginData {
        name: "Prospero",
        id: 718,
        mean_radius: None,
    },
    OriginData {
        name: "Setebos",
        id: 719,
        mean_radius: None,
    },
    OriginData {
        name: "Stephano",
        id: 720,
        mean_radius: None,
    },
    OriginData {
        name: "Trinculo",
        id: 721,
        mean_radius: None,
    },
    OriginData {
        name: "Francisco",
        id: 722,
        mean_radius: None,
    },
    OriginData {
        name: "Margaret",
        id: 723,
        mean_radius: None,
    },
    OriginData {
        name: "Ferdinand",
        id: 724,
        mean_radius: None,
    },
    OriginData {
        name: "Perdita",
        id: 725,
        mean_radius: None,
    },
    OriginData {
        name: "Mab",
        id: 726,
        mean_radius: None,
    },
    OriginData {
        name: "Cupid",
        id: 727,
        mean_radius: None,
    },
    OriginData {
        name: "Triton",
        id: 801,
        mean_radius: Some(1352.6),
    },
    OriginData {
        name: "Nereid",
        id: 802,
        mean_radius: Some(170.0),
    },
    OriginData {
        name: "Naiad",
        id: 803,
        mean_radius: Some(29.0),
    },
    OriginData {
        name: "Thalassa",
        id: 804,
        mean_radius: Some(40.0),
    },
    OriginData {
        name: "Despina",
        id: 805,
        mean_radius: Some(74.0),
    },
    OriginData {
        name: "Galatea",
        id: 806,
        mean_radius: Some(79.0),
    },
    OriginData {
        name: "Larissa",
        id: 807,
        mean_radius: Some(96.0),
    },
    OriginData {
        name: "Proteus",
        id: 808,
        mean_radius: Some(208.0),
    },
    OriginData {
        name: "Halimede",
        id: 809,
        mean_radius: None,
    },
    OriginData {
        name: "Psamathe",
        id: 810,
        mean_radius: None,
    },
    OriginData {
        name: "Sao",
        id: 811,
        mean_radius: None,
    },
    OriginData {
        name: "Laomedeia",
        id: 812,
        mean_radius: None,
    },
    OriginData {
        name: "Neso",
        id: 813,
        mean_radius: None,
    },
    OriginData {
        name: "Charon",
        id: 901,
        mean_radius: Some(606.0),
    },
    OriginData {
        name: "Nix",
        id: 902,
        mean_radius: None,
    },
    OriginData {
        name: "Hydra",
        id: 903,
        mean_radius: None,
    },
    OriginData {
        name: "Kerberos",
        id: 904,
        mean_radius: None,
    },
    OriginData {
        name: "Styx",
        id: 905,
        mean_radius: None,
    },
    // Minor bodies.
    OriginData {
        name: "Gaspra",
        id: 9511010,
        mean_radius: Some(6.1),
    },
    OriginData {
        name: "Ida",
        id: 2431010,
        mean_radius: Some(15.65),
    },
    OriginData {
        name: "Dactyl",
        id: 2431011,
        mean_radius: None,
    },
    OriginData {
        name: "Ceres",
        id: 2000001,
        mean_radius: Some(470.0),
    },
    OriginData {
        name: "Pallas",
        id: 2000002,
        mean_radius: None,
    },
    OriginData {
        name: "Vesta",
        id: 2000004,
        mean_radius: None,
    },
    OriginData {
        name: "Psyche",
        id: 2000016,
        mean_radius: Some(113.0),
    },
    OriginData {
        name: "Lutetia",
        id: 2000021,
        mean_radius: Some(52.5),
    },
    OriginData {
        name: "Kleopatra",
        id: 2000216,
        mean_radius: None,
    },
    OriginData {
        name: "Eros",
        id: 2000433,
        mean_radius: Some(8.45),
    },
    OriginData {
        name: "Davida",
        id: 2000511,
        mean_radius: Some(150.0),
    },
    OriginData {
        name: "Mathilde",
        id: 2000253,
        mean_radius: Some(26.5),
    },
    OriginData {
        name: "Steins",
        id: 2002867,
        mean_radius: Some(2.7),
    },
    OriginData {
        name: "Braille",
        id: 2009969,
        mean_radius: None,
    },
    OriginData {
        name: "Wilson-Harrington",
        id: 2004015,
        mean_radius: None,
    },
    OriginData {
        name: "Toutatis",
        id: 2004179,
        mean_radius: None,
    },
    OriginData {
        name: "Itokawa",
        id: 2025143,
        mean_radius: None,
    },
    OriginData {
        name: "Bennu",
        id: 2101955,
        mean_radius: None,
    },
];

fn ident(name: &str) -> Ident {
    format_ident!("{}", name.replace([' ', '-'], ""))
}

fn ident_uppercase(name: &str) -> Ident {
    format_ident!("{}", name.replace([' ', '-'], "_").to_uppercase())
}

fn get_array_as_radians(kernel: &Kernel, key: &str) -> Option<Vec<f64>> {
    kernel
        .get_double_array(key)
        .map(|array| array.iter().map(|v| v.to_radians()).collect())
}

fn unpair(vec: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut a: Vec<f64> = Vec::with_capacity(vec.len() / 2);
    let mut b: Vec<f64> = Vec::with_capacity(vec.len() / 2);
    for (i, coefficient) in vec.iter().enumerate() {
        if i % 2 == 0 {
            a.push(*coefficient);
        } else {
            b.push(*coefficient);
        }
    }
    (a, b)
}

pub fn generate_bodies(path: &Path, pck: &Kernel, gm: &Kernel) {
    let mut code = quote! {
        use crate::DynOrigin;
        use crate::Elements;
        use crate::MeanRadius;
        use crate::NaifId;
        use crate::Origin;
        use crate::PointMass;
        use crate::Radii;
        use crate::RotationalElement;
        use crate::RotationalElementType;
        use crate::RotationalElements;
        use crate::Spheroid;
        use crate::TriaxialEllipsoid;
        use crate::TryMeanRadius;
        use crate::TryPointMass;
        use crate::TryRotationalElements;
        use crate::TrySpheroid;
        use crate::TryTriaxialEllipsoid;
        use crate::UndefinedOriginPropertyError;
        use alloc::format;
        use alloc::string::{String, ToString};
        use core::fmt::Display;
        use core::fmt::Formatter;
        use lox_core::elements::GravitationalParameter;
        use lox_core::units::Distance;
    };

    let mut point_mass_match_arms = quote! {};
    let mut mean_radius_match_arms = quote! {};
    let mut polar_radius_match_arms = quote! {};
    let mut equatorial_radius_match_arms = quote! {};
    let mut ellipsoid_match_arms = quote! {};

    let mut rotational_elements_match_arms = quote! {};
    let mut rotational_element_rates_match_arms = quote! {};

    let mut tests = quote! {};

    for OriginData {
        name,
        id,
        mean_radius,
    } in ORIGINS
    {
        let ident = ident(name);
        let ident_upper = ident_uppercase(name);

        let doc_string = format!("{name} (NAIF ID: {id}).");
        code.extend(quote! {

            #[doc = #doc_string]
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            #[cfg_attr(feature = "serde", serde(into = "&'static str", try_from = "String"))]
            pub struct #ident;

            impl Origin for #ident {
                fn id(&self) -> NaifId {
                    NaifId(#id)
                }

                fn name(&self) -> &'static str {
                    #name
                }
            }

            impl Display for #ident {
                fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}", self.name())
                }
            }

            impl From<#ident> for &'static str {
                fn from(_: #ident) -> Self {
                    #name
                }
            }

            impl TryFrom<String> for #ident {
                type Error = String;
                fn try_from(s: String) -> Result<Self, Self::Error> {
                    if s == #name { Ok(#ident) } else { Err(format!("expected \"{}\", got \"{}\"", #name, s)) }
                }
            }

            impl From<#ident> for DynOrigin {
                fn from(_: #ident) -> Self {
                    DynOrigin::#ident
                }
            }
        });

        let origin_test_name = format_ident!("test_origin_{}", id as u32);

        tests.extend(quote! {
            #[test]
            fn #origin_test_name() {
                assert_eq!(#ident.id().0, #id);
                assert_eq!(#ident.to_string(), #name);
                assert_eq!(DynOrigin::from(#ident), DynOrigin::#ident);
            }
        });

        // PointMass
        let key = if id == 0 {
            "BODY10_GM".to_string()
        } else {
            format!("BODY{id}_GM")
        };

        let point_mass_test_name = format_ident!("test_point_mass_{}", id as u32);

        if let Some(gm) = gm.get_double_array(&key) {
            let gm = gm.first().unwrap();
            code.extend(quote! {
                impl PointMass for #ident {
                    fn gravitational_parameter(&self) -> GravitationalParameter {
                        GravitationalParameter::km3_per_s2(#gm)
                    }
                }
            });

            point_mass_match_arms.extend(quote! {
                DynOrigin::#ident => Ok(GravitationalParameter::km3_per_s2(#gm)),
            });

            tests.extend(quote! {
                #[test]
                fn #point_mass_test_name() {
                    assert_eq!(#ident.gravitational_parameter(), GravitationalParameter::km3_per_s2(#gm));
                    assert_eq!(DynOrigin::#ident.try_gravitational_parameter(), Ok(GravitationalParameter::km3_per_s2(#gm)));
                }
            });
        } else {
            tests.extend(quote! {
                #[test]
                fn #point_mass_test_name() {
                    assert!(DynOrigin::#ident.try_gravitational_parameter().is_err());
                }
            });
        };

        // Barycenters do not have cartographic properties
        if id < 10 {
            continue;
        }

        let mean_radius_test_name = format_ident!("test_mean_radius_{}", id as u32);

        if let Some(mean_radius) = mean_radius {
            code.extend(quote! {
                impl MeanRadius for #ident {
                    fn mean_radius(&self) -> Distance {
                        Distance::kilometers(#mean_radius)
                    }
                }
            });

            mean_radius_match_arms.extend(quote! {
                DynOrigin::#ident => Ok(Distance::kilometers(#mean_radius)),
            });

            tests.extend(quote! {
                #[test]
                fn #mean_radius_test_name() {
                    assert_eq!(#ident.mean_radius(), Distance::kilometers(#mean_radius));
                    assert_eq!(DynOrigin::#ident.try_mean_radius(), Ok(Distance::kilometers(#mean_radius)));
                }
            });
        } else {
            tests.extend(quote! {
                #[test]
                fn #mean_radius_test_name() {
                    assert!(DynOrigin::#ident.try_mean_radius().is_err());
                }
            });
        }

        // TriaxialEllipsoid / Spheroid
        let key = format!("BODY{id}_RADII");

        let triaxial_test_name = format_ident!("test_tri_axial_{}", id as u32);
        let spheroid_test_name = format_ident!("test_spheroid_{}", id as u32);

        if let Some(radii) = pck.get_double_array(&key) {
            let equatorial = radii[0];
            let along_orbit = radii[1];
            let polar = radii[2];

            code.extend(quote! {
                impl TriaxialEllipsoid for #ident {
                    fn radii(&self) -> Radii {
                        (Distance::kilometers(#equatorial), Distance::kilometers(#along_orbit), Distance::kilometers(#polar))
                    }
                }
            });

            if radii[0] == radii[1] {
                code.extend(quote! {
                    impl Spheroid for #ident {}
                });

                polar_radius_match_arms.extend(quote! {
                    DynOrigin::#ident => Ok(Distance::kilometers(#polar)),
                });
                equatorial_radius_match_arms.extend(quote! {
                    DynOrigin::#ident => Ok(Distance::kilometers(#equatorial)),
                });

                tests.extend(quote! {
                    #[test]
                    fn #spheroid_test_name() {
                        assert_eq!(#ident.polar_radius(), Distance::kilometers(#polar));
                        assert_eq!(DynOrigin::#ident.try_polar_radius(), Ok(Distance::kilometers(#polar)));
                        assert_eq!(#ident.equatorial_radius(), Distance::kilometers(#equatorial));
                        assert_eq!(DynOrigin::#ident.try_equatorial_radius(), Ok(Distance::kilometers(#equatorial)));
                    }

                });
            } else {
                tests.extend(quote! {
                    #[test]
                    fn #spheroid_test_name() {
                        assert!(DynOrigin::#ident.try_polar_radius().is_err());
                        assert!(DynOrigin::#ident.try_equatorial_radius().is_err());
                    }
                });
            }

            ellipsoid_match_arms.extend(quote! {
                DynOrigin::#ident => Ok((Distance::kilometers(#equatorial), Distance::kilometers(#along_orbit), Distance::kilometers(#polar))),
            });

            tests.extend(quote! {
                #[test]
                fn #triaxial_test_name() {
                    assert_eq!(#ident.radii(), (Distance::kilometers(#equatorial), Distance::kilometers(#along_orbit), Distance::kilometers(#polar)));
                    assert_eq!(DynOrigin::#ident.try_radii(), Ok((Distance::kilometers(#equatorial), Distance::kilometers(#along_orbit), Distance::kilometers(#polar))));
                }
            });
        } else {
            tests.extend(quote! {
                #[test]
                fn #triaxial_test_name() {
                    assert!(DynOrigin::#ident.try_radii().is_err());
                }
                #[test]
                fn #spheroid_test_name() {
                    assert!(DynOrigin::#ident.try_polar_radius().is_err());
                    assert!(DynOrigin::#ident.try_equatorial_radius().is_err());
                }
            });
        }

        let ra_key = format!("BODY{id}_POLE_RA");
        let dec_key = format!("BODY{id}_POLE_DEC");
        let pm_key = format!("BODY{id}_PM");
        let nut_prec_ra_key = format!("BODY{id}_NUT_PREC_RA");
        let nut_prec_dec_key = format!("BODY{id}_NUT_PREC_DEC");
        let nut_prec_pm_key = format!("BODY{id}_NUT_PREC_PM");

        let nut_prec_id = id / 100;
        let nut_prec_key = format!("BODY{}_NUT_PREC_ANGLES", nut_prec_id);

        if let (Some(ra), Some(dec), Some(pm)) = (
            get_array_as_radians(pck, &ra_key),
            get_array_as_radians(pck, &dec_key),
            get_array_as_radians(pck, &pm_key),
        ) {
            let (theta0, theta1): (Vec<f64>, Vec<f64>) = get_array_as_radians(pck, &nut_prec_key)
                .as_ref()
                .map(|nut_prec| unpair(nut_prec))
                .unwrap_or_default();

            let ra1 = ra[0];
            let ra2 = ra[1];
            let ra3 = ra.get(2).copied().unwrap_or_default();
            let nut_prec_ra = get_array_as_radians(pck, &nut_prec_ra_key).unwrap_or_default();
            let n = nut_prec_ra.len();
            let theta0_ra = &theta0[0..n];
            let theta1_ra = &theta1[0..n];

            let ra_const_ident = format_ident!("RIGHT_ASCENSION_{}", ident_upper);
            let ra_const = quote! {
                const #ra_const_ident: RotationalElement<#n> = RotationalElement {
                    typ: RotationalElementType::RightAscension,
                    c0: #ra1,
                    c1: #ra2,
                    c2: #ra3,
                    c: [#(#nut_prec_ra),*],
                    theta0: [#(#theta0_ra),*],
                    theta1: [#(#theta1_ra),*],
                };
            };
            let ra = quote! {
                #ra_const_ident.angle(t)
            };
            let ra_dot = quote! {
                #ra_const_ident.angle_dot(t)
            };

            let dec1 = dec[0];
            let dec2 = dec[1];
            let dec3 = dec.get(2).copied().unwrap_or_default();
            let nut_prec_dec = get_array_as_radians(pck, &nut_prec_dec_key).unwrap_or_default();
            let n = nut_prec_dec.len();
            let theta0_dec = &theta0[0..n];
            let theta1_dec = &theta1[0..n];

            let dec_const_ident = format_ident!("DECLINATION_{}", ident_upper);
            let dec_const = quote! {
                const #dec_const_ident: RotationalElement<#n> = RotationalElement {
                    typ: RotationalElementType::Declination,
                    c0: #dec1,
                    c1: #dec2,
                    c2: #dec3,
                    c: [#(#nut_prec_dec),*],
                    theta0: [#(#theta0_dec),*],
                    theta1: [#(#theta1_dec),*],
                };
            };
            let dec = quote! {
                #dec_const_ident.angle(t)
            };
            let dec_dot = quote! {
                #dec_const_ident.angle_dot(t)
            };

            let pm1 = pm[0];
            let pm2 = pm[1];
            let pm3 = pm.get(2).copied().unwrap_or_default();
            let nut_prec_pm = get_array_as_radians(pck, &nut_prec_pm_key).unwrap_or_default();
            let n = nut_prec_pm.len();
            let theta0_pm = &theta0[0..n];
            let theta1_pm = &theta1[0..n];

            let pm_const_ident = format_ident!("ROTATION_{}", ident_upper);
            let pm_const = quote! {
                const #pm_const_ident: RotationalElement<#n> = RotationalElement {
                    typ: RotationalElementType::Rotation,
                    c0: #pm1,
                    c1: #pm2,
                    c2: #pm3,
                    c: [#(#nut_prec_pm),*],
                    theta0: [#(#theta0_pm),*],
                    theta1: [#(#theta1_pm),*],
                };
            };
            let pm = quote! {
                #pm_const_ident.angle(t)
            };
            let pm_dot = quote! {
                #pm_const_ident.angle_dot(t)
            };

            code.extend(quote! {
                #ra_const
                #dec_const
                #pm_const

                impl RotationalElements for #ident {
                    fn rotational_elements(&self, t: f64) -> Elements {
                        (#ra, #dec, #pm)
                    }
                    fn rotational_element_rates(&self, t: f64) -> Elements {
                        (#ra_dot, #dec_dot, #pm_dot)
                    }
                }
            });

            rotational_elements_match_arms.extend(quote! {
                DynOrigin::#ident => Ok(#ident.rotational_elements(t)),
            });

            rotational_element_rates_match_arms.extend(quote! {
                DynOrigin::#ident => Ok(#ident.rotational_element_rates(t)),
            });
        }
    }

    code.extend(quote! {
        impl TryPointMass for DynOrigin {
            fn try_gravitational_parameter(&self) -> Result<GravitationalParameter, UndefinedOriginPropertyError> {
                match self {
                    #point_mass_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "gravitational parameter".to_string(),
                        }
                    ),
                }
            }
        }
        impl TryMeanRadius for DynOrigin {
            fn try_mean_radius(&self) -> Result<Distance, UndefinedOriginPropertyError> {
                match self {
                    #mean_radius_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "mean radius".to_string(),
                        }
                    ),
                }
            }
        }
        impl TryTriaxialEllipsoid for DynOrigin {
            fn try_radii(&self) -> Result<Radii, UndefinedOriginPropertyError> {
                match self {
                    #ellipsoid_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "radii".to_string(),
                        }
                    ),
                }
            }
        }
        impl TrySpheroid for DynOrigin {
            fn try_polar_radius(&self) -> Result<Distance, UndefinedOriginPropertyError> {
                match self {
                    #polar_radius_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "polar radius".to_string(),
                        }
                    ),
                }
            }
            fn try_equatorial_radius(&self) -> Result<Distance, UndefinedOriginPropertyError> {
                match self {
                    #equatorial_radius_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "equatorial radius".to_string(),
                        }
                    ),
                }
            }
        }
        impl TryRotationalElements for DynOrigin {
            fn try_rotational_elements(&self, t: f64)
                -> Result<Elements, UndefinedOriginPropertyError> {
                match self {
                    #rotational_elements_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "rotational elements".to_string(),
                        }
                    ),
                }
            }

            fn try_rotational_element_rates(
                &self,
                t: f64,
            ) -> Result<Elements, UndefinedOriginPropertyError> {
                match self {
                    #rotational_element_rates_match_arms
                    _ => Err(
                        UndefinedOriginPropertyError {
                            origin: self.to_string(),
                            prop: "rotational element rates".to_string(),
                        }
                    ),
                }
            }
        }
    });

    code.extend(quote! {
        #[cfg(test)]
        #[allow(clippy::approx_constant)] // at least one parsed constant is close to TAU
        mod tests {
            use alloc::string::ToString;
            use crate::*;

            #tests
        }
    });

    write_file(path, "generated.rs", code)
}
