// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use crate::{CoordinateOrigin, Earth, J2, J4, NaifId, TryJ2, TryJ4, UndefinedOriginPropertyError};
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};
use core::str::FromStr;
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
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum Origin {
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

impl CoordinateOrigin for Origin {
    fn id(&self) -> NaifId {
        NaifId(*self as i32)
    }

    fn name(&self) -> &'static str {
        match self {
            Origin::Sun => "Sun",

            // Planets.
            Origin::Mercury => "Mercury",
            Origin::Venus => "Venus",
            Origin::Earth => "Earth",
            Origin::Mars => "Mars",
            Origin::Jupiter => "Jupiter",
            Origin::Saturn => "Saturn",
            Origin::Uranus => "Uranus",
            Origin::Neptune => "Neptune",
            Origin::Pluto => "Pluto",

            // Barycenters.
            Origin::SolarSystemBarycenter => "Solar System Barycenter",
            Origin::MercuryBarycenter => "Mercury Barycenter",
            Origin::VenusBarycenter => "Venus Barycenter",
            Origin::EarthBarycenter => "Earth Barycenter",
            Origin::MarsBarycenter => "Mars Barycenter",
            Origin::JupiterBarycenter => "Jupiter Barycenter",
            Origin::SaturnBarycenter => "Saturn Barycenter",
            Origin::UranusBarycenter => "Uranus Barycenter",
            Origin::NeptuneBarycenter => "Neptune Barycenter",
            Origin::PlutoBarycenter => "Pluto Barycenter",

            // Satellites.
            Origin::Moon => "Moon",
            Origin::Phobos => "Phobos",
            Origin::Deimos => "Deimos",
            Origin::Io => "Io",
            Origin::Europa => "Europa",
            Origin::Ganymede => "Ganymede",
            Origin::Callisto => "Callisto",
            Origin::Amalthea => "Amalthea",
            Origin::Himalia => "Himalia",
            Origin::Elara => "Elara",
            Origin::Pasiphae => "Pasiphae",
            Origin::Sinope => "Sinope",
            Origin::Lysithea => "Lysithea",
            Origin::Carme => "Carme",
            Origin::Ananke => "Ananke",
            Origin::Leda => "Leda",
            Origin::Thebe => "Thebe",
            Origin::Adrastea => "Adrastea",
            Origin::Metis => "Metis",
            Origin::Callirrhoe => "Callirrhoe",
            Origin::Themisto => "Themisto",
            Origin::Magaclite => "Magaclite",
            Origin::Taygete => "Taygete",
            Origin::Chaldene => "Chaldene",
            Origin::Harpalyke => "Harpalyke",
            Origin::Kalyke => "Kalyke",
            Origin::Iocaste => "Iocaste",
            Origin::Erinome => "Erinome",
            Origin::Isonoe => "Isonoe",
            Origin::Praxidike => "Praxidike",
            Origin::Autonoe => "Autonoe",
            Origin::Thyone => "Thyone",
            Origin::Hermippe => "Hermippe",
            Origin::Aitne => "Aitne",
            Origin::Eurydome => "Eurydome",
            Origin::Euanthe => "Euanthe",
            Origin::Euporie => "Euporie",
            Origin::Orthosie => "Orthosie",
            Origin::Sponde => "Sponde",
            Origin::Kale => "Kale",
            Origin::Pasithee => "Pasithee",
            Origin::Hegemone => "Hegemone",
            Origin::Mneme => "Mneme",
            Origin::Aoede => "Aoede",
            Origin::Thelxinoe => "Thelxinoe",
            Origin::Arche => "Arche",
            Origin::Kallichore => "Kallichore",
            Origin::Helike => "Helike",
            Origin::Carpo => "Carpo",
            Origin::Eukelade => "Eukelade",
            Origin::Cyllene => "Cyllene",
            Origin::Kore => "Kore",
            Origin::Herse => "Herse",
            Origin::Dia => "Dia",
            Origin::Mimas => "Mimas",
            Origin::Enceladus => "Enceladus",
            Origin::Tethys => "Tethys",
            Origin::Dione => "Dione",
            Origin::Rhea => "Rhea",
            Origin::Titan => "Titan",
            Origin::Hyperion => "Hyperion",
            Origin::Iapetus => "Iapetus",
            Origin::Phoebe => "Phoebe",
            Origin::Janus => "Janus",
            Origin::Epimetheus => "Epimetheus",
            Origin::Helene => "Helene",
            Origin::Telesto => "Telesto",
            Origin::Calypso => "Calypso",
            Origin::Atlas => "Atlas",
            Origin::Prometheus => "Prometheus",
            Origin::Pandora => "Pandora",
            Origin::Pan => "Pan",
            Origin::Ymir => "Ymir",
            Origin::Paaliaq => "Paaliaq",
            Origin::Tarvos => "Tarvos",
            Origin::Ijiraq => "Ijiraq",
            Origin::Suttungr => "Suttungr",
            Origin::Kiviuq => "Kiviuq",
            Origin::Mundilfari => "Mundilfari",
            Origin::Albiorix => "Albiorix",
            Origin::Skathi => "Skathi",
            Origin::Erriapus => "Erriapus",
            Origin::Siarnaq => "Siarnaq",
            Origin::Thrymr => "Thrymr",
            Origin::Narvi => "Narvi",
            Origin::Methone => "Methone",
            Origin::Pallene => "Pallene",
            Origin::Polydeuces => "Polydeuces",
            Origin::Daphnis => "Daphnis",
            Origin::Aegir => "Aegir",
            Origin::Bebhionn => "Bebhionn",
            Origin::Bergelmir => "Bergelmir",
            Origin::Bestla => "Bestla",
            Origin::Farbauti => "Farbauti",
            Origin::Fenrir => "Fenrir",
            Origin::Fornjot => "Fornjot",
            Origin::Hati => "Hati",
            Origin::Hyrrokkin => "Hyrrokkin",
            Origin::Kari => "Kari",
            Origin::Loge => "Loge",
            Origin::Skoll => "Skoll",
            Origin::Surtur => "Surtur",
            Origin::Anthe => "Anthe",
            Origin::Jarnsaxa => "Jarnsaxa",
            Origin::Greip => "Greip",
            Origin::Tarqeq => "Tarqeq",
            Origin::Aegaeon => "Aegaeon",
            Origin::Ariel => "Ariel",
            Origin::Umbriel => "Umbriel",
            Origin::Titania => "Titania",
            Origin::Oberon => "Oberon",
            Origin::Miranda => "Miranda",
            Origin::Cordelia => "Cordelia",
            Origin::Ophelia => "Ophelia",
            Origin::Bianca => "Bianca",
            Origin::Cressida => "Cressida",
            Origin::Desdemona => "Desdemona",
            Origin::Juliet => "Juliet",
            Origin::Portia => "Portia",
            Origin::Rosalind => "Rosalind",
            Origin::Belinda => "Belinda",
            Origin::Puck => "Puck",
            Origin::Caliban => "Caliban",
            Origin::Sycorax => "Sycorax",
            Origin::Prospero => "Prospero",
            Origin::Setebos => "Setebos",
            Origin::Stephano => "Stephano",
            Origin::Trinculo => "Trinculo",
            Origin::Francisco => "Francisco",
            Origin::Margaret => "Margaret",
            Origin::Ferdinand => "Ferdinand",
            Origin::Perdita => "Perdita",
            Origin::Mab => "Mab",
            Origin::Cupid => "Cupid",
            Origin::Triton => "Triton",
            Origin::Nereid => "Nereid",
            Origin::Naiad => "Naiad",
            Origin::Thalassa => "Thalassa",
            Origin::Despina => "Despina",
            Origin::Galatea => "Galatea",
            Origin::Larissa => "Larissa",
            Origin::Proteus => "Proteus",
            Origin::Halimede => "Halimede",
            Origin::Psamathe => "Psamathe",
            Origin::Sao => "Sao",
            Origin::Laomedeia => "Laomedeia",
            Origin::Neso => "Neso",
            Origin::Charon => "Charon",
            Origin::Nix => "Nix",
            Origin::Hydra => "Hydra",
            Origin::Kerberos => "Kerberos",
            Origin::Styx => "Styx",

            // Minor bodies.
            Origin::Gaspra => "Gaspra",
            Origin::Ida => "Ida",
            Origin::Dactyl => "Dactyl",
            Origin::Ceres => "Ceres",
            Origin::Pallas => "Pallas",
            Origin::Vesta => "Vesta",
            Origin::Psyche => "Psyche",
            Origin::Lutetia => "Lutetia",
            Origin::Kleopatra => "Kleopatra",
            Origin::Eros => "Eros",
            Origin::Davida => "Davida",
            Origin::Mathilde => "Mathilde",
            Origin::Steins => "Steins",
            Origin::Braille => "Braille",
            Origin::WilsonHarrington => "Wilson-Harrington",
            Origin::Toutatis => "Toutatis",
            Origin::Itokawa => "Itokawa",
            Origin::Bennu => "Bennu",
        }
    }
}

impl Display for Origin {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TryFrom<i32> for Origin {
    type Error = UnknownOriginId;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            10 => Ok(Origin::Sun),
            199 => Ok(Origin::Mercury),
            299 => Ok(Origin::Venus),
            399 => Ok(Origin::Earth),
            499 => Ok(Origin::Mars),
            599 => Ok(Origin::Jupiter),
            699 => Ok(Origin::Saturn),
            799 => Ok(Origin::Uranus),
            899 => Ok(Origin::Neptune),
            999 => Ok(Origin::Pluto),
            0 => Ok(Origin::SolarSystemBarycenter),
            1 => Ok(Origin::MercuryBarycenter),
            2 => Ok(Origin::VenusBarycenter),
            3 => Ok(Origin::EarthBarycenter),
            4 => Ok(Origin::MarsBarycenter),
            5 => Ok(Origin::JupiterBarycenter),
            6 => Ok(Origin::SaturnBarycenter),
            7 => Ok(Origin::UranusBarycenter),
            8 => Ok(Origin::NeptuneBarycenter),
            9 => Ok(Origin::PlutoBarycenter),
            301 => Ok(Origin::Moon),
            401 => Ok(Origin::Phobos),
            402 => Ok(Origin::Deimos),
            501 => Ok(Origin::Io),
            502 => Ok(Origin::Europa),
            503 => Ok(Origin::Ganymede),
            504 => Ok(Origin::Callisto),
            505 => Ok(Origin::Amalthea),
            506 => Ok(Origin::Himalia),
            507 => Ok(Origin::Elara),
            508 => Ok(Origin::Pasiphae),
            509 => Ok(Origin::Sinope),
            510 => Ok(Origin::Lysithea),
            511 => Ok(Origin::Carme),
            512 => Ok(Origin::Ananke),
            513 => Ok(Origin::Leda),
            514 => Ok(Origin::Thebe),
            515 => Ok(Origin::Adrastea),
            516 => Ok(Origin::Metis),
            517 => Ok(Origin::Callirrhoe),
            518 => Ok(Origin::Themisto),
            519 => Ok(Origin::Magaclite),
            520 => Ok(Origin::Taygete),
            521 => Ok(Origin::Chaldene),
            522 => Ok(Origin::Harpalyke),
            523 => Ok(Origin::Kalyke),
            524 => Ok(Origin::Iocaste),
            525 => Ok(Origin::Erinome),
            526 => Ok(Origin::Isonoe),
            527 => Ok(Origin::Praxidike),
            528 => Ok(Origin::Autonoe),
            529 => Ok(Origin::Thyone),
            530 => Ok(Origin::Hermippe),
            531 => Ok(Origin::Aitne),
            532 => Ok(Origin::Eurydome),
            533 => Ok(Origin::Euanthe),
            534 => Ok(Origin::Euporie),
            535 => Ok(Origin::Orthosie),
            536 => Ok(Origin::Sponde),
            537 => Ok(Origin::Kale),
            538 => Ok(Origin::Pasithee),
            539 => Ok(Origin::Hegemone),
            540 => Ok(Origin::Mneme),
            541 => Ok(Origin::Aoede),
            542 => Ok(Origin::Thelxinoe),
            543 => Ok(Origin::Arche),
            544 => Ok(Origin::Kallichore),
            545 => Ok(Origin::Helike),
            546 => Ok(Origin::Carpo),
            547 => Ok(Origin::Eukelade),
            548 => Ok(Origin::Cyllene),
            549 => Ok(Origin::Kore),
            550 => Ok(Origin::Herse),
            553 => Ok(Origin::Dia),
            601 => Ok(Origin::Mimas),
            602 => Ok(Origin::Enceladus),
            603 => Ok(Origin::Tethys),
            604 => Ok(Origin::Dione),
            605 => Ok(Origin::Rhea),
            606 => Ok(Origin::Titan),
            607 => Ok(Origin::Hyperion),
            608 => Ok(Origin::Iapetus),
            609 => Ok(Origin::Phoebe),
            610 => Ok(Origin::Janus),
            611 => Ok(Origin::Epimetheus),
            612 => Ok(Origin::Helene),
            613 => Ok(Origin::Telesto),
            614 => Ok(Origin::Calypso),
            615 => Ok(Origin::Atlas),
            616 => Ok(Origin::Prometheus),
            617 => Ok(Origin::Pandora),
            618 => Ok(Origin::Pan),
            619 => Ok(Origin::Ymir),
            620 => Ok(Origin::Paaliaq),
            621 => Ok(Origin::Tarvos),
            622 => Ok(Origin::Ijiraq),
            623 => Ok(Origin::Suttungr),
            624 => Ok(Origin::Kiviuq),
            625 => Ok(Origin::Mundilfari),
            626 => Ok(Origin::Albiorix),
            627 => Ok(Origin::Skathi),
            628 => Ok(Origin::Erriapus),
            629 => Ok(Origin::Siarnaq),
            630 => Ok(Origin::Thrymr),
            631 => Ok(Origin::Narvi),
            632 => Ok(Origin::Methone),
            633 => Ok(Origin::Pallene),
            634 => Ok(Origin::Polydeuces),
            635 => Ok(Origin::Daphnis),
            636 => Ok(Origin::Aegir),
            637 => Ok(Origin::Bebhionn),
            638 => Ok(Origin::Bergelmir),
            639 => Ok(Origin::Bestla),
            640 => Ok(Origin::Farbauti),
            641 => Ok(Origin::Fenrir),
            642 => Ok(Origin::Fornjot),
            643 => Ok(Origin::Hati),
            644 => Ok(Origin::Hyrrokkin),
            645 => Ok(Origin::Kari),
            646 => Ok(Origin::Loge),
            647 => Ok(Origin::Skoll),
            648 => Ok(Origin::Surtur),
            649 => Ok(Origin::Anthe),
            650 => Ok(Origin::Jarnsaxa),
            651 => Ok(Origin::Greip),
            652 => Ok(Origin::Tarqeq),
            653 => Ok(Origin::Aegaeon),
            701 => Ok(Origin::Ariel),
            702 => Ok(Origin::Umbriel),
            703 => Ok(Origin::Titania),
            704 => Ok(Origin::Oberon),
            705 => Ok(Origin::Miranda),
            706 => Ok(Origin::Cordelia),
            707 => Ok(Origin::Ophelia),
            708 => Ok(Origin::Bianca),
            709 => Ok(Origin::Cressida),
            710 => Ok(Origin::Desdemona),
            711 => Ok(Origin::Juliet),
            712 => Ok(Origin::Portia),
            713 => Ok(Origin::Rosalind),
            714 => Ok(Origin::Belinda),
            715 => Ok(Origin::Puck),
            716 => Ok(Origin::Caliban),
            717 => Ok(Origin::Sycorax),
            718 => Ok(Origin::Prospero),
            719 => Ok(Origin::Setebos),
            720 => Ok(Origin::Stephano),
            721 => Ok(Origin::Trinculo),
            722 => Ok(Origin::Francisco),
            723 => Ok(Origin::Margaret),
            724 => Ok(Origin::Ferdinand),
            725 => Ok(Origin::Perdita),
            726 => Ok(Origin::Mab),
            727 => Ok(Origin::Cupid),
            801 => Ok(Origin::Triton),
            802 => Ok(Origin::Nereid),
            803 => Ok(Origin::Naiad),
            804 => Ok(Origin::Thalassa),
            805 => Ok(Origin::Despina),
            806 => Ok(Origin::Galatea),
            807 => Ok(Origin::Larissa),
            808 => Ok(Origin::Proteus),
            809 => Ok(Origin::Halimede),
            810 => Ok(Origin::Psamathe),
            811 => Ok(Origin::Sao),
            812 => Ok(Origin::Laomedeia),
            813 => Ok(Origin::Neso),
            901 => Ok(Origin::Charon),
            902 => Ok(Origin::Nix),
            903 => Ok(Origin::Hydra),
            904 => Ok(Origin::Kerberos),
            905 => Ok(Origin::Styx),
            9511010 => Ok(Origin::Gaspra),
            2431010 => Ok(Origin::Ida),
            2431011 => Ok(Origin::Dactyl),
            2000001 => Ok(Origin::Ceres),
            2000002 => Ok(Origin::Pallas),
            2000004 => Ok(Origin::Vesta),
            2000016 => Ok(Origin::Psyche),
            2000021 => Ok(Origin::Lutetia),
            2000216 => Ok(Origin::Kleopatra),
            2000433 => Ok(Origin::Eros),
            2000511 => Ok(Origin::Davida),
            2000253 => Ok(Origin::Mathilde),
            2002867 => Ok(Origin::Steins),
            2009969 => Ok(Origin::Braille),
            2004015 => Ok(Origin::WilsonHarrington),
            2004179 => Ok(Origin::Toutatis),
            2025143 => Ok(Origin::Itokawa),
            2101955 => Ok(Origin::Bennu),
            _ => Err(UnknownOriginId(value)),
        }
    }
}

impl TryFrom<NaifId> for Origin {
    type Error = UnknownOriginId;

    fn try_from(id: NaifId) -> Result<Self, Self::Error> {
        Origin::try_from(id.0)
    }
}

impl FromStr for Origin {
    type Err = UnknownOriginName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sun" | "Sun" => Ok(Origin::Sun),
            "ssb" | "SSB" | "solar system barycenter" | "Solar System Barycenter" => {
                Ok(Origin::SolarSystemBarycenter)
            }
            "mercury barycenter" | "Mercury Barycenter" => Ok(Origin::MercuryBarycenter),
            "venus barycenter" | "Venus Barycenter" => Ok(Origin::VenusBarycenter),
            "earth barycenter" | "Earth Barycenter" => Ok(Origin::EarthBarycenter),
            "mars barycenter" | "Mars Barycenter" => Ok(Origin::MarsBarycenter),
            "jupiter barycenter" | "Jupiter Barycenter" => Ok(Origin::JupiterBarycenter),
            "saturn barycenter" | "Saturn Barycenter" => Ok(Origin::SaturnBarycenter),
            "uranus barycenter" | "Uranus Barycenter" => Ok(Origin::UranusBarycenter),
            "neptune barycenter" | "Neptune Barycenter" => Ok(Origin::NeptuneBarycenter),
            "pluto barycenter" | "Pluto Barycenter" => Ok(Origin::PlutoBarycenter),
            "mercury" | "Mercury" => Ok(Origin::Mercury),
            "venus" | "Venus" => Ok(Origin::Venus),
            "earth" | "Earth" => Ok(Origin::Earth),
            "mars" | "Mars" => Ok(Origin::Mars),
            "jupiter" | "Jupiter" => Ok(Origin::Jupiter),
            "saturn" | "Saturn" => Ok(Origin::Saturn),
            "uranus" | "Uranus" => Ok(Origin::Uranus),
            "neptune" | "Neptune" => Ok(Origin::Neptune),
            "pluto" | "Pluto" => Ok(Origin::Pluto),
            "moon" | "Moon" | "luna" | "Luna" => Ok(Origin::Moon),
            "phobos" | "Phobos" => Ok(Origin::Phobos),
            "deimos" | "Deimos" => Ok(Origin::Deimos),
            "io" | "Io" => Ok(Origin::Io),
            "europa" | "Europa" => Ok(Origin::Europa),
            "ganymede" | "Ganymede" => Ok(Origin::Ganymede),
            "callisto" | "Callisto" => Ok(Origin::Callisto),
            "amalthea" | "Amalthea" => Ok(Origin::Amalthea),
            "himalia" | "Himalia" => Ok(Origin::Himalia),
            "elara" | "Elara" => Ok(Origin::Elara),
            "pasiphae" | "Pasiphae" => Ok(Origin::Pasiphae),
            "sinope" | "Sinope" => Ok(Origin::Sinope),
            "lysithea" | "Lysithea" => Ok(Origin::Lysithea),
            "carme" | "Carme" => Ok(Origin::Carme),
            "ananke" | "Ananke" => Ok(Origin::Ananke),
            "leda" | "Leda" => Ok(Origin::Leda),
            "thebe" | "Thebe" => Ok(Origin::Thebe),
            "adrastea" | "Adrastea" => Ok(Origin::Adrastea),
            "metis" | "Metis" => Ok(Origin::Metis),
            "callirrhoe" | "Callirrhoe" => Ok(Origin::Callirrhoe),
            "themisto" | "Themisto" => Ok(Origin::Themisto),
            "magaclite" | "Magaclite" => Ok(Origin::Magaclite),
            "taygete" | "Taygete" => Ok(Origin::Taygete),
            "chaldene" | "Chaldene" => Ok(Origin::Chaldene),
            "harpalyke" | "Harpalyke" => Ok(Origin::Harpalyke),
            "kalyke" | "Kalyke" => Ok(Origin::Kalyke),
            "iocaste" | "Iocaste" => Ok(Origin::Iocaste),
            "erinome" | "Erinome" => Ok(Origin::Erinome),
            "isonoe" | "Isonoe" => Ok(Origin::Isonoe),
            "praxidike" | "Praxidike" => Ok(Origin::Praxidike),
            "autonoe" | "Autonoe" => Ok(Origin::Autonoe),
            "thyone" | "Thyone" => Ok(Origin::Thyone),
            "hermippe" | "Hermippe" => Ok(Origin::Hermippe),
            "aitne" | "Aitne" => Ok(Origin::Aitne),
            "eurydome" | "Eurydome" => Ok(Origin::Eurydome),
            "euanthe" | "Euanthe" => Ok(Origin::Euanthe),
            "euporie" | "Euporie" => Ok(Origin::Euporie),
            "orthosie" | "Orthosie" => Ok(Origin::Orthosie),
            "sponde" | "Sponde" => Ok(Origin::Sponde),
            "kale" | "Kale" => Ok(Origin::Kale),
            "pasithee" | "Pasithee" => Ok(Origin::Pasithee),
            "hegemone" | "Hegemone" => Ok(Origin::Hegemone),
            "mneme" | "Mneme" => Ok(Origin::Mneme),
            "aoede" | "Aoede" => Ok(Origin::Aoede),
            "thelxinoe" | "Thelxinoe" => Ok(Origin::Thelxinoe),
            "arche" | "Arche" => Ok(Origin::Arche),
            "kallichore" | "Kallichore" => Ok(Origin::Kallichore),
            "helike" | "Helike" => Ok(Origin::Helike),
            "carpo" | "Carpo" => Ok(Origin::Carpo),
            "eukelade" | "Eukelade" => Ok(Origin::Eukelade),
            "cyllene" | "Cyllene" => Ok(Origin::Cyllene),
            "kore" | "Kore" => Ok(Origin::Kore),
            "herse" | "Herse" => Ok(Origin::Herse),
            "dia" | "Dia" => Ok(Origin::Dia),
            "mimas" | "Mimas" => Ok(Origin::Mimas),
            "enceladus" | "Enceladus" => Ok(Origin::Enceladus),
            "tethys" | "Tethys" => Ok(Origin::Tethys),
            "dione" | "Dione" => Ok(Origin::Dione),
            "rhea" | "Rhea" => Ok(Origin::Rhea),
            "titan" | "Titan" => Ok(Origin::Titan),
            "hyperion" | "Hyperion" => Ok(Origin::Hyperion),
            "iapetus" | "Iapetus" => Ok(Origin::Iapetus),
            "phoebe" | "Phoebe" => Ok(Origin::Phoebe),
            "janus" | "Janus" => Ok(Origin::Janus),
            "epimetheus" | "Epimetheus" => Ok(Origin::Epimetheus),
            "helene" | "Helene" => Ok(Origin::Helene),
            "telesto" | "Telesto" => Ok(Origin::Telesto),
            "calypso" | "Calypso" => Ok(Origin::Calypso),
            "atlas" | "Atlas" => Ok(Origin::Atlas),
            "prometheus" | "Prometheus" => Ok(Origin::Prometheus),
            "pandora" | "Pandora" => Ok(Origin::Pandora),
            "pan" | "Pan" => Ok(Origin::Pan),
            "ymir" | "Ymir" => Ok(Origin::Ymir),
            "paaliaq" | "Paaliaq" => Ok(Origin::Paaliaq),
            "tarvos" | "Tarvos" => Ok(Origin::Tarvos),
            "ijiraq" | "Ijiraq" => Ok(Origin::Ijiraq),
            "suttungr" | "Suttungr" => Ok(Origin::Suttungr),
            "kiviuq" | "Kiviuq" => Ok(Origin::Kiviuq),
            "mundilfari" | "Mundilfari" => Ok(Origin::Mundilfari),
            "albiorix" | "Albiorix" => Ok(Origin::Albiorix),
            "skathi" | "Skathi" => Ok(Origin::Skathi),
            "erriapus" | "Erriapus" => Ok(Origin::Erriapus),
            "siarnaq" | "Siarnaq" => Ok(Origin::Siarnaq),
            "thrymr" | "Thrymr" => Ok(Origin::Thrymr),
            "narvi" | "Narvi" => Ok(Origin::Narvi),
            "methone" | "Methone" => Ok(Origin::Methone),
            "pallene" | "Pallene" => Ok(Origin::Pallene),
            "polydeuces" | "Polydeuces" => Ok(Origin::Polydeuces),
            "daphnis" | "Daphnis" => Ok(Origin::Daphnis),
            "aegir" | "Aegir" => Ok(Origin::Aegir),
            "bebhionn" | "Bebhionn" => Ok(Origin::Bebhionn),
            "bergelmir" | "Bergelmir" => Ok(Origin::Bergelmir),
            "bestla" | "Bestla" => Ok(Origin::Bestla),
            "farbauti" | "Farbauti" => Ok(Origin::Farbauti),
            "fenrir" | "Fenrir" => Ok(Origin::Fenrir),
            "fornjot" | "Fornjot" => Ok(Origin::Fornjot),
            "hati" | "Hati" => Ok(Origin::Hati),
            "hyrrokkin" | "Hyrrokkin" => Ok(Origin::Hyrrokkin),
            "kari" | "Kari" => Ok(Origin::Kari),
            "loge" | "Loge" => Ok(Origin::Loge),
            "skoll" | "Skoll" => Ok(Origin::Skoll),
            "surtur" | "Surtur" => Ok(Origin::Surtur),
            "anthe" | "Anthe" => Ok(Origin::Anthe),
            "jarnsaxa" | "Jarnsaxa" => Ok(Origin::Jarnsaxa),
            "greip" | "Greip" => Ok(Origin::Greip),
            "tarqeq" | "Tarqeq" => Ok(Origin::Tarqeq),
            "aegaeon" | "Aegaeon" => Ok(Origin::Aegaeon),
            "ariel" | "Ariel" => Ok(Origin::Ariel),
            "umbriel" | "Umbriel" => Ok(Origin::Umbriel),
            "titania" | "Titania" => Ok(Origin::Titania),
            "oberon" | "Oberon" => Ok(Origin::Oberon),
            "miranda" | "Miranda" => Ok(Origin::Miranda),
            "cordelia" | "Cordelia" => Ok(Origin::Cordelia),
            "ophelia" | "Ophelia" => Ok(Origin::Ophelia),
            "bianca" | "Bianca" => Ok(Origin::Bianca),
            "cressida" | "Cressida" => Ok(Origin::Cressida),
            "desdemona" | "Desdemona" => Ok(Origin::Desdemona),
            "juliet" | "Juliet" => Ok(Origin::Juliet),
            "portia" | "Portia" => Ok(Origin::Portia),
            "rosalind" | "Rosalind" => Ok(Origin::Rosalind),
            "belinda" | "Belinda" => Ok(Origin::Belinda),
            "puck" | "Puck" => Ok(Origin::Puck),
            "caliban" | "Caliban" => Ok(Origin::Caliban),
            "sycorax" | "Sycorax" => Ok(Origin::Sycorax),
            "prospero" | "Prospero" => Ok(Origin::Prospero),
            "setebos" | "Setebos" => Ok(Origin::Setebos),
            "stephano" | "Stephano" => Ok(Origin::Stephano),
            "trinculo" | "Trinculo" => Ok(Origin::Trinculo),
            "francisco" | "Francisco" => Ok(Origin::Francisco),
            "margaret" | "Margaret" => Ok(Origin::Margaret),
            "ferdinand" | "Ferdinand" => Ok(Origin::Ferdinand),
            "perdita" | "Perdita" => Ok(Origin::Perdita),
            "mab" | "Mab" => Ok(Origin::Mab),
            "cupid" | "Cupid" => Ok(Origin::Cupid),
            "triton" | "Triton" => Ok(Origin::Triton),
            "nereid" | "Nereid" => Ok(Origin::Nereid),
            "naiad" | "Naiad" => Ok(Origin::Naiad),
            "thalassa" | "Thalassa" => Ok(Origin::Thalassa),
            "despina" | "Despina" => Ok(Origin::Despina),
            "galatea" | "Galatea" => Ok(Origin::Galatea),
            "larissa" | "Larissa" => Ok(Origin::Larissa),
            "proteus" | "Proteus" => Ok(Origin::Proteus),
            "halimede" | "Halimede" => Ok(Origin::Halimede),
            "psamathe" | "Psamathe" => Ok(Origin::Psamathe),
            "sao" | "Sao" => Ok(Origin::Sao),
            "laomedeia" | "Laomedeia" => Ok(Origin::Laomedeia),
            "neso" | "Neso" => Ok(Origin::Neso),
            "charon" | "Charon" => Ok(Origin::Charon),
            "nix" | "Nix" => Ok(Origin::Nix),
            "hydra" | "Hydra" => Ok(Origin::Hydra),
            "kerberos" | "Kerberos" => Ok(Origin::Kerberos),
            "styx" | "Styx" => Ok(Origin::Styx),

            // Minor bodies.
            "gaspra" | "Gaspra" => Ok(Origin::Gaspra),
            "ida" | "Ida" => Ok(Origin::Ida),
            "dactyl" | "Dactyl" => Ok(Origin::Dactyl),
            "ceres" | "Ceres" => Ok(Origin::Ceres),
            "pallas" | "Pallas" => Ok(Origin::Pallas),
            "vesta" | "Vesta" => Ok(Origin::Vesta),
            "psyche" | "Psyche" => Ok(Origin::Psyche),
            "lutetia" | "Lutetia" => Ok(Origin::Lutetia),
            "kleopatra" | "Kleopatra" => Ok(Origin::Kleopatra),
            "eros" | "Eros" => Ok(Origin::Eros),
            "davida" | "Davida" => Ok(Origin::Davida),
            "mathilde" | "Mathilde" => Ok(Origin::Mathilde),
            "steins" | "Steins" => Ok(Origin::Steins),
            "braille" | "Braille" => Ok(Origin::Braille),
            "wilson-harrington" | "Wilson-Harrington" | "wilson" | "Wilson" => {
                Ok(Origin::WilsonHarrington)
            }
            "toutatis" | "Toutatis" => Ok(Origin::Toutatis),
            "itokawa" | "Itokawa" => Ok(Origin::Itokawa),
            "bennu" | "Bennu" => Ok(Origin::Bennu),

            _ => Err(UnknownOriginName(s.to_owned())),
        }
    }
}

impl TryJ2 for Origin {
    fn try_j2(&self) -> Result<f64, UndefinedOriginPropertyError> {
        match self {
            Origin::Earth => Ok(Earth.j2()),
            _ => Err(UndefinedOriginPropertyError {
                origin: self.to_string(),
                prop: "J2".to_owned(),
            }),
        }
    }
}

impl TryJ4 for Origin {
    fn try_j4(&self) -> Result<f64, UndefinedOriginPropertyError> {
        match self {
            Origin::Earth => Ok(Earth.j4()),
            _ => Err(UndefinedOriginPropertyError {
                origin: self.to_string(),
                prop: "J4".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Origin::Sun)]
    #[case(Origin::Mercury)]
    #[case(Origin::Venus)]
    #[case(Origin::Earth)]
    #[case(Origin::Mars)]
    #[case(Origin::Jupiter)]
    #[case(Origin::Saturn)]
    #[case(Origin::Uranus)]
    #[case(Origin::Neptune)]
    #[case(Origin::Pluto)]
    #[case(Origin::SolarSystemBarycenter)]
    #[case(Origin::MercuryBarycenter)]
    #[case(Origin::VenusBarycenter)]
    #[case(Origin::EarthBarycenter)]
    #[case(Origin::MarsBarycenter)]
    #[case(Origin::JupiterBarycenter)]
    #[case(Origin::SaturnBarycenter)]
    #[case(Origin::UranusBarycenter)]
    #[case(Origin::NeptuneBarycenter)]
    #[case(Origin::PlutoBarycenter)]
    #[case(Origin::Moon)]
    #[case(Origin::Phobos)]
    #[case(Origin::Deimos)]
    #[case(Origin::Io)]
    #[case(Origin::Europa)]
    #[case(Origin::Ganymede)]
    #[case(Origin::Callisto)]
    #[case(Origin::Amalthea)]
    #[case(Origin::Himalia)]
    #[case(Origin::Elara)]
    #[case(Origin::Pasiphae)]
    #[case(Origin::Sinope)]
    #[case(Origin::Lysithea)]
    #[case(Origin::Carme)]
    #[case(Origin::Ananke)]
    #[case(Origin::Leda)]
    #[case(Origin::Thebe)]
    #[case(Origin::Adrastea)]
    #[case(Origin::Metis)]
    #[case(Origin::Callirrhoe)]
    #[case(Origin::Themisto)]
    #[case(Origin::Magaclite)]
    #[case(Origin::Taygete)]
    #[case(Origin::Chaldene)]
    #[case(Origin::Harpalyke)]
    #[case(Origin::Kalyke)]
    #[case(Origin::Iocaste)]
    #[case(Origin::Erinome)]
    #[case(Origin::Isonoe)]
    #[case(Origin::Praxidike)]
    #[case(Origin::Autonoe)]
    #[case(Origin::Thyone)]
    #[case(Origin::Hermippe)]
    #[case(Origin::Aitne)]
    #[case(Origin::Eurydome)]
    #[case(Origin::Euanthe)]
    #[case(Origin::Euporie)]
    #[case(Origin::Orthosie)]
    #[case(Origin::Sponde)]
    #[case(Origin::Kale)]
    #[case(Origin::Pasithee)]
    #[case(Origin::Hegemone)]
    #[case(Origin::Mneme)]
    #[case(Origin::Aoede)]
    #[case(Origin::Thelxinoe)]
    #[case(Origin::Arche)]
    #[case(Origin::Kallichore)]
    #[case(Origin::Helike)]
    #[case(Origin::Carpo)]
    #[case(Origin::Eukelade)]
    #[case(Origin::Cyllene)]
    #[case(Origin::Kore)]
    #[case(Origin::Herse)]
    #[case(Origin::Dia)]
    #[case(Origin::Mimas)]
    #[case(Origin::Enceladus)]
    #[case(Origin::Tethys)]
    #[case(Origin::Dione)]
    #[case(Origin::Rhea)]
    #[case(Origin::Titan)]
    #[case(Origin::Hyperion)]
    #[case(Origin::Iapetus)]
    #[case(Origin::Phoebe)]
    #[case(Origin::Janus)]
    #[case(Origin::Epimetheus)]
    #[case(Origin::Helene)]
    #[case(Origin::Telesto)]
    #[case(Origin::Calypso)]
    #[case(Origin::Atlas)]
    #[case(Origin::Prometheus)]
    #[case(Origin::Pandora)]
    #[case(Origin::Pan)]
    #[case(Origin::Ymir)]
    #[case(Origin::Paaliaq)]
    #[case(Origin::Tarvos)]
    #[case(Origin::Ijiraq)]
    #[case(Origin::Suttungr)]
    #[case(Origin::Kiviuq)]
    #[case(Origin::Mundilfari)]
    #[case(Origin::Albiorix)]
    #[case(Origin::Skathi)]
    #[case(Origin::Erriapus)]
    #[case(Origin::Siarnaq)]
    #[case(Origin::Thrymr)]
    #[case(Origin::Narvi)]
    #[case(Origin::Methone)]
    #[case(Origin::Pallene)]
    #[case(Origin::Polydeuces)]
    #[case(Origin::Daphnis)]
    #[case(Origin::Aegir)]
    #[case(Origin::Bebhionn)]
    #[case(Origin::Bergelmir)]
    #[case(Origin::Bestla)]
    #[case(Origin::Farbauti)]
    #[case(Origin::Fenrir)]
    #[case(Origin::Fornjot)]
    #[case(Origin::Hati)]
    #[case(Origin::Hyrrokkin)]
    #[case(Origin::Kari)]
    #[case(Origin::Loge)]
    #[case(Origin::Skoll)]
    #[case(Origin::Surtur)]
    #[case(Origin::Anthe)]
    #[case(Origin::Jarnsaxa)]
    #[case(Origin::Greip)]
    #[case(Origin::Tarqeq)]
    #[case(Origin::Aegaeon)]
    #[case(Origin::Ariel)]
    #[case(Origin::Umbriel)]
    #[case(Origin::Titania)]
    #[case(Origin::Oberon)]
    #[case(Origin::Miranda)]
    #[case(Origin::Cordelia)]
    #[case(Origin::Ophelia)]
    #[case(Origin::Bianca)]
    #[case(Origin::Cressida)]
    #[case(Origin::Desdemona)]
    #[case(Origin::Juliet)]
    #[case(Origin::Portia)]
    #[case(Origin::Rosalind)]
    #[case(Origin::Belinda)]
    #[case(Origin::Puck)]
    #[case(Origin::Caliban)]
    #[case(Origin::Sycorax)]
    #[case(Origin::Prospero)]
    #[case(Origin::Setebos)]
    #[case(Origin::Stephano)]
    #[case(Origin::Trinculo)]
    #[case(Origin::Francisco)]
    #[case(Origin::Margaret)]
    #[case(Origin::Ferdinand)]
    #[case(Origin::Perdita)]
    #[case(Origin::Mab)]
    #[case(Origin::Cupid)]
    #[case(Origin::Triton)]
    #[case(Origin::Nereid)]
    #[case(Origin::Naiad)]
    #[case(Origin::Thalassa)]
    #[case(Origin::Despina)]
    #[case(Origin::Galatea)]
    #[case(Origin::Larissa)]
    #[case(Origin::Proteus)]
    #[case(Origin::Halimede)]
    #[case(Origin::Psamathe)]
    #[case(Origin::Sao)]
    #[case(Origin::Laomedeia)]
    #[case(Origin::Neso)]
    #[case(Origin::Charon)]
    #[case(Origin::Nix)]
    #[case(Origin::Hydra)]
    #[case(Origin::Kerberos)]
    #[case(Origin::Styx)]
    #[case(Origin::Gaspra)]
    #[case(Origin::Ida)]
    #[case(Origin::Dactyl)]
    #[case(Origin::Ceres)]
    #[case(Origin::Pallas)]
    #[case(Origin::Vesta)]
    #[case(Origin::Psyche)]
    #[case(Origin::Lutetia)]
    #[case(Origin::Kleopatra)]
    #[case(Origin::Eros)]
    #[case(Origin::Davida)]
    #[case(Origin::Mathilde)]
    #[case(Origin::Steins)]
    #[case(Origin::Braille)]
    #[case(Origin::WilsonHarrington)]
    #[case(Origin::Toutatis)]
    #[case(Origin::Itokawa)]
    #[case(Origin::Bennu)]
    fn test_dyn_origin(#[case] exp: Origin) {
        let act = Origin::try_from(exp as i32).unwrap();
        assert_eq!(act, exp);
        let act = Origin::try_from(exp.id()).unwrap();
        assert_eq!(act, exp);
        let act = Origin::from_str(exp.to_string().as_str()).unwrap();
        assert_eq!(act, exp);
    }

    #[test]
    fn test_dyn_origin_unknown_name() {
        assert_eq!(
            Origin::from_str("Rupert"),
            Err(UnknownOriginName("Rupert".to_string()))
        );
    }

    #[test]
    fn test_dyn_origin_unknown_id() {
        assert_eq!(Origin::try_from(666), Err(UnknownOriginId(666)))
    }

    #[test]
    fn test_try_j4_earth() {
        let earth = Origin::Earth;
        let j4 = earth.try_j4().unwrap();
        assert!(j4 < 0.0); // J4 is negative
        assert!(j4.abs() < 1e-5); // O(1e-6)
    }

    #[test]
    fn test_try_j4_undefined_for_moon() {
        let moon = Origin::Moon;
        assert!(moon.try_j4().is_err());
    }
}
