use quote::quote;
use syn::spanned::Spanned;

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

            let (parser_function, parser_error_type, parser_unit_parameter) =
                match field_type.as_str() {
                    "KvnDateTimeValue" => (
                        quote! { parse_kvn_datetime_line },
                        quote! { KvnDeserializerErr::DateTime},
                        quote! {},
                    ),
                    "KvnStringValue" => (
                        quote! { parse_kvn_string_line },
                        quote! { KvnDeserializerErr::String },
                        //@TODO make unit dynamic according to the definition
                        quote! { ,true },
                    ),
                    "KvnNumericValue" => (
                        quote! { parse_kvn_numeric_line },
                        quote! { KvnDeserializerErr::Number },
                        //@TODO make unit dynamic according to the definition
                        quote! { ,true },
                    ),
                    "KvnIntegerValue" => (
                        quote! { parse_kvn_integer_line },
                        quote! { KvnDeserializerErr::Number },
                        //@TODO make unit dynamic according to the definition
                        quote! { ,true },
                    ),
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
                #field_name: #parser_function(
                    #expected_kvn_name,
                    lines.next().ok_or(KvnDeserializerErr::UnexpectedEndOfInput)?
                    #parser_unit_parameter
                ).map_err(|x| #parser_error_type(x))?.1,
            })
        })
        .collect();

    if let Err(e) = field_initializers {
        return e;
    }

    let field_initializers = field_initializers.unwrap();

    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    let deserializer = quote! {
        impl #impl_generics KvnDeserializer<#name> for #name #type_generics
        #where_clause
        {
            fn deserialize<'a>(lines: &mut dyn Iterator<Item = &'a str>) -> Result<#name, KvnDeserializerErr<&'a str>> {
                Ok(#name {
                    #(#field_initializers)*
                })
            }
        }
    };

    deserializer.into()
}
