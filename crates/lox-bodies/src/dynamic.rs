// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::{Earth, J2, NaifId, Origin, TryJ2, UndefinedOriginPropertyError};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

/// Error returned when an origin name is not recognized.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error("no origin with name `{0}` is known")]
pub struct UnknownOriginName(String);

/// Error returned when a NAIF ID does not correspond to a known origin.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error("no origin with NAIF ID `{0}` is known")]
pub struct UnknownOriginId(i32);

/// Enum representation of all known origins, for use in dynamic dispatch contexts.
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive, PartialOrd, Ord,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DynOrigin {
    /// Sun (NAIF ID: 10).
    Sun = 10,

    // Planets.
    /// Mercury (NAIF ID: 199).
    Mercury = 199,
    /// Venus (NAIF ID: 299).
    Venus = 299,
    /// Earth (NAIF ID: 399).
    #[default]
    Earth = 399,
    /// Mars (NAIF ID: 499).
    Mars = 499,
    /// Jupiter (NAIF ID: 599).
    Jupiter = 599,
    /// Saturn (NAIF ID: 699).
    Saturn = 699,
    /// Uranus (NAIF ID: 799).
    Uranus = 799,
    /// Neptune (NAIF ID: 899).
    Neptune = 899,
    /// Pluto (NAIF ID: 999).
    Pluto = 999,

    // Barycenters.
    /// Solar System Barycenter (NAIF ID: 0).
    SolarSystemBarycenter = 0,
    /// Mercury Barycenter (NAIF ID: 1).
    MercuryBarycenter = 1,
    /// Venus Barycenter (NAIF ID: 2).
    VenusBarycenter = 2,
    /// Earth Barycenter (NAIF ID: 3).
    EarthBarycenter = 3,
    /// Mars Barycenter (NAIF ID: 4).
    MarsBarycenter = 4,
    /// Jupiter Barycenter (NAIF ID: 5).
    JupiterBarycenter = 5,
    /// Saturn Barycenter (NAIF ID: 6).
    SaturnBarycenter = 6,
    /// Uranus Barycenter (NAIF ID: 7).
    UranusBarycenter = 7,
    /// Neptune Barycenter (NAIF ID: 8).
    NeptuneBarycenter = 8,
    /// Pluto Barycenter (NAIF ID: 9).
    PlutoBarycenter = 9,

    // Satellites.
    /// Moon (NAIF ID: 301).
    Moon = 301,
    /// Phobos (NAIF ID: 401).
    Phobos = 401,
    /// Deimos (NAIF ID: 402).
    Deimos = 402,
    /// Io (NAIF ID: 501).
    Io = 501,
    /// Europa (NAIF ID: 502).
    Europa = 502,
    /// Ganymede (NAIF ID: 503).
    Ganymede = 503,
    /// Callisto (NAIF ID: 504).
    Callisto = 504,
    /// Amalthea (NAIF ID: 505).
    Amalthea = 505,
    /// Himalia (NAIF ID: 506).
    Himalia = 506,
    /// Elara (NAIF ID: 507).
    Elara = 507,
    /// Pasiphae (NAIF ID: 508).
    Pasiphae = 508,
    /// Sinope (NAIF ID: 509).
    Sinope = 509,
    /// Lysithea (NAIF ID: 510).
    Lysithea = 510,
    /// Carme (NAIF ID: 511).
    Carme = 511,
    /// Ananke (NAIF ID: 512).
    Ananke = 512,
    /// Leda (NAIF ID: 513).
    Leda = 513,
    /// Thebe (NAIF ID: 514).
    Thebe = 514,
    /// Adrastea (NAIF ID: 515).
    Adrastea = 515,
    /// Metis (NAIF ID: 516).
    Metis = 516,
    /// Callirrhoe (NAIF ID: 517).
    Callirrhoe = 517,
    /// Themisto (NAIF ID: 518).
    Themisto = 518,
    /// Magaclite (NAIF ID: 519).
    Magaclite = 519,
    /// Taygete (NAIF ID: 520).
    Taygete = 520,
    /// Chaldene (NAIF ID: 521).
    Chaldene = 521,
    /// Harpalyke (NAIF ID: 522).
    Harpalyke = 522,
    /// Kalyke (NAIF ID: 523).
    Kalyke = 523,
    /// Iocaste (NAIF ID: 524).
    Iocaste = 524,
    /// Erinome (NAIF ID: 525).
    Erinome = 525,
    /// Isonoe (NAIF ID: 526).
    Isonoe = 526,
    /// Praxidike (NAIF ID: 527).
    Praxidike = 527,
    /// Autonoe (NAIF ID: 528).
    Autonoe = 528,
    /// Thyone (NAIF ID: 529).
    Thyone = 529,
    /// Hermippe (NAIF ID: 530).
    Hermippe = 530,
    /// Aitne (NAIF ID: 531).
    Aitne = 531,
    /// Eurydome (NAIF ID: 532).
    Eurydome = 532,
    /// Euanthe (NAIF ID: 533).
    Euanthe = 533,
    /// Euporie (NAIF ID: 534).
    Euporie = 534,
    /// Orthosie (NAIF ID: 535).
    Orthosie = 535,
    /// Sponde (NAIF ID: 536).
    Sponde = 536,
    /// Kale (NAIF ID: 537).
    Kale = 537,
    /// Pasithee (NAIF ID: 538).
    Pasithee = 538,
    /// Hegemone (NAIF ID: 539).
    Hegemone = 539,
    /// Mneme (NAIF ID: 540).
    Mneme = 540,
    /// Aoede (NAIF ID: 541).
    Aoede = 541,
    /// Thelxinoe (NAIF ID: 542).
    Thelxinoe = 542,
    /// Arche (NAIF ID: 543).
    Arche = 543,
    /// Kallichore (NAIF ID: 544).
    Kallichore = 544,
    /// Helike (NAIF ID: 545).
    Helike = 545,
    /// Carpo (NAIF ID: 546).
    Carpo = 546,
    /// Eukelade (NAIF ID: 547).
    Eukelade = 547,
    /// Cyllene (NAIF ID: 548).
    Cyllene = 548,
    /// Kore (NAIF ID: 549).
    Kore = 549,
    /// Herse (NAIF ID: 550).
    Herse = 550,
    /// Dia (NAIF ID: 553).
    Dia = 553,
    /// Mimas (NAIF ID: 601).
    Mimas = 601,
    /// Enceladus (NAIF ID: 602).
    Enceladus = 602,
    /// Tethys (NAIF ID: 603).
    Tethys = 603,
    /// Dione (NAIF ID: 604).
    Dione = 604,
    /// Rhea (NAIF ID: 605).
    Rhea = 605,
    /// Titan (NAIF ID: 606).
    Titan = 606,
    /// Hyperion (NAIF ID: 607).
    Hyperion = 607,
    /// Iapetus (NAIF ID: 608).
    Iapetus = 608,
    /// Phoebe (NAIF ID: 609).
    Phoebe = 609,
    /// Janus (NAIF ID: 610).
    Janus = 610,
    /// Epimetheus (NAIF ID: 611).
    Epimetheus = 611,
    /// Helene (NAIF ID: 612).
    Helene = 612,
    /// Telesto (NAIF ID: 613).
    Telesto = 613,
    /// Calypso (NAIF ID: 614).
    Calypso = 614,
    /// Atlas (NAIF ID: 615).
    Atlas = 615,
    /// Prometheus (NAIF ID: 616).
    Prometheus = 616,
    /// Pandora (NAIF ID: 617).
    Pandora = 617,
    /// Pan (NAIF ID: 618).
    Pan = 618,
    /// Ymir (NAIF ID: 619).
    Ymir = 619,
    /// Paaliaq (NAIF ID: 620).
    Paaliaq = 620,
    /// Tarvos (NAIF ID: 621).
    Tarvos = 621,
    /// Ijiraq (NAIF ID: 622).
    Ijiraq = 622,
    /// Suttungr (NAIF ID: 623).
    Suttungr = 623,
    /// Kiviuq (NAIF ID: 624).
    Kiviuq = 624,
    /// Mundilfari (NAIF ID: 625).
    Mundilfari = 625,
    /// Albiorix (NAIF ID: 626).
    Albiorix = 626,
    /// Skathi (NAIF ID: 627).
    Skathi = 627,
    /// Erriapus (NAIF ID: 628).
    Erriapus = 628,
    /// Siarnaq (NAIF ID: 629).
    Siarnaq = 629,
    /// Thrymr (NAIF ID: 630).
    Thrymr = 630,
    /// Narvi (NAIF ID: 631).
    Narvi = 631,
    /// Methone (NAIF ID: 632).
    Methone = 632,
    /// Pallene (NAIF ID: 633).
    Pallene = 633,
    /// Polydeuces (NAIF ID: 634).
    Polydeuces = 634,
    /// Daphnis (NAIF ID: 635).
    Daphnis = 635,
    /// Aegir (NAIF ID: 636).
    Aegir = 636,
    /// Bebhionn (NAIF ID: 637).
    Bebhionn = 637,
    /// Bergelmir (NAIF ID: 638).
    Bergelmir = 638,
    /// Bestla (NAIF ID: 639).
    Bestla = 639,
    /// Farbauti (NAIF ID: 640).
    Farbauti = 640,
    /// Fenrir (NAIF ID: 641).
    Fenrir = 641,
    /// Fornjot (NAIF ID: 642).
    Fornjot = 642,
    /// Hati (NAIF ID: 643).
    Hati = 643,
    /// Hyrrokkin (NAIF ID: 644).
    Hyrrokkin = 644,
    /// Kari (NAIF ID: 645).
    Kari = 645,
    /// Loge (NAIF ID: 646).
    Loge = 646,
    /// Skoll (NAIF ID: 647).
    Skoll = 647,
    /// Surtur (NAIF ID: 648).
    Surtur = 648,
    /// Anthe (NAIF ID: 649).
    Anthe = 649,
    /// Jarnsaxa (NAIF ID: 650).
    Jarnsaxa = 650,
    /// Greip (NAIF ID: 651).
    Greip = 651,
    /// Tarqeq (NAIF ID: 652).
    Tarqeq = 652,
    /// Aegaeon (NAIF ID: 653).
    Aegaeon = 653,
    /// Ariel (NAIF ID: 701).
    Ariel = 701,
    /// Umbriel (NAIF ID: 702).
    Umbriel = 702,
    /// Titania (NAIF ID: 703).
    Titania = 703,
    /// Oberon (NAIF ID: 704).
    Oberon = 704,
    /// Miranda (NAIF ID: 705).
    Miranda = 705,
    /// Cordelia (NAIF ID: 706).
    Cordelia = 706,
    /// Ophelia (NAIF ID: 707).
    Ophelia = 707,
    /// Bianca (NAIF ID: 708).
    Bianca = 708,
    /// Cressida (NAIF ID: 709).
    Cressida = 709,
    /// Desdemona (NAIF ID: 710).
    Desdemona = 710,
    /// Juliet (NAIF ID: 711).
    Juliet = 711,
    /// Portia (NAIF ID: 712).
    Portia = 712,
    /// Rosalind (NAIF ID: 713).
    Rosalind = 713,
    /// Belinda (NAIF ID: 714).
    Belinda = 714,
    /// Puck (NAIF ID: 715).
    Puck = 715,
    /// Caliban (NAIF ID: 716).
    Caliban = 716,
    /// Sycorax (NAIF ID: 717).
    Sycorax = 717,
    /// Prospero (NAIF ID: 718).
    Prospero = 718,
    /// Setebos (NAIF ID: 719).
    Setebos = 719,
    /// Stephano (NAIF ID: 720).
    Stephano = 720,
    /// Trinculo (NAIF ID: 721).
    Trinculo = 721,
    /// Francisco (NAIF ID: 722).
    Francisco = 722,
    /// Margaret (NAIF ID: 723).
    Margaret = 723,
    /// Ferdinand (NAIF ID: 724).
    Ferdinand = 724,
    /// Perdita (NAIF ID: 725).
    Perdita = 725,
    /// Mab (NAIF ID: 726).
    Mab = 726,
    /// Cupid (NAIF ID: 727).
    Cupid = 727,
    /// Triton (NAIF ID: 801).
    Triton = 801,
    /// Nereid (NAIF ID: 802).
    Nereid = 802,
    /// Naiad (NAIF ID: 803).
    Naiad = 803,
    /// Thalassa (NAIF ID: 804).
    Thalassa = 804,
    /// Despina (NAIF ID: 805).
    Despina = 805,
    /// Galatea (NAIF ID: 806).
    Galatea = 806,
    /// Larissa (NAIF ID: 807).
    Larissa = 807,
    /// Proteus (NAIF ID: 808).
    Proteus = 808,
    /// Halimede (NAIF ID: 809).
    Halimede = 809,
    /// Psamathe (NAIF ID: 810).
    Psamathe = 810,
    /// Sao (NAIF ID: 811).
    Sao = 811,
    /// Laomedeia (NAIF ID: 812).
    Laomedeia = 812,
    /// Neso (NAIF ID: 813).
    Neso = 813,
    /// Charon (NAIF ID: 901).
    Charon = 901,
    /// Nix (NAIF ID: 902).
    Nix = 902,
    /// Hydra (NAIF ID: 903).
    Hydra = 903,
    /// Kerberos (NAIF ID: 904).
    Kerberos = 904,
    /// Styx (NAIF ID: 905).
    Styx = 905,

    // Minor bodies.
    /// Gaspra (NAIF ID: 9511010).
    Gaspra = 9511010,
    /// Ida (NAIF ID: 2431010).
    Ida = 2431010,
    /// Dactyl (NAIF ID: 2431011).
    Dactyl = 2431011,
    /// Ceres (NAIF ID: 2000001).
    Ceres = 2000001,
    /// Pallas (NAIF ID: 2000002).
    Pallas = 2000002,
    /// Vesta (NAIF ID: 2000004).
    Vesta = 2000004,
    /// Psyche (NAIF ID: 2000016).
    Psyche = 2000016,
    /// Lutetia (NAIF ID: 2000021).
    Lutetia = 2000021,
    /// Kleopatra (NAIF ID: 2000216).
    Kleopatra = 2000216,
    /// Eros (NAIF ID: 2000433).
    Eros = 2000433,
    /// Davida (NAIF ID: 2000511).
    Davida = 2000511,
    /// Mathilde (NAIF ID: 2000253).
    Mathilde = 2000253,
    /// Steins (NAIF ID: 2002867).
    Steins = 2002867,
    /// Braille (NAIF ID: 2009969).
    Braille = 2009969,
    /// Wilson-Harrington (NAIF ID: 2004015).
    WilsonHarrington = 2004015,
    /// Toutatis (NAIF ID: 2004179).
    Toutatis = 2004179,
    /// Itokawa (NAIF ID: 2025143).
    Itokawa = 2025143,
    /// Bennu (NAIF ID: 2101955).
    Bennu = 2101955,
}

impl Origin for DynOrigin {
    fn id(&self) -> NaifId {
        NaifId(self.to_i32().unwrap())
    }

    fn name(&self) -> &'static str {
        match self {
            DynOrigin::Sun => "Sun",

            // Planets.
            DynOrigin::Mercury => "Mercury",
            DynOrigin::Venus => "Venus",
            DynOrigin::Earth => "Earth",
            DynOrigin::Mars => "Mars",
            DynOrigin::Jupiter => "Jupiter",
            DynOrigin::Saturn => "Saturn",
            DynOrigin::Uranus => "Uranus",
            DynOrigin::Neptune => "Neptune",
            DynOrigin::Pluto => "Pluto",

            // Barycenters.
            DynOrigin::SolarSystemBarycenter => "Solar System Barycenter",
            DynOrigin::MercuryBarycenter => "Mercury Barycenter",
            DynOrigin::VenusBarycenter => "Venus Barycenter",
            DynOrigin::EarthBarycenter => "Earth Barycenter",
            DynOrigin::MarsBarycenter => "Mars Barycenter",
            DynOrigin::JupiterBarycenter => "Jupiter Barycenter",
            DynOrigin::SaturnBarycenter => "Saturn Barycenter",
            DynOrigin::UranusBarycenter => "Uranus Barycenter",
            DynOrigin::NeptuneBarycenter => "Neptune Barycenter",
            DynOrigin::PlutoBarycenter => "Pluto Barycenter",

            // Satellites.
            DynOrigin::Moon => "Moon",
            DynOrigin::Phobos => "Phobos",
            DynOrigin::Deimos => "Deimos",
            DynOrigin::Io => "Io",
            DynOrigin::Europa => "Europa",
            DynOrigin::Ganymede => "Ganymede",
            DynOrigin::Callisto => "Callisto",
            DynOrigin::Amalthea => "Amalthea",
            DynOrigin::Himalia => "Himalia",
            DynOrigin::Elara => "Elara",
            DynOrigin::Pasiphae => "Pasiphae",
            DynOrigin::Sinope => "Sinope",
            DynOrigin::Lysithea => "Lysithea",
            DynOrigin::Carme => "Carme",
            DynOrigin::Ananke => "Ananke",
            DynOrigin::Leda => "Leda",
            DynOrigin::Thebe => "Thebe",
            DynOrigin::Adrastea => "Adrastea",
            DynOrigin::Metis => "Metis",
            DynOrigin::Callirrhoe => "Callirrhoe",
            DynOrigin::Themisto => "Themisto",
            DynOrigin::Magaclite => "Magaclite",
            DynOrigin::Taygete => "Taygete",
            DynOrigin::Chaldene => "Chaldene",
            DynOrigin::Harpalyke => "Harpalyke",
            DynOrigin::Kalyke => "Kalyke",
            DynOrigin::Iocaste => "Iocaste",
            DynOrigin::Erinome => "Erinome",
            DynOrigin::Isonoe => "Isonoe",
            DynOrigin::Praxidike => "Praxidike",
            DynOrigin::Autonoe => "Autonoe",
            DynOrigin::Thyone => "Thyone",
            DynOrigin::Hermippe => "Hermippe",
            DynOrigin::Aitne => "Aitne",
            DynOrigin::Eurydome => "Eurydome",
            DynOrigin::Euanthe => "Euanthe",
            DynOrigin::Euporie => "Euporie",
            DynOrigin::Orthosie => "Orthosie",
            DynOrigin::Sponde => "Sponde",
            DynOrigin::Kale => "Kale",
            DynOrigin::Pasithee => "Pasithee",
            DynOrigin::Hegemone => "Hegemone",
            DynOrigin::Mneme => "Mneme",
            DynOrigin::Aoede => "Aoede",
            DynOrigin::Thelxinoe => "Thelxinoe",
            DynOrigin::Arche => "Arche",
            DynOrigin::Kallichore => "Kallichore",
            DynOrigin::Helike => "Helike",
            DynOrigin::Carpo => "Carpo",
            DynOrigin::Eukelade => "Eukelade",
            DynOrigin::Cyllene => "Cyllene",
            DynOrigin::Kore => "Kore",
            DynOrigin::Herse => "Herse",
            DynOrigin::Dia => "Dia",
            DynOrigin::Mimas => "Mimas",
            DynOrigin::Enceladus => "Enceladus",
            DynOrigin::Tethys => "Tethys",
            DynOrigin::Dione => "Dione",
            DynOrigin::Rhea => "Rhea",
            DynOrigin::Titan => "Titan",
            DynOrigin::Hyperion => "Hyperion",
            DynOrigin::Iapetus => "Iapetus",
            DynOrigin::Phoebe => "Phoebe",
            DynOrigin::Janus => "Janus",
            DynOrigin::Epimetheus => "Epimetheus",
            DynOrigin::Helene => "Helene",
            DynOrigin::Telesto => "Telesto",
            DynOrigin::Calypso => "Calypso",
            DynOrigin::Atlas => "Atlas",
            DynOrigin::Prometheus => "Prometheus",
            DynOrigin::Pandora => "Pandora",
            DynOrigin::Pan => "Pan",
            DynOrigin::Ymir => "Ymir",
            DynOrigin::Paaliaq => "Paaliaq",
            DynOrigin::Tarvos => "Tarvos",
            DynOrigin::Ijiraq => "Ijiraq",
            DynOrigin::Suttungr => "Suttungr",
            DynOrigin::Kiviuq => "Kiviuq",
            DynOrigin::Mundilfari => "Mundilfari",
            DynOrigin::Albiorix => "Albiorix",
            DynOrigin::Skathi => "Skathi",
            DynOrigin::Erriapus => "Erriapus",
            DynOrigin::Siarnaq => "Siarnaq",
            DynOrigin::Thrymr => "Thrymr",
            DynOrigin::Narvi => "Narvi",
            DynOrigin::Methone => "Methone",
            DynOrigin::Pallene => "Pallene",
            DynOrigin::Polydeuces => "Polydeuces",
            DynOrigin::Daphnis => "Daphnis",
            DynOrigin::Aegir => "Aegir",
            DynOrigin::Bebhionn => "Bebhionn",
            DynOrigin::Bergelmir => "Bergelmir",
            DynOrigin::Bestla => "Bestla",
            DynOrigin::Farbauti => "Farbauti",
            DynOrigin::Fenrir => "Fenrir",
            DynOrigin::Fornjot => "Fornjot",
            DynOrigin::Hati => "Hati",
            DynOrigin::Hyrrokkin => "Hyrrokkin",
            DynOrigin::Kari => "Kari",
            DynOrigin::Loge => "Loge",
            DynOrigin::Skoll => "Skoll",
            DynOrigin::Surtur => "Surtur",
            DynOrigin::Anthe => "Anthe",
            DynOrigin::Jarnsaxa => "Jarnsaxa",
            DynOrigin::Greip => "Greip",
            DynOrigin::Tarqeq => "Tarqeq",
            DynOrigin::Aegaeon => "Aegaeon",
            DynOrigin::Ariel => "Ariel",
            DynOrigin::Umbriel => "Umbriel",
            DynOrigin::Titania => "Titania",
            DynOrigin::Oberon => "Oberon",
            DynOrigin::Miranda => "Miranda",
            DynOrigin::Cordelia => "Cordelia",
            DynOrigin::Ophelia => "Ophelia",
            DynOrigin::Bianca => "Bianca",
            DynOrigin::Cressida => "Cressida",
            DynOrigin::Desdemona => "Desdemona",
            DynOrigin::Juliet => "Juliet",
            DynOrigin::Portia => "Portia",
            DynOrigin::Rosalind => "Rosalind",
            DynOrigin::Belinda => "Belinda",
            DynOrigin::Puck => "Puck",
            DynOrigin::Caliban => "Caliban",
            DynOrigin::Sycorax => "Sycorax",
            DynOrigin::Prospero => "Prospero",
            DynOrigin::Setebos => "Setebos",
            DynOrigin::Stephano => "Stephano",
            DynOrigin::Trinculo => "Trinculo",
            DynOrigin::Francisco => "Francisco",
            DynOrigin::Margaret => "Margaret",
            DynOrigin::Ferdinand => "Ferdinand",
            DynOrigin::Perdita => "Perdita",
            DynOrigin::Mab => "Mab",
            DynOrigin::Cupid => "Cupid",
            DynOrigin::Triton => "Triton",
            DynOrigin::Nereid => "Nereid",
            DynOrigin::Naiad => "Naiad",
            DynOrigin::Thalassa => "Thalassa",
            DynOrigin::Despina => "Despina",
            DynOrigin::Galatea => "Galatea",
            DynOrigin::Larissa => "Larissa",
            DynOrigin::Proteus => "Proteus",
            DynOrigin::Halimede => "Halimede",
            DynOrigin::Psamathe => "Psamathe",
            DynOrigin::Sao => "Sao",
            DynOrigin::Laomedeia => "Laomedeia",
            DynOrigin::Neso => "Neso",
            DynOrigin::Charon => "Charon",
            DynOrigin::Nix => "Nix",
            DynOrigin::Hydra => "Hydra",
            DynOrigin::Kerberos => "Kerberos",
            DynOrigin::Styx => "Styx",

            // Minor bodies.
            DynOrigin::Gaspra => "Gaspra",
            DynOrigin::Ida => "Ida",
            DynOrigin::Dactyl => "Dactyl",
            DynOrigin::Ceres => "Ceres",
            DynOrigin::Pallas => "Pallas",
            DynOrigin::Vesta => "Vesta",
            DynOrigin::Psyche => "Psyche",
            DynOrigin::Lutetia => "Lutetia",
            DynOrigin::Kleopatra => "Kleopatra",
            DynOrigin::Eros => "Eros",
            DynOrigin::Davida => "Davida",
            DynOrigin::Mathilde => "Mathilde",
            DynOrigin::Steins => "Steins",
            DynOrigin::Braille => "Braille",
            DynOrigin::WilsonHarrington => "Wilson-Harrington",
            DynOrigin::Toutatis => "Toutatis",
            DynOrigin::Itokawa => "Itokawa",
            DynOrigin::Bennu => "Bennu",
        }
    }
}

impl Display for DynOrigin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TryFrom<i32> for DynOrigin {
    type Error = UnknownOriginId;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DynOrigin::from_i32(value).ok_or(UnknownOriginId(value))
    }
}

impl TryFrom<NaifId> for DynOrigin {
    type Error = UnknownOriginId;

    fn try_from(id: NaifId) -> Result<Self, Self::Error> {
        DynOrigin::try_from(id.0)
    }
}

impl FromStr for DynOrigin {
    type Err = UnknownOriginName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sun" | "Sun" => Ok(DynOrigin::Sun),
            "ssb" | "SSB" | "solar system barycenter" | "Solar System Barycenter" => {
                Ok(DynOrigin::SolarSystemBarycenter)
            }
            "mercury barycenter" | "Mercury Barycenter" => Ok(DynOrigin::MercuryBarycenter),
            "venus barycenter" | "Venus Barycenter" => Ok(DynOrigin::VenusBarycenter),
            "earth barycenter" | "Earth Barycenter" => Ok(DynOrigin::EarthBarycenter),
            "mars barycenter" | "Mars Barycenter" => Ok(DynOrigin::MarsBarycenter),
            "jupiter barycenter" | "Jupiter Barycenter" => Ok(DynOrigin::JupiterBarycenter),
            "saturn barycenter" | "Saturn Barycenter" => Ok(DynOrigin::SaturnBarycenter),
            "uranus barycenter" | "Uranus Barycenter" => Ok(DynOrigin::UranusBarycenter),
            "neptune barycenter" | "Neptune Barycenter" => Ok(DynOrigin::NeptuneBarycenter),
            "pluto barycenter" | "Pluto Barycenter" => Ok(DynOrigin::PlutoBarycenter),
            "mercury" | "Mercury" => Ok(DynOrigin::Mercury),
            "venus" | "Venus" => Ok(DynOrigin::Venus),
            "earth" | "Earth" => Ok(DynOrigin::Earth),
            "mars" | "Mars" => Ok(DynOrigin::Mars),
            "jupiter" | "Jupiter" => Ok(DynOrigin::Jupiter),
            "saturn" | "Saturn" => Ok(DynOrigin::Saturn),
            "uranus" | "Uranus" => Ok(DynOrigin::Uranus),
            "neptune" | "Neptune" => Ok(DynOrigin::Neptune),
            "pluto" | "Pluto" => Ok(DynOrigin::Pluto),
            "moon" | "Moon" | "luna" | "Luna" => Ok(DynOrigin::Moon),
            "phobos" | "Phobos" => Ok(DynOrigin::Phobos),
            "deimos" | "Deimos" => Ok(DynOrigin::Deimos),
            "io" | "Io" => Ok(DynOrigin::Io),
            "europa" | "Europa" => Ok(DynOrigin::Europa),
            "ganymede" | "Ganymede" => Ok(DynOrigin::Ganymede),
            "callisto" | "Callisto" => Ok(DynOrigin::Callisto),
            "amalthea" | "Amalthea" => Ok(DynOrigin::Amalthea),
            "himalia" | "Himalia" => Ok(DynOrigin::Himalia),
            "elara" | "Elara" => Ok(DynOrigin::Elara),
            "pasiphae" | "Pasiphae" => Ok(DynOrigin::Pasiphae),
            "sinope" | "Sinope" => Ok(DynOrigin::Sinope),
            "lysithea" | "Lysithea" => Ok(DynOrigin::Lysithea),
            "carme" | "Carme" => Ok(DynOrigin::Carme),
            "ananke" | "Ananke" => Ok(DynOrigin::Ananke),
            "leda" | "Leda" => Ok(DynOrigin::Leda),
            "thebe" | "Thebe" => Ok(DynOrigin::Thebe),
            "adrastea" | "Adrastea" => Ok(DynOrigin::Adrastea),
            "metis" | "Metis" => Ok(DynOrigin::Metis),
            "callirrhoe" | "Callirrhoe" => Ok(DynOrigin::Callirrhoe),
            "themisto" | "Themisto" => Ok(DynOrigin::Themisto),
            "magaclite" | "Magaclite" => Ok(DynOrigin::Magaclite),
            "taygete" | "Taygete" => Ok(DynOrigin::Taygete),
            "chaldene" | "Chaldene" => Ok(DynOrigin::Chaldene),
            "harpalyke" | "Harpalyke" => Ok(DynOrigin::Harpalyke),
            "kalyke" | "Kalyke" => Ok(DynOrigin::Kalyke),
            "iocaste" | "Iocaste" => Ok(DynOrigin::Iocaste),
            "erinome" | "Erinome" => Ok(DynOrigin::Erinome),
            "isonoe" | "Isonoe" => Ok(DynOrigin::Isonoe),
            "praxidike" | "Praxidike" => Ok(DynOrigin::Praxidike),
            "autonoe" | "Autonoe" => Ok(DynOrigin::Autonoe),
            "thyone" | "Thyone" => Ok(DynOrigin::Thyone),
            "hermippe" | "Hermippe" => Ok(DynOrigin::Hermippe),
            "aitne" | "Aitne" => Ok(DynOrigin::Aitne),
            "eurydome" | "Eurydome" => Ok(DynOrigin::Eurydome),
            "euanthe" | "Euanthe" => Ok(DynOrigin::Euanthe),
            "euporie" | "Euporie" => Ok(DynOrigin::Euporie),
            "orthosie" | "Orthosie" => Ok(DynOrigin::Orthosie),
            "sponde" | "Sponde" => Ok(DynOrigin::Sponde),
            "kale" | "Kale" => Ok(DynOrigin::Kale),
            "pasithee" | "Pasithee" => Ok(DynOrigin::Pasithee),
            "hegemone" | "Hegemone" => Ok(DynOrigin::Hegemone),
            "mneme" | "Mneme" => Ok(DynOrigin::Mneme),
            "aoede" | "Aoede" => Ok(DynOrigin::Aoede),
            "thelxinoe" | "Thelxinoe" => Ok(DynOrigin::Thelxinoe),
            "arche" | "Arche" => Ok(DynOrigin::Arche),
            "kallichore" | "Kallichore" => Ok(DynOrigin::Kallichore),
            "helike" | "Helike" => Ok(DynOrigin::Helike),
            "carpo" | "Carpo" => Ok(DynOrigin::Carpo),
            "eukelade" | "Eukelade" => Ok(DynOrigin::Eukelade),
            "cyllene" | "Cyllene" => Ok(DynOrigin::Cyllene),
            "kore" | "Kore" => Ok(DynOrigin::Kore),
            "herse" | "Herse" => Ok(DynOrigin::Herse),
            "dia" | "Dia" => Ok(DynOrigin::Dia),
            "mimas" | "Mimas" => Ok(DynOrigin::Mimas),
            "enceladus" | "Enceladus" => Ok(DynOrigin::Enceladus),
            "tethys" | "Tethys" => Ok(DynOrigin::Tethys),
            "dione" | "Dione" => Ok(DynOrigin::Dione),
            "rhea" | "Rhea" => Ok(DynOrigin::Rhea),
            "titan" | "Titan" => Ok(DynOrigin::Titan),
            "hyperion" | "Hyperion" => Ok(DynOrigin::Hyperion),
            "iapetus" | "Iapetus" => Ok(DynOrigin::Iapetus),
            "phoebe" | "Phoebe" => Ok(DynOrigin::Phoebe),
            "janus" | "Janus" => Ok(DynOrigin::Janus),
            "epimetheus" | "Epimetheus" => Ok(DynOrigin::Epimetheus),
            "helene" | "Helene" => Ok(DynOrigin::Helene),
            "telesto" | "Telesto" => Ok(DynOrigin::Telesto),
            "calypso" | "Calypso" => Ok(DynOrigin::Calypso),
            "atlas" | "Atlas" => Ok(DynOrigin::Atlas),
            "prometheus" | "Prometheus" => Ok(DynOrigin::Prometheus),
            "pandora" | "Pandora" => Ok(DynOrigin::Pandora),
            "pan" | "Pan" => Ok(DynOrigin::Pan),
            "ymir" | "Ymir" => Ok(DynOrigin::Ymir),
            "paaliaq" | "Paaliaq" => Ok(DynOrigin::Paaliaq),
            "tarvos" | "Tarvos" => Ok(DynOrigin::Tarvos),
            "ijiraq" | "Ijiraq" => Ok(DynOrigin::Ijiraq),
            "suttungr" | "Suttungr" => Ok(DynOrigin::Suttungr),
            "kiviuq" | "Kiviuq" => Ok(DynOrigin::Kiviuq),
            "mundilfari" | "Mundilfari" => Ok(DynOrigin::Mundilfari),
            "albiorix" | "Albiorix" => Ok(DynOrigin::Albiorix),
            "skathi" | "Skathi" => Ok(DynOrigin::Skathi),
            "erriapus" | "Erriapus" => Ok(DynOrigin::Erriapus),
            "siarnaq" | "Siarnaq" => Ok(DynOrigin::Siarnaq),
            "thrymr" | "Thrymr" => Ok(DynOrigin::Thrymr),
            "narvi" | "Narvi" => Ok(DynOrigin::Narvi),
            "methone" | "Methone" => Ok(DynOrigin::Methone),
            "pallene" | "Pallene" => Ok(DynOrigin::Pallene),
            "polydeuces" | "Polydeuces" => Ok(DynOrigin::Polydeuces),
            "daphnis" | "Daphnis" => Ok(DynOrigin::Daphnis),
            "aegir" | "Aegir" => Ok(DynOrigin::Aegir),
            "bebhionn" | "Bebhionn" => Ok(DynOrigin::Bebhionn),
            "bergelmir" | "Bergelmir" => Ok(DynOrigin::Bergelmir),
            "bestla" | "Bestla" => Ok(DynOrigin::Bestla),
            "farbauti" | "Farbauti" => Ok(DynOrigin::Farbauti),
            "fenrir" | "Fenrir" => Ok(DynOrigin::Fenrir),
            "fornjot" | "Fornjot" => Ok(DynOrigin::Fornjot),
            "hati" | "Hati" => Ok(DynOrigin::Hati),
            "hyrrokkin" | "Hyrrokkin" => Ok(DynOrigin::Hyrrokkin),
            "kari" | "Kari" => Ok(DynOrigin::Kari),
            "loge" | "Loge" => Ok(DynOrigin::Loge),
            "skoll" | "Skoll" => Ok(DynOrigin::Skoll),
            "surtur" | "Surtur" => Ok(DynOrigin::Surtur),
            "anthe" | "Anthe" => Ok(DynOrigin::Anthe),
            "jarnsaxa" | "Jarnsaxa" => Ok(DynOrigin::Jarnsaxa),
            "greip" | "Greip" => Ok(DynOrigin::Greip),
            "tarqeq" | "Tarqeq" => Ok(DynOrigin::Tarqeq),
            "aegaeon" | "Aegaeon" => Ok(DynOrigin::Aegaeon),
            "ariel" | "Ariel" => Ok(DynOrigin::Ariel),
            "umbriel" | "Umbriel" => Ok(DynOrigin::Umbriel),
            "titania" | "Titania" => Ok(DynOrigin::Titania),
            "oberon" | "Oberon" => Ok(DynOrigin::Oberon),
            "miranda" | "Miranda" => Ok(DynOrigin::Miranda),
            "cordelia" | "Cordelia" => Ok(DynOrigin::Cordelia),
            "ophelia" | "Ophelia" => Ok(DynOrigin::Ophelia),
            "bianca" | "Bianca" => Ok(DynOrigin::Bianca),
            "cressida" | "Cressida" => Ok(DynOrigin::Cressida),
            "desdemona" | "Desdemona" => Ok(DynOrigin::Desdemona),
            "juliet" | "Juliet" => Ok(DynOrigin::Juliet),
            "portia" | "Portia" => Ok(DynOrigin::Portia),
            "rosalind" | "Rosalind" => Ok(DynOrigin::Rosalind),
            "belinda" | "Belinda" => Ok(DynOrigin::Belinda),
            "puck" | "Puck" => Ok(DynOrigin::Puck),
            "caliban" | "Caliban" => Ok(DynOrigin::Caliban),
            "sycorax" | "Sycorax" => Ok(DynOrigin::Sycorax),
            "prospero" | "Prospero" => Ok(DynOrigin::Prospero),
            "setebos" | "Setebos" => Ok(DynOrigin::Setebos),
            "stephano" | "Stephano" => Ok(DynOrigin::Stephano),
            "trinculo" | "Trinculo" => Ok(DynOrigin::Trinculo),
            "francisco" | "Francisco" => Ok(DynOrigin::Francisco),
            "margaret" | "Margaret" => Ok(DynOrigin::Margaret),
            "ferdinand" | "Ferdinand" => Ok(DynOrigin::Ferdinand),
            "perdita" | "Perdita" => Ok(DynOrigin::Perdita),
            "mab" | "Mab" => Ok(DynOrigin::Mab),
            "cupid" | "Cupid" => Ok(DynOrigin::Cupid),
            "triton" | "Triton" => Ok(DynOrigin::Triton),
            "nereid" | "Nereid" => Ok(DynOrigin::Nereid),
            "naiad" | "Naiad" => Ok(DynOrigin::Naiad),
            "thalassa" | "Thalassa" => Ok(DynOrigin::Thalassa),
            "despina" | "Despina" => Ok(DynOrigin::Despina),
            "galatea" | "Galatea" => Ok(DynOrigin::Galatea),
            "larissa" | "Larissa" => Ok(DynOrigin::Larissa),
            "proteus" | "Proteus" => Ok(DynOrigin::Proteus),
            "halimede" | "Halimede" => Ok(DynOrigin::Halimede),
            "psamathe" | "Psamathe" => Ok(DynOrigin::Psamathe),
            "sao" | "Sao" => Ok(DynOrigin::Sao),
            "laomedeia" | "Laomedeia" => Ok(DynOrigin::Laomedeia),
            "neso" | "Neso" => Ok(DynOrigin::Neso),
            "charon" | "Charon" => Ok(DynOrigin::Charon),
            "nix" | "Nix" => Ok(DynOrigin::Nix),
            "hydra" | "Hydra" => Ok(DynOrigin::Hydra),
            "kerberos" | "Kerberos" => Ok(DynOrigin::Kerberos),
            "styx" | "Styx" => Ok(DynOrigin::Styx),

            // Minor bodies.
            "gaspra" | "Gaspra" => Ok(DynOrigin::Gaspra),
            "ida" | "Ida" => Ok(DynOrigin::Ida),
            "dactyl" | "Dactyl" => Ok(DynOrigin::Dactyl),
            "ceres" | "Ceres" => Ok(DynOrigin::Ceres),
            "pallas" | "Pallas" => Ok(DynOrigin::Pallas),
            "vesta" | "Vesta" => Ok(DynOrigin::Vesta),
            "psyche" | "Psyche" => Ok(DynOrigin::Psyche),
            "lutetia" | "Lutetia" => Ok(DynOrigin::Lutetia),
            "kleopatra" | "Kleopatra" => Ok(DynOrigin::Kleopatra),
            "eros" | "Eros" => Ok(DynOrigin::Eros),
            "davida" | "Davida" => Ok(DynOrigin::Davida),
            "mathilde" | "Mathilde" => Ok(DynOrigin::Mathilde),
            "steins" | "Steins" => Ok(DynOrigin::Steins),
            "braille" | "Braille" => Ok(DynOrigin::Braille),
            "wilson-harrington" | "Wilson-Harrington" | "wilson" | "Wilson" => {
                Ok(DynOrigin::WilsonHarrington)
            }
            "toutatis" | "Toutatis" => Ok(DynOrigin::Toutatis),
            "itokawa" | "Itokawa" => Ok(DynOrigin::Itokawa),
            "bennu" | "Bennu" => Ok(DynOrigin::Bennu),

            _ => Err(UnknownOriginName(s.to_owned())),
        }
    }
}

impl TryJ2 for DynOrigin {
    fn try_j2(&self) -> Result<f64, UndefinedOriginPropertyError> {
        match self {
            DynOrigin::Earth => Ok(Earth.j2()),
            _ => Err(UndefinedOriginPropertyError {
                origin: self.to_string(),
                prop: "J2".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(DynOrigin::Sun)]
    #[case(DynOrigin::Mercury)]
    #[case(DynOrigin::Venus)]
    #[case(DynOrigin::Earth)]
    #[case(DynOrigin::Mars)]
    #[case(DynOrigin::Jupiter)]
    #[case(DynOrigin::Saturn)]
    #[case(DynOrigin::Uranus)]
    #[case(DynOrigin::Neptune)]
    #[case(DynOrigin::Pluto)]
    #[case(DynOrigin::SolarSystemBarycenter)]
    #[case(DynOrigin::MercuryBarycenter)]
    #[case(DynOrigin::VenusBarycenter)]
    #[case(DynOrigin::EarthBarycenter)]
    #[case(DynOrigin::MarsBarycenter)]
    #[case(DynOrigin::JupiterBarycenter)]
    #[case(DynOrigin::SaturnBarycenter)]
    #[case(DynOrigin::UranusBarycenter)]
    #[case(DynOrigin::NeptuneBarycenter)]
    #[case(DynOrigin::PlutoBarycenter)]
    #[case(DynOrigin::Moon)]
    #[case(DynOrigin::Phobos)]
    #[case(DynOrigin::Deimos)]
    #[case(DynOrigin::Io)]
    #[case(DynOrigin::Europa)]
    #[case(DynOrigin::Ganymede)]
    #[case(DynOrigin::Callisto)]
    #[case(DynOrigin::Amalthea)]
    #[case(DynOrigin::Himalia)]
    #[case(DynOrigin::Elara)]
    #[case(DynOrigin::Pasiphae)]
    #[case(DynOrigin::Sinope)]
    #[case(DynOrigin::Lysithea)]
    #[case(DynOrigin::Carme)]
    #[case(DynOrigin::Ananke)]
    #[case(DynOrigin::Leda)]
    #[case(DynOrigin::Thebe)]
    #[case(DynOrigin::Adrastea)]
    #[case(DynOrigin::Metis)]
    #[case(DynOrigin::Callirrhoe)]
    #[case(DynOrigin::Themisto)]
    #[case(DynOrigin::Magaclite)]
    #[case(DynOrigin::Taygete)]
    #[case(DynOrigin::Chaldene)]
    #[case(DynOrigin::Harpalyke)]
    #[case(DynOrigin::Kalyke)]
    #[case(DynOrigin::Iocaste)]
    #[case(DynOrigin::Erinome)]
    #[case(DynOrigin::Isonoe)]
    #[case(DynOrigin::Praxidike)]
    #[case(DynOrigin::Autonoe)]
    #[case(DynOrigin::Thyone)]
    #[case(DynOrigin::Hermippe)]
    #[case(DynOrigin::Aitne)]
    #[case(DynOrigin::Eurydome)]
    #[case(DynOrigin::Euanthe)]
    #[case(DynOrigin::Euporie)]
    #[case(DynOrigin::Orthosie)]
    #[case(DynOrigin::Sponde)]
    #[case(DynOrigin::Kale)]
    #[case(DynOrigin::Pasithee)]
    #[case(DynOrigin::Hegemone)]
    #[case(DynOrigin::Mneme)]
    #[case(DynOrigin::Aoede)]
    #[case(DynOrigin::Thelxinoe)]
    #[case(DynOrigin::Arche)]
    #[case(DynOrigin::Kallichore)]
    #[case(DynOrigin::Helike)]
    #[case(DynOrigin::Carpo)]
    #[case(DynOrigin::Eukelade)]
    #[case(DynOrigin::Cyllene)]
    #[case(DynOrigin::Kore)]
    #[case(DynOrigin::Herse)]
    #[case(DynOrigin::Dia)]
    #[case(DynOrigin::Mimas)]
    #[case(DynOrigin::Enceladus)]
    #[case(DynOrigin::Tethys)]
    #[case(DynOrigin::Dione)]
    #[case(DynOrigin::Rhea)]
    #[case(DynOrigin::Titan)]
    #[case(DynOrigin::Hyperion)]
    #[case(DynOrigin::Iapetus)]
    #[case(DynOrigin::Phoebe)]
    #[case(DynOrigin::Janus)]
    #[case(DynOrigin::Epimetheus)]
    #[case(DynOrigin::Helene)]
    #[case(DynOrigin::Telesto)]
    #[case(DynOrigin::Calypso)]
    #[case(DynOrigin::Atlas)]
    #[case(DynOrigin::Prometheus)]
    #[case(DynOrigin::Pandora)]
    #[case(DynOrigin::Pan)]
    #[case(DynOrigin::Ymir)]
    #[case(DynOrigin::Paaliaq)]
    #[case(DynOrigin::Tarvos)]
    #[case(DynOrigin::Ijiraq)]
    #[case(DynOrigin::Suttungr)]
    #[case(DynOrigin::Kiviuq)]
    #[case(DynOrigin::Mundilfari)]
    #[case(DynOrigin::Albiorix)]
    #[case(DynOrigin::Skathi)]
    #[case(DynOrigin::Erriapus)]
    #[case(DynOrigin::Siarnaq)]
    #[case(DynOrigin::Thrymr)]
    #[case(DynOrigin::Narvi)]
    #[case(DynOrigin::Methone)]
    #[case(DynOrigin::Pallene)]
    #[case(DynOrigin::Polydeuces)]
    #[case(DynOrigin::Daphnis)]
    #[case(DynOrigin::Aegir)]
    #[case(DynOrigin::Bebhionn)]
    #[case(DynOrigin::Bergelmir)]
    #[case(DynOrigin::Bestla)]
    #[case(DynOrigin::Farbauti)]
    #[case(DynOrigin::Fenrir)]
    #[case(DynOrigin::Fornjot)]
    #[case(DynOrigin::Hati)]
    #[case(DynOrigin::Hyrrokkin)]
    #[case(DynOrigin::Kari)]
    #[case(DynOrigin::Loge)]
    #[case(DynOrigin::Skoll)]
    #[case(DynOrigin::Surtur)]
    #[case(DynOrigin::Anthe)]
    #[case(DynOrigin::Jarnsaxa)]
    #[case(DynOrigin::Greip)]
    #[case(DynOrigin::Tarqeq)]
    #[case(DynOrigin::Aegaeon)]
    #[case(DynOrigin::Ariel)]
    #[case(DynOrigin::Umbriel)]
    #[case(DynOrigin::Titania)]
    #[case(DynOrigin::Oberon)]
    #[case(DynOrigin::Miranda)]
    #[case(DynOrigin::Cordelia)]
    #[case(DynOrigin::Ophelia)]
    #[case(DynOrigin::Bianca)]
    #[case(DynOrigin::Cressida)]
    #[case(DynOrigin::Desdemona)]
    #[case(DynOrigin::Juliet)]
    #[case(DynOrigin::Portia)]
    #[case(DynOrigin::Rosalind)]
    #[case(DynOrigin::Belinda)]
    #[case(DynOrigin::Puck)]
    #[case(DynOrigin::Caliban)]
    #[case(DynOrigin::Sycorax)]
    #[case(DynOrigin::Prospero)]
    #[case(DynOrigin::Setebos)]
    #[case(DynOrigin::Stephano)]
    #[case(DynOrigin::Trinculo)]
    #[case(DynOrigin::Francisco)]
    #[case(DynOrigin::Margaret)]
    #[case(DynOrigin::Ferdinand)]
    #[case(DynOrigin::Perdita)]
    #[case(DynOrigin::Mab)]
    #[case(DynOrigin::Cupid)]
    #[case(DynOrigin::Triton)]
    #[case(DynOrigin::Nereid)]
    #[case(DynOrigin::Naiad)]
    #[case(DynOrigin::Thalassa)]
    #[case(DynOrigin::Despina)]
    #[case(DynOrigin::Galatea)]
    #[case(DynOrigin::Larissa)]
    #[case(DynOrigin::Proteus)]
    #[case(DynOrigin::Halimede)]
    #[case(DynOrigin::Psamathe)]
    #[case(DynOrigin::Sao)]
    #[case(DynOrigin::Laomedeia)]
    #[case(DynOrigin::Neso)]
    #[case(DynOrigin::Charon)]
    #[case(DynOrigin::Nix)]
    #[case(DynOrigin::Hydra)]
    #[case(DynOrigin::Kerberos)]
    #[case(DynOrigin::Styx)]
    #[case(DynOrigin::Gaspra)]
    #[case(DynOrigin::Ida)]
    #[case(DynOrigin::Dactyl)]
    #[case(DynOrigin::Ceres)]
    #[case(DynOrigin::Pallas)]
    #[case(DynOrigin::Vesta)]
    #[case(DynOrigin::Psyche)]
    #[case(DynOrigin::Lutetia)]
    #[case(DynOrigin::Kleopatra)]
    #[case(DynOrigin::Eros)]
    #[case(DynOrigin::Davida)]
    #[case(DynOrigin::Mathilde)]
    #[case(DynOrigin::Steins)]
    #[case(DynOrigin::Braille)]
    #[case(DynOrigin::WilsonHarrington)]
    #[case(DynOrigin::Toutatis)]
    #[case(DynOrigin::Itokawa)]
    #[case(DynOrigin::Bennu)]
    fn test_dyn_origin(#[case] exp: DynOrigin) {
        let act = DynOrigin::try_from(exp.to_i32().unwrap()).unwrap();
        assert_eq!(act, exp);
        let act = DynOrigin::try_from(exp.id()).unwrap();
        assert_eq!(act, exp);
        let act = DynOrigin::from_str(exp.to_string().as_str()).unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_dyn_origin_unknown_name() {
        assert_eq!(
            DynOrigin::from_str("Rupert"),
            Err(UnknownOriginName("Rupert".to_string()))
        );
    }

    #[test]
    fn test_dyn_origin_unknown_id() {
        assert_eq!(DynOrigin::try_from(666), Err(UnknownOriginId(666)))
    }
}
