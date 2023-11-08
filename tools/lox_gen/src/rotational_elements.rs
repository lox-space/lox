/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use lox_core::bodies::{NutationPrecessionCoefficients, PolynomialCoefficients, N_COEFFICIENTS};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use thiserror::Error;

use lox_io::spice::Kernel;

/// Converts a set of PolynomialCoefficients into a TokenStream.
struct TokenizeablePolynomialCoefficients(PolynomialCoefficients);

impl ToTokens for TokenizeablePolynomialCoefficients {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let PolynomialCoefficients(a, b, c, d) = self.0;
        tokens.extend(quote! { PolynomialCoefficients(#a, #b, #c, #d) });
    }
}

/// Converts a set of NutationPrecessionCoefficients into a TokenStream.
struct TokenizeableNutPrecCoefficients(NutationPrecessionCoefficients);

impl ToTokens for TokenizeableNutPrecCoefficients {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NutationPrecessionCoefficients(a, b) = self.0;
        tokens.extend(quote! { NutationPrecessionCoefficients([#(#a),*], [#(#b),*]) });
    }
}

/// Holds the rotational elements for a given body.
///
/// May be parsed directly from PCK data and knows how to represent its code and test components as
/// streams of tokens.
pub(crate) struct RotationalElements<'a, 'b> {
    id: u32,
    ident: &'a Ident,
    right_ascension: [f64; 3],
    declination: [f64; 3],
    prime_meridian: [f64; 3],
    // Not all bodies have trigonometric rotational elements, so we encapsulate these fields in
    // a separate, optional struct.
    trig_elements: Option<TrigonometricElements<'b>>,
}

impl<'a, 'b> RotationalElements<'a, 'b> {
    pub(crate) fn parse(id: u32, ident: &'a Ident, kernel: &'b Kernel) -> Option<Self> {
        let right_ascension_key = format!("BODY{}_POLE_RA", id);
        let declination_key = format!("BODY{}_POLE_DEC", id);
        let prime_meridian_key = format!("BODY{}_PM", id);

        Some(Self {
            id,
            ident,
            right_ascension: get_polynomial_coefficients_or_default(&right_ascension_key, kernel)?,
            declination: get_polynomial_coefficients_or_default(&declination_key, kernel)?,
            prime_meridian: get_polynomial_coefficients_or_default(&prime_meridian_key, kernel)?,
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

    fn barycenter_id(&self) -> u32 {
        self.id / 100
    }
}

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

/// Indicates that a value retrieved from the kernel did not conform to the domain model.
#[derive(Debug, Error)]
pub(crate) enum ParsingError {
    #[error("Kernel coefficients with key {key} had size {actual}, expected at least {actual}")]
    TooFewTerms {
        key: String,
        min: usize,
        actual: usize,
    },
    #[error("Kernel coefficients with key {key} had size {actual}, expected at most {actual}")]
    TooManyTerms {
        key: String,
        max: usize,
        actual: usize,
    },
    #[error("Barycenter nutation precession coefficients with key {key} had an odd number of terms, but an even number is required")]
    OddTerms { key: String },
}

/// Translation layer between the Kernel and RotationalElements.
struct CoefficientKernel<'a>(&'a Kernel);

impl<'a> CoefficientKernel<'a> {
    fn get_polynomial_coefficients_or_default(
        &self,
        id: u32,
        pct: PolynomialCoefficientType,
    ) -> Result<TokenizeablePolynomialCoefficients, Box<ParsingError>> {
        let (key, nut_prec_key) = pct.keys(id);
        let non_trig_coefficients = self.get_non_trig_coefficients_or_default(&key)?;
        let nut_prec_coefficients = self.get_nut_prec_coefficients_or_default(&nut_prec_key)?;
        Ok(TokenizeablePolynomialCoefficients(PolynomialCoefficients(
            non_trig_coefficients[0],
            non_trig_coefficients[1],
            non_trig_coefficients[2],
            nut_prec_coefficients,
        )))
    }

    fn get_non_trig_coefficients_or_default(
        &self,
        key: &str,
    ) -> Result<[f64; 3], Box<ParsingError>> {
        match self.0.get_double_array(key) {
            None => Ok([0.0; 3]),
            Some(polynomials) if polynomials.len() < 2 => {
                Err(Box::new(ParsingError::TooFewTerms {
                    key: key.to_string(),
                    min: 2,
                    actual: polynomials.len(),
                }))
            }
            Some(polynomials) if polynomials.len() > 3 => {
                Err(Box::new(ParsingError::TooManyTerms {
                    key: key.to_string(),
                    max: 3,
                    actual: polynomials.len(),
                }))
            }
            Some(polynomials) => {
                let mut polynomial_coefficients = [0.0; 3];
                polynomial_coefficients.copy_from_slice(&polynomials);
                Ok(polynomial_coefficients)
            }
        }
    }

    fn get_nut_prec_coefficients_or_default(
        &self,
        key: &str,
    ) -> Result<[f64; N_COEFFICIENTS], Box<ParsingError>> {
        match self.0.get_double_array(key) {
            None => Ok([0.0; N_COEFFICIENTS]),
            Some(coefficients) if coefficients.len() <= N_COEFFICIENTS => {
                let mut nut_prec_coefficients = [0.0; N_COEFFICIENTS];
                nut_prec_coefficients.copy_from_slice(&coefficients);
                Some(nut_prec_coefficients)
            }
            Some(coefficients) => Err(Box::new(ParsingError::TooManyTerms {
                key: key.to_string(),
                max: N_COEFFICIENTS,
                actual: coefficients.len(),
            })),
        }
    }

    fn get_barycenter_nut_prec_coefficients(
        &self,
        barycenter_id: u32,
    ) -> Result<TokenizeableNutPrecCoefficients, Box<ParsingError>> {
        let key = format!("BODY{}_NUT_PREC_ANGLES", barycenter_id);
        match self.0.get_double_array(&key) {
            None => Ok(TokenizeableNutPrecCoefficients(
                NutationPrecessionCoefficients::default(),
            )),
            Some(coefficients) if coefficients.len() > N_COEFFICIENTS * 2 => {
                Err(Box::new(ParsingError::TooManyTerms {
                    key: key.to_string(),
                    max: N_COEFFICIENTS * 2,
                    actual: coefficients.len(),
                }))
            }
            Some(coefficients) if coefficients.len() % 2 == 1 => {
                Err(Box::new(ParsingError::OddTerms {
                    key: key.to_string(),
                }))
            }
            Some(coefficients) => {
                let mut a = [0.0; N_COEFFICIENTS];
                let mut b = [0.0; N_COEFFICIENTS];
                coefficients.iter().enumerate().for_each(|(i, c)| {
                    if i % 2 == 0 {
                        a[i / 2] = *c;
                    } else {
                        b[i / 2] = *c;
                    }
                });
                Ok(TokenizeableNutPrecCoefficients(
                    NutationPrecessionCoefficients(a, b),
                ))
            }
        }
    }
}
