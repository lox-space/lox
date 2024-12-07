/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::common::write_file;
use lox_io::spice::Kernel;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use std::path::Path;

const BODIES: [(&str, i32); 190] = [
    ("Sun", 10),
    // Planets.
    ("Mercury", 199),
    ("Venus", 299),
    ("Earth", 399),
    ("Mars", 499),
    ("Jupiter", 599),
    ("Saturn", 699),
    ("Uranus", 799),
    ("Neptune", 899),
    ("Pluto", 999),
    // Barycenters.
    ("Solar System Barycenter", 0),
    ("Mercury Barycenter", 1),
    ("Venus Barycenter", 2),
    ("Earth Barycenter", 3),
    ("Mars Barycenter", 4),
    ("Jupiter Barycenter", 5),
    ("Saturn Barycenter", 6),
    ("Uranus Barycenter", 7),
    ("Neptune Barycenter", 8),
    ("Pluto Barycenter", 9),
    // Satellites.
    ("Moon", 301),
    ("Phobos", 401),
    ("Deimos", 402),
    ("Io", 501),
    ("Europa", 502),
    ("Ganymede", 503),
    ("Callisto", 504),
    ("Amalthea", 505),
    ("Himalia", 506),
    ("Elara", 507),
    ("Pasiphae", 508),
    ("Sinope", 509),
    ("Lysithea", 510),
    ("Carme", 511),
    ("Ananke", 512),
    ("Leda", 513),
    ("Thebe", 514),
    ("Adrastea", 515),
    ("Metis", 516),
    ("Callirrhoe", 517),
    ("Themisto", 518),
    ("Magaclite", 519),
    ("Taygete", 520),
    ("Chaldene", 521),
    ("Harpalyke", 522),
    ("Kalyke", 523),
    ("Iocaste", 524),
    ("Erinome", 525),
    ("Isonoe", 526),
    ("Praxidike", 527),
    ("Autonoe", 528),
    ("Thyone", 529),
    ("Hermippe", 530),
    ("Aitne", 531),
    ("Eurydome", 532),
    ("Euanthe", 533),
    ("Euporie", 534),
    ("Orthosie", 535),
    ("Sponde", 536),
    ("Kale", 537),
    ("Pasithee", 538),
    ("Hegemone", 539),
    ("Mneme", 540),
    ("Aoede", 541),
    ("Thelxinoe", 542),
    ("Arche", 543),
    ("Kallichore", 544),
    ("Helike", 545),
    ("Carpo", 546),
    ("Eukelade", 547),
    ("Cyllene", 548),
    ("Kore", 549),
    ("Herse", 550),
    ("Dia", 553),
    ("Mimas", 601),
    ("Enceladus", 602),
    ("Tethys", 603),
    ("Dione", 604),
    ("Rhea", 605),
    ("Titan", 606),
    ("Hyperion", 607),
    ("Iapetus", 608),
    ("Phoebe", 609),
    ("Janus", 610),
    ("Epimetheus", 611),
    ("Helene", 612),
    ("Telesto", 613),
    ("Calypso", 614),
    ("Atlas", 615),
    ("Prometheus", 616),
    ("Pandora", 617),
    ("Pan", 618),
    ("Ymir", 619),
    ("Paaliaq", 620),
    ("Tarvos", 621),
    ("Ijiraq", 622),
    ("Suttungr", 623),
    ("Kiviuq", 624),
    ("Mundilfari", 625),
    ("Albiorix", 626),
    ("Skathi", 627),
    ("Erriapus", 628),
    ("Siarnaq", 629),
    ("Thrymr", 630),
    ("Narvi", 631),
    ("Methone", 632),
    ("Pallene", 633),
    ("Polydeuces", 634),
    ("Daphnis", 635),
    ("Aegir", 636),
    ("Bebhionn", 637),
    ("Bergelmir", 638),
    ("Bestla", 639),
    ("Farbauti", 640),
    ("Fenrir", 641),
    ("Fornjot", 642),
    ("Hati", 643),
    ("Hyrrokkin", 644),
    ("Kari", 645),
    ("Loge", 646),
    ("Skoll", 647),
    ("Surtur", 648),
    ("Anthe", 649),
    ("Jarnsaxa", 650),
    ("Greip", 651),
    ("Tarqeq", 652),
    ("Aegaeon", 653),
    ("Ariel", 701),
    ("Umbriel", 702),
    ("Titania", 703),
    ("Oberon", 704),
    ("Miranda", 705),
    ("Cordelia", 706),
    ("Ophelia", 707),
    ("Bianca", 708),
    ("Cressida", 709),
    ("Desdemona", 710),
    ("Juliet", 711),
    ("Portia", 712),
    ("Rosalind", 713),
    ("Belinda", 714),
    ("Puck", 715),
    ("Caliban", 716),
    ("Sycorax", 717),
    ("Prospero", 718),
    ("Setebos", 719),
    ("Stephano", 720),
    ("Trinculo", 721),
    ("Francisco", 722),
    ("Margaret", 723),
    ("Ferdinand", 724),
    ("Perdita", 725),
    ("Mab", 726),
    ("Cupid", 727),
    ("Triton", 801),
    ("Nereid", 802),
    ("Naiad", 803),
    ("Thalassa", 804),
    ("Despina", 805),
    ("Galatea", 806),
    ("Larissa", 807),
    ("Proteus", 808),
    ("Halimede", 809),
    ("Psamathe", 810),
    ("Sao", 811),
    ("Laomedeia", 812),
    ("Neso", 813),
    ("Charon", 901),
    ("Nix", 902),
    ("Hydra", 903),
    ("Kerberos", 904),
    ("Styx", 905),
    // Minor bodies.
    ("Gaspra", 9511010),
    ("Ida", 2431010),
    ("Dactyl", 2431011),
    ("Ceres", 2000001),
    ("Pallas", 2000002),
    ("Vesta", 2000004),
    ("Psyche", 2000016),
    ("Lutetia", 2000021),
    ("Kleopatra", 2000216),
    ("Eros", 2000433),
    ("Davida", 2000511),
    ("Mathilde", 2000253),
    ("Steins", 2002867),
    ("Braille", 2009969),
    ("Wilson-Harrington", 2004015),
    ("Toutatis", 2004179),
    ("Itokawa", 2025143),
    ("Bennu", 2101955),
];

fn ident(name: &str) -> Ident {
    format_ident!("{}", name.replace([' ', '-'], ""))
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
        use crate::MaybePointMass;
        use crate::MaybeRotationalElements;
        use crate::MaybeSpheroid;
        use crate::MaybeTriaxialEllipsoid;
        use crate::NaifId;
        use crate::NutationPrecessionCoefficients;
        use crate::Origin;
        use crate::PointMass;
        use crate::PolynomialCoefficients;
        use crate::Radii;
        use crate::RotationalElements;
        use crate::Spheroid;
        use crate::TriaxialEllipsoid;
        use std::fmt::Display;
        use std::fmt::Formatter;
    };

    let mut point_mass_match_arms = quote! {};
    let mut ellipsoid_match_arms = quote! {};
    let mut nutation_precession_match_arms = quote! {};
    let mut right_ascension_match_arms = quote! {};
    let mut declination_match_arms = quote! {};
    let mut prime_meridian_match_arms = quote! {};

    for (name, id) in BODIES {
        let ident = ident(name);

        code.extend(quote! {

            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.name())
                }
            }
        });

        // PointMass
        let key = if id == 0 {
            "BODY10_GM".to_string()
        } else {
            format!("BODY{id}_GM")
        };

        if let Some(gm) = gm.get_double_array(&key) {
            let gm = gm.first().unwrap();
            code.extend(quote! {
                impl PointMass for #ident {
                    fn gravitational_parameter(&self) -> f64 {
                        #gm
                    }
                }
            });

            point_mass_match_arms.extend(quote! {
                DynOrigin::#ident => Some(#gm),
            });
        };

        // Barycenters do not have cartographic properties
        if id < 10 {
            continue;
        }

        // TriaxialEllipsoid / Spheroid
        let key = format!("BODY{id}_RADII");

        if let Some(radii) = pck.get_double_array(&key) {
            code.extend(quote! {
                impl TriaxialEllipsoid for #ident {
                    fn radii(&self) -> Radii {
                        (#(#radii),*)
                    }
                }
            });

            if radii[0] == radii[1] {
                code.extend(quote! {
                    impl Spheroid for #ident {}
                })
            }

            ellipsoid_match_arms.extend(quote! {
                DynOrigin::#ident => Some((#(#radii),*)),
            })
        }

        let ra_key = format!("BODY{id}_POLE_RA");
        let dec_key = format!("BODY{id}_POLE_DEC");
        let pm_key = format!("BODY{id}_PM");
        let nut_prec_ra_key = format!("BODY{id}_NUT_PREC_RA");
        let nut_prec_dec_key = format!("BODY{id}_NUT_PREC_DEC");
        let nut_prec_pm_key = format!("BODY{id}_NUT_PREC_PM");
        let nut_prec_key = format!("BODY{}_NUT_PREC_ANGLES", id / 100);

        if let (Some(ra), Some(dec), Some(pm)) = (
            get_array_as_radians(pck, &ra_key),
            get_array_as_radians(pck, &dec_key),
            get_array_as_radians(pck, &pm_key),
        ) {
            let ra1 = ra[0];
            let ra2 = ra[1];
            let ra3 = ra.get(2).copied().unwrap_or_default();
            let nut_prec_ra = match get_array_as_radians(pck, &nut_prec_ra_key) {
                None => quote! {None},
                Some(coeffs) => quote! {
                    Some(&[#(#coeffs),*])
                },
            };
            let ra = quote! {
                (#ra1, #ra2, #ra3, #nut_prec_ra)
            };

            let dec1 = dec[0];
            let dec2 = dec[1];
            let dec3 = dec.get(2).copied().unwrap_or_default();
            let nut_prec_dec = match get_array_as_radians(pck, &nut_prec_dec_key) {
                None => quote! {None},
                Some(coeffs) => quote! {
                    Some(&[#(#coeffs),*])
                },
            };
            let dec = quote! {
                (#dec1, #dec2, #dec3, #nut_prec_dec)
            };

            let pm1 = pm[0];
            let pm2 = pm[1];
            let pm3 = pm.get(2).copied().unwrap_or_default();
            let nut_prec_pm = match get_array_as_radians(pck, &nut_prec_pm_key) {
                None => quote! {None},
                Some(coeffs) => quote! {
                    Some(&[#(#coeffs),*])
                },
            };
            let pm = quote! {
                (#pm1, #pm2, #pm3, #nut_prec_pm)
            };

            let nut_prec = match get_array_as_radians(pck, &nut_prec_key) {
                None => quote! {(&[] as &[f64], &[] as &[f64])},
                Some(coeffs) => {
                    let (theta0, theta1) = unpair(&coeffs);
                    quote! {
                        (&[#(#theta0),*], &[#(#theta1),*])
                    }
                }
            };

            code.extend(quote! {
                impl RotationalElements for #ident {
                    fn nutation_precession_coefficients(&self) -> NutationPrecessionCoefficients {
                        #nut_prec
                    }
                    fn right_ascension_coefficients(&self) -> PolynomialCoefficients {
                        #ra
                    }
                    fn declination_coefficients(&self) -> PolynomialCoefficients {
                        #dec
                    }
                    fn prime_meridian_coefficients(&self) -> PolynomialCoefficients {
                        #pm
                    }
                }
            });

            nutation_precession_match_arms.extend(quote! {
                DynOrigin::#ident => Some(#nut_prec),
            });

            right_ascension_match_arms.extend(quote! {
                DynOrigin::#ident => Some(#ra),
            });

            declination_match_arms.extend(quote! {
                DynOrigin::#ident => Some(#dec),
            });

            prime_meridian_match_arms.extend(quote! {
                DynOrigin::#ident => Some(#pm),
            });
        }
    }

    code.extend(quote! {
        impl MaybePointMass for DynOrigin {
            fn maybe_gravitational_parameter(&self) -> Option<f64> {
                match self {
                    #point_mass_match_arms
                    _ => None,
                }
            }
        }
        impl MaybeTriaxialEllipsoid for DynOrigin {
            fn maybe_radii(&self) -> Option<Radii> {
                match self {
                    #ellipsoid_match_arms
                    _ => None,
                }
            }
        }
        impl MaybeSpheroid for DynOrigin {}
        impl MaybeRotationalElements for DynOrigin {
            fn maybe_nutation_precession_coefficients(&self) -> Option<NutationPrecessionCoefficients> {
                match self {
                    #nutation_precession_match_arms
                    _ => None,
                }
            }
            fn maybe_right_ascension_coefficients(&self) -> Option<PolynomialCoefficients> {
                match self {
                    #right_ascension_match_arms
                    _ => None,
                }
            }
            fn maybe_declination_coefficients(&self) -> Option<PolynomialCoefficients> {
                match self {
                    #declination_match_arms
                    _ => None,
                }
            }
            fn maybe_prime_meridian_coefficients(&self) -> Option<PolynomialCoefficients> {
                match self {
                    #prime_meridian_match_arms
                    _ => None,
                }
            }
        }
    });

    write_file(path, "generated.rs", code)
}
