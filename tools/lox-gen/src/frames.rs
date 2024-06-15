use std::{fs, path::Path, process::Command};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::common::{AUTO_GENERATION_NOTICE, COPYRIGHT_NOTICE};

pub struct Frame {
    pub ident: &'static str,
    pub name: &'static str,
    pub abbreviation: &'static str,
    pub is_bodyfixed: bool,
}

impl Frame {
    pub fn ident(&self) -> Ident {
        format_ident!("{}", self.ident)
    }
}

fn generate_transform(target: &Frame, frames: &[Frame], code: &mut TokenStream) {
    let target_ident = target.ident();

    let target_type = if target.is_bodyfixed {
        quote!(BodyFixed<#target_ident>)
    } else {
        quote!(#target_ident)
    };

    let mut match_arms = quote! {};

    if target.ident == "Icrf" {
        frames.iter().for_each(|f| {
            let ident = f.ident();

            let f_value = if f.is_bodyfixed {
                quote!(BodyFixed(#ident))
            } else {
                quote!(#ident)
            };

            match_arms.extend(if target.ident == f.ident {
                quote! {
                    PyFrame::#ident => Ok(self.0.with_frame(frame)),
                }
            } else {
                quote! {
                    PyFrame::#ident => self.0.with_frame(#f_value).try_to_frame(frame, provider),
                }
            });
        });
    } else {
        match_arms.extend(quote! {
            PyFrame::Icrf => self.0.with_frame(Icrf).try_to_frame(frame, provider),
            PyFrame::#target_ident => Ok(self.0.with_frame(frame)),
            _ => self
                .try_to_frame(Icrf, provider)?
                .try_to_frame(frame, provider),
        });
    }

    code.extend(quote! {
        impl<T> TryToFrame<#target_type, T> for PyState
        where
            T: FrameTransformationProvider + PyDeltaUt1Provider,
        {
            type Output = State<PyTime, PyBody, #target_type>;
            type Error = T::Error;

            fn try_to_frame(
                &self,
                frame: #target_type,
                provider: &T,
            ) -> Result<State<PyTime, PyBody, #target_type>, T::Error> {
                match self.0.reference_frame() {
                    #match_arms
                }
            }
        }
    })
}

pub fn generate_code(frames: &[Frame]) -> String {
    let mut code = quote!();
    let mut tests = quote!();

    let mut match_arms_name = quote! {};
    let mut match_arms_abbreviation = quote! {};
    let mut match_arms_from_str = quote! {};
    let mut match_arms_impl_pystate = quote! {};

    frames.iter().for_each(|f| {
        let ident = f.ident();
        let test_ident = format_ident!("test_reference_frame_{}", f.abbreviation.to_lowercase());
        let name = f.name;
        let abbreviation = f.abbreviation;
        let abbreviation_lowercase = abbreviation.to_lowercase();
        match_arms_name.extend(quote! {
            PyFrame::#ident => #name.to_string(),
        });
        match_arms_abbreviation.extend(quote! {
            PyFrame::#ident => #abbreviation.to_string(),
        });
        match_arms_from_str.extend(quote! {
            #abbreviation | #abbreviation_lowercase => Ok(PyFrame::#ident),
        });
        match_arms_impl_pystate.extend(if f.ident == "Icrf" {
            quote! {
                PyFrame::#ident => match provider {
                    Some(provider) => Ok(PyState(
                        self.try_to_frame(#ident, provider.get())?
                            .with_frame(PyFrame::#ident),
                    )),
                    None => Ok(PyState(
                        self.try_to_frame(#ident, &PyNoOpOffsetProvider)?
                            .with_frame(PyFrame::#ident),
                    )),
                },
            }
        } else {
            quote! {
                PyFrame::#ident => match provider {
                    Some(provider) => Ok(PyState(
                        self.try_to_frame(BodyFixed(#ident), provider.get())?
                            .with_frame(PyFrame::#ident),
                    )),
                    None => Ok(PyState(
                        self.try_to_frame(BodyFixed(#ident), &PyNoOpOffsetProvider)?
                            .with_frame(PyFrame::#ident),
                    )),
                },

            }
        });

        generate_transform(f, frames, &mut code);

        tests.extend(quote! {
            #[test]
            fn #test_ident() {
                assert_eq!(PyFrame::#ident.name(), #name);
                assert_eq!(PyFrame::#ident.abbreviation(), #abbreviation);
            }
        })
    });

    code.extend(quote! {
        impl PyState {
            pub fn to_frame_generated(
                &self,
                frame: &str,
                provider: Option<&Bound<'_, PyUt1Provider>>,
            ) -> PyResult<Self> {
                let frame: PyFrame = frame.parse()?;
                match frame {
                    #match_arms_impl_pystate
                }
            }
        }

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

        impl FromStr for PyFrame {
            type Err = PyErr;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #match_arms_from_str
                    _ => Err(PyValueError::new_err("unknown reference frame")),
                }
            }
        }
    });

    let module = quote! {
        use crate::frames::{
            BodyFixed, CoordinateSystem, FrameTransformationProvider, Icrf, ReferenceFrame, TryToFrame,
        };
        use crate::python::{PyFrame, PyState};
        use crate::states::State;
        use lox_bodies::python::PyBody;
        use lox_bodies::*;
        use lox_time::python::time::PyTime;
        use lox_time::python::ut1::{PyDeltaUt1Provider, PyNoOpOffsetProvider, PyUt1Provider};
        use pyo3::exceptions::PyValueError;
        use pyo3::{Bound, PyErr, PyResult};
        use std::str::FromStr;

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
            is_bodyfixed: false,
        },
        Frame {
            ident: "Sun",
            name: "IAU Body-Fixed Frame for the Sun",
            abbreviation: "IAU_SUN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mercury",
            name: "IAU Body-Fixed Frame for Mercury",
            abbreviation: "IAU_MERCURY",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Venus",
            name: "IAU Body-Fixed Frame for Venus",
            abbreviation: "IAU_VENUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Earth",
            name: "IAU Body-Fixed Frame for Earth",
            abbreviation: "IAU_EARTH",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mars",
            name: "IAU Body-Fixed Frame for Mars",
            abbreviation: "IAU_MARS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Jupiter",
            name: "IAU Body-Fixed Frame for Jupiter",
            abbreviation: "IAU_JUPITER",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Saturn",
            name: "IAU Body-Fixed Frame for Saturn",
            abbreviation: "IAU_SATURN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Uranus",
            name: "IAU Body-Fixed Frame for Uranus",
            abbreviation: "IAU_URANUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Neptune",
            name: "IAU Body-Fixed Frame for Neptune",
            abbreviation: "IAU_NEPTUNE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pluto",
            name: "IAU Body-Fixed Frame for Pluto",
            abbreviation: "IAU_PLUTO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Moon",
            name: "IAU Body-Fixed Frame for the Moon",
            abbreviation: "IAU_MOON",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Phobos",
            name: "IAU Body-Fixed Frame for Phobos",
            abbreviation: "IAU_PHOBOS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Deimos",
            name: "IAU Body-Fixed Frame for Deimos",
            abbreviation: "IAU_DEIMOS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Io",
            name: "IAU Body-Fixed Frame for Io",
            abbreviation: "IAU_IO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Europa",
            name: "IAU Body-Fixed Frame for Europa",
            abbreviation: "IAU_EUROPA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ganymede",
            name: "IAU Body-Fixed Frame for Ganymede",
            abbreviation: "IAU_GANYMEDE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Callisto",
            name: "IAU Body-Fixed Frame for Callisto",
            abbreviation: "IAU_CALLISTO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Amalthea",
            name: "IAU Body-Fixed Frame for Amalthea",
            abbreviation: "IAU_AMALTHEA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Himalia",
            name: "IAU Body-Fixed Frame for Himalia",
            abbreviation: "IAU_HIMALIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Elara",
            name: "IAU Body-Fixed Frame for Elara",
            abbreviation: "IAU_ELARA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pasiphae",
            name: "IAU Body-Fixed Frame for Pasiphae",
            abbreviation: "IAU_PASIPHAE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Sinope",
            name: "IAU Body-Fixed Frame for Sinope",
            abbreviation: "IAU_SINOPE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Lysithea",
            name: "IAU Body-Fixed Frame for Lysithea",
            abbreviation: "IAU_LYSITHEA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Carme",
            name: "IAU Body-Fixed Frame for Carme",
            abbreviation: "IAU_CARME",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ananke",
            name: "IAU Body-Fixed Frame for Ananke",
            abbreviation: "IAU_ANANKE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Leda",
            name: "IAU Body-Fixed Frame for Leda",
            abbreviation: "IAU_LEDA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Thebe",
            name: "IAU Body-Fixed Frame for Thebe",
            abbreviation: "IAU_THEBE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Adrastea",
            name: "IAU Body-Fixed Frame for Adrastea",
            abbreviation: "IAU_ADRASTEA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Metis",
            name: "IAU Body-Fixed Frame for Metis",
            abbreviation: "IAU_METIS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Callirrhoe",
            name: "IAU Body-Fixed Frame for Callirrhoe",
            abbreviation: "IAU_CALLIRRHOE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Themisto",
            name: "IAU Body-Fixed Frame for Themisto",
            abbreviation: "IAU_THEMISTO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Magaclite",
            name: "IAU Body-Fixed Frame for Magaclite",
            abbreviation: "IAU_MAGACLITE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Taygete",
            name: "IAU Body-Fixed Frame for Taygete",
            abbreviation: "IAU_TAYGETE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Chaldene",
            name: "IAU Body-Fixed Frame for Chaldene",
            abbreviation: "IAU_CHALDENE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Harpalyke",
            name: "IAU Body-Fixed Frame for Harpalyke",
            abbreviation: "IAU_HARPALYKE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kalyke",
            name: "IAU Body-Fixed Frame for Kalyke",
            abbreviation: "IAU_KALYKE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Iocaste",
            name: "IAU Body-Fixed Frame for Iocaste",
            abbreviation: "IAU_IOCASTE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Erinome",
            name: "IAU Body-Fixed Frame for Erinome",
            abbreviation: "IAU_ERINOME",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Isonoe",
            name: "IAU Body-Fixed Frame for Isonoe",
            abbreviation: "IAU_ISONOE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Praxidike",
            name: "IAU Body-Fixed Frame for Praxidike",
            abbreviation: "IAU_PRAXIDIKE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Autonoe",
            name: "IAU Body-Fixed Frame for Autonoe",
            abbreviation: "IAU_AUTONOE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Thyone",
            name: "IAU Body-Fixed Frame for Thyone",
            abbreviation: "IAU_THYONE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Hermippe",
            name: "IAU Body-Fixed Frame for Hermippe",
            abbreviation: "IAU_HERMIPPE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Aitne",
            name: "IAU Body-Fixed Frame for Aitne",
            abbreviation: "IAU_AITNE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Eurydome",
            name: "IAU Body-Fixed Frame for Eurydome",
            abbreviation: "IAU_EURYDOME",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Euanthe",
            name: "IAU Body-Fixed Frame for Euanthe",
            abbreviation: "IAU_EUANTHE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Euporie",
            name: "IAU Body-Fixed Frame for Euporie",
            abbreviation: "IAU_EUPORIE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Orthosie",
            name: "IAU Body-Fixed Frame for Orthosie",
            abbreviation: "IAU_ORTHOSIE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Sponde",
            name: "IAU Body-Fixed Frame for Sponde",
            abbreviation: "IAU_SPONDE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kale",
            name: "IAU Body-Fixed Frame for Kale",
            abbreviation: "IAU_KALE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pasithee",
            name: "IAU Body-Fixed Frame for Pasithee",
            abbreviation: "IAU_PASITHEE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Hegemone",
            name: "IAU Body-Fixed Frame for Hegemone",
            abbreviation: "IAU_HEGEMONE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mneme",
            name: "IAU Body-Fixed Frame for Mneme",
            abbreviation: "IAU_MNEME",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Aoede",
            name: "IAU Body-Fixed Frame for Aoede",
            abbreviation: "IAU_AOEDE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Thelxinoe",
            name: "IAU Body-Fixed Frame for Thelxinoe",
            abbreviation: "IAU_THELXINOE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Arche",
            name: "IAU Body-Fixed Frame for Arche",
            abbreviation: "IAU_ARCHE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kallichore",
            name: "IAU Body-Fixed Frame for Kallichore",
            abbreviation: "IAU_KALLICHORE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Helike",
            name: "IAU Body-Fixed Frame for Helike",
            abbreviation: "IAU_HELIKE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Carpo",
            name: "IAU Body-Fixed Frame for Carpo",
            abbreviation: "IAU_CARPO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Eukelade",
            name: "IAU Body-Fixed Frame for Eukelade",
            abbreviation: "IAU_EUKELADE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Cyllene",
            name: "IAU Body-Fixed Frame for Cyllene",
            abbreviation: "IAU_CYLLENE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kore",
            name: "IAU Body-Fixed Frame for Kore",
            abbreviation: "IAU_KORE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Herse",
            name: "IAU Body-Fixed Frame for Herse",
            abbreviation: "IAU_HERSE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Dia",
            name: "IAU Body-Fixed Frame for Dia",
            abbreviation: "IAU_DIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mimas",
            name: "IAU Body-Fixed Frame for Mimas",
            abbreviation: "IAU_MIMAS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Enceladus",
            name: "IAU Body-Fixed Frame for Enceladus",
            abbreviation: "IAU_ENCELADUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Tethys",
            name: "IAU Body-Fixed Frame for Tethys",
            abbreviation: "IAU_TETHYS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Dione",
            name: "IAU Body-Fixed Frame for Dione",
            abbreviation: "IAU_DIONE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Rhea",
            name: "IAU Body-Fixed Frame for Rhea",
            abbreviation: "IAU_RHEA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Titan",
            name: "IAU Body-Fixed Frame for Titan",
            abbreviation: "IAU_TITAN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Hyperion",
            name: "IAU Body-Fixed Frame for Hyperion",
            abbreviation: "IAU_HYPERION",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Iapetus",
            name: "IAU Body-Fixed Frame for Iapetus",
            abbreviation: "IAU_IAPETUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Phoebe",
            name: "IAU Body-Fixed Frame for Phoebe",
            abbreviation: "IAU_PHOEBE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Janus",
            name: "IAU Body-Fixed Frame for Janus",
            abbreviation: "IAU_JANUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Epimetheus",
            name: "IAU Body-Fixed Frame for Epimetheus",
            abbreviation: "IAU_EPIMETHEUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Helene",
            name: "IAU Body-Fixed Frame for Helene",
            abbreviation: "IAU_HELENE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Telesto",
            name: "IAU Body-Fixed Frame for Telesto",
            abbreviation: "IAU_TELESTO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Calypso",
            name: "IAU Body-Fixed Frame for Calypso",
            abbreviation: "IAU_CALYPSO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Atlas",
            name: "IAU Body-Fixed Frame for Atlas",
            abbreviation: "IAU_ATLAS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Prometheus",
            name: "IAU Body-Fixed Frame for Prometheus",
            abbreviation: "IAU_PROMETHEUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pandora",
            name: "IAU Body-Fixed Frame for Pandora",
            abbreviation: "IAU_PANDORA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pan",
            name: "IAU Body-Fixed Frame for Pan",
            abbreviation: "IAU_PAN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ymir",
            name: "IAU Body-Fixed Frame for Ymir",
            abbreviation: "IAU_YMIR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Paaliaq",
            name: "IAU Body-Fixed Frame for Paaliaq",
            abbreviation: "IAU_PAALIAQ",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Tarvos",
            name: "IAU Body-Fixed Frame for Tarvos",
            abbreviation: "IAU_TARVOS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ijiraq",
            name: "IAU Body-Fixed Frame for Ijiraq",
            abbreviation: "IAU_IJIRAQ",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Suttungr",
            name: "IAU Body-Fixed Frame for Suttungr",
            abbreviation: "IAU_SUTTUNGR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kiviuq",
            name: "IAU Body-Fixed Frame for Kiviuq",
            abbreviation: "IAU_KIVIUQ",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mundilfari",
            name: "IAU Body-Fixed Frame for Mundilfari",
            abbreviation: "IAU_MUNDILFARI",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Albiorix",
            name: "IAU Body-Fixed Frame for Albiorix",
            abbreviation: "IAU_ALBIORIX",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Skathi",
            name: "IAU Body-Fixed Frame for Skathi",
            abbreviation: "IAU_SKATHI",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Erriapus",
            name: "IAU Body-Fixed Frame for Erriapus",
            abbreviation: "IAU_ERRIAPUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Siarnaq",
            name: "IAU Body-Fixed Frame for Siarnaq",
            abbreviation: "IAU_SIARNAQ",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Thrymr",
            name: "IAU Body-Fixed Frame for Thrymr",
            abbreviation: "IAU_THRYMR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Narvi",
            name: "IAU Body-Fixed Frame for Narvi",
            abbreviation: "IAU_NARVI",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Methone",
            name: "IAU Body-Fixed Frame for Methone",
            abbreviation: "IAU_METHONE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pallene",
            name: "IAU Body-Fixed Frame for Pallene",
            abbreviation: "IAU_PALLENE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Polydeuces",
            name: "IAU Body-Fixed Frame for Polydeuces",
            abbreviation: "IAU_POLYDEUCES",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Daphnis",
            name: "IAU Body-Fixed Frame for Daphnis",
            abbreviation: "IAU_DAPHNIS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Aegir",
            name: "IAU Body-Fixed Frame for Aegir",
            abbreviation: "IAU_AEGIR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Bebhionn",
            name: "IAU Body-Fixed Frame for Bebhionn",
            abbreviation: "IAU_BEBHIONN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Bergelmir",
            name: "IAU Body-Fixed Frame for Bergelmir",
            abbreviation: "IAU_BERGELMIR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Bestla",
            name: "IAU Body-Fixed Frame for Bestla",
            abbreviation: "IAU_BESTLA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Farbauti",
            name: "IAU Body-Fixed Frame for Farbauti",
            abbreviation: "IAU_FARBAUTI",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Fenrir",
            name: "IAU Body-Fixed Frame for Fenrir",
            abbreviation: "IAU_FENRIR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Fornjot",
            name: "IAU Body-Fixed Frame for Fornjot",
            abbreviation: "IAU_FORNJOT",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Hati",
            name: "IAU Body-Fixed Frame for Hati",
            abbreviation: "IAU_HATI",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Hyrrokkin",
            name: "IAU Body-Fixed Frame for Hyrrokkin",
            abbreviation: "IAU_HYRROKKIN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kari",
            name: "IAU Body-Fixed Frame for Kari",
            abbreviation: "IAU_KARI",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Loge",
            name: "IAU Body-Fixed Frame for Loge",
            abbreviation: "IAU_LOGE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Skoll",
            name: "IAU Body-Fixed Frame for Skoll",
            abbreviation: "IAU_SKOLL",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Surtur",
            name: "IAU Body-Fixed Frame for Surtur",
            abbreviation: "IAU_SURTUR",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Anthe",
            name: "IAU Body-Fixed Frame for Anthe",
            abbreviation: "IAU_ANTHE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Jarnsaxa",
            name: "IAU Body-Fixed Frame for Jarnsaxa",
            abbreviation: "IAU_JARNSAXA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Greip",
            name: "IAU Body-Fixed Frame for Greip",
            abbreviation: "IAU_GREIP",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Tarqeq",
            name: "IAU Body-Fixed Frame for Tarqeq",
            abbreviation: "IAU_TARQEQ",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Aegaeon",
            name: "IAU Body-Fixed Frame for Aegaeon",
            abbreviation: "IAU_AEGAEON",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ariel",
            name: "IAU Body-Fixed Frame for Ariel",
            abbreviation: "IAU_ARIEL",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Umbriel",
            name: "IAU Body-Fixed Frame for Umbriel",
            abbreviation: "IAU_UMBRIEL",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Titania",
            name: "IAU Body-Fixed Frame for Titania",
            abbreviation: "IAU_TITANIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Oberon",
            name: "IAU Body-Fixed Frame for Oberon",
            abbreviation: "IAU_OBERON",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Miranda",
            name: "IAU Body-Fixed Frame for Miranda",
            abbreviation: "IAU_MIRANDA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Cordelia",
            name: "IAU Body-Fixed Frame for Cordelia",
            abbreviation: "IAU_CORDELIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ophelia",
            name: "IAU Body-Fixed Frame for Ophelia",
            abbreviation: "IAU_OPHELIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Bianca",
            name: "IAU Body-Fixed Frame for Bianca",
            abbreviation: "IAU_BIANCA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Cressida",
            name: "IAU Body-Fixed Frame for Cressida",
            abbreviation: "IAU_CRESSIDA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Desdemona",
            name: "IAU Body-Fixed Frame for Desdemona",
            abbreviation: "IAU_DESDEMONA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Juliet",
            name: "IAU Body-Fixed Frame for Juliet",
            abbreviation: "IAU_JULIET",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Portia",
            name: "IAU Body-Fixed Frame for Portia",
            abbreviation: "IAU_PORTIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Rosalind",
            name: "IAU Body-Fixed Frame for Rosalind",
            abbreviation: "IAU_ROSALIND",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Belinda",
            name: "IAU Body-Fixed Frame for Belinda",
            abbreviation: "IAU_BELINDA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Puck",
            name: "IAU Body-Fixed Frame for Puck",
            abbreviation: "IAU_PUCK",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Caliban",
            name: "IAU Body-Fixed Frame for Caliban",
            abbreviation: "IAU_CALIBAN",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Sycorax",
            name: "IAU Body-Fixed Frame for Sycorax",
            abbreviation: "IAU_SYCORAX",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Prospero",
            name: "IAU Body-Fixed Frame for Prospero",
            abbreviation: "IAU_PROSPERO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Setebos",
            name: "IAU Body-Fixed Frame for Setebos",
            abbreviation: "IAU_SETEBOS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Stephano",
            name: "IAU Body-Fixed Frame for Stephano",
            abbreviation: "IAU_STEPHANO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Trinculo",
            name: "IAU Body-Fixed Frame for Trinculo",
            abbreviation: "IAU_TRINCULO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Francisco",
            name: "IAU Body-Fixed Frame for Francisco",
            abbreviation: "IAU_FRANCISCO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Margaret",
            name: "IAU Body-Fixed Frame for Margaret",
            abbreviation: "IAU_MARGARET",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ferdinand",
            name: "IAU Body-Fixed Frame for Ferdinand",
            abbreviation: "IAU_FERDINAND",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Perdita",
            name: "IAU Body-Fixed Frame for Perdita",
            abbreviation: "IAU_PERDITA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mab",
            name: "IAU Body-Fixed Frame for Mab",
            abbreviation: "IAU_MAB",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Cupid",
            name: "IAU Body-Fixed Frame for Cupid",
            abbreviation: "IAU_CUPID",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Triton",
            name: "IAU Body-Fixed Frame for Triton",
            abbreviation: "IAU_TRITON",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Nereid",
            name: "IAU Body-Fixed Frame for Nereid",
            abbreviation: "IAU_NEREID",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Naiad",
            name: "IAU Body-Fixed Frame for Naiad",
            abbreviation: "IAU_NAIAD",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Thalassa",
            name: "IAU Body-Fixed Frame for Thalassa",
            abbreviation: "IAU_THALASSA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Despina",
            name: "IAU Body-Fixed Frame for Despina",
            abbreviation: "IAU_DESPINA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Galatea",
            name: "IAU Body-Fixed Frame for Galatea",
            abbreviation: "IAU_GALATEA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Larissa",
            name: "IAU Body-Fixed Frame for Larissa",
            abbreviation: "IAU_LARISSA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Proteus",
            name: "IAU Body-Fixed Frame for Proteus",
            abbreviation: "IAU_PROTEUS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Halimede",
            name: "IAU Body-Fixed Frame for Halimede",
            abbreviation: "IAU_HALIMEDE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Psamathe",
            name: "IAU Body-Fixed Frame for Psamathe",
            abbreviation: "IAU_PSAMATHE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Sao",
            name: "IAU Body-Fixed Frame for Sao",
            abbreviation: "IAU_SAO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Laomedeia",
            name: "IAU Body-Fixed Frame for Laomedeia",
            abbreviation: "IAU_LAOMEDEIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Neso",
            name: "IAU Body-Fixed Frame for Neso",
            abbreviation: "IAU_NESO",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Charon",
            name: "IAU Body-Fixed Frame for Charon",
            abbreviation: "IAU_CHARON",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Nix",
            name: "IAU Body-Fixed Frame for Nix",
            abbreviation: "IAU_NIX",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Hydra",
            name: "IAU Body-Fixed Frame for Hydra",
            abbreviation: "IAU_HYDRA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kerberos",
            name: "IAU Body-Fixed Frame for Kerberos",
            abbreviation: "IAU_KERBEROS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Styx",
            name: "IAU Body-Fixed Frame for Styx",
            abbreviation: "IAU_STYX",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Gaspra",
            name: "IAU Body-Fixed Frame for Gaspra",
            abbreviation: "IAU_GASPRA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ida",
            name: "IAU Body-Fixed Frame for Ida",
            abbreviation: "IAU_IDA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Dactyl",
            name: "IAU Body-Fixed Frame for Dactyl",
            abbreviation: "IAU_DACTYL",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Ceres",
            name: "IAU Body-Fixed Frame for Ceres",
            abbreviation: "IAU_CERES",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Pallas",
            name: "IAU Body-Fixed Frame for Pallas",
            abbreviation: "IAU_PALLAS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Vesta",
            name: "IAU Body-Fixed Frame for Vesta",
            abbreviation: "IAU_VESTA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Psyche",
            name: "IAU Body-Fixed Frame for Psyche",
            abbreviation: "IAU_PSYCHE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Lutetia",
            name: "IAU Body-Fixed Frame for Lutetia",
            abbreviation: "IAU_LUTETIA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Kleopatra",
            name: "IAU Body-Fixed Frame for Kleopatra",
            abbreviation: "IAU_KLEOPATRA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Eros",
            name: "IAU Body-Fixed Frame for Eros",
            abbreviation: "IAU_EROS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Davida",
            name: "IAU Body-Fixed Frame for Davida",
            abbreviation: "IAU_DAVIDA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Mathilde",
            name: "IAU Body-Fixed Frame for Mathilde",
            abbreviation: "IAU_MATHILDE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Steins",
            name: "IAU Body-Fixed Frame for Steins",
            abbreviation: "IAU_STEINS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Braille",
            name: "IAU Body-Fixed Frame for Braille",
            abbreviation: "IAU_BRAILLE",
            is_bodyfixed: true,
        },
        Frame {
            ident: "WilsonHarrington",
            name: "IAU Body-Fixed Frame for Wilson-Harrington",
            abbreviation: "IAU_WILSON_HARRINGTON",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Toutatis",
            name: "IAU Body-Fixed Frame for Toutatis",
            abbreviation: "IAU_TOUTATIS",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Itokawa",
            name: "IAU Body-Fixed Frame for Itokawa",
            abbreviation: "IAU_ITOKAWA",
            is_bodyfixed: true,
        },
        Frame {
            ident: "Bennu",
            name: "IAU Body-Fixed Frame for Bennu",
            abbreviation: "IAU_BENNU",
            is_bodyfixed: true,
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
