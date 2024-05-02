/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use quote::quote;
use syn::{spanned::Spanned, Field};

fn generate_call_to_deserializer_for_kvn_type(
    type_name: &str,
    expected_kvn_name: &str,
    field: &Field,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    match type_name {
        "KvnDateTimeValue" => Ok(quote! {
            parse_kvn_datetime_line(
                #expected_kvn_name,
                line,
            ).map_err(|x| KvnDeserializerErr::from(x))
            .map(|x| x.1)
        }),
        "KvnStringValue" => Ok(quote! {
            parse_kvn_string_line(
                #expected_kvn_name,
                line,
                true
            ).map_err(|x| KvnDeserializerErr::from(x))
            .map(|x| x.1)
        }),
        "KvnNumericValue" => Ok(quote! {
            parse_kvn_numeric_line(
                #expected_kvn_name,
                line,
                true
            ).map_err(|x| KvnDeserializerErr::from(x))
            .map(|x| x.1)
        }),
        "KvnIntegerValue" => Ok(quote! {
            parse_kvn_integer_line(
                #expected_kvn_name,
                line,
                true
            ).map_err(|x| KvnDeserializerErr::from(x))
            .map(|x| x.1)
        }),
        _ => Err(syn::Error::new_spanned(
            &field,
            "unsupported field type for `#[derive(KvnDeserialize)]`",
        )
        .into_compile_error()
        .into()),
    }
}

fn generate_call_to_deserializer_for_option_type(
    expected_kvn_name: &str,
    field: &Field,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    if let syn::Type::Path(type_path) = &field.ty {
        let path_part = type_path.path.segments.first();
        if let Some(path_part) = path_part {
            if let syn::PathArguments::AngleBracketed(type_argument) = &path_part.arguments {
                let type_name = type_argument
                    .args
                    .first()
                    .unwrap()
                    .span()
                    .source_text()
                    .unwrap();

                let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                    &type_name.as_ref(),
                    &expected_kvn_name,
                    &field,
                )?;

                return Ok(quote! {
                    lines.next_if(|line|
                        kvn_line_matches_key(
                            #expected_kvn_name,
                            line
                        )
                    ).map(|line| {
                        #deserializer_for_kvn_type
                    }).transpose()?
                });
            }
        }
    }

    return Err(
        syn::Error::new_spanned(&field, "Malformed type for `#[derive(KvnDeserialize)]`")
            .into_compile_error()
            .into(),
    );
}

#[proc_macro_derive(KvnDeserialize)]
pub fn derive_kvn_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    let name = &item.ident;

    let syn::Data::Struct(strukt) = item.data else {
        return syn::Error::new_spanned(
            &item,
            "only structs are supported for `#[derive(KvnDeserialize)]`",
        )
        .into_compile_error()
        .into();
    };

    let fields = match strukt.fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => named,
        _ => {
            return syn::Error::new_spanned(
                &strukt.fields,
                "only named fields are supported for `#[derive(KvnDeserialize)]`",
            )
            .into_compile_error()
            .into()
        }
    };

    let field_initializers: Result<Vec<_>, _> = fields
        .iter()
        .enumerate()
        .map(|(_, field)| {
            let field_name = field.ident.as_ref().unwrap();

            // Unwrap is okay because we only support named structs
            let expected_kvn_name = field_name.span().source_text().unwrap().to_uppercase();

            // Unwrap is okay because we expect this span to come from the source code
            let field_type = field.ty.span().source_text().unwrap();

            let parser = match field_type.as_str() {
                "KvnDateTimeValue" | "KvnStringValue" | "KvnNumericValue" | "KvnIntegerValue" => {
                    let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                        &field_type,
                        &expected_kvn_name,
                        &field,
                    )?;

                    quote! {
                        lines.next().map_or_else(
                            || Err(KvnDeserializerErr::<&str>::UnexpectedEndOfInput),
                            |line| {
                                #deserializer_for_kvn_type
                        })?
                    }
                }
                "Option" => {
                    generate_call_to_deserializer_for_option_type(&expected_kvn_name, &field)?
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &field,
                        "unsupported field type for `#[derive(KvnDeserialize)]`",
                    )
                    .into_compile_error()
                    .into());
                }
            };

            Ok(quote! {
                #field_name: #parser,
            })
        })
        .collect();

    if let Err(e) = field_initializers {
        return e;
    }

    let field_initializers = field_initializers.unwrap();

    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    let deserializer = quote! {
        use std::iter::Peekable;

        use crate::ndm::kvn::parser::{KvnDeserializer, KvnDeserializerErr};
        use crate::ndm::kvn::parser::{
            kvn_line_matches_key,
            parse_kvn_string_line,
            parse_kvn_datetime_line,
            parse_kvn_numeric_line,
            parse_kvn_integer_line
        };

        impl #impl_generics KvnDeserializer<#name> for #name #type_generics
        #where_clause
        {
            fn deserialize<'a>(lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>)
            -> Result<#name, KvnDeserializerErr<&'a str>> {

                Ok(#name {
                    #(#field_initializers)*
                })
            }
        }
    };

    deserializer.into()
}
