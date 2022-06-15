use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{CommonDerivedTypeInfo, NamedFieldsInfo};

/// Return a token stream that implements `DeserializeFromValue<E>` for the given derived struct with named fields
pub fn generate_derive_struct_impl(
    info: CommonDerivedTypeInfo,
    fields: NamedFieldsInfo,
) -> TokenStream {
    let CommonDerivedTypeInfo {
        impl_trait_tokens,
        unknown_key,
        err_ty,
    } = info;

    let NamedFieldsInfo {
        field_names,
        field_tys,
        field_defaults,
        missing_field_errors,
        key_names,
    } = fields;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(value: jayson::Value<V>, current_location: jayson::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                match value {
                    // The value must always be a map
                    jayson::Value::Map(map) => {
                        // Start by declaring all the fields as mutable optionals
                        // Their initial value is given by the precomputed `#field_defaults`,
                        // see [NamedFieldsInfo] and [NamedFieldsInfo::parse].
                        //
                        // The initial value of #field_names is `None` if the field has no initial value and
                        // thus must be given by the map, and `Some` otherwise.
                        #(
                            let mut #field_names = #field_defaults;
                        )*
                        // We traverse the entire map instead of looking for specific keys, because we want
                        // to handle the case where a key is unknown and the attribute `deny_unknown_fields` was used.
                        for (key, value) in jayson::Map::into_iter(map) {
                            match key.as_str() {
                                // For each known key, look at the corresponding value and try to deserialize it
                                #(
                                    #key_names => {
                                        #field_names = ::std::option::Option::Some(
                                            <#field_tys as jayson::DeserializeFromValue<#err_ty>>::deserialize_from_value(
                                                jayson::IntoValue::into_value(value),
                                                current_location.push_key(key.as_str())
                                            )?
                                        );
                                    }
                                )*
                                // For an unknownn key, use the precomputed #unknown_key token stream
                                key => { #unknown_key }
                            }
                        }

                        // If the deserialization was successful, then all #field_names are `Some(..)`
                        // Otherwise, it means the map was missing a key
                        ::std::result::Result::Ok(Self {
                            #(
                                #field_names : #field_names.ok_or_else(|| #missing_field_errors)?,
                            )*
                        })
                    }
                    // this is the case where the value is not a map
                    _ => {
                        ::std::result::Result::Err(
                            <#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                &[jayson::ValueKind::Map],
                                current_location
                            )
                        )
                    }
                }
            }
        }
    }
}
