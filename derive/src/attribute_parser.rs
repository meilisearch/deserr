use proc_macro2::{Ident, Span};
use syn::{
    parenthesized,
    parse::{ParseBuffer, ParseStream},
    parse2, Attribute, Expr, ExprPath, GenericParam, LitStr, Token, WherePredicate,
};

/// Attributes that are applied to fields.
///
/// There are currently three supported field attributes: `rename`, `default`, and `missing_field_error`.
/// For example:
/// ```ignore
/// struct X {
///     #[jayson(rename = "apple", default, missing_field_error = MissingFruitError::Apple)]
///     fruit: Apple,
///     #[jayson(rename = "key")]
///     #[jayson(default = get_key().to_string())]
///     b: Key
/// }
/// ```
#[derive(Default, Debug, Clone)]
pub struct FieldAttributesInfo {
    /// Whether the key corresponding to the field should be renamed to something different
    /// than the identifier of the field.
    pub rename: Option<LitStr>,
    /// The default value to deserialise to when the field is missing.
    pub default: Option<DefaultFieldAttribute>,
    /// The error to return when the field is missing and no default value exists.
    pub missing_field_error: Option<Expr>,

    /// Span of the `default` attribute, if any, for compile error reporting purposes
    default_span: Option<Span>,
}

/// The value of the `default` field attribute
#[derive(Debug, Clone)]
pub enum DefaultFieldAttribute {
    /// `#[jayson(default)]`
    ///
    /// The default value is given by the type's `std::default::Default` implementation
    DefaultTrait,
    /// `#[jayson(default = expression)]`
    ///
    /// The default value is given by the given expression
    Function(Expr),
}

impl FieldAttributesInfo {
    /// Merges the other field attributes into `self`.
    ///
    /// This is used to transform a list of attributes, such as the following:
    /// ```ignore
    /// #[jayson(rename = "a")]
    /// #[jayson(default)]
    /// ```
    /// into a single equivalent attribute:
    /// ```ignore
    /// #[jayson(rename = "a", default)]
    /// ```
    ///
    /// An error is returned iff the same attribute is defined twice.
    fn merge(&mut self, other: FieldAttributesInfo) -> Result<(), syn::Error> {
        if let Some(rename) = other.rename {
            if let Some(self_rename) = &self.rename {
                return Err(syn::Error::new_spanned(
                    self_rename,
                    "The `rename` field attribute is defined twice.",
                ));
            }
            self.rename = Some(rename)
        }
        if let Some(default) = other.default {
            if let Some(self_default_span) = &self.default_span {
                return Err(syn::Error::new(
                    *self_default_span,
                    "The `default` field attribute is defined twice.",
                ));
            }
            self.default = Some(default)
        }
        if let Some(missing_field_error) = other.missing_field_error {
            if let Some(self_missing_field_error) = &self.missing_field_error {
                return Err(syn::Error::new_spanned(
                    self_missing_field_error,
                    "The `missing_field_error` field attribute is defined twice.",
                ));
            }
            self.missing_field_error = Some(missing_field_error)
        }

        Ok(())
    }
}
fn parse_rename(input: &ParseBuffer) -> Result<LitStr, syn::Error> {
    let _eq = input.parse::<Token![=]>()?;
    let ident = input.parse::<LitStr>()?;
    // #[jayson( ... rename = ident )]
    Ok(ident)
}

impl syn::parse::Parse for FieldAttributesInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = FieldAttributesInfo::default();
        // parse starting right after #[jayson .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[jayson( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[jayson( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename" => {
                    this.rename = Some(parse_rename(&input)?);
                }
                "default" => {
                    if input.peek(Token![=]) {
                        let _eq = input.parse::<Token![=]>()?;
                        let expr = input.parse::<Expr>()?;
                        // #[jayson( ... default = expr )]
                        this.default = Some(DefaultFieldAttribute::Function(expr));
                    } else {
                        this.default = Some(DefaultFieldAttribute::DefaultTrait);
                    }
                    this.default_span = Some(attr_name.span());
                }
                "missing_field_error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let expr = input.parse::<Expr>()?;
                    // #[jayson( ... missing_field_error = expr )]
                    this.missing_field_error = Some(expr);
                }
                _ => {
                    let message = format!("Unknown jayson field attribute: {}", attr_name);
                    return Result::Err(syn::Error::new_spanned(attr_name, message));
                }
            }

            if input.peek(Token![,]) {
                let _comma = input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
                continue;
            } else if input.is_empty() {
                break;
            } else {
                return Result::Err(syn::Error::new(input.span(), "Expected end of attribute"));
            }
        }
        Ok(this)
    }
}

/// Parses an array of `syn::Attribute` into a single `FieldAttributesInfo` containing the
/// relevant information for the generation of the code deserialising each field.
pub fn read_jayson_field_attributes(
    attributes: &[Attribute],
) -> Result<FieldAttributesInfo, syn::Error> {
    let mut this = FieldAttributesInfo::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "jayson" {
                continue;
            }
            let other = parse2::<FieldAttributesInfo>(attribute.tokens.clone())?;
            this.merge(other)?;
        } else {
            continue;
        }
    }
    Ok(this)
}

/// The value of the `default` field attribute
#[derive(Debug, Clone)]
pub enum RenameAll {
    /// `#[jayson(rename_all = camelCase)]`
    CamelCase,
    /// `#[jayson(rename_all = lowercase)]`
    LowerCase,
}
/// The value of the `tag` field attribute
#[derive(Debug, Clone)]
pub enum TagType {
    /// `#[jayson(tag = "somestring")]`
    Internal(String),
    /// An external tag is the default value, when there is no `tag` attribute.
    External,
}
impl Default for TagType {
    fn default() -> Self {
        Self::External
    }
}
/// The value of the `deny_unknown_fields` field attribute
#[derive(Debug, Clone)]
pub enum DenyUnknownFields {
    /// `#[jayson(deny_unknown_fields)]`
    ///
    /// Unknown fields should return a default error.
    DefaultError,
    /// `#[jayson(deny_unknown_fields = func)]`
    ///
    /// Unknown fields return an error defined by the function of type `Fn(&str) -> CustomError`.
    /// The input to the function is the name of the unknown field.
    Function(syn::ExprPath),
}

/// Attributes that are applied to containers
///
/// There are currently four supported container attributes: `error`, `tag`, `rename_all`, `deny_unknown_fields`.
/// For example:
/// ```ignore
/// #[jayson(error = MyError, tag = "sometag", rename_all = camelCase)]
/// #[jayson(deny_unknown_fields = some_func)]
/// enum X { ... }
/// ```
///
/// The `tag` attribute can only be applied to enums.
#[derive(Default, Debug, Clone)]
pub struct ContainerAttributesInfo {
    pub rename_all: Option<RenameAll>,
    pub err_ty: Option<syn::Type>,
    pub tag: TagType,
    pub deny_unknown_fields: Option<DenyUnknownFields>,

    pub generic_params: Vec<GenericParam>,
    pub where_predicates: Vec<WherePredicate>,

    rename_all_span: Option<Span>,
    tag_span: Option<Span>,
    deny_unknown_fields_span: Option<Span>,
}
impl ContainerAttributesInfo {
    /// Merges the other data attributes into `self`.
    ///
    /// This is used to transform a list of attributes, such as the following:
    /// ```ignore
    /// #[jayson(rename_all = lowercase)]
    /// #[jayson(error = MyError)]
    /// ```
    /// into a single equivalent attribute:
    /// ```ignore
    /// #[jayson(rename_all = lowercase, error = MyError)]
    /// ```
    ///
    /// An error is returned iff the same attribute is defined twice.
    fn merge(&mut self, other: Self) -> Result<(), syn::Error> {
        if let Some(rename_all) = other.rename_all {
            if let Some(self_rename_all_span) = self.rename_all_span {
                return Err(syn::Error::new(
                    self_rename_all_span,
                    "The `rename_all` attribute is defined twice.",
                ));
            }
            self.rename_all = Some(rename_all)
        }
        if let Some(err_ty) = other.err_ty {
            if let Some(self_err_ty) = &self.err_ty {
                return Err(syn::Error::new_spanned(
                    self_err_ty,
                    "The `error` attribute is defined twice.",
                ));
            }
            self.err_ty = Some(err_ty)
        }
        if let TagType::Internal(x) = other.tag {
            if let Some(self_tag_span) = self.tag_span {
                return Err(syn::Error::new(
                    self_tag_span,
                    "The `tag` attribute is defined twice.",
                ));
            }
            self.tag = TagType::Internal(x)
        }
        if let Some(x) = other.deny_unknown_fields {
            if let Some(self_deny_unknown_fields_span) = &self.deny_unknown_fields_span {
                return Err(syn::Error::new(
                    *self_deny_unknown_fields_span,
                    "The `deny_unknown_fields` attribute is defined twice.",
                ));
            }
            self.deny_unknown_fields = Some(x);
        }
        self.generic_params.extend(other.generic_params);
        self.where_predicates.extend(other.where_predicates);

        Ok(())
    }
    /// Merges the variant attributes into `self`.
    ///
    /// This is used to combine the container attributes of the whole enum
    /// with variant attributes to obtain the container attributes relevant
    /// to the fields inside the enum variant.
    pub fn merge_variant(&mut self, other: &VariantAttributesInfo) {
        self.rename_all = other.rename_all.clone();
    }
}
fn parse_rename_all(input: &ParseBuffer) -> Result<RenameAll, syn::Error> {
    let _eq = input.parse::<Token![=]>()?;
    let ident = input.parse::<Ident>()?;
    // #[jayson( ... rename_all = ident )]
    let rename_all = match ident.to_string().as_str() {
        "camelCase" => RenameAll::CamelCase,
        "lowercase" => RenameAll::LowerCase,
        _ => {
            return Result::Err(syn::Error::new_spanned(
                ident,
                "rename_all can either be equal to `camelCase` or `lowercase`",
            ));
        }
    };
    Ok(rename_all)
}

impl syn::parse::Parse for ContainerAttributesInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = ContainerAttributesInfo::default();
        // parse starting right after #[jayson .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[jayson( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[jayson( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename_all" => {
                    let rename_all = parse_rename_all(&input)?;
                    this.rename_all = Some(rename_all);
                    this.rename_all_span = Some(attr_name.span());
                }
                "tag" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let lit = input.parse::<LitStr>()?;
                    // #[jayson( ... tag = "lit" )]
                    this.tag = TagType::Internal(lit.value());
                    this.tag_span = Some(attr_name.span());
                }
                "error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let err_ty = input.parse::<syn::Type>()?;
                    // #[jayson( ... error = err_ty )]
                    this.err_ty = Some(err_ty);
                }
                "deny_unknown_fields" => {
                    if input.peek(Token![=]) {
                        let _eq = input.parse::<Token![=]>()?;
                        let func = input.parse::<ExprPath>()?;
                        // #[jayson( ... deny_unknown_fields = func )]
                        this.deny_unknown_fields = Some(DenyUnknownFields::Function(func));
                    } else {
                        this.deny_unknown_fields = Some(DenyUnknownFields::DefaultError);
                    }
                    this.deny_unknown_fields_span = Some(attr_name.span());
                }
                "generic_param" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let param = input.parse::<GenericParam>()?;
                    // #[jayson( ... generic_params = P )]
                    this.generic_params.push(param);
                }
                "where_predicate" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let pred = input.parse::<WherePredicate>()?;
                    // #[jayson( ... where_predicate = P: Display + Debug )]
                    this.where_predicates.push(pred);
                }
                _ => {
                    let message = format!("Unknown jayson container attribute: {}", attr_name);
                    return Result::Err(syn::Error::new_spanned(attr_name, message));
                }
            }

            if input.peek(Token![,]) {
                let _comma = input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
                continue;
            } else if input.is_empty() {
                break;
            } else {
                return Result::Err(syn::Error::new(input.span(), "Expected end of attribute"));
            }
        }
        Ok(this)
    }
}

/// Parses an array of `syn::Attribute` into a single `FieldAttributesInfo` containing the
/// relevant information for the generation of the code deserialising each field.
pub fn read_jayson_container_attributes(
    attributes: &[Attribute],
) -> Result<ContainerAttributesInfo, syn::Error> {
    let mut this = ContainerAttributesInfo::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "jayson" {
                continue;
            }
            let other = parse2::<ContainerAttributesInfo>(attribute.tokens.clone())?;
            this.merge(other)?;
        } else {
            continue;
        }
    }
    Ok(this)
}

/// Attributes that are applied to enum variants
///
/// There are currently two supported variant attributes: `rename` and `rename_all`.
/// For example:
/// ```ignore
/// enum X {
///     #[jayson(rename = "Apple")]
///     A
///     #[jayson(rename = "Pear", rename_all = camelCase)]
///     P { type_of_pear: PearType }
/// }
/// ```
#[derive(Default, Debug)]
pub struct VariantAttributesInfo {
    pub rename_all: Option<RenameAll>,
    pub rename: Option<LitStr>,
    rename_all_span: Option<Span>,
}
impl VariantAttributesInfo {
    /// Merges the other data attributes into `self`.
    ///
    /// This is used to transform a list of attributes, such as the following:
    /// ```ignore
    /// #[jayson(rename_all = lowercase)]
    /// #[jayson(error = MyError)]
    /// ```
    /// into a single equivalent attribute:
    /// ```ignore
    /// #[jayson(rename_all = lowercase, error = MyError)]
    /// ```
    ///
    /// An error is returned iff the same attribute is defined twice.
    fn merge(&mut self, other: Self) -> Result<(), syn::Error> {
        if let Some(rename_all) = other.rename_all {
            if let Some(self_rename_all_span) = self.rename_all_span {
                return Err(syn::Error::new(
                    self_rename_all_span,
                    "The `rename_all` attribute is defined twice.",
                ));
            }
            self.rename_all = Some(rename_all)
        }
        if let Some(rename) = other.rename {
            if let Some(self_rename) = &self.rename {
                return Err(syn::Error::new_spanned(
                    self_rename,
                    "The `rename` attribute is defined twice.",
                ));
            }
            self.rename = Some(rename)
        }

        Ok(())
    }
}
impl syn::parse::Parse for VariantAttributesInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = VariantAttributesInfo::default();
        // parse starting right after #[jayson .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[jayson( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[jayson( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename" => {
                    this.rename = Some(parse_rename(&input)?);
                }
                "rename_all" => {
                    this.rename_all = Some(parse_rename_all(&input)?);
                    this.rename_all_span = Some(attr_name.span());
                }
                _ => {
                    let message = format!("Unknown jayson variant attribute: {}", attr_name);
                    return Result::Err(syn::Error::new_spanned(attr_name, message));
                }
            }

            if input.peek(Token![,]) {
                let _comma = input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
                continue;
            } else if input.is_empty() {
                break;
            } else {
                return Result::Err(syn::Error::new(input.span(), "Expected end of attribute"));
            }
        }
        Ok(this)
    }
}

/// Parses an array of `syn::Attribute` into a single `FieldAttributesInfo` containing the
/// relevant information for the generation of the code deserialising each field.
pub fn read_jayson_variant_attributes(
    attributes: &[Attribute],
) -> Result<VariantAttributesInfo, syn::Error> {
    let mut this = VariantAttributesInfo::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "jayson" {
                continue;
            }
            let other = parse2::<VariantAttributesInfo>(attribute.tokens.clone())?;
            this.merge(other)?;
        } else {
            continue;
        }
    }
    Ok(this)
}
