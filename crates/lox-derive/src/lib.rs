// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Procedural derive macros for the Lox ecosystem.
//!
//! Provides `#[derive(ApproxEq)]` for automatic approximate-equality
//! implementations against [`lox_test_utils::approx_eq::ApproxEq`].

#![warn(missing_docs)]

use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, GenericParam, Generics, Index, parse_macro_input, parse_quote,
};

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            // Add a trait bound to each type parameter
            type_param.bounds.push(parse_quote!(::std::fmt::Debug));
        }
    }
    generics
}

/// Derives the `ApproxEq` trait for structs with named or unnamed fields.
///
/// All fields must implement `ApproxEq` and `Debug`. The generated implementation
/// compares each field individually, collecting per-field results.
#[proc_macro_derive(ApproxEq)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);

    let lox_test_utils = match crate_name("lox-test-utils") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::lox_test_utils),
    };

    let fields: Vec<proc_macro2::TokenStream> = match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|f| {
                    let f = f.ident.expect("named field must have an identifier");
                    quote! {#f}
                })
                .collect::<Vec<proc_macro2::TokenStream>>(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(idx, _)| {
                    let idx = Index::from(idx);
                    quote! {#idx}
                })
                .collect(),
            Fields::Unit => {
                return Error::new(ident.span(), "unit structs are not supported")
                    .into_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new(ident.span(), format!("{} is not a struct", ident))
                .into_compile_error()
                .into();
        }
    }
    .iter()
    .map(|f| {
        quote! {
            results.merge(stringify!(#f).to_string(), self.#f.approx_eq(&rhs.#f, atol, rtol));
        }
    })
    .collect();

    let generics = add_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let results = quote! {#lox_test_utils::approx_eq::results::ApproxEqResults};

    let output = quote! {
        impl #impl_generics #lox_test_utils::approx_eq::ApproxEq for #ident #ty_generics #where_clause {
            fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> #results {
                let mut results = #results::new();
                #(#fields)*
                results
            }
        }
    };
    output.into()
}
