/*
 * Copyright (c) 2024. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use lox_io::spice::Kernel;

use crate::bodies::BodyDef;
use crate::rotational_elements::{CoefficientKernel, RotationalElements};

pub trait Generator {
    fn imports(&self) -> Vec<Ident>;
    fn generate_code(&self, code: &mut TokenStream, tests: &mut TokenStream, body: &BodyDef);
}

pub struct BaseGenerator;

impl Generator for BaseGenerator {
    fn imports(&self) -> Vec<Ident> {
        vec![]
    }

    fn generate_code(&self, code: &mut TokenStream, _tests: &mut TokenStream, body: &BodyDef) {
        let ident = body.ident();

        code.extend(quote! {
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
            pub struct #ident;
        });

        if let Some(body_trait) = body.body_trait() {
            code.extend(quote! {
                impl #body_trait for #ident {}
            })
        }
    }
}

pub struct BodyGenerator;

impl Generator for BodyGenerator {
    fn imports(&self) -> Vec<Ident> {
        vec![format_ident!("Body"), format_ident!("NaifId")]
    }

    fn generate_code(&self, code: &mut TokenStream, tests: &mut TokenStream, body: &BodyDef) {
        let ident = body.ident();
        let name = body.name;
        let id = body.id;

        code.extend(quote! {
            impl Body for #ident {
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

        let test = test_name("body", body);

        tests.extend(quote! {
            #[test]
            fn #test() {
                assert_eq!(#ident.id(), NaifId(#id));
                assert_eq!(#ident.name(), #name);
                assert_eq!(format!("{}", #ident), #name);
            }
        });
    }
}

pub struct SpheroidGenerator {
    pub pck: Kernel,
}

impl Generator for SpheroidGenerator {
    fn imports(&self) -> Vec<Ident> {
        vec![format_ident!("Ellipsoid"), format_ident!("Spheroid")]
    }

    fn generate_code(&self, code: &mut TokenStream, tests: &mut TokenStream, body: &BodyDef) {
        let ident = body.ident();
        let id = body.id;

        let radii = format!("BODY{id}_RADII");
        if let Some(radii) = self.pck.get_double_array(&radii) {
            let equatorial = radii.first().expect("radius should be here");
            let polar = radii.get(2).expect("radius should be here");
            let mean = (2.0 * equatorial + polar) / 3.0;

            code.extend(quote! {
                impl Ellipsoid for #ident {
                    fn polar_radius(&self) -> f64 {
                        #polar
                    }
                    fn mean_radius(&self) -> f64 {
                        #mean
                    }
                }

                impl Spheroid for #ident {
                    fn equatorial_radius(&self) -> f64 {
                        #equatorial
                    }
                }
            });

            let test = test_name("spheroid", body);

            tests.extend(quote! {
                #[test]
                fn #test() {
                    assert_eq!(#ident.polar_radius(), #polar);
                    assert_eq!(#ident.mean_radius(), #mean);
                    assert_eq!(#ident.equatorial_radius(), #equatorial);
                }
            });
        }
    }
}

pub struct TriAxialGenerator {
    pub pck: Kernel,
}

impl Generator for TriAxialGenerator {
    fn imports(&self) -> Vec<Ident> {
        vec![format_ident!("Ellipsoid"), format_ident!("TriAxial")]
    }

    fn generate_code(&self, code: &mut TokenStream, tests: &mut TokenStream, body: &BodyDef) {
        let ident = body.ident();
        let id = body.id;
        let key = format!("BODY{id}_RADII");
        if let Some(radii) = self.pck.get_double_array(&key) {
            let subplanetary = radii.first().expect("radius should be here");
            let along_orbit = radii.get(1).expect("radius should be here");
            let polar = radii.get(2).expect("radius should be here");
            let mean = (subplanetary + along_orbit + polar) / 3.0;

            code.extend(quote! {
                impl Ellipsoid for #ident {
                    fn polar_radius(&self) -> f64 {
                        #polar
                    }
                    fn mean_radius(&self) -> f64 {
                        #mean
                    }
                }

                impl TriAxial for #ident {
                    fn subplanetary_radius(&self) -> f64 {
                        #subplanetary
                    }
                    fn along_orbit_radius(&self) -> f64 {
                        #along_orbit
                    }
                }
            });

            let test = test_name("tri_axial", body);

            tests.extend(quote! {
                #[test]
                fn #test() {
                    assert_eq!(#ident.polar_radius(), #polar);
                    assert_eq!(#ident.mean_radius(), #mean);
                    assert_eq!(#ident.subplanetary_radius(), #subplanetary);
                    assert_eq!(#ident.along_orbit_radius(), #along_orbit);
                }
            });
        }
    }
}

pub struct PointMassGenerator {
    pub gm: Kernel,
}

impl Generator for PointMassGenerator {
    fn imports(&self) -> Vec<Ident> {
        vec![format_ident!("PointMass")]
    }

    fn generate_code(&self, code: &mut TokenStream, tests: &mut TokenStream, body: &BodyDef) {
        let ident = body.ident();
        let id = body.id;

        let key = format!("BODY{id}_GM");
        if let Some(gm) = self.gm.get_double_array(&key) {
            let gm = gm.first().unwrap();
            code.extend(quote! {
                impl PointMass for #ident {
                    fn gravitational_parameter(&self) -> f64 {
                        #gm
                    }
                }
            });

            let test = test_name("point_mass", body);

            tests.extend(quote! {
                #[test]
                fn #test() {
                    assert_eq!(#ident.gravitational_parameter(), #gm);
                }
            });
        };
    }
}

pub struct RotationalElementsGenerator {
    pub pck: Kernel,
}

impl Generator for RotationalElementsGenerator {
    fn imports(&self) -> Vec<Ident> {
        vec![
            format_ident!("RotationalElements"),
            format_ident!("PolynomialCoefficients"),
            format_ident!("NutationPrecessionCoefficients"),
        ]
    }

    fn generate_code(&self, code: &mut TokenStream, tests: &mut TokenStream, body: &BodyDef) {
        let ident = body.ident();
        let id = body.id;

        let elements =
            match RotationalElements::parse(id as u32, &ident, CoefficientKernel(&self.pck)) {
                Ok(elements) => elements,
                Err(err) => panic!("failed to parse rotational elements for {}: {}", ident, err),
            };

        code.extend(elements.code_tokens());
        tests.extend(elements.test_tokens());
    }
}

fn test_name(trait_name: &str, body: &BodyDef) -> Ident {
    format_ident!("test_{}_{}", trait_name, body.id as u32)
}
