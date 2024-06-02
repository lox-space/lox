use std::{fs, path::Path, process::Command};

use proc_macro2::Ident;
use quote::{format_ident, quote};

use crate::common::{AUTO_GENERATION_NOTICE, COPYRIGHT_NOTICE};

pub struct Frame {
    pub ident: &'static str,
    pub name: &'static str,
    pub abbreviation: &'static str,
}

impl Frame {
    pub fn ident(&self) -> Ident {
        format_ident!("{}", self.ident)
    }
}

pub fn generate_code(frames: &[Frame]) -> String {
    let mut code = quote!();
    let mut tests = quote!();

    let mut match_arms_name = quote! {};
    let mut match_arms_abbreviation = quote! {};

    frames.iter().for_each(|f| {
        let ident = f.ident();
        let test_ident = format_ident!("test_reference_frame_{}", f.abbreviation.to_lowercase());
        let name = f.name;
        let abbreviation = f.abbreviation;
        match_arms_name.extend(quote! {
            PyFrame::#ident => #name.to_string(),
        });
        match_arms_abbreviation.extend(quote! {
            PyFrame::#ident => #abbreviation.to_string(),
        });

        tests.extend(quote! {
            #[test]
            fn #test_ident() {
                assert_eq!(PyFrame::#ident.name(), #name);
                assert_eq!(PyFrame::#ident.abbreviation(), #abbreviation);
            }
        })
    });

    code.extend(quote! {
        impl ReferenceFrame for PyFrame {
            fn name(&self) -> String {
                match self {
                    #match_arms_name
                }
            }

            fn abbreviation(&self) -> String {
                match self {
                    #match_arms_abbreviation
                }
            }
        }
    });

    let module = quote! {
        use crate::frames::ReferenceFrame;
        use crate::python::PyFrame;

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

pub fn generate_frames(dir: &Path) {
    let frames = vec![
        Frame {
            ident: "Icrf",
            name: "International Celestial Reference Frame",
            abbreviation: "ICRF",
        },
        Frame {
            ident: "IauSun",
            name: "IAU Body-Fixed Frame for the Sun",
            abbreviation: "IAU_SUN",
        },
        Frame {
            ident: "IauMercury",
            name: "IAU Body-Fixed Frame for Mercury",
            abbreviation: "IAU_MERCURY",
        },
        Frame {
            ident: "IauVenus",
            name: "IAU Body-Fixed Frame for Venus",
            abbreviation: "IAU_VENUS",
        },
        Frame {
            ident: "IauEarth",
            name: "IAU Body-Fixed Frame for Earth",
            abbreviation: "IAU_EARTH",
        },
        Frame {
            ident: "IauMars",
            name: "IAU Body-Fixed Frame for Mars",
            abbreviation: "IAU_MARS",
        },
        Frame {
            ident: "IauJupiter",
            name: "IAU Body-Fixed Frame for Jupiter",
            abbreviation: "IAU_JUPITER",
        },
        Frame {
            ident: "IauSaturn",
            name: "IAU Body-Fixed Frame for Saturn",
            abbreviation: "IAU_SATURN",
        },
        Frame {
            ident: "IauUranus",
            name: "IAU Body-Fixed Frame for Uranus",
            abbreviation: "IAU_URANUS",
        },
        Frame {
            ident: "IauNeptune",
            name: "IAU Body-Fixed Frame for Neptune",
            abbreviation: "IAU_NEPTUNE",
        },
        Frame {
            ident: "IauPluto",
            name: "IAU Body-Fixed Frame for Pluto",
            abbreviation: "IAU_PLUTO",
        },
        Frame {
            ident: "IauMoon",
            name: "IAU Body-Fixed Frame for the Moon",
            abbreviation: "IAU_MOON",
        },
        Frame {
            ident: "IauPhobos",
            name: "IAU Body-Fixed Frame for Phobos",
            abbreviation: "IAU_PHOBOS",
        },
        Frame {
            ident: "IauDeimos",
            name: "IAU Body-Fixed Frame for Deimos",
            abbreviation: "IAU_DEIMOS",
        },
        Frame {
            ident: "IauIo",
            name: "IAU Body-Fixed Frame for Io",
            abbreviation: "IAU_IO",
        },
        Frame {
            ident: "IauEuropa",
            name: "IAU Body-Fixed Frame for Europa",
            abbreviation: "IAU_EUROPA",
        },
        Frame {
            ident: "IauGanymede",
            name: "IAU Body-Fixed Frame for Ganymede",
            abbreviation: "IAU_GANYMEDE",
        },
        Frame {
            ident: "IauCallisto",
            name: "IAU Body-Fixed Frame for Callisto",
            abbreviation: "IAU_CALLISTO",
        },
        Frame {
            ident: "IauAmalthea",
            name: "IAU Body-Fixed Frame for Amalthea",
            abbreviation: "IAU_AMALTHEA",
        },
        Frame {
            ident: "IauHimalia",
            name: "IAU Body-Fixed Frame for Himalia",
            abbreviation: "IAU_HIMALIA",
        },
        Frame {
            ident: "IauElara",
            name: "IAU Body-Fixed Frame for Elara",
            abbreviation: "IAU_ELARA",
        },
        Frame {
            ident: "IauPasiphae",
            name: "IAU Body-Fixed Frame for Pasiphae",
            abbreviation: "IAU_PASIPHAE",
        },
        Frame {
            ident: "IauSinope",
            name: "IAU Body-Fixed Frame for Sinope",
            abbreviation: "IAU_SINOPE",
        },
        Frame {
            ident: "IauLysithea",
            name: "IAU Body-Fixed Frame for Lysithea",
            abbreviation: "IAU_LYSITHEA",
        },
        Frame {
            ident: "IauCarme",
            name: "IAU Body-Fixed Frame for Carme",
            abbreviation: "IAU_CARME",
        },
        Frame {
            ident: "IauAnanke",
            name: "IAU Body-Fixed Frame for Ananke",
            abbreviation: "IAU_ANANKE",
        },
        Frame {
            ident: "IauLeda",
            name: "IAU Body-Fixed Frame for Leda",
            abbreviation: "IAU_LEDA",
        },
        Frame {
            ident: "IauThebe",
            name: "IAU Body-Fixed Frame for Thebe",
            abbreviation: "IAU_THEBE",
        },
        Frame {
            ident: "IauAdrastea",
            name: "IAU Body-Fixed Frame for Adrastea",
            abbreviation: "IAU_ADRASTEA",
        },
        Frame {
            ident: "IauMetis",
            name: "IAU Body-Fixed Frame for Metis",
            abbreviation: "IAU_METIS",
        },
        Frame {
            ident: "IauCallirrhoe",
            name: "IAU Body-Fixed Frame for Callirrhoe",
            abbreviation: "IAU_CALLIRRHOE",
        },
        Frame {
            ident: "IauThemisto",
            name: "IAU Body-Fixed Frame for Themisto",
            abbreviation: "IAU_THEMISTO",
        },
        Frame {
            ident: "IauMagaclite",
            name: "IAU Body-Fixed Frame for Magaclite",
            abbreviation: "IAU_MAGACLITE",
        },
        Frame {
            ident: "IauTaygete",
            name: "IAU Body-Fixed Frame for Taygete",
            abbreviation: "IAU_TAYGETE",
        },
        Frame {
            ident: "IauChaldene",
            name: "IAU Body-Fixed Frame for Chaldene",
            abbreviation: "IAU_CHALDENE",
        },
        Frame {
            ident: "IauHarpalyke",
            name: "IAU Body-Fixed Frame for Harpalyke",
            abbreviation: "IAU_HARPALYKE",
        },
        Frame {
            ident: "IauKalyke",
            name: "IAU Body-Fixed Frame for Kalyke",
            abbreviation: "IAU_KALYKE",
        },
        Frame {
            ident: "IauIocaste",
            name: "IAU Body-Fixed Frame for Iocaste",
            abbreviation: "IAU_IOCASTE",
        },
        Frame {
            ident: "IauErinome",
            name: "IAU Body-Fixed Frame for Erinome",
            abbreviation: "IAU_ERINOME",
        },
        Frame {
            ident: "IauIsonoe",
            name: "IAU Body-Fixed Frame for Isonoe",
            abbreviation: "IAU_ISONOE",
        },
        Frame {
            ident: "IauPraxidike",
            name: "IAU Body-Fixed Frame for Praxidike",
            abbreviation: "IAU_PRAXIDIKE",
        },
        Frame {
            ident: "IauAutonoe",
            name: "IAU Body-Fixed Frame for Autonoe",
            abbreviation: "IAU_AUTONOE",
        },
        Frame {
            ident: "IauThyone",
            name: "IAU Body-Fixed Frame for Thyone",
            abbreviation: "IAU_THYONE",
        },
        Frame {
            ident: "IauHermippe",
            name: "IAU Body-Fixed Frame for Hermippe",
            abbreviation: "IAU_HERMIPPE",
        },
        Frame {
            ident: "IauAitne",
            name: "IAU Body-Fixed Frame for Aitne",
            abbreviation: "IAU_AITNE",
        },
        Frame {
            ident: "IauEurydome",
            name: "IAU Body-Fixed Frame for Eurydome",
            abbreviation: "IAU_EURYDOME",
        },
        Frame {
            ident: "IauEuanthe",
            name: "IAU Body-Fixed Frame for Euanthe",
            abbreviation: "IAU_EUANTHE",
        },
        Frame {
            ident: "IauEuporie",
            name: "IAU Body-Fixed Frame for Euporie",
            abbreviation: "IAU_EUPORIE",
        },
        Frame {
            ident: "IauOrthosie",
            name: "IAU Body-Fixed Frame for Orthosie",
            abbreviation: "IAU_ORTHOSIE",
        },
        Frame {
            ident: "IauSponde",
            name: "IAU Body-Fixed Frame for Sponde",
            abbreviation: "IAU_SPONDE",
        },
        Frame {
            ident: "IauKale",
            name: "IAU Body-Fixed Frame for Kale",
            abbreviation: "IAU_KALE",
        },
        Frame {
            ident: "IauPasithee",
            name: "IAU Body-Fixed Frame for Pasithee",
            abbreviation: "IAU_PASITHEE",
        },
        Frame {
            ident: "IauHegemone",
            name: "IAU Body-Fixed Frame for Hegemone",
            abbreviation: "IAU_HEGEMONE",
        },
        Frame {
            ident: "IauMneme",
            name: "IAU Body-Fixed Frame for Mneme",
            abbreviation: "IAU_MNEME",
        },
        Frame {
            ident: "IauAoede",
            name: "IAU Body-Fixed Frame for Aoede",
            abbreviation: "IAU_AOEDE",
        },
        Frame {
            ident: "IauThelxinoe",
            name: "IAU Body-Fixed Frame for Thelxinoe",
            abbreviation: "IAU_THELXINOE",
        },
        Frame {
            ident: "IauArche",
            name: "IAU Body-Fixed Frame for Arche",
            abbreviation: "IAU_ARCHE",
        },
        Frame {
            ident: "IauKallichore",
            name: "IAU Body-Fixed Frame for Kallichore",
            abbreviation: "IAU_KALLICHORE",
        },
        Frame {
            ident: "IauHelike",
            name: "IAU Body-Fixed Frame for Helike",
            abbreviation: "IAU_HELIKE",
        },
        Frame {
            ident: "IauCarpo",
            name: "IAU Body-Fixed Frame for Carpo",
            abbreviation: "IAU_CARPO",
        },
        Frame {
            ident: "IauEukelade",
            name: "IAU Body-Fixed Frame for Eukelade",
            abbreviation: "IAU_EUKELADE",
        },
        Frame {
            ident: "IauCyllene",
            name: "IAU Body-Fixed Frame for Cyllene",
            abbreviation: "IAU_CYLLENE",
        },
        Frame {
            ident: "IauKore",
            name: "IAU Body-Fixed Frame for Kore",
            abbreviation: "IAU_KORE",
        },
        Frame {
            ident: "IauHerse",
            name: "IAU Body-Fixed Frame for Herse",
            abbreviation: "IAU_HERSE",
        },
        Frame {
            ident: "IauDia",
            name: "IAU Body-Fixed Frame for Dia",
            abbreviation: "IAU_DIA",
        },
        Frame {
            ident: "IauMimas",
            name: "IAU Body-Fixed Frame for Mimas",
            abbreviation: "IAU_MIMAS",
        },
        Frame {
            ident: "IauEnceladus",
            name: "IAU Body-Fixed Frame for Enceladus",
            abbreviation: "IAU_ENCELADUS",
        },
        Frame {
            ident: "IauTethys",
            name: "IAU Body-Fixed Frame for Tethys",
            abbreviation: "IAU_TETHYS",
        },
        Frame {
            ident: "IauDione",
            name: "IAU Body-Fixed Frame for Dione",
            abbreviation: "IAU_DIONE",
        },
        Frame {
            ident: "IauRhea",
            name: "IAU Body-Fixed Frame for Rhea",
            abbreviation: "IAU_RHEA",
        },
        Frame {
            ident: "IauTitan",
            name: "IAU Body-Fixed Frame for Titan",
            abbreviation: "IAU_TITAN",
        },
        Frame {
            ident: "IauHyperion",
            name: "IAU Body-Fixed Frame for Hyperion",
            abbreviation: "IAU_HYPERION",
        },
        Frame {
            ident: "IauIapetus",
            name: "IAU Body-Fixed Frame for Iapetus",
            abbreviation: "IAU_IAPETUS",
        },
        Frame {
            ident: "IauPhoebe",
            name: "IAU Body-Fixed Frame for Phoebe",
            abbreviation: "IAU_PHOEBE",
        },
        Frame {
            ident: "IauJanus",
            name: "IAU Body-Fixed Frame for Janus",
            abbreviation: "IAU_JANUS",
        },
        Frame {
            ident: "IauEpimetheus",
            name: "IAU Body-Fixed Frame for Epimetheus",
            abbreviation: "IAU_EPIMETHEUS",
        },
        Frame {
            ident: "IauHelene",
            name: "IAU Body-Fixed Frame for Helene",
            abbreviation: "IAU_HELENE",
        },
        Frame {
            ident: "IauTelesto",
            name: "IAU Body-Fixed Frame for Telesto",
            abbreviation: "IAU_TELESTO",
        },
        Frame {
            ident: "IauCalypso",
            name: "IAU Body-Fixed Frame for Calypso",
            abbreviation: "IAU_CALYPSO",
        },
        Frame {
            ident: "IauAtlas",
            name: "IAU Body-Fixed Frame for Atlas",
            abbreviation: "IAU_ATLAS",
        },
        Frame {
            ident: "IauPrometheus",
            name: "IAU Body-Fixed Frame for Prometheus",
            abbreviation: "IAU_PROMETHEUS",
        },
        Frame {
            ident: "IauPandora",
            name: "IAU Body-Fixed Frame for Pandora",
            abbreviation: "IAU_PANDORA",
        },
        Frame {
            ident: "IauPan",
            name: "IAU Body-Fixed Frame for Pan",
            abbreviation: "IAU_PAN",
        },
        Frame {
            ident: "IauYmir",
            name: "IAU Body-Fixed Frame for Ymir",
            abbreviation: "IAU_YMIR",
        },
        Frame {
            ident: "IauPaaliaq",
            name: "IAU Body-Fixed Frame for Paaliaq",
            abbreviation: "IAU_PAALIAQ",
        },
        Frame {
            ident: "IauTarvos",
            name: "IAU Body-Fixed Frame for Tarvos",
            abbreviation: "IAU_TARVOS",
        },
        Frame {
            ident: "IauIjiraq",
            name: "IAU Body-Fixed Frame for Ijiraq",
            abbreviation: "IAU_IJIRAQ",
        },
        Frame {
            ident: "IauSuttungr",
            name: "IAU Body-Fixed Frame for Suttungr",
            abbreviation: "IAU_SUTTUNGR",
        },
        Frame {
            ident: "IauKiviuq",
            name: "IAU Body-Fixed Frame for Kiviuq",
            abbreviation: "IAU_KIVIUQ",
        },
        Frame {
            ident: "IauMundilfari",
            name: "IAU Body-Fixed Frame for Mundilfari",
            abbreviation: "IAU_MUNDILFARI",
        },
        Frame {
            ident: "IauAlbiorix",
            name: "IAU Body-Fixed Frame for Albiorix",
            abbreviation: "IAU_ALBIORIX",
        },
        Frame {
            ident: "IauSkathi",
            name: "IAU Body-Fixed Frame for Skathi",
            abbreviation: "IAU_SKATHI",
        },
        Frame {
            ident: "IauErriapus",
            name: "IAU Body-Fixed Frame for Erriapus",
            abbreviation: "IAU_ERRIAPUS",
        },
        Frame {
            ident: "IauSiarnaq",
            name: "IAU Body-Fixed Frame for Siarnaq",
            abbreviation: "IAU_SIARNAQ",
        },
        Frame {
            ident: "IauThrymr",
            name: "IAU Body-Fixed Frame for Thrymr",
            abbreviation: "IAU_THRYMR",
        },
        Frame {
            ident: "IauNarvi",
            name: "IAU Body-Fixed Frame for Narvi",
            abbreviation: "IAU_NARVI",
        },
        Frame {
            ident: "IauMethone",
            name: "IAU Body-Fixed Frame for Methone",
            abbreviation: "IAU_METHONE",
        },
        Frame {
            ident: "IauPallene",
            name: "IAU Body-Fixed Frame for Pallene",
            abbreviation: "IAU_PALLENE",
        },
        Frame {
            ident: "IauPolydeuces",
            name: "IAU Body-Fixed Frame for Polydeuces",
            abbreviation: "IAU_POLYDEUCES",
        },
        Frame {
            ident: "IauDaphnis",
            name: "IAU Body-Fixed Frame for Daphnis",
            abbreviation: "IAU_DAPHNIS",
        },
        Frame {
            ident: "IauAegir",
            name: "IAU Body-Fixed Frame for Aegir",
            abbreviation: "IAU_AEGIR",
        },
        Frame {
            ident: "IauBebhionn",
            name: "IAU Body-Fixed Frame for Bebhionn",
            abbreviation: "IAU_BEBHIONN",
        },
        Frame {
            ident: "IauBergelmir",
            name: "IAU Body-Fixed Frame for Bergelmir",
            abbreviation: "IAU_BERGELMIR",
        },
        Frame {
            ident: "IauBestla",
            name: "IAU Body-Fixed Frame for Bestla",
            abbreviation: "IAU_BESTLA",
        },
        Frame {
            ident: "IauFarbauti",
            name: "IAU Body-Fixed Frame for Farbauti",
            abbreviation: "IAU_FARBAUTI",
        },
        Frame {
            ident: "IauFenrir",
            name: "IAU Body-Fixed Frame for Fenrir",
            abbreviation: "IAU_FENRIR",
        },
        Frame {
            ident: "IauFornjot",
            name: "IAU Body-Fixed Frame for Fornjot",
            abbreviation: "IAU_FORNJOT",
        },
        Frame {
            ident: "IauHati",
            name: "IAU Body-Fixed Frame for Hati",
            abbreviation: "IAU_HATI",
        },
        Frame {
            ident: "IauHyrrokkin",
            name: "IAU Body-Fixed Frame for Hyrrokkin",
            abbreviation: "IAU_HYRROKKIN",
        },
        Frame {
            ident: "IauKari",
            name: "IAU Body-Fixed Frame for Kari",
            abbreviation: "IAU_KARI",
        },
        Frame {
            ident: "IauLoge",
            name: "IAU Body-Fixed Frame for Loge",
            abbreviation: "IAU_LOGE",
        },
        Frame {
            ident: "IauSkoll",
            name: "IAU Body-Fixed Frame for Skoll",
            abbreviation: "IAU_SKOLL",
        },
        Frame {
            ident: "IauSurtur",
            name: "IAU Body-Fixed Frame for Surtur",
            abbreviation: "IAU_SURTUR",
        },
        Frame {
            ident: "IauAnthe",
            name: "IAU Body-Fixed Frame for Anthe",
            abbreviation: "IAU_ANTHE",
        },
        Frame {
            ident: "IauJarnsaxa",
            name: "IAU Body-Fixed Frame for Jarnsaxa",
            abbreviation: "IAU_JARNSAXA",
        },
        Frame {
            ident: "IauGreip",
            name: "IAU Body-Fixed Frame for Greip",
            abbreviation: "IAU_GREIP",
        },
        Frame {
            ident: "IauTarqeq",
            name: "IAU Body-Fixed Frame for Tarqeq",
            abbreviation: "IAU_TARQEQ",
        },
        Frame {
            ident: "IauAegaeon",
            name: "IAU Body-Fixed Frame for Aegaeon",
            abbreviation: "IAU_AEGAEON",
        },
        Frame {
            ident: "IauAriel",
            name: "IAU Body-Fixed Frame for Ariel",
            abbreviation: "IAU_ARIEL",
        },
        Frame {
            ident: "IauUmbriel",
            name: "IAU Body-Fixed Frame for Umbriel",
            abbreviation: "IAU_UMBRIEL",
        },
        Frame {
            ident: "IauTitania",
            name: "IAU Body-Fixed Frame for Titania",
            abbreviation: "IAU_TITANIA",
        },
        Frame {
            ident: "IauOberon",
            name: "IAU Body-Fixed Frame for Oberon",
            abbreviation: "IAU_OBERON",
        },
        Frame {
            ident: "IauMiranda",
            name: "IAU Body-Fixed Frame for Miranda",
            abbreviation: "IAU_MIRANDA",
        },
        Frame {
            ident: "IauCordelia",
            name: "IAU Body-Fixed Frame for Cordelia",
            abbreviation: "IAU_CORDELIA",
        },
        Frame {
            ident: "IauOphelia",
            name: "IAU Body-Fixed Frame for Ophelia",
            abbreviation: "IAU_OPHELIA",
        },
        Frame {
            ident: "IauBianca",
            name: "IAU Body-Fixed Frame for Bianca",
            abbreviation: "IAU_BIANCA",
        },
        Frame {
            ident: "IauCressida",
            name: "IAU Body-Fixed Frame for Cressida",
            abbreviation: "IAU_CRESSIDA",
        },
        Frame {
            ident: "IauDesdemona",
            name: "IAU Body-Fixed Frame for Desdemona",
            abbreviation: "IAU_DESDEMONA",
        },
        Frame {
            ident: "IauJuliet",
            name: "IAU Body-Fixed Frame for Juliet",
            abbreviation: "IAU_JULIET",
        },
        Frame {
            ident: "IauPortia",
            name: "IAU Body-Fixed Frame for Portia",
            abbreviation: "IAU_PORTIA",
        },
        Frame {
            ident: "IauRosalind",
            name: "IAU Body-Fixed Frame for Rosalind",
            abbreviation: "IAU_ROSALIND",
        },
        Frame {
            ident: "IauBelinda",
            name: "IAU Body-Fixed Frame for Belinda",
            abbreviation: "IAU_BELINDA",
        },
        Frame {
            ident: "IauPuck",
            name: "IAU Body-Fixed Frame for Puck",
            abbreviation: "IAU_PUCK",
        },
        Frame {
            ident: "IauCaliban",
            name: "IAU Body-Fixed Frame for Caliban",
            abbreviation: "IAU_CALIBAN",
        },
        Frame {
            ident: "IauSycorax",
            name: "IAU Body-Fixed Frame for Sycorax",
            abbreviation: "IAU_SYCORAX",
        },
        Frame {
            ident: "IauProspero",
            name: "IAU Body-Fixed Frame for Prospero",
            abbreviation: "IAU_PROSPERO",
        },
        Frame {
            ident: "IauSetebos",
            name: "IAU Body-Fixed Frame for Setebos",
            abbreviation: "IAU_SETEBOS",
        },
        Frame {
            ident: "IauStephano",
            name: "IAU Body-Fixed Frame for Stephano",
            abbreviation: "IAU_STEPHANO",
        },
        Frame {
            ident: "IauTrinculo",
            name: "IAU Body-Fixed Frame for Trinculo",
            abbreviation: "IAU_TRINCULO",
        },
        Frame {
            ident: "IauFrancisco",
            name: "IAU Body-Fixed Frame for Francisco",
            abbreviation: "IAU_FRANCISCO",
        },
        Frame {
            ident: "IauMargaret",
            name: "IAU Body-Fixed Frame for Margaret",
            abbreviation: "IAU_MARGARET",
        },
        Frame {
            ident: "IauFerdinand",
            name: "IAU Body-Fixed Frame for Ferdinand",
            abbreviation: "IAU_FERDINAND",
        },
        Frame {
            ident: "IauPerdita",
            name: "IAU Body-Fixed Frame for Perdita",
            abbreviation: "IAU_PERDITA",
        },
        Frame {
            ident: "IauMab",
            name: "IAU Body-Fixed Frame for Mab",
            abbreviation: "IAU_MAB",
        },
        Frame {
            ident: "IauCupid",
            name: "IAU Body-Fixed Frame for Cupid",
            abbreviation: "IAU_CUPID",
        },
        Frame {
            ident: "IauTriton",
            name: "IAU Body-Fixed Frame for Triton",
            abbreviation: "IAU_TRITON",
        },
        Frame {
            ident: "IauNereid",
            name: "IAU Body-Fixed Frame for Nereid",
            abbreviation: "IAU_NEREID",
        },
        Frame {
            ident: "IauNaiad",
            name: "IAU Body-Fixed Frame for Naiad",
            abbreviation: "IAU_NAIAD",
        },
        Frame {
            ident: "IauThalassa",
            name: "IAU Body-Fixed Frame for Thalassa",
            abbreviation: "IAU_THALASSA",
        },
        Frame {
            ident: "IauDespina",
            name: "IAU Body-Fixed Frame for Despina",
            abbreviation: "IAU_DESPINA",
        },
        Frame {
            ident: "IauGalatea",
            name: "IAU Body-Fixed Frame for Galatea",
            abbreviation: "IAU_GALATEA",
        },
        Frame {
            ident: "IauLarissa",
            name: "IAU Body-Fixed Frame for Larissa",
            abbreviation: "IAU_LARISSA",
        },
        Frame {
            ident: "IauProteus",
            name: "IAU Body-Fixed Frame for Proteus",
            abbreviation: "IAU_PROTEUS",
        },
        Frame {
            ident: "IauHalimede",
            name: "IAU Body-Fixed Frame for Halimede",
            abbreviation: "IAU_HALIMEDE",
        },
        Frame {
            ident: "IauPsamathe",
            name: "IAU Body-Fixed Frame for Psamathe",
            abbreviation: "IAU_PSAMATHE",
        },
        Frame {
            ident: "IauSao",
            name: "IAU Body-Fixed Frame for Sao",
            abbreviation: "IAU_SAO",
        },
        Frame {
            ident: "IauLaomedeia",
            name: "IAU Body-Fixed Frame for Laomedeia",
            abbreviation: "IAU_LAOMEDEIA",
        },
        Frame {
            ident: "IauNeso",
            name: "IAU Body-Fixed Frame for Neso",
            abbreviation: "IAU_NESO",
        },
        Frame {
            ident: "IauCharon",
            name: "IAU Body-Fixed Frame for Charon",
            abbreviation: "IAU_CHARON",
        },
        Frame {
            ident: "IauNix",
            name: "IAU Body-Fixed Frame for Nix",
            abbreviation: "IAU_NIX",
        },
        Frame {
            ident: "IauHydra",
            name: "IAU Body-Fixed Frame for Hydra",
            abbreviation: "IAU_HYDRA",
        },
        Frame {
            ident: "IauKerberos",
            name: "IAU Body-Fixed Frame for Kerberos",
            abbreviation: "IAU_KERBEROS",
        },
        Frame {
            ident: "IauStyx",
            name: "IAU Body-Fixed Frame for Styx",
            abbreviation: "IAU_STYX",
        },
        Frame {
            ident: "IauGaspra",
            name: "IAU Body-Fixed Frame for Gaspra",
            abbreviation: "IAU_GASPRA",
        },
        Frame {
            ident: "IauIda",
            name: "IAU Body-Fixed Frame for Ida",
            abbreviation: "IAU_IDA",
        },
        Frame {
            ident: "IauDactyl",
            name: "IAU Body-Fixed Frame for Dactyl",
            abbreviation: "IAU_DACTYL",
        },
        Frame {
            ident: "IauCeres",
            name: "IAU Body-Fixed Frame for Ceres",
            abbreviation: "IAU_CERES",
        },
        Frame {
            ident: "IauPallas",
            name: "IAU Body-Fixed Frame for Pallas",
            abbreviation: "IAU_PALLAS",
        },
        Frame {
            ident: "IauVesta",
            name: "IAU Body-Fixed Frame for Vesta",
            abbreviation: "IAU_VESTA",
        },
        Frame {
            ident: "IauPsyche",
            name: "IAU Body-Fixed Frame for Psyche",
            abbreviation: "IAU_PSYCHE",
        },
        Frame {
            ident: "IauLutetia",
            name: "IAU Body-Fixed Frame for Lutetia",
            abbreviation: "IAU_LUTETIA",
        },
        Frame {
            ident: "IauKleopatra",
            name: "IAU Body-Fixed Frame for Kleopatra",
            abbreviation: "IAU_KLEOPATRA",
        },
        Frame {
            ident: "IauEros",
            name: "IAU Body-Fixed Frame for Eros",
            abbreviation: "IAU_EROS",
        },
        Frame {
            ident: "IauDavida",
            name: "IAU Body-Fixed Frame for Davida",
            abbreviation: "IAU_DAVIDA",
        },
        Frame {
            ident: "IauMathilde",
            name: "IAU Body-Fixed Frame for Mathilde",
            abbreviation: "IAU_MATHILDE",
        },
        Frame {
            ident: "IauSteins",
            name: "IAU Body-Fixed Frame for Steins",
            abbreviation: "IAU_STEINS",
        },
        Frame {
            ident: "IauBraille",
            name: "IAU Body-Fixed Frame for Braille",
            abbreviation: "IAU_BRAILLE",
        },
        Frame {
            ident: "IauWilsonHarrington",
            name: "IAU Body-Fixed Frame for Wilson-Harrington",
            abbreviation: "IAU_WILSON_HARRINGTON",
        },
        Frame {
            ident: "IauToutatis",
            name: "IAU Body-Fixed Frame for Toutatis",
            abbreviation: "IAU_TOUTATIS",
        },
        Frame {
            ident: "IauItokawa",
            name: "IAU Body-Fixed Frame for Itokawa",
            abbreviation: "IAU_ITOKAWA",
        },
        Frame {
            ident: "IauBennu",
            name: "IAU Body-Fixed Frame for Bennu",
            abbreviation: "IAU_BENNU",
        },
    ];

    let mut code = String::from(COPYRIGHT_NOTICE);
    code.push_str(AUTO_GENERATION_NOTICE);
    code.push_str(&generate_code(&frames));

    let out = dir.join("generated.rs");
    fs::write(&out, code).expect("file should be writeable");

    Command::new("rustfmt")
        .args([out.to_str().unwrap()])
        .status()
        .expect("formatting should work");
}
