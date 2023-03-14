use lox_io::spice::Kernel;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn main() {
    let pck = include_str!("../../../data/pck00011.tpc");
    let kernel = Kernel::from_string(pck).expect("parsing should succeed");
    let code = planets(&kernel).to_string();
    let file = Path::new(file!());
    let out = file
        .parent()
        .unwrap()
        .join("../../../crates/lox_core/src/bodies/planets.rs");
    fs::write(&out, code).expect("file should be writeable");
    Command::new("rustfmt")
        .args([out.to_str().unwrap()])
        .status()
        .expect("formatting should work");
}

const PLANETS: [(i32, &str); 9] = [
    (199, "Mercury"),
    (299, "Venus"),
    (399, "Earth"),
    (499, "Mars"),
    (599, "Jupiter"),
    (699, "Saturn"),
    (799, "Uranus"),
    (899, "Neptune"),
    (999, "Pluto"),
];

fn planets(kernel: &Kernel) -> TokenStream {
    let tokens = PLANETS.iter().map(|(id, name)| {
        let ident = format_ident!("{}", name);
        let mut tokens = quote! {
            pub struct #ident;

            impl NaifId for #ident {
                fn id() -> i32 {
                    #id
                }
            }
        };

        let radii = format!("BODY{id}_RADII");
        if let Some(radii) = kernel.get_double_array(&radii) {
            let max_eq = radii.first().expect("radius should be here");
            let min_eq = radii.get(1).expect("radius should be here");
            let polar = radii.get(2).expect("radius should be here");
            let mean = (max_eq + min_eq + polar) / 3.0;
            tokens = quote! {
                #tokens

                impl Ellipsoid for #ident {
                    fn max_equatorial_radius() -> f64 {
                        #max_eq
                    }
                    fn min_equatorial_radius() -> f64 {
                        #min_eq
                    }
                    fn polar_radius() -> f64 {
                        #polar
                    }
                    fn mean_radius() -> f64 {
                        #mean
                    }
                }
            }
        }

        tokens
    });

    quote! {
        use super::{NaifId,Ellipsoid};
        #(#tokens)*
    }
}
