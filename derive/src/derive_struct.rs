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
        err_ty,
        validate,
    } = info;

    let fields_impl = crate::generate_named_fields_impl(&fields, &err_ty, quote! { Self });

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(jayson_value__: jayson::Value<V>, jayson_location__: jayson::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                let jayson_final__ = match jayson_value__ {
                    // The value must always be a map
                    jayson::Value::Map(jayson_map__) => {
                        let mut jayson_error__ = None;
                        #fields_impl
                    }
                    // this is the case where the value is not a map
                    v @ _ => {
                        ::std::result::Result::Err(
                            jayson::take_result_content(<#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                None,
                                v.kind(),
                                &[jayson::ValueKind::Map],
                                jayson_location__
                            ))
                        )
                    }
                }?;
                #validate
            }
        }
    }
}
