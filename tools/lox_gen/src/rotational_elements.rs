/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use thiserror::Error;

use lox_io::spice::Kernel;

use crate::naif_ids::is_planet;

/// Converts [lox_core::bodies::PolynomialCoefficients] into a TokenStream.
pub struct TokenizeablePolynomialCoefficients(f64, f64, f64, Vec<f64>);

impl ToTokens for TokenizeablePolynomialCoefficients {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(a, b, c, d) = self;
        // The explicit type annotation is required to support the case of an empty slice.
        tokens.extend(quote! { (#a, #b, #c, &[#(#d),*] as &[f64]) });
    }
}

/// Converts [lox_core::bodies::NutationPrecessionCoefficients] into a TokenStream.
#[derive(Default)]
pub struct TokenizeableNutPrecCoefficients((Vec<f64>, Vec<f64>));

impl ToTokens for TokenizeableNutPrecCoefficients {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (a, b) = &self.0;
        // The explicit type annotations are required to support the case of an empty slice.
        tokens.extend(quote! { (&[#(#a),*] as &[f64], &[#(#b),*] as &[f64]) });
    }
}

/// Holds the rotational elements for a given body.
///
/// May be parsed directly from PCK data and knows how to represent its code and test components as
/// streams of tokens.
///
/// Wherever data is not available for a given body, the corresponding field will be empty.
pub(crate) struct RotationalElements<'a> {
    id: u32,
    ident: &'a Ident,
    right_ascension: TokenizeablePolynomialCoefficients,
    declination: TokenizeablePolynomialCoefficients,
    prime_meridian: TokenizeablePolynomialCoefficients,

    /// barycenter_nut_prec_coefficients will be the type default for all bodies except the prime
    /// bodies of their respective systems.
    barycenter_nut_prec_coefficients: TokenizeableNutPrecCoefficients,
}

impl<'a> RotationalElements<'a> {
    pub(crate) fn parse(
        id: u32,
        ident: &'a Ident,
        kernel: impl GetPolynomialCoefficients,
    ) -> Result<Self, Box<CoefficientsError>> {
        Ok(Self {
            id,
            ident,
            right_ascension: kernel.get_right_ascension_coefficients_or_default(id)?,
            declination: kernel.get_declination_coefficients_or_default(id)?,
            prime_meridian: kernel.get_prime_meridian_coefficients_or_default(id)?,
            barycenter_nut_prec_coefficients: kernel
                .get_barycenter_nut_prec_coefficients_or_default(id)?,
        })
    }

    /// Returns the TokenStream corresponding to the [lox_core::bodies::BodyRotationalElements] impl.
    pub(crate) fn code_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let barycenter_nut_prec = &self.barycenter_nut_prec_coefficients;
        let right_ascension = &self.right_ascension;
        let declination = &self.declination;
        let prime_meridian = &self.prime_meridian;

        quote! {
            #[allow(clippy::approx_constant)]
            impl RotationalElements for #ident {
                const NUTATION_PRECESSION_COEFFICIENTS: NutationPrecessionCoefficients = #barycenter_nut_prec;
                const RIGHT_ASCENSION_COEFFICIENTS: PolynomialCoefficients = #right_ascension;
                const DECLINATION_COEFFICIENTS: PolynomialCoefficients = #declination;
                const PRIME_MERIDIAN_COEFFICIENTS: PolynomialCoefficients = #prime_meridian;
            }
        }
    }

    /// Returns the TokenStream testing the [lox_core::bodies::BodyRotationalElements] impl.
    pub(crate) fn test_tokens(&self) -> TokenStream {
        let ident = self.ident;
        let barycenter_nut_prec = &self.barycenter_nut_prec_coefficients;
        let right_ascension = &self.right_ascension;
        let declination = &self.declination;
        let prime_meridian = &self.prime_meridian;

        let barycenter_nut_prec_test_name = format_ident!(
            "test_rotational_elements_nutation_precession_coefficients_{}",
            self.id
        );

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

        quote! {
            #[test]
            fn #barycenter_nut_prec_test_name() {
                assert_eq!(#barycenter_nut_prec, #ident::NUTATION_PRECESSION_COEFFICIENTS)
            }

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
}

/// Indicates that a value retrieved from the kernel did not conform to the domain model.
#[derive(Debug, Error)]
pub enum CoefficientsError {
    #[error("kernel coefficients with key {key} had size {actual}, expected at least {actual}")]
    TooFew {
        key: String,
        min: usize,
        actual: usize,
    },
    #[error("kernel coefficients with key {key} had size {actual}, expected at most {actual}")]
    TooMany {
        key: String,
        max: usize,
        actual: usize,
    },
    #[error(
        "barycenter nutation precession coefficients with key {key} had an odd number of \
        terms, but an even number is required"
    )]
    OddNumber { key: String },
}

/// GetPolynomialCoefficients supports translation between a raw PCK kernel and RotationalElements.
pub trait GetPolynomialCoefficients {
    fn get_right_ascension_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>>;

    fn get_declination_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>>;

    fn get_prime_meridian_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>>;

    fn get_barycenter_nut_prec_coefficients_or_default(
        &self,
        barycenter_id: u32,
    ) -> Result<TokenizeableNutPrecCoefficients, Box<CoefficientsError>>;
}

/// Concrete translator from a raw PCK kernel to RotationalElements. Produces coefficients in
/// radians.
pub struct CoefficientKernel<'a>(pub &'a Kernel);

/// The type of coefficient to be retrieved from the kernel.
enum PolynomialCoefficientType {
    RightAscension,
    Declination,
    PrimeMeridian,
}

impl PolynomialCoefficientType {
    /// Returns the pair of kernel keys required to retrieve the trig and non-trig coefficients for
    /// a given body and coefficient type.  
    fn keys(&self, id: u32) -> (String, String) {
        match self {
            PolynomialCoefficientType::RightAscension => (
                format!("BODY{}_POLE_RA", id),
                format!("BODY{}_NUT_PREC_RA", id),
            ),
            PolynomialCoefficientType::Declination => (
                format!("BODY{}_POLE_DEC", id),
                format!("BODY{}_NUT_PREC_DEC", id),
            ),
            PolynomialCoefficientType::PrimeMeridian => {
                (format!("BODY{}_PM", id), format!("BODY{}_NUT_PREC_PM", id))
            }
        }
    }
}

impl GetPolynomialCoefficients for CoefficientKernel<'_> {
    fn get_right_ascension_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>> {
        self.get_polynomial_coefficients_or_default(id, PolynomialCoefficientType::RightAscension)
    }

    fn get_declination_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>> {
        self.get_polynomial_coefficients_or_default(id, PolynomialCoefficientType::Declination)
    }

    fn get_prime_meridian_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>> {
        self.get_polynomial_coefficients_or_default(id, PolynomialCoefficientType::PrimeMeridian)
    }

    fn get_barycenter_nut_prec_coefficients_or_default(
        &self,
        id: u32,
    ) -> Result<TokenizeableNutPrecCoefficients, Box<CoefficientsError>> {
        if !is_planet(id as i32) {
            return Ok(TokenizeableNutPrecCoefficients::default());
        }

        let barycenter_id = id / 100;

        let key = format!("BODY{}_NUT_PREC_ANGLES", barycenter_id);
        match self.0.get_double_array(&key) {
            None => Ok(TokenizeableNutPrecCoefficients::default()),
            // The raw kernel data is an array of implicit pairs, so the number of coefficients must
            // always be even.
            Some(coefficients) if coefficients.len() % 2 == 1 => {
                Err(Box::new(CoefficientsError::OddNumber {
                    key: key.to_string(),
                }))
            }
            // Split the implicit pairs into two index-matched vecs.
            Some(coefficients) => Ok(TokenizeableNutPrecCoefficients(unpair(coefficients))),
        }
    }
}

impl<'a> CoefficientKernel<'a> {
    fn get_polynomial_coefficients_or_default(
        &self,
        id: u32,
        pct: PolynomialCoefficientType,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<CoefficientsError>> {
        let (key, nut_prec_key) = pct.keys(id);
        let non_trig_coefficients = self.get_non_trig_coefficients_or_default(&key)?;
        let nut_prec_coefficients = self.get_nut_prec_coefficients_or_default(&nut_prec_key)?;

        Ok(TokenizeablePolynomialCoefficients(
            non_trig_coefficients[0],
            non_trig_coefficients[1],
            non_trig_coefficients[2],
            nut_prec_coefficients,
        ))
    }

    fn get_non_trig_coefficients_or_default(
        &self,
        key: &str,
    ) -> Result<[f64; 3], Box<CoefficientsError>> {
        match self.0.get_double_array(key) {
            None => Ok([0.0; 3]),
            Some(coefficients) if coefficients.len() < 2 => {
                Err(Box::new(CoefficientsError::TooFew {
                    key: key.to_string(),
                    min: 2,
                    actual: coefficients.len(),
                }))
            }
            Some(coefficients) if coefficients.len() > 3 => {
                Err(Box::new(CoefficientsError::TooMany {
                    key: key.to_string(),
                    max: 3,
                    actual: coefficients.len(),
                }))
            }
            Some(coefficients) => Ok([
                coefficients[0].to_radians(),
                coefficients[1].to_radians(),
                coefficients.get(2).copied().unwrap_or(0.0).to_radians(),
            ]),
        }
    }

    fn get_nut_prec_coefficients_or_default(
        &self,
        key: &str,
    ) -> Result<Vec<f64>, Box<CoefficientsError>> {
        match self.0.get_double_array(key) {
            None => Ok(vec![]),
            Some(coefficients) => Ok(coefficients.iter().map(|c| c.to_radians()).collect()),
        }
    }
}

fn unpair(vec: &Vec<f64>) -> (Vec<f64>, Vec<f64>) {
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
