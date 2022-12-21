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
            fn deserialize_from_value<V: ::deserr::IntoValue>(deserr_value__: ::deserr::Value<V>, deserr_location__: ::deserr::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                let deserr_final__ = match deserr_value__ {
                    // The value must always be a map
                    ::deserr::Value::Map(deserr_map__) => {
                        let mut deserr_error__ = None;
                        #fields_impl
                    }
                    // this is the case where the value is not a map
                    v => {
                        ::std::result::Result::Err(
                            ::deserr::take_result_content(<#err_ty as ::deserr::DeserializeError>::error::<V>(
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
