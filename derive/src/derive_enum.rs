use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{
    CommonDerivedTypeInfo, NamedFieldsInfo,
    VariantData::{Named, Unit},
    VariantInfo,
};

/// Return a token stream that implements `DeserializeFromValue<E>` for the given derived enum with internal tag
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
        ..
    } = info;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(jayson_value__: jayson::Value<V>, jayson_location__: jayson::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                // The value must always be a map
                match jayson_value__ {
                    jayson::Value::Map(mut map) => {
                        let tag_value = jayson::Map::remove(&mut map, #tag).ok_or_else(|| {
                            jayson::take_result_content(<#err_ty as jayson::DeserializeError>::missing_field(
                                None,
                                #tag,
                                jayson_location__
                            ))
                        })?;
                        let tag_value_string = match tag_value.into_value() {
                            jayson::Value::String(x) => x,
                            v @ _ => {
                                return ::std::result::Result::Err(
                                    <#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                        None,
                                        v.kind(),
                                        &[jayson::ValueKind::String],
                                        jayson_location__.push_key(#tag)
                                    )?
                                );
                            }
                        };

                        match tag_value_string.as_str() {
                            #(#variants_impls)*
                            // this is the case where the tag exists and is a string, but its value does not
                            // correspond to any valid enum variant name
                            _ => {
                                ::std::result::Result::Err(
                                    <#err_ty as jayson::DeserializeError>::unexpected(
                                        None,
                                        // TODO: expected one of {expected_tags_list}, found {actual_tag} error message
                                        "Incorrect tag value",
                                        jayson_location__
                                    )?
                                )
                            }
                        }
                    }
                    // this is the case where the value is not a map
                    v @ _ => {
                        ::std::result::Result::Err(
                            <#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                None,
                                v.kind(),
                                &[jayson::ValueKind::Map],
                                jayson_location__
                            )?
                        )
                    }
                }
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
            let NamedFieldsInfo {
                field_names,
                field_tys,
                field_defaults,
                missing_field_errors,
                key_names,
                unknown_key,
            } = fields;

            // The code here is virtually identical to the code of `generate_derive_struct_impl`
            quote! {
                #variant_key_name => {
                    let mut jayson_error__ = None;
                    // Start by declaring all the fields as mutable optionals
                    // Their initial value is given by the precomputed `#field_defaults`,
                    // see [NamedFieldsInfo] and [NamedFieldsInfo::parse].
                    //
                    // The initial value of #field_names is `None` if the field has no initial value and
                    // thus must be given by the map, and `Some` otherwise.
                    #(
                        let mut #field_names : jayson::FieldState<_> = #field_defaults .into();
                    )*
                    // We traverse the entire map instead of looking for specific keys, because we want
                    // to handle the case where a key is unknown and the attribute `deny_unknown_fields` was used.
                    for (jayson_key__, jayson_value__) in jayson::Map::into_iter(map) {
                        match jayson_key__.as_str() {
                            // For each known key, look at the corresponding value and try to deserialize it
                            #(
                                #key_names => {
                                    #field_names = match
                                        <#field_tys as jayson::DeserializeFromValue<#err_ty>>::deserialize_from_value(
                                            jayson::IntoValue::into_value(jayson_value__),
                                            jayson_location__.push_key(jayson_key__.as_str())
                                        ) {
                                            Ok(x) => jayson::FieldState::Some(x),
                                            Err(e) => {
                                                jayson_error__ = Some(<#err_ty as jayson::MergeWithError<_>>::merge(jayson_error__, e)?);
                                                jayson::FieldState::Err
                                            }
                                        };
                                }
                            )*
                            // For an unknownn key, use the precomputed #unknown_key token stream
                            jayson_key__ => { #unknown_key }
                        }
                    }
                    // Now we check whether any field was missing
                    #(
                        if #field_names .is_missing() {
                            #missing_field_errors
                        }
                    )*

                    if let Some(jayson_error__) = jayson_error__ {
                        ::std::result::Result::Err(jayson_error__)
                    } else {
                        // If the deserialization was successful, then all #field_names are `Some(..)`
                        // Otherwise, an error was thrown earlier
                        ::std::result::Result::Ok(Self::#variant_ident {
                            #(
                                #field_names : #field_names.unwrap(),
                            )*
                        })
                    }
                }
            }
        }
    }
}
