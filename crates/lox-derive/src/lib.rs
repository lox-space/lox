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
    with_keyword_matching: bool,
    unpack_value: bool,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    let unpack_insert = if unpack_value {
        quote! { .value }
    } else {
        quote! {}
    };

    match type_name {
        "String" | "f64" | "i32" | "u64" | "NonNegativeDouble" | "NegativeDouble"
        | "PositiveDouble" => {
            let parser = match type_name {
                "String" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_string_line(
                        next_line
                    ).map_err(|x| crate::ndm::kvn::KvnDeserializerErr::from(x))?
                },
                "f64" | "NonNegativeDouble" | "NegativeDouble" | "PositiveDouble" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_numeric_line(
                        next_line,
                        true, //@TODO
                    ).map_err(|x| crate::ndm::kvn::KvnDeserializerErr::from(x))?
                },
                "i32" | "u64" => quote! {
                    crate::ndm::kvn::parser::parse_kvn_integer_line(
                        next_line,
                        true, //@TODO
                    ).map_err(|x| crate::ndm::kvn::KvnDeserializerErr::from(x))?
                },
                // Assumes the match list here exhaustively matches the one from above
                _ => unreachable!(),
            };

            if with_keyword_matching {
                Ok(quote! {
                    match lines.peek() {
                        None => Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                            keyword: #expected_kvn_name.to_string()
                        }),
                        Some(next_line) => {
                            let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key(
                                #expected_kvn_name,
                                next_line,
                            )?;

                            let result = if line_matches {
                                let next_line = lines.next().unwrap();

                                Ok(#parser #unpack_insert)
                            } else {
                                Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedKeyword {
                                    found: next_line.to_string(),
                                    expected: #expected_kvn_name.to_string(),
                                })
                            };

                            result
                        }
                    }
                })
            } else {
                Ok(quote! {
                   {
                      let next_line = lines.next().unwrap();
                      #parser #unpack_insert
                   }
                })
            }
        }
        _ => Ok(quote! {
           {
                let has_next_line = lines.peek().is_some();

                let result = if has_next_line {
                    #type_name_new::deserialize(lines)
                } else {
                    Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                          keyword: #expected_kvn_name.to_string()
                    })
                };

                result
            }
        }),
    }
}

fn get_generic_type_argument(field: &Field) -> Option<(String, &syn::Path)> {
    if let syn::Type::Path(type_path) = &field.ty {
        let path_part = type_path.path.segments.first();
        if let Some(path_part) = path_part {
            if let syn::PathArguments::AngleBracketed(type_argument) = &path_part.arguments {
                if let Some(syn::GenericArgument::Type(r#type)) = &type_argument.args.first() {
                    if let syn::Type::Path(r#type) = r#type {
                        return Some((
                            r#type
                                .path
                                .segments
                                .clone()
                                .into_iter()
                                .map(|ident| ident.span().source_text().unwrap())
                                .reduce(|a, b| a + "::" + &b)
                                .unwrap(),
                            &r#type.path,
                        ));
                    }
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
    // @TODO
    let (type_name, type_name_new) = get_generic_type_argument(field).ok_or(
        syn::Error::new_spanned(field, "Malformed type for `#[derive(KvnDeserialize)]`")
            .into_compile_error(),
    )?;

    let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
        type_name.as_ref(),
        type_name_new,
        expected_kvn_name,
        true,
        true,
    )?;

    let condition_shortcut = match type_name.as_str() {
        "String" | "f64" | "i32" | "u64" => quote! {},
        _ => quote! { ! #type_name_new::should_check_key_match() || },
    };

    let value = match type_name.as_ref() {
        "NonNegativeDouble" | "NegativeDouble" | "PositiveDouble" => {
            quote! { #type_name_new (item) }
        }
        _ => quote! { item },
    };

    Ok(quote! {
        match lines.peek() {
            None => None,
            Some(next_line) => {
                let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key(
                    #expected_kvn_name,
                    next_line,
                )?;

                if #condition_shortcut line_matches {
                    let result = #deserializer_for_kvn_type;

                    match result {
                        Ok(item) => Some(#value),
                        Err(crate::ndm::kvn::KvnDeserializerErr::UnexpectedKeyword { .. }) |
                        Err(crate::ndm::kvn::KvnDeserializerErr::UnexpectedEndOfInput { .. }) => None,
                        Err(e) => Err(e)?,
                    }
                } else {
                    None
                }
            }
        }
    })
}

fn generate_call_to_deserializer_for_vec_type(
    expected_kvn_name: &str,
    field: &Field,
) -> Result<proc_macro2::TokenStream, proc_macro::TokenStream> {
    let (type_name, type_ident) = get_generic_type_argument(field).ok_or(
        syn::Error::new_spanned(field, "Malformed type for `#[derive(KvnDeserialize)]`")
            .into_compile_error(),
    )?;

    let expected_kvn_name = expected_kvn_name.trim_end_matches("_LIST");

    let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
        type_name.as_ref(),
        type_ident,
        expected_kvn_name,
        true,
        true,
    )?;

    return Ok(quote! {
        {
            let mut items: Vec<#type_ident> = Vec::new();

            loop {
                let result = #deserializer_for_kvn_type;

                match result {
                    Ok(item) => items.push(item),
                    Err(crate::ndm::kvn::KvnDeserializerErr::UnexpectedKeyword { .. }) |
                    Err(crate::ndm::kvn::KvnDeserializerErr::UnexpectedEndOfInput { .. }) => break,
                    Err(e) => Err(e)?,
                }
            }

            items
        }
    });
}

fn is_value_unit_struct(item: &DeriveInput) -> bool {
    item.attrs.iter().any(|attr| {
        attr.path().is_ident("kvn")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("value_unit_struct") {
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                })
                .is_ok()
    })
}

fn extract_type_path(ty: &syn::Type) -> Option<&syn::Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

fn handle_version_field(
    type_name: &proc_macro2::Ident,
    field: &syn::Field,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let string_type_name = type_name.to_string();

    if !string_type_name.ends_with("Type") {
        return Err(syn::Error::new_spanned(
            type_name,
            "Types with \"version\" field should be of the form SomethingType",
        )
        .into_compile_error());
    }

    let message_type_name = string_type_name
        .trim_end_matches("Type")
        .to_string()
        .to_uppercase();

    let field_name = field.ident.as_ref().unwrap();

    // 7.4.4 Keywords must be uppercase and must not contain blanks
    let expected_kvn_name = format!("CCSDS_{}_VERS", message_type_name);

    // Unwrap is okay because we expect this span to come from the source code
    let field_type = extract_type_path(&field.ty)
        .unwrap()
        .span()
        .source_text()
        .unwrap();
    let field_type_new = extract_type_path(&field.ty).unwrap();

    let parser = generate_call_to_deserializer_for_kvn_type(
        &field_type,
        field_type_new,
        &expected_kvn_name,
        true,
        true,
    )?;

    Ok(quote! {
        #field_name: #parser?,
    })
}

fn deserializer_for_struct_with_named_fields(
    type_name: &proc_macro2::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    is_value_unit_struct: bool,
) -> proc_macro2::TokenStream {
    if &type_name.to_string() == "UserDefinedType" {
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
                            field,
                            "The first field in a value unit struct should be called \"base\"",
                        )
                        .into_compile_error();
                    }

                    // Unwrap is okay because we expect this span to come from the source code
                    let local_field_type = extract_type_path(&field.ty)
                        .unwrap()
                        .span()
                        .source_text()
                        .unwrap();
                    let local_field_type_new = extract_type_path(&field.ty).unwrap();

                    match local_field_type.as_str() {
                        "KvnDateTimeValue" | "String" | "f64" | "i32" | "NonNegativeDouble"
                        | "NegativeDouble" | "PositiveDouble" => {
                            match generate_call_to_deserializer_for_kvn_type(
                                &local_field_type,
                                local_field_type_new,
                                "Blala",
                                false,
                                false,
                            ) {
                                Ok(deserializer_for_kvn_type) => {
                                    deserializer = Some(deserializer_for_kvn_type)
                                }
                                Err(e) => return e.into(),
                            }
                        }

                        _ => {
                            return syn::Error::new_spanned(
                                field,
                                "Unsupported field type for deserializer",
                            )
                            .into_compile_error()
                        }
                    };

                    field_type = Some(local_field_type);
                    field_type_new = Some(local_field_type_new);
                }
                1 => {
                    if field_name.as_str() != "units" && field_name.as_str() != "parameter" {
                        return syn::Error::new_spanned(
                             field,
                             "The second field in a value unit struct should be called \"units\" or \"parameter\"",
                         )
                         .into_compile_error();
                    }

                    unit_type = get_generic_type_argument(field).map(|x| x.0);
                    unit_field_name_ident = Some(field_name_ident);
                }
                _ => {
                    return syn::Error::new_spanned(
                        field,
                        "Only two fields are allowed: \"base\" and (\"units\" or \"parameters\"",
                    )
                    .into_compile_error()
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
            None => syn::Error::new_spanned(fields, "Unable to create deserializer for struct")
                .into_compile_error(),
            Some(deserializer) => quote! {
                let kvn_value = #deserializer;
                Ok(#type_name {
                    base: #base,
                    #unit_field_name_ident: kvn_value.unit.map(|unit| #unit_type_ident (unit)),
                })
            },
        }
    } else {
        let other_field_deserializers: Result<Vec<_>, _> = fields
             .iter()
             .map(|field| {
                let field_name = field.ident.as_ref().unwrap();

                // Unwrap is okay because we only support named structs
                // 7.4.4 Keywords must be uppercase and must not contain blanks
                let expected_kvn_name = field_name.span().source_text().unwrap().to_uppercase();

                // Unwrap is okay because we expect this span to come from the source code
                let field_type = extract_type_path(&field.ty).unwrap().span().source_text().unwrap();
                let field_type_new = extract_type_path(&field.ty).unwrap();

                if field_name == "version" {
                    return handle_version_field(type_name, field);
                }

                let parser = match field_type.as_str() {
                    "String" | "f64" | "i32" => {
                        let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                            &field_type,
                            field_type_new,
                            &expected_kvn_name,
                            true,
                            true,
                        )?;

                        quote! {
                            #deserializer_for_kvn_type?
                        }
                    }
                    "Option" => {
                        generate_call_to_deserializer_for_option_type(
                            &expected_kvn_name,
                            field
                        )?
                    }
                    "Vec" => generate_call_to_deserializer_for_vec_type(&expected_kvn_name, field)?,
                    _ => {

                        let condition_shortcut = match field_type.as_str() {
                            "String" => quote! {},
                            _ => quote! { ! #field_type_new::should_check_key_match() || },
                        };

                        quote! {
                            match lines.peek() {
                                None => Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                                    keyword: #expected_kvn_name.to_string()
                                })?,
                                Some(next_line) => {
                                    let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key(
                                        #expected_kvn_name,
                                        next_line,
                                    )?;

                                    if #condition_shortcut line_matches {
                                        #field_type_new::deserialize(lines)?
                                    } else {
                                        Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedKeyword {
                                            found: next_line.to_string(),
                                            expected: #expected_kvn_name.to_string(),
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

        if let Err(e) = other_field_deserializers {
            return e;
        }

        let other_field_deserializers = other_field_deserializers.unwrap();

        quote! {
            Ok(#type_name {
                #(#other_field_deserializers)*
            })
        }
    }
}

fn deserializers_for_struct_with_unnamed_fields(
    type_name: &proc_macro2::Ident,
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let field = fields
        .first()
        .expect("We expect exactly one item in structs with unnamed fields");

    if &type_name.to_string() == "EpochType" {
        return quote! {
            Ok(#type_name (
                crate::ndm::kvn::parser::parse_kvn_datetime_line(
                    lines.next().unwrap()
                ).map_err(|x| crate::ndm::kvn::KvnDeserializerErr::from(x))
                .map(|x| x)?.full_value
            ))
        };
    }

    // Unwrap is okay because we expect this span to come from the source code
    let field_type_new = extract_type_path(&field.ty).unwrap();
    let field_type = field_type_new.span().source_text().unwrap();

    let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
        &field_type,
        field_type_new,
        "Blalala",
        false,
        true,
    );

    let deserializer_for_kvn_type = match deserializer_for_kvn_type {
        Ok(deserializer_for_kvn_type) => deserializer_for_kvn_type,
        Err(e) => return e.into(),
    };

    quote! {
        Ok(#type_name (
            #deserializer_for_kvn_type
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
        impl #impl_generics crate::ndm::kvn::KvnDeserializer for #type_name #type_generics
        #where_clause
        {
            fn deserialize<'a>(lines: &mut ::std::iter::Peekable<impl Iterator<Item = &'a str>>)
            -> Result<#type_name, crate::ndm::kvn::KvnDeserializerErr<String>> {
                #struct_deserializer
            }

            fn should_check_key_match () -> bool {
                #should_check_key_match
            }
        }
    };

    struct_deserializer.into()
}
