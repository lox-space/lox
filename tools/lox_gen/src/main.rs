/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use lazy_static::lazy_static;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use lox_core::bodies::*;
use lox_io::spice::Kernel;

use crate::rotational_elements::{CoefficientKernel, RotationalElements};

mod rotational_elements;

type Generator = fn(
    imports: &mut HashSet<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: i32,
    data: &Data,
);

type Task = (String, Vec<Box<dyn Body>>, Vec<Generator>);

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
    let tasks: Vec<Task> = vec![
        (
            "sun".to_owned(),
            vec![Box::new(Sun)],
            vec![point_mass, spheroid, rotational_elements],
        ),
        (
            "barycenters".to_owned(),
            vec![
                Box::new(SolarSystemBarycenter),
                Box::new(MercuryBarycenter),
                Box::new(VenusBarycenter),
                Box::new(EarthBarycenter),
                Box::new(MarsBarycenter),
                Box::new(JupiterBarycenter),
                Box::new(SaturnBarycenter),
                Box::new(UranusBarycenter),
                Box::new(NeptuneBarycenter),
                Box::new(PlutoBarycenter),
            ],
            vec![point_mass],
        ),
        (
            "planets".to_owned(),
            vec![
                Box::new(Mercury),
                Box::new(Venus),
                Box::new(Earth),
                Box::new(Mars),
                Box::new(Jupiter),
                Box::new(Saturn),
                Box::new(Uranus),
                Box::new(Neptune),
                Box::new(Pluto),
            ],
            vec![point_mass, spheroid, rotational_elements],
        ),
        (
            "satellites".to_owned(),
            vec![
                Box::new(Moon),
                Box::new(Phobos),
                Box::new(Deimos),
                Box::new(Io),
                Box::new(Europa),
                Box::new(Ganymede),
                Box::new(Callisto),
                Box::new(Amalthea),
                Box::new(Himalia),
                Box::new(Elara),
                Box::new(Pasiphae),
                Box::new(Sinope),
                Box::new(Lysithea),
                Box::new(Carme),
                Box::new(Ananke),
                Box::new(Leda),
                Box::new(Thebe),
                Box::new(Adrastea),
                Box::new(Metis),
                Box::new(Callirrhoe),
                Box::new(Themisto),
                Box::new(Magaclite),
                Box::new(Taygete),
                Box::new(Chaldene),
                Box::new(Harpalyke),
                Box::new(Kalyke),
                Box::new(Iocaste),
                Box::new(Erinome),
                Box::new(Isonoe),
                Box::new(Praxidike),
                Box::new(Autonoe),
                Box::new(Thyone),
                Box::new(Hermippe),
                Box::new(Aitne),
                Box::new(Eurydome),
                Box::new(Euanthe),
                Box::new(Euporie),
                Box::new(Orthosie),
                Box::new(Sponde),
                Box::new(Kale),
                Box::new(Pasithee),
                Box::new(Hegemone),
                Box::new(Mneme),
                Box::new(Aoede),
                Box::new(Thelxinoe),
                Box::new(Arche),
                Box::new(Kallichore),
                Box::new(Helike),
                Box::new(Carpo),
                Box::new(Eukelade),
                Box::new(Cyllene),
                Box::new(Kore),
                Box::new(Herse),
                Box::new(Dia),
                Box::new(Mimas),
                Box::new(Enceladus),
                Box::new(Tethys),
                Box::new(Dione),
                Box::new(Rhea),
                Box::new(Titan),
                Box::new(Hyperion),
                Box::new(Iapetus),
                Box::new(Phoebe),
                Box::new(Janus),
                Box::new(Epimetheus),
                Box::new(Helene),
                Box::new(Telesto),
                Box::new(Calypso),
                Box::new(Atlas),
                Box::new(Prometheus),
                Box::new(Pandora),
                Box::new(Pan),
                Box::new(Ymir),
                Box::new(Paaliaq),
                Box::new(Tarvos),
                Box::new(Ijiraq),
                Box::new(Suttungr),
                Box::new(Kiviuq),
                Box::new(Mundilfari),
                Box::new(Albiorix),
                Box::new(Skathi),
                Box::new(Erriapus),
                Box::new(Siarnaq),
                Box::new(Thrymr),
                Box::new(Narvi),
                Box::new(Methone),
                Box::new(Pallene),
                Box::new(Polydeuces),
                Box::new(Daphnis),
                Box::new(Aegir),
                Box::new(Bebhionn),
                Box::new(Bergelmir),
                Box::new(Bestla),
                Box::new(Farbauti),
                Box::new(Fenrir),
                Box::new(Fornjot),
                Box::new(Hati),
                Box::new(Hyrrokkin),
                Box::new(Kari),
                Box::new(Loge),
                Box::new(Skoll),
                Box::new(Surtur),
                Box::new(Anthe),
                Box::new(Jarnsaxa),
                Box::new(Greip),
                Box::new(Tarqeq),
                Box::new(Aegaeon),
                Box::new(Ariel),
                Box::new(Umbriel),
                Box::new(Titania),
                Box::new(Oberon),
                Box::new(Miranda),
                Box::new(Cordelia),
                Box::new(Ophelia),
                Box::new(Bianca),
                Box::new(Cressida),
                Box::new(Desdemona),
                Box::new(Juliet),
                Box::new(Portia),
                Box::new(Rosalind),
                Box::new(Belinda),
                Box::new(Puck),
                Box::new(Caliban),
                Box::new(Sycorax),
                Box::new(Prospero),
                Box::new(Setebos),
                Box::new(Stephano),
                Box::new(Trinculo),
                Box::new(Francisco),
                Box::new(Margaret),
                Box::new(Ferdinand),
                Box::new(Perdita),
                Box::new(Mab),
                Box::new(Cupid),
                Box::new(Triton),
                Box::new(Nereid),
                Box::new(Naiad),
                Box::new(Thalassa),
                Box::new(Despina),
                Box::new(Galatea),
                Box::new(Larissa),
                Box::new(Proteus),
                Box::new(Halimede),
                Box::new(Psamathe),
                Box::new(Sao),
                Box::new(Laomedeia),
                Box::new(Neso),
                Box::new(Charon),
                Box::new(Nix),
                Box::new(Hydra),
                Box::new(Kerberos),
                Box::new(Styx),
            ],
            vec![point_mass, tri_axial, rotational_elements],
        ),
        (
            "minor".to_owned(),
            vec![
                Box::new(Gaspra),
                Box::new(Ida),
                Box::new(Dactyl),
                Box::new(Ceres),
                Box::new(Pallas),
                Box::new(Vesta),
                Box::new(Psyche),
                Box::new(Lutetia),
                Box::new(Kleopatra),
                Box::new(Eros),
                Box::new(Davida),
                Box::new(Mathilde),
                Box::new(Steins),
                Box::new(Braille),
                Box::new(WilsonHarrington),
                Box::new(Toutatis),
                Box::new(Itokawa),
                Box::new(Bennu),
            ],
            vec![point_mass, tri_axial, rotational_elements],
        ),
    ];
    tasks
        .iter()
        .for_each(|(file, bodies, generators)| write_file(file, bodies, generators, &data));
}

const COPYRIGHT_NOTICE: &str = "/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */\n\n";

const AUTO_GENERATION_NOTICE: &str = "// Auto-generated by `lox_gen`. Do not edit!\n\n";

lazy_static! {
    static ref TARGET_DIR: PathBuf = {
        let parent = Path::new(file!()).parent().unwrap();
        parent.join(Path::new("../../../crates/lox_core/src/bodies/generated/"))
    };
}

fn write_file(file: &str, bodies: &[Box<dyn Body>], generators: &[Generator], data: &Data) {
    let mut code = String::from(COPYRIGHT_NOTICE);
    code.push_str(AUTO_GENERATION_NOTICE);
    code.push_str(&generate_code(bodies, generators, data));

    let out = TARGET_DIR.join(format!("{}.rs", file));
    fs::write(&out, code).expect("file should be writeable");

    Command::new("rustfmt")
        .args([out.to_str().unwrap()])
        .status()
        .expect("formatting should work");
}

fn generate_code(bodies: &[Box<dyn Body>], generators: &[Generator], data: &Data) -> String {
    let mut imports: HashSet<Ident> = HashSet::new();
    let mut code = quote!();
    let mut tests = quote!();

    bodies.iter().for_each(|body| {
        let ident = format_ident!("{}", body.name().replace([' ', '-'], ""));
        generators.iter().for_each(|generator| {
            generator(
                &mut imports,
                &mut code,
                &mut tests,
                &ident,
                body.id().0,
                data,
            )
        })
    });

    let imports_iter = imports.iter();
    let module = quote! {
        use crate::bodies::{#(#imports_iter),*};

        #code

        #[cfg(test)]
        #[allow(clippy::approx_constant)] // at least one parsed constant is close to TAU
        mod tests {
            use crate::bodies::*;

            #tests
        }
    };
    module.to_string()
}

fn spheroid(
    imports: &mut HashSet<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: i32,
    data: &Data,
) {
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

        let test_name = format_ident!("test_spheroid_{}", id as u32);

        *tests = quote! {
            #tests

            #[test]
            fn #test_name() {
                assert_eq!(#ident::polar_radius(), #polar);
                assert_eq!(#ident::mean_radius(), #mean);
                assert_eq!(#ident::equatorial_radius(), #equatorial);
            }
        };

        // Imports are added after any early returns or panics, guaranteeing that the trait is
        // implemented for ident and avoiding unused imports.
        imports.extend([
            ident.clone(),
            format_ident!("Ellipsoid"),
            format_ident!("Spheroid"),
        ]);
    }
}

fn tri_axial(
    imports: &mut HashSet<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: i32,
    data: &Data,
) {
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

        let test_name = format_ident!("test_tri_axial_{}", id as u32);

        *tests = quote! {
            #tests

            #[test]
            fn #test_name() {
                assert_eq!(#ident::polar_radius(), #polar);
                assert_eq!(#ident::mean_radius(), #mean);
                assert_eq!(#ident::subplanetary_radius(), #subplanetary);
                assert_eq!(#ident::along_orbit_radius(), #along_orbit);
            }
        };

        // Imports are added after any early returns or panics, guaranteeing that the trait is
        // implemented for ident and avoiding unused imports.
        imports.extend([
            ident.clone(),
            format_ident!("Ellipsoid"),
            format_ident!("TriAxial"),
        ]);
    }
}

fn point_mass(
    imports: &mut HashSet<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: i32,
    data: &Data,
) {
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

        let test_name = format_ident!("test_point_mass_{}", id as u32);

        *tests = quote! {
            #tests

            #[test]
            fn #test_name() {
                assert_eq!(#ident::gravitational_parameter(), #gm);
            }
        };

        // Imports are added after any early returns or panics, guaranteeing that the trait is
        // implemented for ident and avoiding unused imports.
        imports.extend([ident.clone(), format_ident!("PointMass")]);
    };
}

/// Generates implementations for [lox_core::bodies::RotationalElements].
fn rotational_elements(
    imports: &mut HashSet<Ident>,
    code: &mut TokenStream,
    tests: &mut TokenStream,
    ident: &Ident,
    id: i32,
    data: &Data,
) {
    let elements = match RotationalElements::parse(id as u32, ident, CoefficientKernel(&data.pck)) {
        Ok(elements) => elements,
        Err(err) => panic!("failed to parse rotational elements for {}: {}", ident, err),
    };

    // Imports are added after any early returns or panics, guaranteeing that the trait is
    // implemented for ident and avoiding unused imports.
    imports.extend([
        ident.clone(),
        format_ident!("RotationalElements"),
        format_ident!("PolynomialCoefficients"),
        format_ident!("NutationPrecessionCoefficients"),
    ]);
    code.extend(elements.code_tokens());
    tests.extend(elements.test_tokens());
}
