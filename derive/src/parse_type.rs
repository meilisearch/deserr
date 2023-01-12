use crate::attribute_parser::{
    read_deserr_container_attributes, read_deserr_field_attributes, read_deserr_variant_attributes,
    validate_container_attributes, AttributeFrom, ContainerAttributesInfo, DefaultFieldAttribute,
    DenyUnknownFields, FunctionReturningError, RenameAll, TagType,
};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_quote, Data, DeriveInput, WherePredicate};

/// Contains all the information needed to generate a
/// `DeserializeFromValue` implementation for the derived type,
/// in a conveniently preprocessed form.
///
/// It is created via `[DerivedTypeInfo::parse]`
pub struct DerivedTypeInfo {
    /// Information that is common to both structs and enums
    pub common: CommonDerivedTypeInfo,
    /// Information specific to structs or enums
    pub data: TraitImplementationInfo,
}

/// The subset of [`DerivedTypeInfo`] that contains information
/// common to both structs and enums.
pub struct CommonDerivedTypeInfo {
    /// A token stream representing the `impl<..> DeserializeFromValue for #ident .. where ..` line.
    pub impl_trait_tokens: TokenStream,
    /// The custom error type `E` that is the generic parameter
    /// of the derived `DeserializeFromValue<E>` trait implementation.
    ///
    /// It is relevant to the `error` attribute, which is necessary for now.
    pub err_ty: syn::Type,

    pub validate: TokenStream,
}

/// The subset of [`DerivedTypeInfo`] that contains information
/// specific to structs or enums
#[allow(clippy::large_enum_variant)]
pub enum TraitImplementationInfo {
    Struct(NamedFieldsInfo),
    Enum {
        tag: TagType,
        variants: Vec<VariantInfo>,
    },
    UserProvidedFunction {
        from_attr: AttributeFrom,
    },
}

/// Contains all the information needed to generate the deserialization code
/// for a specific enum variant
pub struct VariantInfo {
    /// The identifier of the enum variant
    pub ident: Ident,

    /// Describes the kind of variant and its content
    pub data: VariantData,

    /// The key name (in the serialized value) that represents this variant.
    ///
    /// It is relevant to the `rename` and `rename_all` attributes
    pub key_name: String,
}

/// Contains the information needed to generate the deserialization code
/// for the content of an enum variant.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum VariantData {
    /// The variant is a unit variant, such as `Option::None`
    Unit,

    /// The variant is a variant with named fields, such as `Position { line: usize, col: usize }`
    Named(NamedFieldsInfo),
}

impl DerivedTypeInfo {
    pub fn parse(input: DeriveInput) -> syn::Result<Self> {
        // First, read the attributes on the derived input
        // e.g. `#[deserr(error = MyError, tag = "mytag", rename_all = camelCase)]`
        let attrs = read_deserr_container_attributes(&input.attrs)?;

        validate_container_attributes(&attrs, &input)?;

        // The error type as given by the attribute #[deserr(error = err_ty)]
        let user_provided_err_ty: Option<&syn::Type> = attrs.err_ty.as_ref();
        let err_ty = user_provided_err_ty
            .cloned()
            .unwrap_or_else(|| parse_quote!(__Deserr_E));

        // Now we build the TraitImplementationInfo structure

        let data = if let Some(from) = &attrs.from {
            // if there was a container `from` attribute, then it doesn't matter what the derived input
            // is, we just call the provided function to deserialise it
            TraitImplementationInfo::UserProvidedFunction {
                from_attr: from.clone(),
            }
        } else {
            // Otherwise, we parse derive information specific to structs or enums
            match input.data {
                Data::Struct(s) => match s.fields {
                    syn::Fields::Named(fields) => TraitImplementationInfo::Struct(
                        NamedFieldsInfo::parse(fields, &attrs, &err_ty)?,
                    ),
                    syn::Fields::Unnamed(fields) => return Err(syn::Error::new(
                        fields.span(),
                        "Tuple structs aren't supported by the DeserializeFromValue derive macro",
                    )),
                    syn::Fields::Unit => return Err(syn::Error::new(
                        Span::call_site(),
                        "Unit structs aren't supported by the DeserializeFromValue derive macro",
                    )),
                },
                Data::Enum(e) => {
                    // parse a VariantInfo for each variant in the enum
                    let mut parsed_variants = vec![];
                    for variant in e.variants {
                        let variant_attrs = read_deserr_variant_attributes(&variant.attrs)?;

                        let renamed = variant_attrs.rename.as_ref().map(|i| i.value());

                        // The key in the serialized value representing the variant, which is influenced by the
                        // `rename` and `rename_all` attributes
                        let key_name = key_name_for_ident(
                            variant.ident.to_string(),
                            attrs.rename_all.as_ref(),
                            renamed.as_deref(),
                        );

                        let mut effective_container_attrs = attrs.clone();
                        effective_container_attrs.merge_variant(&variant_attrs);

                        // Parse derive info for the content of the variants
                        let data = match variant.fields {
                        syn::Fields::Named(fields) => {
                            VariantData::Named(NamedFieldsInfo::parse(fields, &effective_container_attrs, &err_ty)?)
                        }
                        syn::Fields::Unnamed(u) => return Err(syn::Error::new(
                        u.span(),
                        "Enum variants with unnamed associated data aren't supported by the DeserializeFromValue derive macro.",
                    )),
                        syn::Fields::Unit => VariantData::Unit,
                    };
                        parsed_variants.push(VariantInfo {
                            ident: variant.ident,
                            key_name,
                            data,
                        });
                    }
                    TraitImplementationInfo::Enum {
                        tag: attrs.tag,
                        variants: parsed_variants,
                    }
                }
                Data::Union(u) => {
                    return Err(syn::Error::new(
                        u.union_token.span,
                        "Unions aren't supported by the DeserializeFromValue derive macro",
                    ))
                }
            }
        };

        // Create the token stream representing the line:
        // ```
        //  impl<generics: bounds> DeserializeFromValue<err_ty> for MyType<generics>
        //      where where_clause, generics: DeserializeFromValue<err_ty>
        // ```
        // The generics and where clause are given by the original generics and where clause of the derived type,
        // with the additional requirement that each generic parameter implements `DeserializeFromValue<err_ty>`
        let impl_trait_tokens = {
            // The goal of creating these simple bindings is to be able to reference them in a quote! macro
            let ident = input.ident;

            // append the additional clause to the existing where clause
            let mut new_predicates = input
                .generics
                .type_params()
                .map::<WherePredicate, _>(|param| {
                    let param = &param.ident;
                    parse_quote!(#param : ::deserr::DeserializeFromValue<#err_ty>)
                })
                .collect::<Vec<_>>();

            let mut generics_for_trait_impl = input.generics.clone();

            if user_provided_err_ty.is_none() {
                generics_for_trait_impl.params.push(parse_quote!(#err_ty));
                new_predicates.push(parse_quote!(
                    #err_ty : ::deserr::DeserializeError
                ));
            }
            match &data {
                TraitImplementationInfo::Struct(NamedFieldsInfo {
                    field_from_errors, ..
                }) => {
                    for field_from_error in field_from_errors.iter().flatten() {
                        new_predicates.push(parse_quote!(
                            #err_ty : ::deserr::MergeWithError<#field_from_error>
                        ))
                    }
                }
                TraitImplementationInfo::Enum { variants, .. } => {
                    for variant in variants {
                        match &variant.data {
                            VariantData::Unit => continue,
                            VariantData::Named(variant_info) => {
                                for field_from_error in
                                    variant_info.field_from_errors.iter().flatten()
                                {
                                    new_predicates.push(parse_quote!(
                                        #err_ty : ::deserr::MergeWithError<#field_from_error>
                                    ));
                                }
                            }
                        }
                    }
                }
                TraitImplementationInfo::UserProvidedFunction { .. } => {}
            }

            // Add MergeWithError<FromFunctionError> requirement
            if let Some(from) = &attrs.from {
                let from_error = &from.function.error_ty;
                new_predicates.push(parse_quote!(
                    #err_ty : ::deserr::MergeWithError<#from_error>
                ));
            }
            // Add MergeWithError<ValidateFunctionError> requirement
            if let Some(validate) = &attrs.validate {
                let validate_error = &validate.error_ty;
                new_predicates.push(parse_quote!(
                    #err_ty : ::deserr::MergeWithError<#validate_error>
                ));
            }

            // Add FieldTy: DeserializeFromValue<ErrTy> for each field with the needs_predicate attribute
            {
                let collect_needs_pred = |fields: &NamedFieldsInfo| {
                    fields
                        .field_tys
                        .iter()
                        .zip(fields.needs_predicate.iter())
                        .filter_map(|(ty, pred)| if *pred { Some(ty.clone()) } else { None })
                        .collect::<Vec<_>>()
                };
                let all_fields_needing_pred = match &data {
                    TraitImplementationInfo::Struct(fields) => collect_needs_pred(fields),
                    TraitImplementationInfo::Enum { variants, .. } => variants
                        .iter()
                        .flat_map(|v| match &v.data {
                            VariantData::Named(fields) => collect_needs_pred(fields),
                            _ => vec![],
                        })
                        .collect(),
                    TraitImplementationInfo::UserProvidedFunction { .. } => {
                        vec![]
                    }
                };
                for field_ty in all_fields_needing_pred {
                    new_predicates.push(parse_quote! {
                        #field_ty : ::deserr::DeserializeFromValue<#err_ty>
                    });
                }
            }

            generics_for_trait_impl
                .params
                .extend(attrs.generic_params.clone());

            let mut generics = input.generics.clone();

            // existing generics/where clause
            let (_, ty_generics, ..) = input.generics.split_for_impl();
            let (impl_generics, ..) = generics_for_trait_impl.split_for_impl();

            generics
                .make_where_clause()
                .predicates
                .extend(new_predicates);

            let mut bounded_where_clause = generics.where_clause.unwrap();
            bounded_where_clause
                .predicates
                .extend(attrs.where_predicates.clone());

            quote! {
                impl #impl_generics ::deserr::DeserializeFromValue<#err_ty> for #ident #ty_generics #bounded_where_clause
            }
        };

        let validate = if let Some(validate_func) = attrs.validate {
            let FunctionReturningError {
                function: validate_func,
                error_ty: func_error_type,
            } = validate_func;
            quote! {
                #validate_func (deserr_final__, deserr_location__) .map_err(|validate_error__|{
                    ::deserr::take_result_content(
                        <#err_ty as ::deserr::MergeWithError<#func_error_type>>::merge(
                            None,
                            validate_error__,
                            deserr_location__
                        )
                    )
                })
            }
        } else {
            quote! { Ok(deserr_final__) }
        };

        Ok(Self {
            common: CommonDerivedTypeInfo {
                impl_trait_tokens,
                err_ty,
                validate,
            },
            data,
        })
    }
}

/// Contains the information needed to generate the deserialization code
/// for named fields, whether they reside in a struct or an enum variant.
///
/// Note that each field in this structure is a vector. All the vectors have the
/// same length, corresponding to the number of fields. So this structure is essentially
/// the same as a hypothetical `Vec<NamedFieldInfo>`.
///
/// The reason it is designed in this way is to be able to `quote!` its content easily.
/// For example:
/// ```ignore
/// let NamedFieldsInfo { field_names, field_tys, .. } = named_fields;
/// quote! {
///     struct S { #( #field_names : #field_tys )* }
/// }
/// ```
#[derive(Debug)]
pub struct NamedFieldsInfo {
    pub field_names: Vec<syn::Ident>,
    pub field_tys: Vec<syn::Type>,
    pub field_defaults: Vec<TokenStream>,
    pub field_errs: Vec<syn::Type>,

    pub field_from_fns: Vec<Option<TokenStream>>,
    pub field_from_errors: Vec<Option<syn::Type>>,

    pub field_maps: Vec<TokenStream>,
    pub missing_field_errors: Vec<TokenStream>,
    pub key_names: Vec<String>,

    pub needs_predicate: Vec<bool>,
    /// A token stream representing the code to handle an unknown field key.
    ///
    /// It is relevant to the `deny_unknown_fields` attribute.
    pub unknown_key: TokenStream,
}

impl NamedFieldsInfo {
    fn parse(
        fields: syn::FieldsNamed,
        data_attrs: &ContainerAttributesInfo,
        err_ty: &syn::Type,
    ) -> syn::Result<Self> {
        // the identifier of the field
        let mut field_names = vec![];
        // the type of the field or the type of the `from` if there was one
        let mut field_tys = vec![];
        // the key (in the serialised value) corresponding to the field
        // influenced by the `rename` and `rename_all` attributes
        let mut key_names = vec![];
        // the token stream that give the optional value of the field when its key is missing
        // influenced by the `default` attribute
        let mut field_defaults = vec![];
        // the type of the error used to deserialize the field
        let mut field_errs = vec![];
        // the token stream representing the error to return when the field is missing and has no default value
        let mut missing_field_errors = vec![];
        // an Option of token stream which maps the deserialised field value from one type to another
        let mut field_from_fns = vec![];
        // The list of error types that can be returned by the `from` clauses
        let mut field_from_errors = vec![];
        // the token stream which maps the deserialised field value
        let mut field_maps = vec![];
        // `true` iff the field has the needs_predicate attribute
        let mut needs_predicate = vec![];

        let mut fields_extra = fields
            .named
            .into_iter()
            .map(|field| {
                let attrs = read_deserr_field_attributes(&field.attrs)?;
                Ok((field, attrs))
            })
            .collect::<Result<Vec<_>, syn::Error>>()?;

        // We put all the non-skipped fields at the beginning, so that when we iterate
        // over the non-skipped key names, we can access their corresponding field names
        // using the same index.
        fields_extra.sort_by_key(|x| x.1.skipped);

        for (field, attrs) in fields_extra.iter() {
            let field_name = field.ident.clone().unwrap();
            let field_ty = &field.ty;

            let field_default = if let Some(default) = &attrs.default {
                match default {
                    // #[deserr(default)] => use the Default trait
                    DefaultFieldAttribute::DefaultTrait => {
                        quote! { ::deserr::FieldState::Some(::std::default::Default::default()) }
                    }
                    // #[deserr(default = expr)] => use the given expression
                    DefaultFieldAttribute::Function(expr) => {
                        quote! { ::deserr::FieldState::Some(#expr) }
                    }
                }
            } else if attrs.skipped {
                quote! { ::deserr::FieldState::Some(::std::default::Default::default()) }
            } else {
                quote! { ::deserr::FieldState::Missing }
            };

            let field_ty = match attrs.from {
                Some(ref from) => from.from_ty.clone(),
                None => field_ty.clone(),
            };

            let field_map = match &attrs.map {
                Some(func) => {
                    quote! {
                        #func
                    }
                }
                None => {
                    quote! { ::std::convert::identity }
                }
            };

            field_names.push(field_name);
            field_tys.push(field_ty.clone());
            field_defaults.push(field_default);
            field_maps.push(field_map);
            needs_predicate.push(attrs.needs_predicate);
        }

        for (field, attrs) in fields_extra.into_iter().filter(|x| !x.1.skipped) {
            let field_ty = &field.ty;
            let field_name = field.ident.clone().unwrap();

            let renamed = attrs.rename.as_ref().map(|i| i.value());
            let key_name = key_name_for_ident(
                field_name.to_string(),
                data_attrs.rename_all.as_ref(),
                renamed.as_deref(),
            );
            let error = match attrs.error {
                Some(error) => error,
                None => data_attrs
                    .err_ty
                    .clone()
                    .unwrap_or_else(|| parse_quote!(__Deserr_E)),
            };

            let field_ty = match attrs.from {
                Some(ref from) => from.from_ty.clone(),
                None => field_ty.clone(),
            };

            let field_from_fn = match attrs.from {
                Some(ref from) => {
                    let fun = &from.function.function;
                    if from.is_ref {
                        Some(quote! { |val: #field_ty | #fun(&val) })
                    } else {
                        Some(quote! { #fun })
                    }
                }
                None => None,
            };

            let field_from_error = attrs
                .from
                .as_ref()
                .map(|from| from.function.error_ty.clone());

            let missing_field_error = match &attrs.missing_field_error {
                Some(error_function) => {
                    quote! {
                        let deserr_e__ = #error_function ( #key_name, deserr_location__ ) ;
                        deserr_error__ = ::std::option::Option::Some(<#err_ty as ::deserr::MergeWithError<_>>::merge(
                            deserr_error__,
                            deserr_e__,
                            deserr_location__
                        )?);
                    }
                }
                None => {
                    quote! {
                        deserr_error__ = ::std::option::Option::Some(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                            deserr_error__,
                            ::deserr::ErrorKind::MissingField {
                                field: #key_name,
                            },
                            deserr_location__
                        )?);
                    }
                }
            };

            key_names.push(key_name.clone());
            field_errs.push(error);
            field_from_fns.push(field_from_fn);
            field_from_errors.push(field_from_error);
            missing_field_errors.push(missing_field_error);
        }

        // Create the token stream representing the code to handle an unknown field key.
        // By default, we ignore unknown keys, so the token stream is empty.
        //
        // If the #[deserr(deny_unknown_fields)] or #[deserr(deny_unknown_fields = func)] attribute exists,
        // we return an error: either the default error, or an error created by the custom function given by
        // the user.
        let unknown_key = match &data_attrs.deny_unknown_fields {
            Some(DenyUnknownFields::DefaultError) => {
                // Here we must give as argument the accepted keys
                quote! {
                    deserr_error__ = ::std::option::Option::Some(<#err_ty as ::deserr::DeserializeError>::error::<V>(
                        deserr_error__,
                        ::deserr::ErrorKind::UnknownKey {
                            key: deserr_key__,
                            accepted: &[#(#key_names),*],
                        },
                        deserr_location__
                    )?);
                }
            }
            Some(DenyUnknownFields::Function(func)) => quote! {
                let deserr_e__ = #func (deserr_key__, &[#(#key_names),*], deserr_location__) ;
                deserr_error__ = ::std::option::Option::Some(<#err_ty as ::deserr::MergeWithError<_>>::merge(
                    deserr_error__,
                    deserr_e__,
                    deserr_location__,
                )?);
            },
            None => quote! {},
        };

        Ok(Self {
            field_names,
            field_tys,
            key_names,
            field_defaults,
            field_errs,
            field_from_fns,
            field_from_errors,
            field_maps,
            needs_predicate,
            missing_field_errors,
            unknown_key,
        })
    }
}

/// Transforms the given `ident` string according to the rules of the `rename` and `rename_all` attributes
fn key_name_for_ident(
    ident: String,
    rename_all: Option<&RenameAll>,
    rename: Option<&str>,
) -> String {
    match rename {
        Some(name) => name.to_string(),
        None => match rename_all {
            Some(RenameAll::CamelCase) => ident.to_case(Case::Camel),
            Some(RenameAll::LowerCase) => ident.to_lowercase(),
            None => ident,
        },
    }
}
