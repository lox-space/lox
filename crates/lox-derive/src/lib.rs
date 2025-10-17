/*
 * Copyright (c) 2023. Helge Eichhorn and the LOX contributors
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::iter::zip;

use darling::{FromDeriveInput, FromMeta, util::Flag};
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Field, parse_macro_input};

#[derive(FromMeta)]
struct Scales {
    ut1: Flag,
    tdb: Flag,
    dynamic: Flag,
    multi: Flag,
}

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(lox_time))]
struct Opts {
    disable: Option<Scales>,
    error: Option<Ident>,
}

const SCALES: [&str; 6] = ["Tai", "Tcb", "Tcg", "Tdb", "Tt", "Ut1"];

#[proc_macro_derive(OffsetProvider, attributes(lox_time))]
pub fn derive_offset_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let lox_time = match crate_name("lox-time") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::lox_time),
    };

    let mut output = quote! {
        impl #lox_time::offsets::OffsetProvider for #ident {}
    };

    let eop_error = quote! {#lox_time::offsets::MissingEopProviderError};
    let delta = quote! {#lox_time::deltas::TimeDelta};
    let try_offset = quote! {#lox_time::offsets::TryOffset};
    let scales: Vec<proc_macro2::TokenStream> = SCALES
        .iter()
        .map(|scale| {
            let scale = format_ident!("{}", scale);
            quote! {#lox_time::time_scales::#scale}
        })
        .collect();
    let tai = scales[0].clone();
    let tcb = scales[1].clone();
    let tcg = scales[2].clone();
    let tdb = scales[3].clone();
    let tt = scales[4].clone();
    let ut1 = scales[5].clone();

    // No-ops

    for scale in &scales {
        output.extend(quote! {
            impl #try_offset<#scale, #scale> for #ident {
                type Error = ::core::convert::Infallible;

                fn try_offset(&self, _origin: #scale, _target: #scale, _delta: #lox_time::deltas::TimeDelta) -> Result<#delta, Self::Error> {
                    Ok(#delta::default())
                }
            }
        })
    }

    // TAI <-> TT

    let d_tai_tt = quote! {#lox_time::offsets::D_TAI_TT};

    output.extend(quote! {
        impl #try_offset<#tai, #tt> for #ident
        {
            type Error = ::core::convert::Infallible;

            fn try_offset(
                &self,
                _origin: #tai,
                _target: #tt,
                _delta: #delta,
            ) -> Result<#delta, Self::Error> {
                Ok(#d_tai_tt)
            }
        }

        impl #try_offset<#tt, #tai> for #ident
        {
            type Error = ::core::convert::Infallible;

            fn try_offset(
                &self,
                _origin: #tt,
                _target: #tai,
                _delta: #delta,
            ) -> Result<#delta, Self::Error> {
                Ok(-#d_tai_tt)
            }
        }
    });

    // TT <-> TCG

    let tt_to_tcg = quote! {#lox_time::offsets::tt_to_tcg};
    let tcg_to_tt = quote! {#lox_time::offsets::tcg_to_tt};

    output.extend(quote! {
        impl #try_offset<#tt, #tcg> for #ident
        {
            type Error = ::core::convert::Infallible;

            fn try_offset(
                &self,
                _origin: #tt,
                _target: #tcg,
                delta: #delta,
            ) -> Result<#delta, Self::Error> {
                Ok(#tt_to_tcg(delta))
            }
        }

        impl #try_offset<#tcg, #tt> for #ident
        {
            type Error = ::core::convert::Infallible;

            fn try_offset(
                &self,
                _origin: #tcg,
                _target: #tt,
                delta: #delta,
            ) -> Result<#delta, Self::Error> {
                Ok(#tcg_to_tt(delta))
            }
        }
    });

    // TCB <-> TDB

    let tdb_to_tcb = quote! {#lox_time::offsets::tdb_to_tcb};
    let tcb_to_tdb = quote! {#lox_time::offsets::tcb_to_tdb};

    output.extend(quote! {
        impl #try_offset<#tdb, #tcb> for #ident
        {
            type Error = ::core::convert::Infallible;

            fn try_offset(
                &self,
                _origin: #tdb,
                _target: #tcb,
                delta: #delta,
            ) -> Result<#delta, Self::Error> {
                Ok(#tdb_to_tcb(delta))
            }
        }

        impl #try_offset<#tcb, #tdb> for #ident
        {
            type Error = ::core::convert::Infallible;

            fn try_offset(
                &self,
                _origin: #tcb,
                _target: #tdb,
                delta: #delta,
            ) -> Result<#delta, Self::Error> {
                Ok(#tcb_to_tdb(delta))
            }
        }
    });

    // TT <-> TDB

    let disable_tdb = opts
        .disable
        .as_ref()
        .is_some_and(|disable| disable.tdb.is_present());

    if !disable_tdb {
        let tdb_to_tt = quote! {#lox_time::offsets::tdb_to_tt};
        let tt_to_tdb = quote! {#lox_time::offsets::tt_to_tdb};

        output.extend(quote! {
            impl #try_offset<#tdb, #tt> for #ident
            {
                type Error = ::core::convert::Infallible;

                fn try_offset(
                    &self,
                    _origin: #tdb,
                    _target: #tt,
                    delta: #delta,
                ) -> Result<#delta, Self::Error> {
                    Ok(#tdb_to_tt(delta))
                }
            }

            impl #try_offset<#tt, #tdb> for #ident
            {
                type Error = ::core::convert::Infallible;

                fn try_offset(
                    &self,
                    _origin: #tt,
                    _target: #tdb,
                    delta: #delta,
                ) -> Result<#delta, Self::Error> {
                    Ok(#tt_to_tdb(delta))
                }
            }
        });
    }

    // UT1

    let disable_ut1 = opts
        .disable
        .as_ref()
        .is_some_and(|disable| disable.ut1.is_present());

    if !disable_ut1 {
        for scale in scales.split_last().unwrap().1 {
            output.extend(quote! {
                impl #try_offset<#ut1, #scale> for #ident
                {
                    type Error = #eop_error;

                    fn try_offset(
                        &self,
                        _origin: #ut1,
                        _target: #scale,
                        delta: #delta,
                    ) -> Result<#delta, Self::Error> {
                        Err(#eop_error)
                    }
                }

                impl #try_offset<#scale, #ut1> for #ident
                {
                    type Error = #eop_error;

                    fn try_offset(
                        &self,
                        _origin: #scale,
                        _target: #ut1,
                        delta: #delta,
                    ) -> Result<#delta, Self::Error> {
                        Err(#eop_error)
                    }
                }
            });
        }
    }

    // DynTimeScale

    let disable_dyn = opts
        .disable
        .as_ref()
        .is_some_and(|disable| disable.dynamic.is_present());

    if !disable_dyn {
        let dyn_scale = quote! {#lox_time::time_scales::DynTimeScale};
        let dyn_scales: Vec<proc_macro2::TokenStream> = SCALES
            .iter()
            .map(|scale| {
                let scale = format_ident!("{}", scale);
                quote! {#dyn_scale::#scale}
            })
            .collect();
        let error = opts
            .error
            .map(|err| {
                let err = quote! {#err};
                // FIXME: Remove once `!` lands on stable.
                output.extend(quote! {
                    impl From<::core::convert::Infallible> for #err {
                        fn from(_: ::core::convert::Infallible) -> Self {
                            #err::default()
                        }
                    }
                });
                err
            })
            .unwrap_or(eop_error.clone());

        let mut arms = quote! {};

        for (dyn_scale1, scale1) in zip(&dyn_scales, &scales) {
            for (dyn_scale2, scale2) in zip(&dyn_scales, &scales) {
                if scale1.to_string() == scale2.to_string() {
                    continue;
                }
                arms.extend(quote! {
                    (#dyn_scale1, #dyn_scale2) => {
                        Ok(self.try_offset(#scale1, #scale2, delta)?)
                    }
                })
            }
        }

        output.extend(quote! {
            impl #try_offset<#dyn_scale, #dyn_scale> for #ident {
                type Error = #error;

                fn try_offset(&self, origin: #dyn_scale, target: #dyn_scale, delta: #delta) -> Result<#delta, Self::Error> {
                    match (origin, target) {
                        #arms
                        (_, _) => Ok(#delta::default()),
                    }
                }
            }
        });

        for scale in scales.split_last().unwrap().1 {
            let mut arms1 = quote! {};
            let mut arms2 = quote! {};

            for (dyn_scale, scale) in zip(&dyn_scales, &scales) {
                arms1.extend(quote! {
                    #dyn_scale => Ok(self.try_offset(#scale, target, delta)?),
                });
                arms2.extend(quote! {
                    #dyn_scale => Ok(self.try_offset(origin, #scale, delta)?),
                });
            }

            output.extend(quote! {
                impl #try_offset<#dyn_scale, #scale> for #ident {
                    type Error = #error;

                    fn try_offset(&self, origin: #dyn_scale, target: #scale, delta: #delta) -> Result<#delta, Self::Error> {
                        match origin {
                            #arms1
                        }
                    }
                }

                impl #try_offset<#scale, #dyn_scale> for #ident {
                    type Error = #error;

                    fn try_offset(&self, origin: #scale, target: #dyn_scale, delta: #delta) -> Result<#delta, Self::Error> {
                        match target {
                            #arms2
                        }
                    }
                }
            });
        }
    }

    // Two-step transformations

    let disable_multi = opts
        .disable
        .as_ref()
        .is_some_and(|disable| disable.multi.is_present());

    if !disable_multi && !disable_tdb {
        let multis = [
            (&tai, &tt, &tdb),
            (&tdb, &tt, &tcg),
            (&tai, &tt, &tcg),
            (&tai, &tdb, &tcb),
            (&tt, &tdb, &tcb),
            (&tcb, &tdb, &tcg),
        ];

        let two_step = quote! {#lox_time::offsets::two_step_offset};

        for (origin, via, target) in multis {
            output.extend(quote!{
                impl #try_offset<#origin, #target> for #ident
                {
                    type Error = ::core::convert::Infallible;

                    fn try_offset(&self, origin: #origin, target: #target, delta: #delta) -> Result<#delta, Self::Error> {
                        Ok(#two_step(self, origin, #via, target, delta))
                    }
                }

                impl #try_offset<#target, #origin> for #ident
                {
                    type Error = ::core::convert::Infallible;

                    fn try_offset(&self, origin: #target, target: #origin, delta: #delta) -> Result<#delta, Self::Error> {
                        Ok(#two_step(self, origin, #via, target, delta))
                    }
                }
            });
        }
    }

    output.into()
}

fn generate_call_to_deserializer_for_covariance_matrix_kvn_type(
    expected_kvn_name: &str,
) -> proc_macro2::TokenStream {
    quote! {
        match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
            None => Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                keyword: #expected_kvn_name.to_string()
            }),
            Some(next_line) => {
                let result = crate::ndm::kvn::parser::parse_kvn_covariance_matrix(
                    lines,
                ).map_err(|x| match crate::ndm::kvn::KvnDeserializerErr::from(x) {
                    crate::ndm::kvn::KvnDeserializerErr::InvalidCovarianceMatrixFormat { .. } => crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedKeyword {
                        // This is empty because we just want to tell the
                        // vector iterator to stop the iteration.
                        found: "".to_string(),
                        expected: "".to_string(),
                    },
                    crate::ndm::kvn::KvnDeserializerErr::UnexpectedEndOfInput { keyword } => crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                        keyword
                    },
                    e => e,
                })?;

                Ok(result)
            }
        }
    }
}

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
                    match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
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
        "common::StateVectorAccType" => Ok(quote! {
            match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
                None => Err(crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                    keyword: #expected_kvn_name.to_string()
                }),
                Some(next_line) => {
                    let result = crate::ndm::kvn::parser::parse_kvn_state_vector(
                        next_line,
                    ).map_err(|x| match crate::ndm::kvn::KvnDeserializerErr::from(x) {
                        crate::ndm::kvn::KvnDeserializerErr::InvalidStateVectorFormat { .. } => crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedKeyword {
                            // This is empty because we just want to tell the
                            // vector iterator to stop the iteration.
                            found: "".to_string(),
                            expected: "".to_string(),
                        },
                        e => e,
                    }).map(|x| x.into());

                    if result.is_ok() {
                        let _ = lines.next().unwrap();
                    }

                    result
                }
            }
        }),
        _ => Ok(quote! {
           {
                let has_next_line = crate::ndm::kvn::parser::get_next_nonempty_line(lines).is_some();

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
                if let Some(syn::GenericArgument::Type(syn::Type::Path(r#type))) =
                    &type_argument.args.first()
                {
                    return Some((
                        r#type
                            .path
                            .segments
                            .clone()
                            .into_iter()
                            .map(|ident| ident.to_token_stream().to_string())
                            .reduce(|a, b| a + "::" + &b)
                            .unwrap(),
                        &r#type.path,
                    ));
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
    let (type_name, type_ident) = get_generic_type_argument(field).ok_or(
        syn::Error::new_spanned(field, "Malformed type for `#[derive(KvnDeserialize)]`")
            .into_compile_error(),
    )?;

    let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
        type_name.as_ref(),
        type_ident,
        expected_kvn_name,
        true,
        true,
    )?;

    let condition_shortcut = match type_name.as_str() {
        "String" | "f64" | "i32" | "u64" => quote! {},
        _ => quote! { ! #type_ident::should_check_key_match() || },
    };

    let value = match type_name.as_ref() {
        "NonNegativeDouble" | "NegativeDouble" | "PositiveDouble" => {
            quote! { #type_ident (item) }
        }
        _ => quote! { item },
    };

    Ok(quote! {
        match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
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

    Ok(quote! {
        {
            let mut items: Vec<#type_ident> = Vec::new();

            let mut is_retry = false;

            loop {
                let result = #deserializer_for_kvn_type;

                match result {
                    Ok(item) => {
                        is_retry = false;
                        items.push(item)
                    },
                    Err(crate::ndm::kvn::KvnDeserializerErr::UnexpectedKeyword { .. }) |
                    Err(crate::ndm::kvn::KvnDeserializerErr::UnexpectedEndOfInput { .. }) => if is_retry {
                        break;
                    } else {
                        is_retry = true;
                        continue;
                    },
                    Err(e) => Err(e)?,
                }
            }

            items
        }
    })
}

fn get_prefix_and_postfix_keyword(attrs: &[syn::Attribute]) -> Option<(String, String)> {
    let mut keyword: Option<syn::LitStr> = None;

    for attr in attrs.iter() {
        if !attr.path().is_ident("kvn") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix_and_postfix_keyword") {
                let value = meta.value()?;
                keyword = value.parse()?;

                Ok(())
            } else {
                Err(meta.error("unsupported attribute"))
            }
        });
    }

    keyword.map(|keyword| {
        let keyword = keyword.value().to_uppercase();

        (format!("{keyword}_START"), format!("{keyword}_STOP"))
    })
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
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), proc_macro2::TokenStream> {
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
    let expected_kvn_name = format!("CCSDS_{message_type_name}_VERS");

    // Unwrap is okay because we expect this to be a well defined type path,
    // because this is not a general-purpose proc macro, but one that we
    // control the input to ourselves.
    let field_type = extract_type_path(&field.ty)
        .unwrap()
        .to_token_stream()
        .to_string();
    let field_type_new = extract_type_path(&field.ty).unwrap();

    let parser = generate_call_to_deserializer_for_kvn_type(
        &field_type,
        field_type_new,
        &expected_kvn_name,
        true,
        true,
    )?;

    Ok((
        quote! { let #field_name = #parser?; },
        quote! { #field_name, },
    ))
}

fn deserializer_for_struct_with_named_fields(
    type_name: &proc_macro2::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    is_value_unit_struct: bool,
    struct_level_prefix_and_postfix_keyword: Option<(String, String)>,
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
            // Unwrap is okay because we only support named structs
            let field_name_ident = field.ident.as_ref().unwrap();

            let field_name = field_name_ident.to_token_stream().to_string();

            match index {
                0 => {
                    if field_name.as_str() != "base" {
                        return syn::Error::new_spanned(
                            field,
                            "The first field in a value unit struct should be called \"base\"",
                        )
                        .into_compile_error();
                    }

                    // Unwrap is okay because we expect this to be a well defined type path,
                    // because this is not a general-purpose proc macro, but one that we
                    // control the input to ourselves.
                    let local_field_type = extract_type_path(&field.ty)
                        .unwrap()
                        .to_token_stream()
                        .to_string();
                    let local_field_type_new = extract_type_path(&field.ty).unwrap();

                    match local_field_type.as_str() {
                        "KvnDateTimeValue" | "String" | "f64" | "i32" | "NonNegativeDouble"
                        | "NegativeDouble" | "PositiveDouble" => {
                            match generate_call_to_deserializer_for_kvn_type(
                                &local_field_type,
                                local_field_type_new,
                                "undefined",
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
                            .into_compile_error();
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
                    .into_compile_error();
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
        let field_deserializers: Result<Vec<_>, _> = fields.iter().filter(|field| {
            // For OemCovarianceMatrixType we filter the types which start with
            // cx, cy and cz because we populate those differently
            if type_name != "OemCovarianceMatrixType" {
                return true
            }

            // Unwrap is okay because we only support named structs
            let field_name = field.ident.as_ref().unwrap().to_token_stream().to_string();

            !field_name.starts_with("cx")
                && !field_name.starts_with("cy")
                && !field_name.starts_with("cz")
        }).map(|field| {
                let field_name = field.ident.as_ref().unwrap();

                // Unwrap is okay because we only support named structs
                // 7.4.4 Keywords must be uppercase and must not contain blanks
                let expected_kvn_name = field_name.to_token_stream().to_string().to_uppercase();

                // Unwrap is okay because we expect this to be a well defined type path,
                // because this is not a general-purpose proc macro, but one that we
                // control the input to ourselves.
                let field_type_new = extract_type_path(&field.ty).unwrap();

                // Unwrap is okay becuase we always expect at least one type
                let field_main_type = field_type_new.segments.iter()
                    .next_back()
                    .unwrap()
                    .ident
                    .to_string();

                if field_name == "version" {
                    return handle_version_field(type_name, field);
                }

                let parser = match field_main_type.as_str() {
                    "String" | "f64" | "i32" => {
                        let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
                            &field_main_type,
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

                        let condition_shortcut = match field_main_type.as_str() {
                            "String" => quote! {},
                            _ => quote! { ! #field_type_new::should_check_key_match() || },
                        };

                        quote! {
                            match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
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

                let field_prefix_and_postfix_keyword_checks = get_prefix_and_postfix_keyword(&field.attrs);

                let wrapped_parser = add_prefix_and_postfix_keyword_checks(field_prefix_and_postfix_keyword_checks, parser, true);

                Ok((
                    quote! { let #field_name = #wrapped_parser; },
                    quote! { #field_name, }
                ))
             })
             .collect();

        if let Err(e) = field_deserializers {
            return e;
        }

        let mut field_deserializers = field_deserializers.unwrap();

        if type_name == "OemCovarianceMatrixType" {
            let covariance_matrix_parser =
                generate_call_to_deserializer_for_covariance_matrix_kvn_type("COVARIANCE_MATRIX");

            field_deserializers.push((
                quote! { let covariance_matrix = #covariance_matrix_parser?; },
                quote! {},
            ));

            for (field, field_type) in [
                ("cx_x", "PositionCovarianceType"),
                ("cy_x", "PositionCovarianceType"),
                ("cy_y", "PositionCovarianceType"),
                ("cz_x", "PositionCovarianceType"),
                ("cz_y", "PositionCovarianceType"),
                ("cz_z", "PositionCovarianceType"),
                ("cx_dot_x", "PositionVelocityCovarianceType"),
                ("cx_dot_y", "PositionVelocityCovarianceType"),
                ("cx_dot_z", "PositionVelocityCovarianceType"),
                ("cx_dot_x_dot", "VelocityCovarianceType"),
                ("cy_dot_x", "PositionVelocityCovarianceType"),
                ("cy_dot_y", "PositionVelocityCovarianceType"),
                ("cy_dot_z", "PositionVelocityCovarianceType"),
                ("cy_dot_x_dot", "VelocityCovarianceType"),
                ("cy_dot_y_dot", "VelocityCovarianceType"),
                ("cz_dot_x", "PositionVelocityCovarianceType"),
                ("cz_dot_y", "PositionVelocityCovarianceType"),
                ("cz_dot_z", "PositionVelocityCovarianceType"),
                ("cz_dot_x_dot", "VelocityCovarianceType"),
                ("cz_dot_y_dot", "VelocityCovarianceType"),
                ("cz_dot_z_dot", "VelocityCovarianceType"),
            ] {
                let field_ident = syn::Ident::new(field, Span::call_site());
                let type_ident = syn::Ident::new(field_type, Span::call_site());

                field_deserializers.push((
                    quote! {},
                    quote! {
                        #field_ident: #type_ident {
                            base: covariance_matrix.#field_ident,
                            units: None,
                        },
                    },
                ));
            }
        }

        let (field_deserializers, fields): (Vec<_>, Vec<_>) =
            field_deserializers.into_iter().unzip();

        let parser_to_wrap = quote! {
            #(#field_deserializers)*

            Ok(#type_name {
                #(#fields)*
            })
        };

        let wrapped_parser = add_prefix_and_postfix_keyword_checks(
            struct_level_prefix_and_postfix_keyword,
            parser_to_wrap,
            false,
        );

        quote! {
            #wrapped_parser
        }
    }
}

fn add_prefix_and_postfix_keyword_checks(
    prefix_and_postfix_keyword: Option<(String, String)>,
    parser_to_wrap: proc_macro2::TokenStream,
    is_field: bool,
) -> proc_macro2::TokenStream {
    match prefix_and_postfix_keyword {
        None => parser_to_wrap,
        Some((prefix_keyword, postfix_keyword)) => {
            let mismatch_handler = if is_field {
                quote! { Default::default() }
            } else {
                quote! {
                    Err(
                        crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedKeyword {
                            found: next_line.to_string(),
                            expected: #prefix_keyword.to_string(),
                        },
                    )?
                }
            };

            quote! {

                match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
                    None => Err(
                        crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                            keyword: #prefix_keyword.to_string(),
                        },
                    )?,

                    Some(next_line) => {
                        let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key(
                            #prefix_keyword,
                            next_line,
                        )?;

                        if line_matches {
                            lines.next().unwrap();

                            let result = { #parser_to_wrap };

                            match crate::ndm::kvn::parser::get_next_nonempty_line(lines) {
                                None =>  Err(
                                    crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedEndOfInput {
                                        keyword: #postfix_keyword.to_string(),
                                    },
                                )?,

                                Some(next_line) => {
                                    let line_matches = crate::ndm::kvn::parser::kvn_line_matches_key(
                                        #postfix_keyword,
                                        next_line,
                                    )?;

                                    if line_matches {
                                        lines.next().unwrap();
                                    } else {
                                        Err(
                                            crate::ndm::kvn::KvnDeserializerErr::<String>::UnexpectedKeyword {
                                                found: next_line.to_string(),
                                                expected: #postfix_keyword.to_string(),
                                            },
                                        )?
                                    }
                                }
                            };

                            result
                        } else {
                            #mismatch_handler
                        }
                    }
                }
            }
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

    // Unwrap is okay because we expect this to be a well defined type path,
    // because this is not a general-purpose proc macro, but one that we
    // control the input to ourselves.
    let field_type_new = extract_type_path(&field.ty).unwrap();
    let field_type = field_type_new.to_token_stream().to_string();

    let deserializer_for_kvn_type = generate_call_to_deserializer_for_kvn_type(
        &field_type,
        field_type_new,
        "unnamed field",
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
    let prefix_and_postfix_keyword = get_prefix_and_postfix_keyword(&item.attrs);

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
            deserializer_for_struct_with_named_fields(
                type_name,
                &named,
                is_value_unit_struct,
                prefix_and_postfix_keyword,
            ),
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
            .into();
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
