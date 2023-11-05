/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fs;
use std::path::Path;
use std::process::Command;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::rotational_elements::RotationalElements;
use lox_io::spice::Kernel;
use naif_ids::{Body, BARYCENTERS, MINOR_BODIES, PLANETS, SATELLITES, SUN};

mod naif_ids;
mod rotational_elements;

type Generator = fn(
    imports: &mut Vec<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: &i32,
    data: &Data,
);

struct Data {
    pck: Kernel,
    gm: Kernel,
}

pub fn main() {
    let pck = Kernel::from_string(include_str!("../../../data/pck00011.tpc"))
        .expect("parsing should succeed");
    let gm = Kernel::from_string(include_str!("../../../data/gm_de440.tpc"))
        .expect("parsing should succeed");
    let data = Data { pck, gm };
    let bodies: [(&str, Vec<Body>, Vec<Generator>); 5] = [
        (
            "sun",
            Vec::from(SUN),
            vec![naif_id, point_mass, spheroid, rotational_elements],
        ),
        (
            "barycenters",
            Vec::from(BARYCENTERS),
            vec![naif_id, point_mass],
        ),
        (
            "planets",
            Vec::from(PLANETS),
            vec![naif_id, point_mass, spheroid, rotational_elements],
        ),
        (
            "satellites",
            Vec::from(SATELLITES),
            vec![naif_id, point_mass, tri_axial, rotational_elements],
        ),
        (
            "minor",
            Vec::from(MINOR_BODIES),
            vec![naif_id, point_mass, tri_axial, rotational_elements],
        ),
    ];
    bodies
        .iter()
        .for_each(|(file, bodies, generators)| write_file(file, bodies, generators, &data));
}

fn write_file(file: &str, bodies: &[Body], generators: &[Generator], data: &Data) {
    let path = format!("../../../crates/lox_core/src/bodies/{file}.rs");
    let mut code = String::from("// Auto-generated by `lox_gen`. Do not edit!\n");
    code.push_str(&generate_code(bodies, generators, data));
    let file = Path::new(file!());
    let out = file.parent().unwrap().join(path);
    fs::write(&out, code).expect("file should be writeable");
    Command::new("rustfmt")
        .args([out.to_str().unwrap()])
        .status()
        .expect("formatting should work");
}

fn generate_code(bodies: &[Body], generators: &[Generator], data: &Data) -> String {
    let mut imports: Vec<Ident> = Vec::new();
    let mut code = quote!();
    let mut tests = quote!();

    bodies.iter().for_each(|(id, name)| {
        let ident = format_ident!("{}", name);
        code = quote! {
            #code
            #[derive(Debug, Clone, Copy, Eq, PartialEq)]
            pub struct #ident;
        };
        generators
            .iter()
            .for_each(|generator| generator(&mut imports, &mut code, &mut tests, &ident, id, data))
    });

    let module = quote! {
        use super::{#(#imports),*};

        #code

        #[cfg(test)]
        mod tests {
            use super::*;

            #tests
        }
    };
    module.to_string()
}

fn naif_id(
    imports: &mut Vec<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: &i32,
    _data: &Data,
) {
    let trait_name = format_ident!("NaifId");

    if !imports.contains(&trait_name) {
        imports.push(trait_name);
    }

    *code = quote! {
        #code

        impl NaifId for #ident {
            fn id() -> i32 {
                #id
            }
        }
    };

    let test_name = format_ident!("test_naif_id_{}", *id as u32);

    *tests = quote! {
        #tests

        #[test]
        fn #test_name() {
            assert_eq!(#ident::id(), #id)
        }
    }
}

fn spheroid(
    imports: &mut Vec<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: &i32,
    data: &Data,
) {
    let trait_name = format_ident!("Ellipsoid");
    if !imports.contains(&trait_name) {
        imports.push(trait_name);
    }

    let trait_name = format_ident!("Spheroid");
    if !imports.contains(&trait_name) {
        imports.push(trait_name);
    }

    let radii = format!("BODY{id}_RADII");
    if let Some(radii) = data.pck.get_double_array(&radii) {
        let equatorial = radii.first().expect("radius should be here");
        let polar = radii.get(2).expect("radius should be here");
        let mean = (2.0 * equatorial + polar) / 3.0;

        *code = quote! {
            #code

            impl Ellipsoid for #ident {
                fn polar_radius() -> f64 {
                    #polar
                }
                fn mean_radius() -> f64 {
                    #mean
                }
            }

            impl Spheroid for #ident {
                fn equatorial_radius() -> f64 {
                    #equatorial
                }
            }
        };

        let test_name = format_ident!("test_spheroid_{}", *id as u32);

        *tests = quote! {
            #tests

            #[test]
            fn #test_name() {
                assert_eq!(#ident::polar_radius(), #polar);
                assert_eq!(#ident::mean_radius(), #mean);
                assert_eq!(#ident::equatorial_radius(), #equatorial);
            }
        }
    }
}

fn tri_axial(
    imports: &mut Vec<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: &i32,
    data: &Data,
) {
    let trait_name = format_ident!("Ellipsoid");
    if !imports.contains(&trait_name) {
        imports.push(trait_name);
    }

    let trait_name = format_ident!("TriAxial");
    if !imports.contains(&trait_name) {
        imports.push(trait_name);
    }

    let radii = format!("BODY{id}_RADII");
    if let Some(radii) = data.pck.get_double_array(&radii) {
        let subplanetary = radii.first().expect("radius should be here");
        let along_orbit = radii.get(1).expect("radius should be here");
        let polar = radii.get(2).expect("radius should be here");
        let mean = (subplanetary + along_orbit + polar) / 3.0;

        *code = quote! {
            #code

            impl Ellipsoid for #ident {
                fn polar_radius() -> f64 {
                    #polar
                }
                fn mean_radius() -> f64 {
                    #mean
                }
            }

            impl TriAxial for #ident {
                fn subplanetary_radius() -> f64 {
                    #subplanetary
                }
                fn along_orbit_radius() -> f64 {
                    #along_orbit
                }
            }
        };

        let test_name = format_ident!("test_tri_axial_{}", *id as u32);

        *tests = quote! {
            #tests

            #[test]
            fn #test_name() {
                assert_eq!(#ident::polar_radius(), #polar);
                assert_eq!(#ident::mean_radius(), #mean);
                assert_eq!(#ident::subplanetary_radius(), #subplanetary);
                assert_eq!(#ident::along_orbit_radius(), #along_orbit);
            }
        }
    }
}

fn point_mass(
    imports: &mut Vec<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: &i32,
    data: &Data,
) {
    let trait_name = format_ident!("PointMass");
    if !imports.contains(&trait_name) {
        imports.push(trait_name);
    }
    let key = format!("BODY{id}_GM");
    if let Some(gm) = data.gm.get_double_array(&key) {
        let gm = gm.first().unwrap();
        *code = quote! {
            #code

            impl PointMass for #ident {
                fn gravitational_parameter() -> f64 {
                    #gm
                }
            }
        };

        let test_name = format_ident!("test_point_mass_{}", *id as u32);

        *tests = quote! {
            #tests

            #[test]
            fn #test_name() {
                assert_eq!(#ident::gravitational_parameter(), #gm);
            }
        }
    };
}

/// Generates implementations for [lox_core::bodies::RotationalElements].
fn rotational_elements(
    imports: &mut Vec<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: &i32,
    data: &Data,
) {
    let shared_imports = vec![
        format_ident!("RotationalElements"),
        format_ident!("PolynomialCoefficient"),
    ];

    for import in shared_imports {
        if !imports.contains(&import) {
            imports.push(import)
        }
    }

    let rot_el = if let Some(rot_el) = RotationalElements::parse(*id, ident, data) {
        rot_el
    } else {
        return;
    };

    code.extend(rot_el.code_tokens());
    tests.extend(rot_el.test_tokens());
}
