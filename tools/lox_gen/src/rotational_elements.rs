/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::Data;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

type PolynomialCoefficients = (f64, f64, f64);

/// Holds the rotational elements for a given body.
///
/// May be parsed directly from PCK data and knows how to represent its code and test components as
/// streams of tokens.
pub(crate) struct RotationalElements<'a> {
    id: i32,
    ident: &'a Ident,
    right_ascension: PolynomialCoefficients,
    declination: PolynomialCoefficients,
    prime_meridian: PolynomialCoefficients,
}

impl<'a> RotationalElements<'a> {
    pub(crate) fn parse(id: i32, ident: &'a Ident, data: &Data) -> Option<Self> {
        let right_ascension_key = format!("BODY{}_POLE_RA", id);
        let right_ascension = get_polynomial_coefficients(&right_ascension_key, data)?;

        let declination_key = format!("BODY{}_POLE_DEC", id);
        let declination = get_polynomial_coefficients(&declination_key, data)?;

        let prime_meridian_key = format!("BODY{}_PM", id);
        let prime_meridian = get_polynomial_coefficients(&prime_meridian_key, data)?;

        Some(Self {
            id,
            ident,
            right_ascension,
            declination,
            prime_meridian,
        })
    }

    pub(crate) fn code_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let right_ascension = self.right_ascension();
        let declination = self.declination();
        let prime_meridian = self.prime_meridian();

        quote! {
            impl RotationalElements for #ident {
                const RIGHT_ASCENSION_COEFFICIENTS: [PolynomialCoefficient; 3] = #right_ascension;
                const DECLINATION_COEFFICIENTS: [PolynomialCoefficient; 3] = #declination;
                const PRIME_MERIDIAN_COEFFICIENTS: [PolynomialCoefficient; 3] = #prime_meridian;
            }
        }
    }

    pub(crate) fn test_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let right_ascension = self.right_ascension();
        let declination = self.declination();
        let prime_meridian = self.prime_meridian();

        let right_ascension_test_name =
            format_ident!("test_right_ascension_coefficients_{}", self.id as u32);
        let declination_test_name =
            format_ident!("test_declination_coefficients_{}", self.id as u32);
        let prime_meridian_test_name =
            format_ident!("test_prime_meridian_coefficients_{}", self.id as u32);

        quote! {
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
        }
    }

    fn right_ascension(&self) -> TokenStream {
        let (ra_0, ra_1, ra_2) = self.right_ascension;
        quote! { [#ra_0, #ra_1, #ra_2] }
    }

    fn declination(&self) -> TokenStream {
        let (dec_0, dec_1, dec_2) = self.declination;
        quote! { [#dec_0, #dec_1, #dec_2] }
    }

    fn prime_meridian(&self) -> TokenStream {
        let (pm_0, pm_1, pm_2) = self.prime_meridian;
        quote! { [#pm_0, #pm_1, #pm_2] }
    }
}

fn get_polynomial_coefficients(key: &str, data: &Data) -> Option<PolynomialCoefficients> {
    match data.pck.get_double_array(key) {
        None => None,
        Some(polynomials) if polynomials.len() == 2 => Some((polynomials[0], polynomials[1], 0.0)),
        Some(polynomials) if polynomials.len() == 3 => {
            Some((polynomials[0], polynomials[1], polynomials[2]))
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
