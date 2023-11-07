/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use lox_io::spice::Kernel;

/// Holds the rotational elements for a given body.
///
/// May be parsed directly from PCK data and knows how to represent its code and test components as
/// streams of tokens.
pub(crate) struct BodyRotationalElements<'a, 'b> {
    id: u32,
    ident: &'a Ident,
    right_ascension: [f64; 3],
    declination: [f64; 3],
    prime_meridian: [f64; 3],
    // Not all bodies have trigonometric rotational elements, so we encapsulate these fields in
    // a separate, optional struct.
    trig_elements: Option<TrigonometricElements<'b>>,
}

impl<'a, 'b> BodyRotationalElements<'a, 'b> {
    pub(crate) fn parse(id: u32, ident: &'a Ident, kernel: &'b Kernel) -> Option<Self> {
        let right_ascension_key = format!("BODY{}_POLE_RA", id);
        let declination_key = format!("BODY{}_POLE_DEC", id);
        let prime_meridian_key = format!("BODY{}_PM", id);

        Some(Self {
            id,
            ident,
            right_ascension: get_polynomial_coefficients(&right_ascension_key, kernel)?,
            declination: get_polynomial_coefficients(&declination_key, kernel)?,
            prime_meridian: get_polynomial_coefficients(&prime_meridian_key, kernel)?,
            trig_elements: TrigonometricElements::parse(id, kernel),
        })
    }

    /// Returns the TokenStream corresponding to the [lox_core::bodies::BodyRotationalElements] impl.
    pub(crate) fn code_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let right_ascension = array_tokens_for(&self.right_ascension);
        let declination = array_tokens_for(&self.declination);
        let prime_meridian = array_tokens_for(&self.prime_meridian);

        let mut tokens = quote! {
            #[allow(clippy::approx_constant)]
            impl BodyRotationalElements for #ident {
                const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = #right_ascension;
                const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = #declination;
                const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = #prime_meridian;
            }
        };

        if let Some(trig_elements) = &self.trig_elements {
            tokens.extend(trig_elements.code_tokens(self.ident));
        }

        tokens
    }

    /// Returns the TokenStream testing the [lox_core::bodies::BodyRotationalElements] impl.
    pub(crate) fn test_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let right_ascension = array_tokens_for(&self.right_ascension);
        let declination = array_tokens_for(&self.declination);
        let prime_meridian = array_tokens_for(&self.prime_meridian);

        let right_ascension_test_name = format_ident!(
            "test_rotational_elements_right_ascension_coefficients_{}",
            self.id
        );
        let declination_test_name = format_ident!(
            "test_rotational_elements_declination_coefficients_{}",
            self.id
        );
        let prime_meridian_test_name = format_ident!(
            "test_rotational_elements_prime_meridian_coefficients_{}",
            self.id
        );

        let mut tokens = quote! {
            #[test]
            fn #right_ascension_test_name() {
                assert_eq!(#right_ascension, #ident::RIGHT_ASCENSION_COEFFICIENTS)
            }

            #[test]
            fn #declination_test_name() {
                assert_eq!(#declination, #ident::DECLINATION_COEFFICIENTS)
            }

            #[test]
            fn #prime_meridian_test_name() {
                assert_eq!(#prime_meridian, #ident::PRIME_MERIDIAN_COEFFICIENTS)
            }
        };

        if let Some(trig_elements) = &self.trig_elements {
            tokens.extend(trig_elements.test_tokens(self.id, self.ident));
        }

        tokens
    }

    pub(crate) fn has_trig_elements(&self) -> bool {
        self.trig_elements.is_some()
    }
}

/// The trigonometric rotational elements for a given body and, optionally, its system, where such
/// data is available.
struct TrigonometricElements<'a> {
    nut_prec_right_ascension: &'a Vec<f64>,
    nut_prec_declination: &'a Vec<f64>,
    nut_prec_prime_meridian: &'a Vec<f64>,
}

impl<'a> TrigonometricElements<'a> {
    fn parse(id: u32, kernel: &'a Kernel) -> Option<Self> {
        let nut_prec_right_ascension_key = format!("BODY{}_NUT_PREC_RA", id);
        let nut_prec_declination_key = format!("BODY{}_NUT_PREC_DEC", id);
        let nut_prec_prime_meridian_key = format!("BODY{}_NUT_PREC_PM", id);

        Some(Self {
            nut_prec_right_ascension: kernel.get_double_array(&nut_prec_right_ascension_key)?,
            nut_prec_declination: kernel.get_double_array(&nut_prec_declination_key)?,
            nut_prec_prime_meridian: kernel.get_double_array(&nut_prec_prime_meridian_key)?,
        })
    }

    /// Returns the TokenStream corresponding to the [lox_core::bodies::BodyTrigRotationalElements] impl.
    fn code_tokens(&self, ident: &Ident) -> TokenStream {
        let nut_prec_ra = slice_tokens_for(self.nut_prec_right_ascension);
        let nut_prec_dec = slice_tokens_for(self.nut_prec_declination);
        let nut_prec_pm = slice_tokens_for(self.nut_prec_prime_meridian);

        quote! {
            #[allow(clippy::approx_constant)]
            impl BodyTrigRotationalElements for #ident {
                const NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS: &'static [PolynomialCoefficient] = #nut_prec_ra;
                const NUT_PREC_DECLINATION_COEFFICIENTS: &'static [PolynomialCoefficient] = #nut_prec_dec;
                const NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS: &'static [PolynomialCoefficient] = #nut_prec_pm;
            }
        }
    }

    /// Returns the TokenStream testing the [lox_core::bodies::BodyTrigRotationalElements] impl.
    fn test_tokens(&self, id: u32, ident: &Ident) -> TokenStream {
        let nut_prec_ra_test_name = format_ident!(
            "test_trig_rotational_elements_nut_prec_right_ascension_coefficients{}",
            id
        );
        let nut_prec_dec_test_name = format_ident!(
            "test_trig_rotational_elements_nut_prec_declination_coefficients{}",
            id
        );
        let nut_prec_pm_test_name = format_ident!(
            "test_trig_rotational_elements_nut_prec_prime_meridian_coefficients{}",
            id
        );

        let nut_prec_ra = slice_tokens_for(self.nut_prec_right_ascension);
        let nut_prec_dec = slice_tokens_for(self.nut_prec_declination);
        let nut_prec_pm = slice_tokens_for(self.nut_prec_prime_meridian);

        quote! {
            #[test]
            fn #nut_prec_ra_test_name() {
                assert_eq!(#nut_prec_ra, #ident::NUT_PREC_RIGHT_ASCENSION_COEFFICIENTS)
            }

            #[test]
            fn #nut_prec_dec_test_name() {
                assert_eq!(#nut_prec_dec, #ident::NUT_PREC_DECLINATION_COEFFICIENTS)
            }

            #[test]
            fn #nut_prec_pm_test_name() {
                assert_eq!(#nut_prec_pm, #ident::NUT_PREC_PRIME_MERIDIAN_COEFFICIENTS)
            }
        }
    }
}

/// The trigonometric rotational elements for a given system.
pub(crate) struct BarycenterTrigElements<'a, 'b> {
    id: u32,
    ident: &'a Ident,
    nut_prec_angles: &'b Vec<f64>,
}

impl<'a, 'b> BarycenterTrigElements<'a, 'b> {
    pub(crate) fn parse(id: u32, ident: &'a Ident, kernel: &'b Kernel) -> Option<Self> {
        let nut_prec_angles_key = format!("BODY{}_NUT_PREC_ANGLES", id);

        Some(Self {
            id,
            ident,
            nut_prec_angles: kernel.get_double_array(&nut_prec_angles_key)?,
        })
    }

    /// Returns the TokenStream corresponding to the [lox_core::bodies::BarycenterRotationalElements] impl.
    pub(crate) fn code_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let nut_prec_angles = slice_tokens_for(self.nut_prec_angles);

        quote! {
            #[allow(clippy::approx_constant)]
            impl BarycenterTrigRotationalElements for #ident {
                const NUT_PREC_ANGLES: &'static [PolynomialCoefficient] = #nut_prec_angles;
            }
        }
    }

    /// Returns the TokenStream testing the [lox_core::bodies::BarycenterRotationalElements] impl.
    pub(crate) fn test_tokens(&self) -> TokenStream {
        let nut_prec_angles_test_name = format_ident!(
            "test_barycenter_trig_rotational_elements_nut_prec_angles_{}",
            self.id
        );
        let ident = self.ident;
        let nut_prec_angles = slice_tokens_for(self.nut_prec_angles);

        quote! {
            #[test]
            fn #nut_prec_angles_test_name() {
                assert_eq!(#nut_prec_angles, #ident::NUT_PREC_ANGLES)
            }
        }
    }
}

fn get_polynomial_coefficients(key: &str, kernel: &Kernel) -> Option<[f64; 3]> {
    match kernel.get_double_array(key) {
        None => None,
        Some(polynomials) if polynomials.len() == 2 => Some([polynomials[0], polynomials[1], 0.0]),
        Some(polynomials) if polynomials.len() == 3 => {
            Some([polynomials[0], polynomials[1], polynomials[2]])
        }
        Some(polynomials) => {
            panic!(
                "PCK DoubleArray with key {} had size {}, expected 2 <= size <= 3",
                key,
                polynomials.len(),
            )
        }
    }
}

fn array_tokens_for(data: &[f64]) -> TokenStream {
    quote! { [#(#data),*] }
}

fn slice_tokens_for(data: &[f64]) -> TokenStream {
    quote! { &[#(#data),*] }
}
