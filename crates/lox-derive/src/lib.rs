// SPDX-FileCopyrightText: 2024 Andrei Zisu <matzipan@gmail.com>
// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Procedural derive macros for the Lox ecosystem.
//!
//! Provides `#[derive(ApproxEq)]` for automatic approximate-equality
//! implementations against `lox_approx::ApproxEq`.

#![warn(missing_docs)]

use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, GenericParam, Generics, Member, parse_macro_input,
    parse_quote,
};

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            // Add a trait bound to each type parameter
            type_param.bounds.push(parse_quote!(::core::fmt::Debug));
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

    let lox_approx = match crate_name("lox-approx") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::lox_approx),
    };

    let fields: Vec<Member> = match &data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) | Fields::Unnamed(_) => data.fields.members().collect(),
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
    };

    let results = quote! {#lox_approx::ApproxEqResults};

    let comparisons = fields.iter().map(|f| {
        let name = match f {
            Member::Named(ident) => ident.to_string(),
            Member::Unnamed(index) => index.index.to_string(),
        };
        quote! {
            results.merge(#name, self.#f.approx_eq(&rhs.#f, atol, rtol));
        }
    });

    let generics = add_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl #impl_generics #lox_approx::ApproxEq for #ident #ty_generics #where_clause {
            fn approx_eq(&self, rhs: &Self, atol: f64, rtol: f64) -> #results {
                let mut results = #results::new();
                #(#comparisons)*
                results
            }
        }
    };
    output.into()
}
