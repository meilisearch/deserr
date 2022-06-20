use crate::{
    attribute_parser::{AttributeFrom, FunctionReturningError},
    parse_type::CommonDerivedTypeInfo,
};
use proc_macro2::TokenStream;
use quote::quote;

/// Return a token stream that implements `DeserializeFromValue<E>` by calling the user-provided function
pub fn generate_derive_user_function(
    info: CommonDerivedTypeInfo,
    from_attr: AttributeFrom,
) -> TokenStream {
    let CommonDerivedTypeInfo {
        impl_trait_tokens,
        err_ty,
        validate,
    } = info;

    let AttributeFrom {
        is_ref,
        from_ty,
        function:
            FunctionReturningError {
                function,
                error_ty: function_error_ty,
            },
        ..
    } = from_attr;

    let function_call = if is_ref {
        quote! { #function (&jayson_from__) }
    } else {
        quote! { #function (jayson_from__) }
    };

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(jayson_value__: jayson::Value<V>, jayson_location__: jayson::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                // first create the intermediate from_ty
                let jayson_from__ = <#from_ty as jayson::DeserializeFromValue<#err_ty>>::deserialize_from_value(jayson_value__, jayson_location__)?;
                // then apply the function to it
                let jayson_final__ = #function_call.map_err(|e| {
                    // then map the error to the final error type
                    jayson::take_result_content(
                        <#err_ty as jayson::MergeWithError<#function_error_ty>>::merge(None, e, jayson_location__)
                    )
                })?;
                #validate
            }
        }
    }
}
