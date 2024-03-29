use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{
    CommonDerivedTypeInfo,
    VariantData::{Named, Unit},
    VariantInfo,
};

/// Return a token stream that implements `Deserr<E>` for the given derived enum with internal tag
pub fn generate_derive_tagged_enum_impl(
    info: CommonDerivedTypeInfo,
    tag: String,
    variants: Vec<VariantInfo>,
) -> TokenStream {
    // `variant_impls` is the token stream of the code responsible for deserialising
    // all the fields of the enum variants and returning the fully deserialised enum.
    let variants_impls = variants
        .into_iter()
        .map(|v| generate_derive_tagged_enum_variant_impl(&info, &v))
        .collect::<Vec<_>>();

    let CommonDerivedTypeInfo {
        impl_trait_tokens,
        err_ty,
        validate,
    } = info;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: ::deserr::IntoValue>(deserr_value__: ::deserr::Value<V>, deserr_location__: ::deserr::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                // The value must always be a map
                let deserr_final__ = match deserr_value__ {
                    ::deserr::Value::Map(mut deserr_map__) => {
                        let tag_value = ::deserr::Map::remove(&mut deserr_map__, #tag).ok_or_else(|| {
                            ::deserr::take_cf_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                                None,
                                ::deserr::ErrorKind::MissingField {
                                    field: #tag,
                                },
                                deserr_location__
                            ))
                        })?;
                        let tag_value_string = match tag_value.into_value() {
                            ::deserr::Value::String(x) => x,
                            v => {
                                return ::std::result::Result::Err(
                                    ::deserr::take_cf_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                                        None,
                                        ::deserr::ErrorKind::IncorrectValueKind {
                                            actual: v,
                                            accepted: &[::deserr::ValueKind::String],
                                        },
                                        deserr_location__.push_key(#tag)
                                    ))
                                );
                            }
                        };

                        match tag_value_string.as_str() {
                            #(#variants_impls)*
                            // this is the case where the tag exists and is a string, but its value does not
                            // correspond to any valid enum variant name
                            _ => {
                                ::std::result::Result::Err(
                                    ::deserr::take_cf_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                                        None,
                                        // TODO: expected one of {expected_tags_list}, found {actual_tag} error message
                                        ::deserr::ErrorKind::Unexpected {
                                            msg: "Incorrect tag value".to_string(),
                                        },
                                        deserr_location__
                                    ))
                                )
                            }
                        }
                    },
                    // this is the case where the value is not a map
                    v => {
                        ::std::result::Result::Err(
                            ::deserr::take_cf_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                                None,
                                ::deserr::ErrorKind::IncorrectValueKind {
                                    actual: v,
                                    accepted: &[::deserr::ValueKind::Map],
                                },
                                deserr_location__
                            ))
                        )
                    }
                }?;
                #validate
            }
        }
    }
}

/// Create a token stream that deserialises all the fields of the enum variant and return
/// the fully deserialised enum.
///
/// The context of the token stream is:
///
/// ```ignore
/// let map: Map
/// match tag_value_string.as_str() {
///     === here ===
///     key => { .. }
/// }
/// ```
///
fn generate_derive_tagged_enum_variant_impl(
    info: &CommonDerivedTypeInfo,
    variant: &VariantInfo,
) -> TokenStream {
    let CommonDerivedTypeInfo { err_ty, .. } = info;

    let VariantInfo {
        ident: variant_ident,
        data,
        key_name: variant_key_name,
    } = variant;

    match data {
        Unit => {
            // If the enum variant is a unit variant, there is nothing else to do.
            quote! {
                #variant_key_name => {
                    ::std::result::Result::Ok(Self::#variant_ident)
                }
            }
        }
        Named(fields) => {
            let fields_impl = crate::generate_named_fields_impl(
                fields,
                err_ty,
                quote! { Self :: #variant_ident },
            );
            // The code here is virtually identical to the code of `generate_derive_struct_impl`
            quote! {
                #variant_key_name => {
                    let mut deserr_error__ = None;
                    #fields_impl
                }
            }
        }
    }
}

/// Create a token stream that deserialises all the fields of the enum variant and return
/// the fully deserialised enum.
/// /!\ Currently, we only support untagged enum that only contains unit variants.
///
/// The context of the token stream is:
///
/// ```ignore
/// let map: Map
/// match tag_value_string.as_str() {
///     === here ===
///     key => { .. }
/// }
/// ```
///
pub fn generate_derive_untagged_enum_impl(
    info: CommonDerivedTypeInfo,
    variants: Vec<VariantInfo>,
) -> TokenStream {
    // all the variant of the enum as a slice of `&str`
    let all_variants_as_str = variants
        .iter()
        .map(|v| &v.key_name)
        .map(|v| quote!(#v, ))
        .collect::<TokenStream>();
    let all_variants_as_str = quote!(&[#all_variants_as_str]);

    // `variant_impls` is the token stream of the code responsible for deserialising
    // the enum variants and returning the correct variant. Since we've already ensured
    // the enum was untagged, we can re-use the `generate_derive_tagged_enum_variant_impl`
    // function and only use the `Unit` part of the match.
    let variants_impls = variants
        .into_iter()
        .map(|v| generate_derive_tagged_enum_variant_impl(&info, &v))
        .collect::<Vec<_>>();

    let CommonDerivedTypeInfo {
        impl_trait_tokens,
        err_ty,
        validate,
    } = info;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: ::deserr::IntoValue>(deserr_value__: ::deserr::Value<V>, deserr_location__: ::deserr::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                // The value must always be a string
                let deserr_final__ = match deserr_value__ {
                    ::deserr::Value::String(s) => {
                        match s.as_str() {
                            #(#variants_impls)*
                            // this is the case where the tag exists and is a string, but its value does not
                            // correspond to any valid enum variant name
                            s => {
                                ::std::result::Result::Err(
                                    ::deserr::take_cf_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                                        None,
                                        ::deserr::ErrorKind::UnknownValue {
                                            value: s,
                                            accepted: #all_variants_as_str,
                                        },
                                        deserr_location__
                                    ))
                                )
                            }
                        }
                    },
                    // this is the case where the value is not a String
                    v => {
                        ::std::result::Result::Err(
                            ::deserr::take_cf_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                                None,
                                ::deserr::ErrorKind::IncorrectValueKind {
                                    actual: v,
                                    accepted: &[::deserr::ValueKind::String],
                                },
                                deserr_location__
                            ))
                        )
                    }
                }?;
                #validate
            }
        }
    }
}
