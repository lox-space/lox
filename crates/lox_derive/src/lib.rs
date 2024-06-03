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
    type_name_new: &syn::Path,
    expected_kvn_name: &str,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    match type_name {
        "f64" | "String" | "i32" | "u64" => {
            let parser = match type_name {
                "String" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_string_line_new(
                        next_line
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "f64" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_numeric_line_new(
                        next_line,
                        true, //@TODO
                    ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                    .map(|x| x.1)?
                },
                "i32" | "u64" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_integer_line_new(
                        next_line,
                        true, //@TODO
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
                        let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key_new(
                            #expected_kvn_name,
                            next_line,
                        )?;

                        let result = if line_matches {
                            let next_line = lines.next().unwrap();

                            Ok(#parser.value)
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

        "KvnNumericValue" | "KvnStringValue" | "KvnIntegerValue" => {
            let parser = match type_name {
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
                        let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key_new(
                            #expected_kvn_name,
                            next_line,
                        )?;

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

        type_value => Ok(quote! {
            {
                let has_next_line = lines.peek().is_some();

                let result = if has_next_line {
                    #type_name_new::deserialize(lines)
                } else {
                    Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedEndOfInput {
                        keyword: #expected_kvn_name
                    })
                };

                result
            }
        }),
    }
}

//@TODO unify with above
fn generate_call_to_deserializer_for_kvn_type_new(
    type_name: &str,
    type_name_new: &syn::Path,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    match type_name {
        "f64" | "String" | "i32" | "NonNegativeDouble" | "NegativeDouble" | "PositiveDouble" => {
            let parser = match type_name {
                "String" => {
                    quote! {
                        crate::ndm::kvn::parser::parse_kvn_string_line_new(
                            lines.next().unwrap()
                        ).map_err(|x| crate::ndm::kvn::parser::KvnDeserializerErr::from(x))
                        .map(|x| x.1)?
                    }
                }
                "f64" | "NonNegativeDouble" | "NegativeDouble" | "PositiveDouble" => quote! {
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
        }
        type_value => {
            Ok(quote! {
                {
                    let has_next_line = lines.peek().is_some();

                    let result = if has_next_line {
                        #type_name_new::deserialize(lines)
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

fn get_generic_type_argument(field: &Field) -> Option<String> {
    if let syn::Type::Path(type_path) = &field.ty {
        let path_part = type_path.path.segments.first();
        if let Some(path_part) = path_part {
            if let syn::PathArguments::AngleBracketed(type_argument) = &path_part.arguments {
                return Some(
                    type_argument
                        .args
                        .first()
                        .unwrap()
                        .span()
                        .source_text()
                        .unwrap(),
                );
            }
        }
    }

    None
}

fn get_generic_type_argument_new(field: &Field) -> Option<&syn::Path> {
    if let syn::Type::Path(type_path) = &field.ty {
        let path_part = type_path.path.segments.first();
        if let Some(path_part) = path_part {
            if let syn::PathArguments::AngleBracketed(type_argument) = &path_part.arguments {
                if let Some(syn::GenericArgument::Type(r#type)) = &type_argument.args.first() {
                    return extract_type_path(r#type);
                }
            }
        }
    }

    None
}

fn generate_call_to_deserializer_for_option_type(
    expected_kvn_name: &str,
    field: &Field,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    let type_name = get_generic_type_argument(field);

    // @TODO
    let type_name_new = get_generic_type_argument_new(field).unwrap();

    match type_name {
        None => {
            return Err(syn::Error::new_spanned(
                &field,
                "Malformed type for `#[derive(KvnDeserialize)]`",
            )
            .into_compile_error()
            .into())
        }

        Some(type_name) => {
            let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                &type_name.as_ref(),
                &type_name_new,
                &expected_kvn_name,
            )?;

            let condition_shortcut = match type_name.as_str() {
                "String" | "f64" | "i32" | "u64" => quote! {},
                _ => quote! { ! #type_name_new::should_check_key_match() || },
            };

            return Ok(quote! {
                match lines.peek() {
                    None => None,
                    Some(next_line) => {
                        let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key_new(
                            #expected_kvn_name,
                            next_line,
                        )?;

                        if #condition_shortcut line_matches {
                            let result = #deserializer_for_kvn_type;

                            match result {
                                Ok(item) => Some(item),
                                Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedKeyword { .. }) |
                                Err(crate::ndm::kvn::parser::KvnDeserializerErr::UnexpectedEndOfInput { .. }) => None,
                                Err(e) => Err(e)?,
                            }
                        } else {
                            None
                        }
                    }
                }
            });
        }
    }
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

                //@TODO
                let bla = get_generic_type_argument_new(field).unwrap();

                let expected_kvn_name = expected_kvn_name.trim_end_matches("_LIST");

                let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                    &type_name.as_ref(),
                    &bla,
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

fn extract_type_path(ty: &syn::Type) -> Option<&syn::Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

fn deserializer_for_struct_with_named_fields(
    type_name: &proc_macro2::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    is_value_unit_struct: bool,
) -> proc_macro2::TokenStream {
    if type_name.to_string() == "UserDefinedType" {
        //@TODO
        return quote! {
            Ok(Default::default())
        };
    }

    if is_value_unit_struct {
        let mut deserializer = None;
        let mut unit_type: Option<String> = None;
        let mut unit_field_name_ident: Option<&proc_macro2::Ident> = None;
        let mut field_type: Option<String> = None;
        let mut field_type_new: Option<&syn::Path> = None;

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
                        .into();
                    }

                    // Unwrap is okay because we expect this span to come from the source code
                    let local_field_type = extract_type_path(&field.ty)
                        .unwrap()
                        .span()
                        .source_text()
                        .unwrap();
                    let local_field_type_new = extract_type_path(&field.ty).unwrap();

                    let deserializer = match local_field_type.as_str() {
                        "KvnDateTimeValue" | "String" | "f64" | "i32" | "NonNegativeDouble"
                        | "NegativeDouble" | "PositiveDouble" => {
                            match generate_call_to_deserializer_for_kvn_type_new(
                                &local_field_type,
                                &local_field_type_new,
                            ) {
                                Ok(deserializer_for_kvn_type) => {
                                    deserializer = Some(deserializer_for_kvn_type)
                                }
                                Err(e) => return e.into(),
                            }
                        }

                        _ => {
                            return syn::Error::new_spanned(
                                &field,
                                "Unsupported field type for deserializer",
                            )
                            .into_compile_error()
                            .into()
                        }
                    };

                    field_type = Some(local_field_type);
                    field_type_new = Some(local_field_type_new);

                    deserializer
                }
                1 => {
                    if field_name.as_str() != "units" && field_name.as_str() != "parameter" {
                        return syn::Error::new_spanned(
                             &field,
                             "The second field in a value unit struct should be called \"units\" or \"parameter\"",
                         )
                         .into_compile_error()
                         .into();
                    }

                    unit_type = get_generic_type_argument(field);
                    unit_field_name_ident = Some(field_name_ident);
                }
                _ => {
                    return syn::Error::new_spanned(
                        &field,
                        "Only two fields are allowed: \"base\" and (\"units\" or \"parameters\"",
                    )
                    .into_compile_error()
                    .into()
                }
            }
        }

        // This unwrap is okay because we know the field exists. If it didn't exist we would've thrown an error.
        let unit_type = unit_type.unwrap();
        let unit_field_name_ident = unit_field_name_ident.unwrap();
        let field_type = field_type.unwrap();
        let field_type_new = field_type_new.unwrap();

        let unit_type_ident = syn::Ident::new(&unit_type, Span::call_site());

        let base = match field_type.as_str() {
            "NonNegativeDouble" | "NegativeDouble" | "PositiveDouble" => {
                quote! { #field_type_new (kvn_value.value) }
            }
            _ => quote! { kvn_value.value },
        };

        match deserializer {
            None => syn::Error::new_spanned(&fields, "Unable to create deserializer for struct")
                .into_compile_error()
                .into(),
            Some(deserializer) => quote! {
                let kvn_value = #deserializer;
                Ok(#type_name {
                    base: #base,
                    #unit_field_name_ident: kvn_value.unit.map(|unit| #unit_type_ident (unit)),
                })
            },
        }
    } else {
        let field_deserializers: Result<Vec<_>, _> = fields
             .iter()
             .enumerate()
             .map(|(_, field)| {
                let field_name = field.ident.as_ref().unwrap();

                // Unwrap is okay because we only support named structs
                // 7.4.4 Keywords must be uppercase and must not contain blanks
                let expected_kvn_name = field_name.span().source_text().unwrap().to_uppercase();

                // Unwrap is okay because we expect this span to come from the source code
                let field_type = extract_type_path(&field.ty).unwrap().span().source_text().unwrap();
                let field_type_new = extract_type_path(&field.ty).unwrap();

                let parser = match field_type.as_str() {
                    "EpochType" | "KvnStringValue" | "KvnNumericValue" | "KvnIntegerValue" | "String" | "f64" | "i32" => {
                        let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                            &field_type,
                            field_type_new,
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

                        let condition_shortcut = match field_type.as_str() {
                            "String" => quote! {},
                            _ => quote! { ! #field_type_new::should_check_key_match() || },
                        };
            
                        quote! {
                            match lines.peek() {
                                None => Err(crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::UnexpectedEndOfInput {
                                    keyword: #expected_kvn_name
                                })?,
                                Some(next_line) => {
                                    let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key_new(
                                        #expected_kvn_name,
                                        next_line,
                                    )?;
            
                                    if #condition_shortcut line_matches {
                                        #field_type_new::deserialize(lines)?
                                    } else {
                                        Err(crate::ndm::kvn::parser::KvnDeserializerErr::<&str>::UnexpectedKeyword {
                                            found: next_line,
                                            expected: #expected_kvn_name,
                                        })?
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

        if let Err(e) = field_deserializers {
            return e;
        }

        let field_deserializers = field_deserializers.unwrap();

        quote! {
            Ok(#type_name {
                #(#field_deserializers)*
            })
        }
    }
}

fn deserializers_for_struct_with_unnamed_fields(
    type_name: &proc_macro2::Ident,
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let field_deserializers: Result<Vec<_>, _> = fields
        .iter()
        .enumerate()
        .map(|(_, field)| {
            // Unwrap is okay because we expect this span to come from the source code
            let field_type = extract_type_path(&field.ty)
                .unwrap()
                .span()
                .source_text()
                .unwrap();
            let field_type_new = extract_type_path(&field.ty).unwrap();

            let deserializer_for_kvn_type =
                generate_call_to_deserializer_for_kvn_type_new(&field_type, field_type_new)?;

            Ok(quote! {
                #deserializer_for_kvn_type.value,
            })
        })
        .collect();

    if let Err(e) = field_deserializers {
        return e;
    }

    let field_deserializers = field_deserializers.unwrap();

    quote! {
        Ok(#type_name (
            #(#field_deserializers)*
        ))
    }
}

#[proc_macro_derive(KvnDeserialize, attributes(kvn))]
pub fn derive_kvn_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);
    let type_name = &item.ident;
    let is_value_unit_struct = is_value_unit_struct(&item);

    let syn::Data::Struct(strukt) = item.data else {
        return syn::Error::new_spanned(
            &item,
            "only structs are supported for `#[derive(KvnDeserialize)]`",
        )
        .into_compile_error()
        .into();
    };

    let (struct_deserializer, should_check_key_match) = match strukt.fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => (
            deserializer_for_struct_with_named_fields(type_name, &named, is_value_unit_struct),
            is_value_unit_struct,
        ),
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => (
            deserializers_for_struct_with_unnamed_fields(type_name, &unnamed),
            true,
        ),
        _ => {
            return syn::Error::new_spanned(
                &strukt.fields,
                "only named fields are supported for `#[derive(KvnDeserialize)]`",
            )
            .into_compile_error()
            .into()
        }
    };

    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    let struct_deserializer = quote! {
        impl #impl_generics crate::ndm::kvn::parser::KvnDeserializer for #type_name #type_generics
        #where_clause
        {
            fn deserialize<'a>(lines: &mut ::std::iter::Peekable<impl Iterator<Item = &'a str>>)
            -> Result<#type_name, crate::ndm::kvn::parser::KvnDeserializerErr<&'a str>> {

                #struct_deserializer
            }

            fn should_check_key_match () -> bool {
                #should_check_key_match
            }
        }
    };

    struct_deserializer.into()
}
