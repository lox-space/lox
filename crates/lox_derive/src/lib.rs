/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Field};

fn generate_call_to_deserializer_for_kvn_type(
    type_name: &str,
    expected_kvn_name: &str,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    match type_name {
        "KvnDateTimeValue" | "KvnNumericValue" | "KvnStringValue" | "KvnIntegerValue" => {
            let parser = match type_name {
                "KvnDateTimeValue" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_datetime_line(
                        #expected_kvn_name,
                        next_line,
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "KvnStringValue" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_string_line(
                        #expected_kvn_name,
                        next_line,
                        true
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "KvnNumericValue" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_numeric_line(
                        #expected_kvn_name,
                        next_line,
                        true
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "KvnIntegerValue" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_integer_line(
                        #expected_kvn_name,
                        next_line,
                        true
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                // Assumes the match list here exhaustively matches the one from above
                _ => unreachable!(),
            };

            Ok(quote! {
                match lines.peek() {
                    None => Err(crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::UnexpectedEndOfInput {
                        keyword: #expected_kvn_name
                    }),
                    Some(next_line) => {
                        let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key(
                            #expected_kvn_name,
                            next_line,
                        );

                        let result = if line_matches {
                            let next_line = lines.next().unwrap();

                            Ok(#parser)
                        } else {
                            Err(crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::UnexpectedKeyword {
                                found: next_line,
                                expected: #expected_kvn_name,
                            })
                        };

                        result
                    }
                }
            })
        }

        type_value => {
            let type_ident = syn::Ident::new(&type_value, Span::call_site());

            Ok(quote! {
                {
                    let has_next_line = lines.peek().is_some();

                    let result = if has_next_line {
                        #type_ident::deserialize(lines)
                    } else {
                        Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedEndOfInput {
                            keyword: #expected_kvn_name
                        })
                    };

                    result
                }
            })
        }
    }
}

fn generate_call_to_deserializer_for_kvn_type_new(
    type_name: &str,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    match type_name {
        "KvnDateTimeValue" | "f64" | "String" | "i32" => {
            let parser = match type_name {
                "KvnDateTimeValue" => quote! { //@TODO
                    crate::ndm::kvn::parser::parse_kvn_datetime_line_new(
                        lines.next().unwrap(),
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "String" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_string_line_new(
                        lines.next().unwrap(),
                        true
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "f64" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_numeric_line_new(
                        lines.next().unwrap(),
                        true
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "i32" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_integer_line_new(
                        lines.next().unwrap(),
                        true
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                // Assumes the match list here exhaustively matches the one from above
                _ => unreachable!(),
            };

            Ok(parser)
        },
        type_value => {
            let type_ident = syn::Ident::new(&type_value, Span::call_site());

            Ok(quote! {
                {
                    let has_next_line = lines.peek().is_some();

                    let result = if has_next_line {
                        #type_ident::deserialize(lines)
                    } else {
                        Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedEndOfInput {
                            keyword: "Blala" //@TODO
                        })
                    };

                    result
                }
            })
        }
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
                )?;

                return Ok(quote! {
                    {
                        let result = #deserializer_for_kvn_type;

                        match result {
                            Ok(item) => Some(item),
                            Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedKeyword { .. }) |
                            Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedEndOfInput { .. }) => None,
                            Err(e) => Err(e)?,
                        }
                    }
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

fn generate_call_to_deserializer_for_vec_type(
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

                let expected_kvn_name = expected_kvn_name.trim_end_matches("_LIST");

                let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                    &type_name.as_ref(),
                    &expected_kvn_name,
                )?;

                let type_ident = syn::Ident::new(&type_name, Span::call_site());

                return Ok(quote! {
                    {
                        let mut items: Vec<#type_ident> = Vec::new();

                        loop {
                            let result = #deserializer_for_kvn_type;

                            match result {
                                Ok(item) => items.push(item),
                                Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedKeyword { .. }) |
                                Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedEndOfInput { .. }) => break,
                                Err(e) => Err(e)?,
                            }
                        }

                        items
                    }
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

fn is_value_unit_struct(item: &DeriveInput) -> bool {
    for attr in item.attrs.iter() {
        if attr.path().is_ident("kvn") {
            if attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("value_unit_struct") {
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                })
                .is_ok()
            {
                return true;
            }
        }
    }


    false
}

#[proc_macro_derive(KvnDeserialize, attributes(kvn))]
pub fn derive_kvn_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    let name = &item.ident;
    let is_value_unit_struct = is_value_unit_struct(&item);

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

    let deserializer = if is_value_unit_struct {
        let mut deserializer = None;

        for (index, field) in fields.iter().enumerate() {
            let field_name_ident = field.ident.as_ref().unwrap();

            // Unwrap is okay because we only support named structs
            let field_name = field_name_ident.span().source_text().unwrap();

            match index {
                0 => {
                    if field_name.as_str() != "base" {
                        return syn::Error::new_spanned(
                            &field,
                    "The first field in a value unit struct should be called \"base\"",
                        )
                        .into_compile_error()
                        .into()
                    }

                    // Unwrap is okay because we expect this span to come from the source code
                    let field_type = field.ty.span().source_text().unwrap();

                    
                    match field_type.as_str() {
                        "KvnDateTimeValue" | "String" | "f64" | "i32" => {
                            match generate_call_to_deserializer_for_kvn_type_new(&field_type) {
                                Ok(deserializer_for_kvn_type) => deserializer = Some(deserializer_for_kvn_type),
                                Err(e) => return e,
                            }
                        },
                        
                        _ => return syn::Error::new_spanned(
                            &field,
                    "Unsupported field type for deserializer",
                        )
                        .into_compile_error()
                        .into()
                    }
                },
                1 => if field_name.as_str() != "units" {
                    return syn::Error::new_spanned(
                        &field,
                "The second field in a value unit struct should be called \"units\"",
                    )
                    .into_compile_error()
                    .into()
                },
                _ => return syn::Error::new_spanned(
                        &field,
                "Only two fields \"base\" and \"units\" are called",
                    )
                    .into_compile_error()
                    .into()
            }

            
        }
        
        match deserializer { 
            None => return syn::Error::new_spanned(
                    &fields,
            "Unable to create deserializer for struct",
                )
                .into_compile_error()
                .into(),
            Some(deserializer) => quote! {
                let kvn_value = #deserializer;
                Ok(#name {
                    base: kvn_value.value,
                    units: kvn_value.unit,
                })
            }
        }
    } else {
        let field_initializers: Result<Vec<_>, _> = fields
            .iter()
            .enumerate()
            .map(|(_, field)| {
                let field_name = field.ident.as_ref().unwrap();

                // Unwrap is okay because we only support named structs
                let expected_kvn_name = field_name.span().source_text().unwrap().to_uppercase();

                // Unwrap is okay because we expect this span to come from the source code
                let field_type = field.ty.span().source_text().unwrap();

                let type_ident = syn::Ident::new(&field_type, Span::call_site());

                let parser = match field_type.as_str() {
                    "KvnDateTimeValue" | "KvnStringValue" | "KvnNumericValue" | "KvnIntegerValue" => {
                        let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                            &field_type,
                            &expected_kvn_name,
                        )?;

                        quote! {
                            #deserializer_for_kvn_type?
                        }
                    }
                    "Option" => {
                        generate_call_to_deserializer_for_option_type(&expected_kvn_name, &field)?
                    }
                    "Vec" => generate_call_to_deserializer_for_vec_type(&expected_kvn_name, &field)?,
                    _ => {
                        quote! {
                            {
                                match lines.peek() {
                                    None => Err(
                                        crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::UnexpectedEndOfInput {
                                            keyword: #expected_kvn_name,
                                        },
                                    )?,
                                    Some(next_line) => {
                                        let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key_new(
                                            #expected_kvn_name,
                                            next_line,
                                        );
                
                                        match line_matches {
                                            Ok(true) => #type_ident::deserialize(lines),
                                            Ok(false) => Err(
                                                crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::UnexpectedKeyword {
                                                    found: next_line,
                                                    expected: #expected_kvn_name,
                                                },
                                            ),
                                            Err(crate::ndm::kvn::parser::KvnKeyMatchErr::KeywordNotFound { expected }) => Err(
                                                crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::KeywordNotFound {
                                                    expected,
                                                },
                                            ),
                                        }?
                                    }
                                }
                            }
                        }
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
            
        quote! {
            Ok(#name {
                #(#field_initializers)*
            })
        }
    };



    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    let deserializer = quote! {
        impl #impl_generics crate::ndm::kvn::parser::KvnDeserializer for #name #type_generics
        #where_clause
        {
            fn deserialize<'a>(lines: &mut ::std::iter::Peekable<impl Iterator<Item = &'a str>>)
            -> Result<#name, crate::ndm::kvn::parser::KvnDeserializerErr<&'a str>> {

                #deserializer
            }
        }
    };

    deserializer.into()
}
