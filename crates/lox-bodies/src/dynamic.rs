use crate::{NaifId, Origin};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error("no origin with name `{0}` is known")]
pub struct UnknownOriginName(String);

#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error("no origin with NAIF ID `{0}` is known")]
pub struct UnknownOriginId(i32);

#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive, PartialOrd, Ord,
)]
pub enum DynOrigin {
    Sun = 10,

    // Planets.
    Mercury = 199,
    Venus = 299,
    #[default]
    Earth = 399,
    Mars = 499,
    Jupiter = 599,
    Saturn = 699,
    Uranus = 799,
    Neptune = 899,
    Pluto = 999,

    // Barycenters.
    SolarSystemBarycenter = 0,
    MercuryBarycenter = 1,
    VenusBarycenter = 2,
    EarthBarycenter = 3,
    MarsBarycenter = 4,
    JupiterBarycenter = 5,
    SaturnBarycenter = 6,
    UranusBarycenter = 7,
    NeptuneBarycenter = 8,
    PlutoBarycenter = 9,

    // Satellites.
    Moon = 301,
    Phobos = 401,
    Deimos = 402,
    Io = 501,
    Europa = 502,
    Ganymede = 503,
    Callisto = 504,
    Amalthea = 505,
    Himalia = 506,
    Elara = 507,
    Pasiphae = 508,
    Sinope = 509,
    Lysithea = 510,
    Carme = 511,
    Ananke = 512,
    Leda = 513,
    Thebe = 514,
    Adrastea = 515,
    Metis = 516,
    Callirrhoe = 517,
    Themisto = 518,
    Magaclite = 519,
    Taygete = 520,
    Chaldene = 521,
    Harpalyke = 522,
    Kalyke = 523,
    Iocaste = 524,
    Erinome = 525,
    Isonoe = 526,
    Praxidike = 527,
    Autonoe = 528,
    Thyone = 529,
    Hermippe = 530,
    Aitne = 531,
    Eurydome = 532,
    Euanthe = 533,
    Euporie = 534,
    Orthosie = 535,
    Sponde = 536,
    Kale = 537,
    Pasithee = 538,
    Hegemone = 539,
    Mneme = 540,
    Aoede = 541,
    Thelxinoe = 542,
    Arche = 543,
    Kallichore = 544,
    Helike = 545,
    Carpo = 546,
    Eukelade = 547,
    Cyllene = 548,
    Kore = 549,
    Herse = 550,
    Dia = 553,
    Mimas = 601,
    Enceladus = 602,
    Tethys = 603,
    Dione = 604,
    Rhea = 605,
    Titan = 606,
    Hyperion = 607,
    Iapetus = 608,
    Phoebe = 609,
    Janus = 610,
    Epimetheus = 611,
    Helene = 612,
    Telesto = 613,
    Calypso = 614,
    Atlas = 615,
    Prometheus = 616,
    Pandora = 617,
    Pan = 618,
    Ymir = 619,
    Paaliaq = 620,
    Tarvos = 621,
    Ijiraq = 622,
    Suttungr = 623,
    Kiviuq = 624,
    Mundilfari = 625,
    Albiorix = 626,
    Skathi = 627,
    Erriapus = 628,
    Siarnaq = 629,
    Thrymr = 630,
    Narvi = 631,
    Methone = 632,
    Pallene = 633,
    Polydeuces = 634,
    Daphnis = 635,
    Aegir = 636,
    Bebhionn = 637,
    Bergelmir = 638,
    Bestla = 639,
    Farbauti = 640,
    Fenrir = 641,
    Fornjot = 642,
    Hati = 643,
    Hyrrokkin = 644,
    Kari = 645,
    Loge = 646,
    Skoll = 647,
    Surtur = 648,
    Anthe = 649,
    Jarnsaxa = 650,
    Greip = 651,
    Tarqeq = 652,
    Aegaeon = 653,
    Ariel = 701,
    Umbriel = 702,
    Titania = 703,
    Oberon = 704,
    Miranda = 705,
    Cordelia = 706,
    Ophelia = 707,
    Bianca = 708,
    Cressida = 709,
    Desdemona = 710,
    Juliet = 711,
    Portia = 712,
    Rosalind = 713,
    Belinda = 714,
    Puck = 715,
    Caliban = 716,
    Sycorax = 717,
    Prospero = 718,
    Setebos = 719,
    Stephano = 720,
    Trinculo = 721,
    Francisco = 722,
    Margaret = 723,
    Ferdinand = 724,
    Perdita = 725,
    Mab = 726,
    Cupid = 727,
    Triton = 801,
    Nereid = 802,
    Naiad = 803,
    Thalassa = 804,
    Despina = 805,
    Galatea = 806,
    Larissa = 807,
    Proteus = 808,
    Halimede = 809,
    Psamathe = 810,
    Sao = 811,
    Laomedeia = 812,
    Neso = 813,
    Charon = 901,
    Nix = 902,
    Hydra = 903,
    Kerberos = 904,
    Styx = 905,

    // Minor bodies.
    Gaspra = 9511010,
    Ida = 2431010,
    Dactyl = 2431011,
    Ceres = 2000001,
    Pallas = 2000002,
    Vesta = 2000004,
    Psyche = 2000016,
    Lutetia = 2000021,
    Kleopatra = 2000216,
    Eros = 2000433,
    Davida = 2000511,
    Mathilde = 2000253,
    Steins = 2002867,
    Braille = 2009969,
    WilsonHarrington = 2004015,
    Toutatis = 2004179,
    Itokawa = 2025143,
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
