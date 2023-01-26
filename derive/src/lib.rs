extern crate proc_macro;

mod attribute_parser;
mod derive_enum;
mod derive_named_fields;
mod derive_struct;
mod derive_user_provided_function;
mod parse_type;

use attribute_parser::TagType;
use derive_named_fields::generate_named_fields_impl;
use parse_type::{DerivedTypeInfo, TraitImplementationInfo, VariantData};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DeserializeFromValue, attributes(deserr, serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match DerivedTypeInfo::parse(input) {
        Ok(derived_type_info) => match derived_type_info.data {
            TraitImplementationInfo::Struct(fields) => {
                derive_struct::generate_derive_struct_impl(derived_type_info.common, fields).into()
            }
            TraitImplementationInfo::Enum { tag, variants } => match tag {
                TagType::Internal(tag_key) => derive_enum::generate_derive_tagged_enum_impl(
                    derived_type_info.common,
                    tag_key,
                    variants,
                )
                .into(),
                TagType::External
                    if variants
                        .iter()
                        .all(|variant| matches!(variant.data, VariantData::Unit)) =>
                {
                    derive_enum::generate_derive_untagged_enum_impl(
                        derived_type_info.common,
                        variants,
                    )
                    .into()
                }
                TagType::External =>
                    syn::Error::new(
                        Span::call_site(),
                        r##"Externally tagged enums are not supported yet by deserr. Add #[deserr(tag = "some_tag_key")]"##,
                ).to_compile_error().into()
            },
            TraitImplementationInfo::UserProvidedFunction { from_attr } => {
                derive_user_provided_function::generate_derive_user_function(
                    derived_type_info.common,
                    from_attr,
                )
                .into()
            }
        },
        Err(e) => e.to_compile_error().into(),
    }
}
