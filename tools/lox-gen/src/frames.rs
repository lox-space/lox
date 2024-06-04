use std::{fs, path::Path, process::Command};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::common::{AUTO_GENERATION_NOTICE, COPYRIGHT_NOTICE};

pub struct Frame {
    pub ident: &'static str,
    pub name: &'static str,
    pub abbreviation: &'static str,
    pub body: Option<&'static str>,
}

impl Frame {
    pub fn ident(&self) -> Ident {
        format_ident!("{}", self.ident)
    }
}

fn generate_transform(target: &Frame, frames: &[Frame], code: &mut TokenStream) {
    let target_type = if let Some(body) = target.body {
        let ident = format_ident!("{}", body);
        quote!(BodyFixed<#ident>)
    } else {
        let ident = target.ident();
        quote!(#ident)
    };
    code.extend(quote! {
        impl TryToFrame<PyTime, PyBody, #target_type, NoOpFrameTransformationProvider, PyErr> for PyState {
            fn try_to_frame(&self, frame: #target_type, _: &NoOpFrameTransformationProvider) -> PyResult<State<PyTime, PyBody, #target_type>> {
                todo!()
            }
        }
    })
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

        generate_transform(f, &[], &mut code);

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
        use lox_bodies::*;
        use lox_bodies::python::PyBody;
        use lox_time::python::time::PyTime;
        use pyo3::{PyErr, PyResult};
        use crate::frames::{BodyFixed, Icrf, NoOpFrameTransformationProvider, ReferenceFrame};
        use crate::python::{PyFrame, PyState};
        use crate::states::{State, TryToFrame};

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
            body: None,
        },
        Frame {
            ident: "IauSun",
            name: "IAU Body-Fixed Frame for the Sun",
            abbreviation: "IAU_SUN",
            body: Some("Sun"),
        },
        Frame {
            ident: "IauMercury",
            name: "IAU Body-Fixed Frame for Mercury",
            abbreviation: "IAU_MERCURY",
            body: Some("Mercury"),
        },
        Frame {
            ident: "IauVenus",
            name: "IAU Body-Fixed Frame for Venus",
            abbreviation: "IAU_VENUS",
            body: Some("Venus"),
        },
        Frame {
            ident: "IauEarth",
            name: "IAU Body-Fixed Frame for Earth",
            abbreviation: "IAU_EARTH",
            body: Some("Earth"),
        },
        Frame {
            ident: "IauMars",
            name: "IAU Body-Fixed Frame for Mars",
            abbreviation: "IAU_MARS",
            body: Some("Mars"),
        },
        Frame {
            ident: "IauJupiter",
            name: "IAU Body-Fixed Frame for Jupiter",
            abbreviation: "IAU_JUPITER",
            body: Some("Jupiter"),
        },
        Frame {
            ident: "IauSaturn",
            name: "IAU Body-Fixed Frame for Saturn",
            abbreviation: "IAU_SATURN",
            body: Some("Saturn"),
        },
        Frame {
            ident: "IauUranus",
            name: "IAU Body-Fixed Frame for Uranus",
            abbreviation: "IAU_URANUS",
            body: Some("Uranus"),
        },
        Frame {
            ident: "IauNeptune",
            name: "IAU Body-Fixed Frame for Neptune",
            abbreviation: "IAU_NEPTUNE",
            body: Some("Neptune"),
        },
        Frame {
            ident: "IauPluto",
            name: "IAU Body-Fixed Frame for Pluto",
            abbreviation: "IAU_PLUTO",
            body: Some("Pluto"),
        },
        Frame {
            ident: "IauMoon",
            name: "IAU Body-Fixed Frame for the Moon",
            abbreviation: "IAU_MOON",
            body: Some("Moon"),
        },
        Frame {
            ident: "IauPhobos",
            name: "IAU Body-Fixed Frame for Phobos",
            abbreviation: "IAU_PHOBOS",
            body: Some("Phobos"),
        },
        Frame {
            ident: "IauDeimos",
            name: "IAU Body-Fixed Frame for Deimos",
            abbreviation: "IAU_DEIMOS",
            body: Some("Deimos"),
        },
        Frame {
            ident: "IauIo",
            name: "IAU Body-Fixed Frame for Io",
            abbreviation: "IAU_IO",
            body: Some("Io"),
        },
        Frame {
            ident: "IauEuropa",
            name: "IAU Body-Fixed Frame for Europa",
            abbreviation: "IAU_EUROPA",
            body: Some("Europa"),
        },
        Frame {
            ident: "IauGanymede",
            name: "IAU Body-Fixed Frame for Ganymede",
            abbreviation: "IAU_GANYMEDE",
            body: Some("Ganymede"),
        },
        Frame {
            ident: "IauCallisto",
            name: "IAU Body-Fixed Frame for Callisto",
            abbreviation: "IAU_CALLISTO",
            body: Some("Callisto"),
        },
        Frame {
            ident: "IauAmalthea",
            name: "IAU Body-Fixed Frame for Amalthea",
            abbreviation: "IAU_AMALTHEA",
            body: Some("Amalthea"),
        },
        Frame {
            ident: "IauHimalia",
            name: "IAU Body-Fixed Frame for Himalia",
            abbreviation: "IAU_HIMALIA",
            body: Some("Himalia"),
        },
        Frame {
            ident: "IauElara",
            name: "IAU Body-Fixed Frame for Elara",
            abbreviation: "IAU_ELARA",
            body: Some("Elara"),
        },
        Frame {
            ident: "IauPasiphae",
            name: "IAU Body-Fixed Frame for Pasiphae",
            abbreviation: "IAU_PASIPHAE",
            body: Some("Pasiphae"),
        },
        Frame {
            ident: "IauSinope",
            name: "IAU Body-Fixed Frame for Sinope",
            abbreviation: "IAU_SINOPE",
            body: Some("Sinope"),
        },
        Frame {
            ident: "IauLysithea",
            name: "IAU Body-Fixed Frame for Lysithea",
            abbreviation: "IAU_LYSITHEA",
            body: Some("Lysithea"),
        },
        Frame {
            ident: "IauCarme",
            name: "IAU Body-Fixed Frame for Carme",
            abbreviation: "IAU_CARME",
            body: Some("Carme"),
        },
        Frame {
            ident: "IauAnanke",
            name: "IAU Body-Fixed Frame for Ananke",
            abbreviation: "IAU_ANANKE",
            body: Some("Ananke"),
        },
        Frame {
            ident: "IauLeda",
            name: "IAU Body-Fixed Frame for Leda",
            abbreviation: "IAU_LEDA",
            body: Some("Leda"),
        },
        Frame {
            ident: "IauThebe",
            name: "IAU Body-Fixed Frame for Thebe",
            abbreviation: "IAU_THEBE",
            body: Some("Thebe"),
        },
        Frame {
            ident: "IauAdrastea",
            name: "IAU Body-Fixed Frame for Adrastea",
            abbreviation: "IAU_ADRASTEA",
            body: Some("Adrastea"),
        },
        Frame {
            ident: "IauMetis",
            name: "IAU Body-Fixed Frame for Metis",
            abbreviation: "IAU_METIS",
            body: Some("Metis"),
        },
        Frame {
            ident: "IauCallirrhoe",
            name: "IAU Body-Fixed Frame for Callirrhoe",
            abbreviation: "IAU_CALLIRRHOE",
            body: Some("Callirrhoe"),
        },
        Frame {
            ident: "IauThemisto",
            name: "IAU Body-Fixed Frame for Themisto",
            abbreviation: "IAU_THEMISTO",
            body: Some("Themisto"),
        },
        Frame {
            ident: "IauMagaclite",
            name: "IAU Body-Fixed Frame for Magaclite",
            abbreviation: "IAU_MAGACLITE",
            body: Some("Magaclite"),
        },
        Frame {
            ident: "IauTaygete",
            name: "IAU Body-Fixed Frame for Taygete",
            abbreviation: "IAU_TAYGETE",
            body: Some("Taygete"),
        },
        Frame {
            ident: "IauChaldene",
            name: "IAU Body-Fixed Frame for Chaldene",
            abbreviation: "IAU_CHALDENE",
            body: Some("Chaldene"),
        },
        Frame {
            ident: "IauHarpalyke",
            name: "IAU Body-Fixed Frame for Harpalyke",
            abbreviation: "IAU_HARPALYKE",
            body: Some("Harpalyke"),
        },
        Frame {
            ident: "IauKalyke",
            name: "IAU Body-Fixed Frame for Kalyke",
            abbreviation: "IAU_KALYKE",
            body: Some("Kalyke"),
        },
        Frame {
            ident: "IauIocaste",
            name: "IAU Body-Fixed Frame for Iocaste",
            abbreviation: "IAU_IOCASTE",
            body: Some("Iocaste"),
        },
        Frame {
            ident: "IauErinome",
            name: "IAU Body-Fixed Frame for Erinome",
            abbreviation: "IAU_ERINOME",
            body: Some("Erinome"),
        },
        Frame {
            ident: "IauIsonoe",
            name: "IAU Body-Fixed Frame for Isonoe",
            abbreviation: "IAU_ISONOE",
            body: Some("Isonoe"),
        },
        Frame {
            ident: "IauPraxidike",
            name: "IAU Body-Fixed Frame for Praxidike",
            abbreviation: "IAU_PRAXIDIKE",
            body: Some("Praxidike"),
        },
        Frame {
            ident: "IauAutonoe",
            name: "IAU Body-Fixed Frame for Autonoe",
            abbreviation: "IAU_AUTONOE",
            body: Some("Autonoe"),
        },
        Frame {
            ident: "IauThyone",
            name: "IAU Body-Fixed Frame for Thyone",
            abbreviation: "IAU_THYONE",
            body: Some("Thyone"),
        },
        Frame {
            ident: "IauHermippe",
            name: "IAU Body-Fixed Frame for Hermippe",
            abbreviation: "IAU_HERMIPPE",
            body: Some("Hermippe"),
        },
        Frame {
            ident: "IauAitne",
            name: "IAU Body-Fixed Frame for Aitne",
            abbreviation: "IAU_AITNE",
            body: Some("Aitne"),
        },
        Frame {
            ident: "IauEurydome",
            name: "IAU Body-Fixed Frame for Eurydome",
            abbreviation: "IAU_EURYDOME",
            body: Some("Eurydome"),
        },
        Frame {
            ident: "IauEuanthe",
            name: "IAU Body-Fixed Frame for Euanthe",
            abbreviation: "IAU_EUANTHE",
            body: Some("Euanthe"),
        },
        Frame {
            ident: "IauEuporie",
            name: "IAU Body-Fixed Frame for Euporie",
            abbreviation: "IAU_EUPORIE",
            body: Some("Euporie"),
        },
        Frame {
            ident: "IauOrthosie",
            name: "IAU Body-Fixed Frame for Orthosie",
            abbreviation: "IAU_ORTHOSIE",
            body: Some("Orthosie"),
        },
        Frame {
            ident: "IauSponde",
            name: "IAU Body-Fixed Frame for Sponde",
            abbreviation: "IAU_SPONDE",
            body: Some("Sponde"),
        },
        Frame {
            ident: "IauKale",
            name: "IAU Body-Fixed Frame for Kale",
            abbreviation: "IAU_KALE",
            body: Some("Kale"),
        },
        Frame {
            ident: "IauPasithee",
            name: "IAU Body-Fixed Frame for Pasithee",
            abbreviation: "IAU_PASITHEE",
            body: Some("Pasithee"),
        },
        Frame {
            ident: "IauHegemone",
            name: "IAU Body-Fixed Frame for Hegemone",
            abbreviation: "IAU_HEGEMONE",
            body: Some("Hegemone"),
        },
        Frame {
            ident: "IauMneme",
            name: "IAU Body-Fixed Frame for Mneme",
            abbreviation: "IAU_MNEME",
            body: Some("Mneme"),
        },
        Frame {
            ident: "IauAoede",
            name: "IAU Body-Fixed Frame for Aoede",
            abbreviation: "IAU_AOEDE",
            body: Some("Aoede"),
        },
        Frame {
            ident: "IauThelxinoe",
            name: "IAU Body-Fixed Frame for Thelxinoe",
            abbreviation: "IAU_THELXINOE",
            body: Some("Thelxinoe"),
        },
        Frame {
            ident: "IauArche",
            name: "IAU Body-Fixed Frame for Arche",
            abbreviation: "IAU_ARCHE",
            body: Some("Arche"),
        },
        Frame {
            ident: "IauKallichore",
            name: "IAU Body-Fixed Frame for Kallichore",
            abbreviation: "IAU_KALLICHORE",
            body: Some("Kallichore"),
        },
        Frame {
            ident: "IauHelike",
            name: "IAU Body-Fixed Frame for Helike",
            abbreviation: "IAU_HELIKE",
            body: Some("Helike"),
        },
        Frame {
            ident: "IauCarpo",
            name: "IAU Body-Fixed Frame for Carpo",
            abbreviation: "IAU_CARPO",
            body: Some("Carpo"),
        },
        Frame {
            ident: "IauEukelade",
            name: "IAU Body-Fixed Frame for Eukelade",
            abbreviation: "IAU_EUKELADE",
            body: Some("Eukelade"),
        },
        Frame {
            ident: "IauCyllene",
            name: "IAU Body-Fixed Frame for Cyllene",
            abbreviation: "IAU_CYLLENE",
            body: Some("Cyllene"),
        },
        Frame {
            ident: "IauKore",
            name: "IAU Body-Fixed Frame for Kore",
            abbreviation: "IAU_KORE",
            body: Some("Kore"),
        },
        Frame {
            ident: "IauHerse",
            name: "IAU Body-Fixed Frame for Herse",
            abbreviation: "IAU_HERSE",
            body: Some("Herse"),
        },
        Frame {
            ident: "IauDia",
            name: "IAU Body-Fixed Frame for Dia",
            abbreviation: "IAU_DIA",
            body: Some("Dia"),
        },
        Frame {
            ident: "IauMimas",
            name: "IAU Body-Fixed Frame for Mimas",
            abbreviation: "IAU_MIMAS",
            body: Some("Mimas"),
        },
        Frame {
            ident: "IauEnceladus",
            name: "IAU Body-Fixed Frame for Enceladus",
            abbreviation: "IAU_ENCELADUS",
            body: Some("Enceladus"),
        },
        Frame {
            ident: "IauTethys",
            name: "IAU Body-Fixed Frame for Tethys",
            abbreviation: "IAU_TETHYS",
            body: Some("Tethys"),
        },
        Frame {
            ident: "IauDione",
            name: "IAU Body-Fixed Frame for Dione",
            abbreviation: "IAU_DIONE",
            body: Some("Dione"),
        },
        Frame {
            ident: "IauRhea",
            name: "IAU Body-Fixed Frame for Rhea",
            abbreviation: "IAU_RHEA",
            body: Some("Rhea"),
        },
        Frame {
            ident: "IauTitan",
            name: "IAU Body-Fixed Frame for Titan",
            abbreviation: "IAU_TITAN",
            body: Some("Titan"),
        },
        Frame {
            ident: "IauHyperion",
            name: "IAU Body-Fixed Frame for Hyperion",
            abbreviation: "IAU_HYPERION",
            body: Some("Hyperion"),
        },
        Frame {
            ident: "IauIapetus",
            name: "IAU Body-Fixed Frame for Iapetus",
            abbreviation: "IAU_IAPETUS",
            body: Some("Iapetus"),
        },
        Frame {
            ident: "IauPhoebe",
            name: "IAU Body-Fixed Frame for Phoebe",
            abbreviation: "IAU_PHOEBE",
            body: Some("Phoebe"),
        },
        Frame {
            ident: "IauJanus",
            name: "IAU Body-Fixed Frame for Janus",
            abbreviation: "IAU_JANUS",
            body: Some("Janus"),
        },
        Frame {
            ident: "IauEpimetheus",
            name: "IAU Body-Fixed Frame for Epimetheus",
            abbreviation: "IAU_EPIMETHEUS",
            body: Some("Epimetheus"),
        },
        Frame {
            ident: "IauHelene",
            name: "IAU Body-Fixed Frame for Helene",
            abbreviation: "IAU_HELENE",
            body: Some("Helene"),
        },
        Frame {
            ident: "IauTelesto",
            name: "IAU Body-Fixed Frame for Telesto",
            abbreviation: "IAU_TELESTO",
            body: Some("Telesto"),
        },
        Frame {
            ident: "IauCalypso",
            name: "IAU Body-Fixed Frame for Calypso",
            abbreviation: "IAU_CALYPSO",
            body: Some("Calypso"),
        },
        Frame {
            ident: "IauAtlas",
            name: "IAU Body-Fixed Frame for Atlas",
            abbreviation: "IAU_ATLAS",
            body: Some("Atlas"),
        },
        Frame {
            ident: "IauPrometheus",
            name: "IAU Body-Fixed Frame for Prometheus",
            abbreviation: "IAU_PROMETHEUS",
            body: Some("Prometheus"),
        },
        Frame {
            ident: "IauPandora",
            name: "IAU Body-Fixed Frame for Pandora",
            abbreviation: "IAU_PANDORA",
            body: Some("Pandora"),
        },
        Frame {
            ident: "IauPan",
            name: "IAU Body-Fixed Frame for Pan",
            abbreviation: "IAU_PAN",
            body: Some("Pan"),
        },
        Frame {
            ident: "IauYmir",
            name: "IAU Body-Fixed Frame for Ymir",
            abbreviation: "IAU_YMIR",
            body: Some("Ymir"),
        },
        Frame {
            ident: "IauPaaliaq",
            name: "IAU Body-Fixed Frame for Paaliaq",
            abbreviation: "IAU_PAALIAQ",
            body: Some("Paaliaq"),
        },
        Frame {
            ident: "IauTarvos",
            name: "IAU Body-Fixed Frame for Tarvos",
            abbreviation: "IAU_TARVOS",
            body: Some("Tarvos"),
        },
        Frame {
            ident: "IauIjiraq",
            name: "IAU Body-Fixed Frame for Ijiraq",
            abbreviation: "IAU_IJIRAQ",
            body: Some("Ijiraq"),
        },
        Frame {
            ident: "IauSuttungr",
            name: "IAU Body-Fixed Frame for Suttungr",
            abbreviation: "IAU_SUTTUNGR",
            body: Some("Suttungr"),
        },
        Frame {
            ident: "IauKiviuq",
            name: "IAU Body-Fixed Frame for Kiviuq",
            abbreviation: "IAU_KIVIUQ",
            body: Some("Kiviuq"),
        },
        Frame {
            ident: "IauMundilfari",
            name: "IAU Body-Fixed Frame for Mundilfari",
            abbreviation: "IAU_MUNDILFARI",
            body: Some("Mundilfari"),
        },
        Frame {
            ident: "IauAlbiorix",
            name: "IAU Body-Fixed Frame for Albiorix",
            abbreviation: "IAU_ALBIORIX",
            body: Some("Albiorix"),
        },
        Frame {
            ident: "IauSkathi",
            name: "IAU Body-Fixed Frame for Skathi",
            abbreviation: "IAU_SKATHI",
            body: Some("Skathi"),
        },
        Frame {
            ident: "IauErriapus",
            name: "IAU Body-Fixed Frame for Erriapus",
            abbreviation: "IAU_ERRIAPUS",
            body: Some("Erriapus"),
        },
        Frame {
            ident: "IauSiarnaq",
            name: "IAU Body-Fixed Frame for Siarnaq",
            abbreviation: "IAU_SIARNAQ",
            body: Some("Siarnaq"),
        },
        Frame {
            ident: "IauThrymr",
            name: "IAU Body-Fixed Frame for Thrymr",
            abbreviation: "IAU_THRYMR",
            body: Some("Thrymr"),
        },
        Frame {
            ident: "IauNarvi",
            name: "IAU Body-Fixed Frame for Narvi",
            abbreviation: "IAU_NARVI",
            body: Some("Narvi"),
        },
        Frame {
            ident: "IauMethone",
            name: "IAU Body-Fixed Frame for Methone",
            abbreviation: "IAU_METHONE",
            body: Some("Methone"),
        },
        Frame {
            ident: "IauPallene",
            name: "IAU Body-Fixed Frame for Pallene",
            abbreviation: "IAU_PALLENE",
            body: Some("Pallene"),
        },
        Frame {
            ident: "IauPolydeuces",
            name: "IAU Body-Fixed Frame for Polydeuces",
            abbreviation: "IAU_POLYDEUCES",
            body: Some("Polydeuces"),
        },
        Frame {
            ident: "IauDaphnis",
            name: "IAU Body-Fixed Frame for Daphnis",
            abbreviation: "IAU_DAPHNIS",
            body: Some("Daphnis"),
        },
        Frame {
            ident: "IauAegir",
            name: "IAU Body-Fixed Frame for Aegir",
            abbreviation: "IAU_AEGIR",
            body: Some("Aegir"),
        },
        Frame {
            ident: "IauBebhionn",
            name: "IAU Body-Fixed Frame for Bebhionn",
            abbreviation: "IAU_BEBHIONN",
            body: Some("Bebhionn"),
        },
        Frame {
            ident: "IauBergelmir",
            name: "IAU Body-Fixed Frame for Bergelmir",
            abbreviation: "IAU_BERGELMIR",
            body: Some("Bergelmir"),
        },
        Frame {
            ident: "IauBestla",
            name: "IAU Body-Fixed Frame for Bestla",
            abbreviation: "IAU_BESTLA",
            body: Some("Bestla"),
        },
        Frame {
            ident: "IauFarbauti",
            name: "IAU Body-Fixed Frame for Farbauti",
            abbreviation: "IAU_FARBAUTI",
            body: Some("Farbauti"),
        },
        Frame {
            ident: "IauFenrir",
            name: "IAU Body-Fixed Frame for Fenrir",
            abbreviation: "IAU_FENRIR",
            body: Some("Fenrir"),
        },
        Frame {
            ident: "IauFornjot",
            name: "IAU Body-Fixed Frame for Fornjot",
            abbreviation: "IAU_FORNJOT",
            body: Some("Fornjot"),
        },
        Frame {
            ident: "IauHati",
            name: "IAU Body-Fixed Frame for Hati",
            abbreviation: "IAU_HATI",
            body: Some("Hati"),
        },
        Frame {
            ident: "IauHyrrokkin",
            name: "IAU Body-Fixed Frame for Hyrrokkin",
            abbreviation: "IAU_HYRROKKIN",
            body: Some("Hyrrokkin"),
        },
        Frame {
            ident: "IauKari",
            name: "IAU Body-Fixed Frame for Kari",
            abbreviation: "IAU_KARI",
            body: Some("Kari"),
        },
        Frame {
            ident: "IauLoge",
            name: "IAU Body-Fixed Frame for Loge",
            abbreviation: "IAU_LOGE",
            body: Some("Loge"),
        },
        Frame {
            ident: "IauSkoll",
            name: "IAU Body-Fixed Frame for Skoll",
            abbreviation: "IAU_SKOLL",
            body: Some("Skoll"),
        },
        Frame {
            ident: "IauSurtur",
            name: "IAU Body-Fixed Frame for Surtur",
            abbreviation: "IAU_SURTUR",
            body: Some("Surtur"),
        },
        Frame {
            ident: "IauAnthe",
            name: "IAU Body-Fixed Frame for Anthe",
            abbreviation: "IAU_ANTHE",
            body: Some("Anthe"),
        },
        Frame {
            ident: "IauJarnsaxa",
            name: "IAU Body-Fixed Frame for Jarnsaxa",
            abbreviation: "IAU_JARNSAXA",
            body: Some("Jarnsaxa"),
        },
        Frame {
            ident: "IauGreip",
            name: "IAU Body-Fixed Frame for Greip",
            abbreviation: "IAU_GREIP",
            body: Some("Greip"),
        },
        Frame {
            ident: "IauTarqeq",
            name: "IAU Body-Fixed Frame for Tarqeq",
            abbreviation: "IAU_TARQEQ",
            body: Some("Tarqeq"),
        },
        Frame {
            ident: "IauAegaeon",
            name: "IAU Body-Fixed Frame for Aegaeon",
            abbreviation: "IAU_AEGAEON",
            body: Some("Aegaeon"),
        },
        Frame {
            ident: "IauAriel",
            name: "IAU Body-Fixed Frame for Ariel",
            abbreviation: "IAU_ARIEL",
            body: Some("Ariel"),
        },
        Frame {
            ident: "IauUmbriel",
            name: "IAU Body-Fixed Frame for Umbriel",
            abbreviation: "IAU_UMBRIEL",
            body: Some("Umbriel"),
        },
        Frame {
            ident: "IauTitania",
            name: "IAU Body-Fixed Frame for Titania",
            abbreviation: "IAU_TITANIA",
            body: Some("Titania"),
        },
        Frame {
            ident: "IauOberon",
            name: "IAU Body-Fixed Frame for Oberon",
            abbreviation: "IAU_OBERON",
            body: Some("Oberon"),
        },
        Frame {
            ident: "IauMiranda",
            name: "IAU Body-Fixed Frame for Miranda",
            abbreviation: "IAU_MIRANDA",
            body: Some("Miranda"),
        },
        Frame {
            ident: "IauCordelia",
            name: "IAU Body-Fixed Frame for Cordelia",
            abbreviation: "IAU_CORDELIA",
            body: Some("Cordelia"),
        },
        Frame {
            ident: "IauOphelia",
            name: "IAU Body-Fixed Frame for Ophelia",
            abbreviation: "IAU_OPHELIA",
            body: Some("Ophelia"),
        },
        Frame {
            ident: "IauBianca",
            name: "IAU Body-Fixed Frame for Bianca",
            abbreviation: "IAU_BIANCA",
            body: Some("Bianca"),
        },
        Frame {
            ident: "IauCressida",
            name: "IAU Body-Fixed Frame for Cressida",
            abbreviation: "IAU_CRESSIDA",
            body: Some("Cressida"),
        },
        Frame {
            ident: "IauDesdemona",
            name: "IAU Body-Fixed Frame for Desdemona",
            abbreviation: "IAU_DESDEMONA",
            body: Some("Desdemona"),
        },
        Frame {
            ident: "IauJuliet",
            name: "IAU Body-Fixed Frame for Juliet",
            abbreviation: "IAU_JULIET",
            body: Some("Juliet"),
        },
        Frame {
            ident: "IauPortia",
            name: "IAU Body-Fixed Frame for Portia",
            abbreviation: "IAU_PORTIA",
            body: Some("Portia"),
        },
        Frame {
            ident: "IauRosalind",
            name: "IAU Body-Fixed Frame for Rosalind",
            abbreviation: "IAU_ROSALIND",
            body: Some("Rosalind"),
        },
        Frame {
            ident: "IauBelinda",
            name: "IAU Body-Fixed Frame for Belinda",
            abbreviation: "IAU_BELINDA",
            body: Some("Belinda"),
        },
        Frame {
            ident: "IauPuck",
            name: "IAU Body-Fixed Frame for Puck",
            abbreviation: "IAU_PUCK",
            body: Some("Puck"),
        },
        Frame {
            ident: "IauCaliban",
            name: "IAU Body-Fixed Frame for Caliban",
            abbreviation: "IAU_CALIBAN",
            body: Some("Caliban"),
        },
        Frame {
            ident: "IauSycorax",
            name: "IAU Body-Fixed Frame for Sycorax",
            abbreviation: "IAU_SYCORAX",
            body: Some("Sycorax"),
        },
        Frame {
            ident: "IauProspero",
            name: "IAU Body-Fixed Frame for Prospero",
            abbreviation: "IAU_PROSPERO",
            body: Some("Prospero"),
        },
        Frame {
            ident: "IauSetebos",
            name: "IAU Body-Fixed Frame for Setebos",
            abbreviation: "IAU_SETEBOS",
            body: Some("Setebos"),
        },
        Frame {
            ident: "IauStephano",
            name: "IAU Body-Fixed Frame for Stephano",
            abbreviation: "IAU_STEPHANO",
            body: Some("Stephano"),
        },
        Frame {
            ident: "IauTrinculo",
            name: "IAU Body-Fixed Frame for Trinculo",
            abbreviation: "IAU_TRINCULO",
            body: Some("Trinculo"),
        },
        Frame {
            ident: "IauFrancisco",
            name: "IAU Body-Fixed Frame for Francisco",
            abbreviation: "IAU_FRANCISCO",
            body: Some("Francisco"),
        },
        Frame {
            ident: "IauMargaret",
            name: "IAU Body-Fixed Frame for Margaret",
            abbreviation: "IAU_MARGARET",
            body: Some("Margaret"),
        },
        Frame {
            ident: "IauFerdinand",
            name: "IAU Body-Fixed Frame for Ferdinand",
            abbreviation: "IAU_FERDINAND",
            body: Some("Ferdinand"),
        },
        Frame {
            ident: "IauPerdita",
            name: "IAU Body-Fixed Frame for Perdita",
            abbreviation: "IAU_PERDITA",
            body: Some("Perdita"),
        },
        Frame {
            ident: "IauMab",
            name: "IAU Body-Fixed Frame for Mab",
            abbreviation: "IAU_MAB",
            body: Some("Mab"),
        },
        Frame {
            ident: "IauCupid",
            name: "IAU Body-Fixed Frame for Cupid",
            abbreviation: "IAU_CUPID",
            body: Some("Cupid"),
        },
        Frame {
            ident: "IauTriton",
            name: "IAU Body-Fixed Frame for Triton",
            abbreviation: "IAU_TRITON",
            body: Some("Triton"),
        },
        Frame {
            ident: "IauNereid",
            name: "IAU Body-Fixed Frame for Nereid",
            abbreviation: "IAU_NEREID",
            body: Some("Nereid"),
        },
        Frame {
            ident: "IauNaiad",
            name: "IAU Body-Fixed Frame for Naiad",
            abbreviation: "IAU_NAIAD",
            body: Some("Naiad"),
        },
        Frame {
            ident: "IauThalassa",
            name: "IAU Body-Fixed Frame for Thalassa",
            abbreviation: "IAU_THALASSA",
            body: Some("Thalassa"),
        },
        Frame {
            ident: "IauDespina",
            name: "IAU Body-Fixed Frame for Despina",
            abbreviation: "IAU_DESPINA",
            body: Some("Despina"),
        },
        Frame {
            ident: "IauGalatea",
            name: "IAU Body-Fixed Frame for Galatea",
            abbreviation: "IAU_GALATEA",
            body: Some("Galatea"),
        },
        Frame {
            ident: "IauLarissa",
            name: "IAU Body-Fixed Frame for Larissa",
            abbreviation: "IAU_LARISSA",
            body: Some("Larissa"),
        },
        Frame {
            ident: "IauProteus",
            name: "IAU Body-Fixed Frame for Proteus",
            abbreviation: "IAU_PROTEUS",
            body: Some("Proteus"),
        },
        Frame {
            ident: "IauHalimede",
            name: "IAU Body-Fixed Frame for Halimede",
            abbreviation: "IAU_HALIMEDE",
            body: Some("Halimede"),
        },
        Frame {
            ident: "IauPsamathe",
            name: "IAU Body-Fixed Frame for Psamathe",
            abbreviation: "IAU_PSAMATHE",
            body: Some("Psamathe"),
        },
        Frame {
            ident: "IauSao",
            name: "IAU Body-Fixed Frame for Sao",
            abbreviation: "IAU_SAO",
            body: Some("Sao"),
        },
        Frame {
            ident: "IauLaomedeia",
            name: "IAU Body-Fixed Frame for Laomedeia",
            abbreviation: "IAU_LAOMEDEIA",
            body: Some("Laomedeia"),
        },
        Frame {
            ident: "IauNeso",
            name: "IAU Body-Fixed Frame for Neso",
            abbreviation: "IAU_NESO",
            body: Some("Neso"),
        },
        Frame {
            ident: "IauCharon",
            name: "IAU Body-Fixed Frame for Charon",
            abbreviation: "IAU_CHARON",
            body: Some("Charon"),
        },
        Frame {
            ident: "IauNix",
            name: "IAU Body-Fixed Frame for Nix",
            abbreviation: "IAU_NIX",
            body: Some("Nix"),
        },
        Frame {
            ident: "IauHydra",
            name: "IAU Body-Fixed Frame for Hydra",
            abbreviation: "IAU_HYDRA",
            body: Some("Hydra"),
        },
        Frame {
            ident: "IauKerberos",
            name: "IAU Body-Fixed Frame for Kerberos",
            abbreviation: "IAU_KERBEROS",
            body: Some("Kerberos"),
        },
        Frame {
            ident: "IauStyx",
            name: "IAU Body-Fixed Frame for Styx",
            abbreviation: "IAU_STYX",
            body: Some("Styx"),
        },
        Frame {
            ident: "IauGaspra",
            name: "IAU Body-Fixed Frame for Gaspra",
            abbreviation: "IAU_GASPRA",
            body: Some("Gaspra"),
        },
        Frame {
            ident: "IauIda",
            name: "IAU Body-Fixed Frame for Ida",
            abbreviation: "IAU_IDA",
            body: Some("Ida"),
        },
        Frame {
            ident: "IauDactyl",
            name: "IAU Body-Fixed Frame for Dactyl",
            abbreviation: "IAU_DACTYL",
            body: Some("Dactyl"),
        },
        Frame {
            ident: "IauCeres",
            name: "IAU Body-Fixed Frame for Ceres",
            abbreviation: "IAU_CERES",
            body: Some("Ceres"),
        },
        Frame {
            ident: "IauPallas",
            name: "IAU Body-Fixed Frame for Pallas",
            abbreviation: "IAU_PALLAS",
            body: Some("Pallas"),
        },
        Frame {
            ident: "IauVesta",
            name: "IAU Body-Fixed Frame for Vesta",
            abbreviation: "IAU_VESTA",
            body: Some("Vesta"),
        },
        Frame {
            ident: "IauPsyche",
            name: "IAU Body-Fixed Frame for Psyche",
            abbreviation: "IAU_PSYCHE",
            body: Some("Psyche"),
        },
        Frame {
            ident: "IauLutetia",
            name: "IAU Body-Fixed Frame for Lutetia",
            abbreviation: "IAU_LUTETIA",
            body: Some("Lutetia"),
        },
        Frame {
            ident: "IauKleopatra",
            name: "IAU Body-Fixed Frame for Kleopatra",
            abbreviation: "IAU_KLEOPATRA",
            body: Some("Kleopatra"),
        },
        Frame {
            ident: "IauEros",
            name: "IAU Body-Fixed Frame for Eros",
            abbreviation: "IAU_EROS",
            body: Some("Eros"),
        },
        Frame {
            ident: "IauDavida",
            name: "IAU Body-Fixed Frame for Davida",
            abbreviation: "IAU_DAVIDA",
            body: Some("Davida"),
        },
        Frame {
            ident: "IauMathilde",
            name: "IAU Body-Fixed Frame for Mathilde",
            abbreviation: "IAU_MATHILDE",
            body: Some("Mathilde"),
        },
        Frame {
            ident: "IauSteins",
            name: "IAU Body-Fixed Frame for Steins",
            abbreviation: "IAU_STEINS",
            body: Some("Steins"),
        },
        Frame {
            ident: "IauBraille",
            name: "IAU Body-Fixed Frame for Braille",
            abbreviation: "IAU_BRAILLE",
            body: Some("Braille"),
        },
        Frame {
            ident: "IauWilsonHarrington",
            name: "IAU Body-Fixed Frame for Wilson-Harrington",
            abbreviation: "IAU_WILSON_HARRINGTON",
            body: Some("WilsonHarrington"),
        },
        Frame {
            ident: "IauToutatis",
            name: "IAU Body-Fixed Frame for Toutatis",
            abbreviation: "IAU_TOUTATIS",
            body: Some("Toutatis"),
        },
        Frame {
            ident: "IauItokawa",
            name: "IAU Body-Fixed Frame for Itokawa",
            abbreviation: "IAU_ITOKAWA",
            body: Some("Itokawa"),
        },
        Frame {
            ident: "IauBennu",
            name: "IAU Body-Fixed Frame for Bennu",
            abbreviation: "IAU_BENNU",
            body: Some("Bennu"),
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
