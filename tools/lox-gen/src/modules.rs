/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use proc_macro2::Ident;
use quote::quote;

use lox_io::spice::Kernel;

use crate::bodies::BodyDef;
use crate::generators::{
    BaseGenerator, BodyGenerator, Generator, PointMassGenerator, RotationalElementsGenerator,
    SpheroidGenerator, TriAxialGenerator,
};

pub fn generate_modules(dir: &Path, pck: &Kernel, gm: &Kernel) {
    let modules = vec![
        Module {
            name: "sun",
            bodies: vec![BodyDef {
                name: "Sun",
                id: 10,
                body_trait: None,
            }],
            generators: vec![
                Box::new(BaseGenerator),
                Box::new(BodyGenerator),
                Box::new(SpheroidGenerator { pck: pck.clone() }),
                Box::new(PointMassGenerator { gm: gm.clone() }),
                Box::new(RotationalElementsGenerator { pck: pck.clone() }),
            ],
        },
        Module {
            name: "barycenters",
            bodies: vec![
                BodyDef {
                    name: "Solar System Barycenter",
                    id: 0,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Mercury Barycenter",
                    id: 1,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Venus Barycenter",
                    id: 2,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Earth Barycenter",
                    id: 3,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Mars Barycenter",
                    id: 4,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Jupiter Barycenter",
                    id: 5,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Saturn Barycenter",
                    id: 6,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Uranus Barycenter",
                    id: 7,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Neptune Barycenter",
                    id: 8,
                    body_trait: Some("Barycenter"),
                },
                BodyDef {
                    name: "Pluto Barycenter",
                    id: 9,
                    body_trait: Some("Barycenter"),
                },
            ],
            generators: vec![
                Box::new(BaseGenerator),
                Box::new(BodyGenerator),
                Box::new(PointMassGenerator { gm: gm.clone() }),
            ],
        },
        Module {
            name: "planets",
            bodies: vec![
                BodyDef {
                    name: "Mercury",
                    id: 199,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Venus",
                    id: 299,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Earth",
                    id: 399,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Mars",
                    id: 499,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Jupiter",
                    id: 599,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Saturn",
                    id: 699,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Uranus",
                    id: 799,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Neptune",
                    id: 899,
                    body_trait: Some("Planet"),
                },
                BodyDef {
                    name: "Pluto",
                    id: 999,
                    body_trait: Some("Planet"),
                },
            ],
            generators: vec![
                Box::new(BaseGenerator),
                Box::new(BodyGenerator),
                Box::new(SpheroidGenerator { pck: pck.clone() }),
                Box::new(PointMassGenerator { gm: gm.clone() }),
                Box::new(RotationalElementsGenerator { pck: pck.clone() }),
            ],
        },
        Module {
            name: "satellites",
            bodies: vec![
                BodyDef {
                    name: "Moon",
                    id: 301,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Phobos",
                    id: 401,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Deimos",
                    id: 402,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Io",
                    id: 501,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Europa",
                    id: 502,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Ganymede",
                    id: 503,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Callisto",
                    id: 504,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Amalthea",
                    id: 505,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Himalia",
                    id: 506,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Elara",
                    id: 507,
                    body_trait: None,
                },
                BodyDef {
                    name: "Pasiphae",
                    id: 508,
                    body_trait: None,
                },
                BodyDef {
                    name: "Sinope",
                    id: 509,
                    body_trait: None,
                },
                BodyDef {
                    name: "Lysithea",
                    id: 510,
                    body_trait: None,
                },
                BodyDef {
                    name: "Carme",
                    id: 511,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ananke",
                    id: 512,
                    body_trait: None,
                },
                BodyDef {
                    name: "Leda",
                    id: 513,
                    body_trait: None,
                },
                BodyDef {
                    name: "Thebe",
                    id: 514,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Adrastea",
                    id: 515,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Metis",
                    id: 516,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Callirrhoe",
                    id: 517,
                    body_trait: None,
                },
                BodyDef {
                    name: "Themisto",
                    id: 518,
                    body_trait: None,
                },
                BodyDef {
                    name: "Magaclite",
                    id: 519,
                    body_trait: None,
                },
                BodyDef {
                    name: "Taygete",
                    id: 520,
                    body_trait: None,
                },
                BodyDef {
                    name: "Chaldene",
                    id: 521,
                    body_trait: None,
                },
                BodyDef {
                    name: "Harpalyke",
                    id: 522,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kalyke",
                    id: 523,
                    body_trait: None,
                },
                BodyDef {
                    name: "Iocaste",
                    id: 524,
                    body_trait: None,
                },
                BodyDef {
                    name: "Erinome",
                    id: 525,
                    body_trait: None,
                },
                BodyDef {
                    name: "Isonoe",
                    id: 526,
                    body_trait: None,
                },
                BodyDef {
                    name: "Praxidike",
                    id: 527,
                    body_trait: None,
                },
                BodyDef {
                    name: "Autonoe",
                    id: 528,
                    body_trait: None,
                },
                BodyDef {
                    name: "Thyone",
                    id: 529,
                    body_trait: None,
                },
                BodyDef {
                    name: "Hermippe",
                    id: 530,
                    body_trait: None,
                },
                BodyDef {
                    name: "Aitne",
                    id: 531,
                    body_trait: None,
                },
                BodyDef {
                    name: "Eurydome",
                    id: 532,
                    body_trait: None,
                },
                BodyDef {
                    name: "Euanthe",
                    id: 533,
                    body_trait: None,
                },
                BodyDef {
                    name: "Euporie",
                    id: 534,
                    body_trait: None,
                },
                BodyDef {
                    name: "Orthosie",
                    id: 535,
                    body_trait: None,
                },
                BodyDef {
                    name: "Sponde",
                    id: 536,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kale",
                    id: 537,
                    body_trait: None,
                },
                BodyDef {
                    name: "Pasithee",
                    id: 538,
                    body_trait: None,
                },
                BodyDef {
                    name: "Hegemone",
                    id: 539,
                    body_trait: None,
                },
                BodyDef {
                    name: "Mneme",
                    id: 540,
                    body_trait: None,
                },
                BodyDef {
                    name: "Aoede",
                    id: 541,
                    body_trait: None,
                },
                BodyDef {
                    name: "Thelxinoe",
                    id: 542,
                    body_trait: None,
                },
                BodyDef {
                    name: "Arche",
                    id: 543,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kallichore",
                    id: 544,
                    body_trait: None,
                },
                BodyDef {
                    name: "Helike",
                    id: 545,
                    body_trait: None,
                },
                BodyDef {
                    name: "Carpo",
                    id: 546,
                    body_trait: None,
                },
                BodyDef {
                    name: "Eukelade",
                    id: 547,
                    body_trait: None,
                },
                BodyDef {
                    name: "Cyllene",
                    id: 548,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kore",
                    id: 549,
                    body_trait: None,
                },
                BodyDef {
                    name: "Herse",
                    id: 550,
                    body_trait: None,
                },
                BodyDef {
                    name: "Dia",
                    id: 553,
                    body_trait: None,
                },
                BodyDef {
                    name: "Mimas",
                    id: 601,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Enceladus",
                    id: 602,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Tethys",
                    id: 603,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Dione",
                    id: 604,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Rhea",
                    id: 605,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Titan",
                    id: 606,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Hyperion",
                    id: 607,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Iapetus",
                    id: 608,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Phoebe",
                    id: 609,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Janus",
                    id: 610,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Epimetheus",
                    id: 611,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Helene",
                    id: 612,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Telesto",
                    id: 613,
                    body_trait: None,
                },
                BodyDef {
                    name: "Calypso",
                    id: 614,
                    body_trait: None,
                },
                BodyDef {
                    name: "Atlas",
                    id: 615,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Prometheus",
                    id: 616,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Pandora",
                    id: 617,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Pan",
                    id: 618,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ymir",
                    id: 619,
                    body_trait: None,
                },
                BodyDef {
                    name: "Paaliaq",
                    id: 620,
                    body_trait: None,
                },
                BodyDef {
                    name: "Tarvos",
                    id: 621,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ijiraq",
                    id: 622,
                    body_trait: None,
                },
                BodyDef {
                    name: "Suttungr",
                    id: 623,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kiviuq",
                    id: 624,
                    body_trait: None,
                },
                BodyDef {
                    name: "Mundilfari",
                    id: 625,
                    body_trait: None,
                },
                BodyDef {
                    name: "Albiorix",
                    id: 626,
                    body_trait: None,
                },
                BodyDef {
                    name: "Skathi",
                    id: 627,
                    body_trait: None,
                },
                BodyDef {
                    name: "Erriapus",
                    id: 628,
                    body_trait: None,
                },
                BodyDef {
                    name: "Siarnaq",
                    id: 629,
                    body_trait: None,
                },
                BodyDef {
                    name: "Thrymr",
                    id: 630,
                    body_trait: None,
                },
                BodyDef {
                    name: "Narvi",
                    id: 631,
                    body_trait: None,
                },
                BodyDef {
                    name: "Methone",
                    id: 632,
                    body_trait: None,
                },
                BodyDef {
                    name: "Pallene",
                    id: 633,
                    body_trait: None,
                },
                BodyDef {
                    name: "Polydeuces",
                    id: 634,
                    body_trait: None,
                },
                BodyDef {
                    name: "Daphnis",
                    id: 635,
                    body_trait: None,
                },
                BodyDef {
                    name: "Aegir",
                    id: 636,
                    body_trait: None,
                },
                BodyDef {
                    name: "Bebhionn",
                    id: 637,
                    body_trait: None,
                },
                BodyDef {
                    name: "Bergelmir",
                    id: 638,
                    body_trait: None,
                },
                BodyDef {
                    name: "Bestla",
                    id: 639,
                    body_trait: None,
                },
                BodyDef {
                    name: "Farbauti",
                    id: 640,
                    body_trait: None,
                },
                BodyDef {
                    name: "Fenrir",
                    id: 641,
                    body_trait: None,
                },
                BodyDef {
                    name: "Fornjot",
                    id: 642,
                    body_trait: None,
                },
                BodyDef {
                    name: "Hati",
                    id: 643,
                    body_trait: None,
                },
                BodyDef {
                    name: "Hyrrokkin",
                    id: 644,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kari",
                    id: 645,
                    body_trait: None,
                },
                BodyDef {
                    name: "Loge",
                    id: 646,
                    body_trait: None,
                },
                BodyDef {
                    name: "Skoll",
                    id: 647,
                    body_trait: None,
                },
                BodyDef {
                    name: "Surtur",
                    id: 648,
                    body_trait: None,
                },
                BodyDef {
                    name: "Anthe",
                    id: 649,
                    body_trait: None,
                },
                BodyDef {
                    name: "Jarnsaxa",
                    id: 650,
                    body_trait: None,
                },
                BodyDef {
                    name: "Greip",
                    id: 651,
                    body_trait: None,
                },
                BodyDef {
                    name: "Tarqeq",
                    id: 652,
                    body_trait: None,
                },
                BodyDef {
                    name: "Aegaeon",
                    id: 653,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ariel",
                    id: 701,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Umbriel",
                    id: 702,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Titania",
                    id: 703,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Oberon",
                    id: 704,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Miranda",
                    id: 705,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Cordelia",
                    id: 706,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ophelia",
                    id: 707,
                    body_trait: None,
                },
                BodyDef {
                    name: "Bianca",
                    id: 708,
                    body_trait: None,
                },
                BodyDef {
                    name: "Cressida",
                    id: 709,
                    body_trait: None,
                },
                BodyDef {
                    name: "Desdemona",
                    id: 710,
                    body_trait: None,
                },
                BodyDef {
                    name: "Juliet",
                    id: 711,
                    body_trait: None,
                },
                BodyDef {
                    name: "Portia",
                    id: 712,
                    body_trait: None,
                },
                BodyDef {
                    name: "Rosalind",
                    id: 713,
                    body_trait: None,
                },
                BodyDef {
                    name: "Belinda",
                    id: 714,
                    body_trait: None,
                },
                BodyDef {
                    name: "Puck",
                    id: 715,
                    body_trait: None,
                },
                BodyDef {
                    name: "Caliban",
                    id: 716,
                    body_trait: None,
                },
                BodyDef {
                    name: "Sycorax",
                    id: 717,
                    body_trait: None,
                },
                BodyDef {
                    name: "Prospero",
                    id: 718,
                    body_trait: None,
                },
                BodyDef {
                    name: "Setebos",
                    id: 719,
                    body_trait: None,
                },
                BodyDef {
                    name: "Stephano",
                    id: 720,
                    body_trait: None,
                },
                BodyDef {
                    name: "Trinculo",
                    id: 721,
                    body_trait: None,
                },
                BodyDef {
                    name: "Francisco",
                    id: 722,
                    body_trait: None,
                },
                BodyDef {
                    name: "Margaret",
                    id: 723,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ferdinand",
                    id: 724,
                    body_trait: None,
                },
                BodyDef {
                    name: "Perdita",
                    id: 725,
                    body_trait: None,
                },
                BodyDef {
                    name: "Mab",
                    id: 726,
                    body_trait: None,
                },
                BodyDef {
                    name: "Cupid",
                    id: 727,
                    body_trait: None,
                },
                BodyDef {
                    name: "Triton",
                    id: 801,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Nereid",
                    id: 802,
                    body_trait: None,
                },
                BodyDef {
                    name: "Naiad",
                    id: 803,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Thalassa",
                    id: 804,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Despina",
                    id: 805,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Galatea",
                    id: 806,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Larissa",
                    id: 807,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Proteus",
                    id: 808,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Halimede",
                    id: 809,
                    body_trait: None,
                },
                BodyDef {
                    name: "Psamathe",
                    id: 810,
                    body_trait: None,
                },
                BodyDef {
                    name: "Sao",
                    id: 811,
                    body_trait: None,
                },
                BodyDef {
                    name: "Laomedeia",
                    id: 812,
                    body_trait: None,
                },
                BodyDef {
                    name: "Neso",
                    id: 813,
                    body_trait: None,
                },
                BodyDef {
                    name: "Charon",
                    id: 901,
                    body_trait: Some("Satellite"),
                },
                BodyDef {
                    name: "Nix",
                    id: 902,
                    body_trait: None,
                },
                BodyDef {
                    name: "Hydra",
                    id: 903,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kerberos",
                    id: 904,
                    body_trait: None,
                },
                BodyDef {
                    name: "Styx",
                    id: 905,
                    body_trait: None,
                },
            ],
            generators: vec![
                Box::new(BaseGenerator),
                Box::new(BodyGenerator),
                Box::new(TriAxialGenerator { pck: pck.clone() }),
                Box::new(PointMassGenerator { gm: gm.clone() }),
                Box::new(RotationalElementsGenerator { pck: pck.clone() }),
            ],
        },
        Module {
            name: "minor",
            bodies: vec![
                BodyDef {
                    name: "Gaspra",
                    id: 9511010,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ida",
                    id: 2431010,
                    body_trait: None,
                },
                BodyDef {
                    name: "Dactyl",
                    id: 2431011,
                    body_trait: None,
                },
                BodyDef {
                    name: "Ceres",
                    id: 2000001,
                    body_trait: Some("MinorBody"),
                },
                BodyDef {
                    name: "Pallas",
                    id: 2000002,
                    body_trait: None,
                },
                BodyDef {
                    name: "Vesta",
                    id: 2000004,
                    body_trait: Some("MinorBody"),
                },
                BodyDef {
                    name: "Psyche",
                    id: 2000016,
                    body_trait: Some("MinorBody"),
                },
                BodyDef {
                    name: "Lutetia",
                    id: 2000021,
                    body_trait: None,
                },
                BodyDef {
                    name: "Kleopatra",
                    id: 2000216,
                    body_trait: None,
                },
                BodyDef {
                    name: "Eros",
                    id: 2000433,
                    body_trait: Some("MinorBody"),
                },
                BodyDef {
                    name: "Davida",
                    id: 2000511,
                    body_trait: Some("MinorBody"),
                },
                BodyDef {
                    name: "Mathilde",
                    id: 2000253,
                    body_trait: None,
                },
                BodyDef {
                    name: "Steins",
                    id: 2002867,
                    body_trait: None,
                },
                BodyDef {
                    name: "Braille",
                    id: 2009969,
                    body_trait: None,
                },
                BodyDef {
                    name: "Wilson-Harrington",
                    id: 2004015,
                    body_trait: None,
                },
                BodyDef {
                    name: "Toutatis",
                    id: 2004179,
                    body_trait: None,
                },
                BodyDef {
                    name: "Itokawa",
                    id: 2025143,
                    body_trait: None,
                },
                BodyDef {
                    name: "Bennu",
                    id: 2101955,
                    body_trait: None,
                },
            ],
            generators: vec![
                Box::new(BaseGenerator),
                Box::new(BodyGenerator),
                Box::new(TriAxialGenerator { pck: pck.clone() }),
                Box::new(PointMassGenerator { gm: gm.clone() }),
                Box::new(RotationalElementsGenerator { pck: pck.clone() }),
            ],
        },
    ];

    modules.iter().for_each(|module| module.write(dir));
}

const COPYRIGHT_NOTICE: &str = "/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */\n\n";

const AUTO_GENERATION_NOTICE: &str = "// Auto-generated by `lox-gen`. Do not edit!\n\n";

struct Module {
    name: &'static str,
    bodies: Vec<BodyDef>,
    generators: Vec<Box<dyn Generator>>,
}

impl Module {
    fn write(&self, dir: &Path) {
        let mut code = String::from(COPYRIGHT_NOTICE);
        code.push_str(AUTO_GENERATION_NOTICE);
        code.push_str(&self.generate_code());

        let out = dir.join(format!("{}.rs", self.name));
        fs::write(&out, code).expect("file should be writeable");

        Command::new("rustfmt")
            .args([out.to_str().unwrap()])
            .status()
            .expect("formatting should work");
    }

    fn generate_code(&self) -> String {
        let mut imports: HashSet<Ident> = HashSet::new();
        let mut code = quote!();
        let mut tests = quote!();

        self.bodies.iter().for_each(|body| {
            body.body_trait()
                .map(|body_trait| imports.insert(body_trait));
            self.generators.iter().for_each(|generator| {
                generator.generate_code(&mut code, &mut tests, body);
                imports.extend(generator.imports());
            });
        });

        let imports_iter = imports.iter();
        let module = quote! {
            use std::fmt::{Display, Formatter};

            use crate::{#(#imports_iter),*};

            #code

            #[cfg(test)]
            #[allow(clippy::approx_constant)] // at least one parsed constant is close to TAU
            mod tests {
                use super::*;

                #tests
            }
        };
        module.to_string()
    }
}
