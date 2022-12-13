use crate::parse_type::NamedFieldsInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_named_fields_impl(
    fields: &NamedFieldsInfo,
    err_ty: &syn::Type,
    create: TokenStream,
) -> TokenStream {
    let NamedFieldsInfo {
        field_names,
        field_tys,
        field_defaults,
        field_errs,
        field_maps,
        missing_field_errors,
        key_names,
        unknown_key,
        needs_predicate: _,
    } = fields;
    quote! {
        // Start by declaring all the fields as mutable optionals
        // Their initial value is given by the precomputed `#field_defaults`,
        // see [NamedFieldsInfo] and [NamedFieldsInfo::parse].
        //
        // The initial value of #field_names is `None` if the field has no initial value and
        // thus must be given by the map, and `Some` otherwise.
        #(
            let mut #field_names : ::deserr::FieldState<_> = #field_defaults .into();
        )*
        // We traverse the entire map instead of looking for specific keys, because we want
        // to handle the case where a key is unknown and the attribute `deny_unknown_fields` was used.
        for (deserr_key__, deserr_value__) in ::deserr::Map::into_iter(deserr_map__) {
            match deserr_key__.as_str() {
                // For each known key, look at the corresponding value and try to deserialize it
                #(
                    #key_names => {
                        #field_names = match
                            <#field_tys as ::deserr::DeserializeFromValue<#field_errs>>::deserialize_from_value(
                                ::deserr::IntoValue::into_value(deserr_value__),
                                deserr_location__.push_key(deserr_key__.as_str())
                            ) {
                                Ok(x) => ::deserr::FieldState::Some(x),
                                Err(e) => {
                                    deserr_error__ = Some(<#err_ty as ::deserr::MergeWithError<_>>::merge(
                                        deserr_error__,
                                        e,
                                        deserr_location__.push_key(deserr_key__.as_str())
                                    )?);
                                    ::deserr::FieldState::Err
                                }
                            };
                    }
                )*
                // For an unknown key, use the precomputed #unknown_key token stream
                deserr_key__ => {
                    #unknown_key
                }
            }
        }
        // Now we check whether any field was missing
        #(
            if #field_names .is_missing() {
                #missing_field_errors
            }
        )*

        if let Some(deserr_error__) = deserr_error__ {
            ::std::result::Result::Err(deserr_error__)
        } else {
            // If the deserialization was successful, then all #field_names are `Some(..)`
            // Otherwise, an error was thrown earlier
            ::std::result::Result::Ok(#create {
                #(
                    #field_names : #field_names.map(#field_maps).unwrap(),
                )*
            })
        }

    }
}
