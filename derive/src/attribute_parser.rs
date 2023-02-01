use proc_macro2::{Ident, Span};
use syn::{
    parenthesized,
    parse::{ParseBuffer, ParseStream},
    parse2, Attribute, DeriveInput, Expr, ExprPath, GenericParam, LitStr, Token, WherePredicate,
};

// pub struct MapFieldAttribute {
//     // from_ty: Option<syn::Type>,
//     map_func: syn::ExprPath,
// }

/// Attributes that are applied to fields.
#[derive(Default, Debug, Clone)]
pub struct FieldAttributesInfo {
    /// Whether the key corresponding to the field should be renamed to something different
    /// than the identifier of the field.
    pub rename: Option<LitStr>,
    /// The default value to deserialise to when the field is missing.
    pub default: Option<DefaultFieldAttribute>,
    /// The error to return when the field is missing and no default value exists.
    pub missing_field_error: Option<ExprPath>,
    /// The type of the error used to deserialize the field
    pub error: Option<syn::Type>,
    /// The function to apply to the result after it has been deserialised successfully
    pub map: Option<syn::ExprPath>,
    /// The function used to deserialize the whole type
    pub from: Option<AttributeFrom>,
    /// The function used to deserialize the whole type
    pub try_from: Option<AttributeTryFrom>,
    /// Whether an additional where clause should be added to deserialize this field
    pub needs_predicate: bool,
    /// Whether the field should be skipped
    pub skipped: bool,

    /// Span of the `default` attribute, if any, for compile error reporting purposes
    default_span: Option<Span>,
}

/// The value of the `default` field attribute
#[derive(Debug, Clone)]
pub enum DefaultFieldAttribute {
    /// `#[deserr(default)]`
    ///
    /// The default value is given by the type's `std::default::Default` implementation
    DefaultTrait,
    /// `#[deserr(default = expression)]`
    ///
    /// The default value is given by the given expression
    Function(Expr),
}

impl FieldAttributesInfo {
    /// Merges the other field attributes into `self`.
    ///
    /// This is used to transform a list of attributes, such as the following:
    /// ```ignore
    /// #[deserr(rename = "a")]
    /// #[deserr(default)]
    /// ```
    /// into a single equivalent attribute:
    /// ```ignore
    /// #[deserr(rename = "a", default)]
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
        if let Some(error) = other.error {
            if let Some(self_error) = &self.error {
                return Err(syn::Error::new_spanned(
                    self_error,
                    "The `error` field attribute is defined twice.",
                ));
            }
            self.error = Some(error)
        }
        if let Some(map) = other.map {
            if let Some(self_map) = &self.map {
                return Err(syn::Error::new_spanned(
                    self_map,
                    "The `map` field attribute is defined twice.",
                ));
            }
            self.map = Some(map)
        }
        if let Some(from) = other.from {
            if let Some(_self_from) = &self.from {
                return Err(syn::Error::new(
                    from.span,
                    "The `from` field attribute is defined twice.",
                ));
            } else if let Some(self_try_from) = &self.try_from {
                return Err(syn::Error::new(
                    self_try_from.span,
                    "The `from` and `try_from` attributes can't be used together.",
                ));
            }
            self.from = Some(from)
        }
        if let Some(try_from) = other.try_from {
            if let Some(_self_from) = &self.try_from {
                return Err(syn::Error::new(
                    try_from.span,
                    "The `try_from` field attribute is defined twice.",
                ));
            } else if let Some(self_from) = &self.from {
                return Err(syn::Error::new(
                    self_from.span,
                    "The `try_from` and `from` attributes can't be used together.",
                ));
            }
            self.try_from = Some(try_from)
        }
        self.needs_predicate |= other.needs_predicate;
        self.skipped |= other.skipped;

        Ok(())
    }
}
fn parse_rename(input: &ParseBuffer) -> Result<LitStr, syn::Error> {
    let _eq = input.parse::<Token![=]>()?;
    let ident = input.parse::<LitStr>()?;
    // #[deserr( ... rename = ident )]
    Ok(ident)
}

impl syn::parse::Parse for FieldAttributesInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = FieldAttributesInfo::default();
        // parse starting right after #[deserr .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[deserr( .... )]

        loop {
            let mut other = FieldAttributesInfo::default();
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[deserr( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename" => {
                    other.rename = Some(parse_rename(&input)?);
                }
                "default" => {
                    if input.peek(Token![=]) {
                        let _eq = input.parse::<Token![=]>()?;
                        let expr = input.parse::<Expr>()?;
                        // #[deserr( ... default = expr )]
                        other.default = Some(DefaultFieldAttribute::Function(expr));
                    } else {
                        other.default = Some(DefaultFieldAttribute::DefaultTrait);
                    }
                    other.default_span = Some(attr_name.span());
                }
                "missing_field_error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let func = input.parse::<ExprPath>()?;
                    // #[deserr( ... missing_field_error = func )]
                    other.missing_field_error = Some(func);
                }
                "needs_predicate" => other.needs_predicate = true,
                "error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let err_ty = input.parse::<syn::Type>()?;
                    // #[deserr( ... error = err_ty )]
                    other.error = Some(err_ty);
                }
                "map" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let func = input.parse::<syn::ExprPath>()?;
                    // #[deserr( ... map = func )]
                    other.map = Some(func);
                }
                "from" => {
                    let from_attr = parse_attribute_from(attr_name.span(), &input)?;
                    // #[deserr( .. from(from_ty) = function::path::<_>)]
                    other.from = Some(from_attr);
                }
                "try_from" => {
                    let try_from_attr = parse_attribute_try_from(attr_name.span(), &input)?;
                    // #[deserr( .. try_from(from_ty) = function::path::<_> -> to_ty )]
                    other.try_from = Some(try_from_attr);
                }
                "skip" => {
                    other.skipped = true;
                }
                _ => {
                    let message = format!("Unknown deserr field attribute: {}", attr_name);
                    return Result::Err(syn::Error::new_spanned(attr_name, message));
                }
            }
            this.merge(other)?;

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
pub fn read_deserr_field_attributes(
    attributes: &[Attribute],
) -> Result<FieldAttributesInfo, syn::Error> {
    let mut this = FieldAttributesInfo::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "deserr" {
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
    /// `#[deserr(rename_all = camelCase)]`
    CamelCase,
    /// `#[deserr(rename_all = lowercase)]`
    LowerCase,
}

/// The value of the `tag` field attribute
#[derive(Debug, Clone)]
pub enum TagType {
    /// `#[deserr(tag = "somestring")]`
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
    /// `#[deserr(deny_unknown_fields)]`
    ///
    /// Unknown fields should return a default error.
    DefaultError,
    /// `#[deserr(deny_unknown_fields = func)]`
    ///
    /// Unknown fields return an error defined by the function of type `Fn(&str) -> CustomError`.
    /// The input to the function is the name of the unknown field.
    Function(syn::ExprPath),
}

#[derive(Debug, Clone)]
pub struct AttributeTryFrom {
    pub is_ref: bool,
    pub try_from_ty: syn::Type,
    pub function: FunctionReturningError,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct AttributeFrom {
    pub is_ref: bool,
    pub from_ty: syn::Type,
    pub function: ExprPath,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionReturningError {
    pub function: ExprPath,
    pub error_ty: syn::Type,
}

/// Attributes that are applied to containers
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

    /// The function used to deserialize the whole container
    pub from: Option<AttributeFrom>,
    /// The function used to deserialize the whole container
    pub try_from: Option<AttributeTryFrom>,

    /// A function to call on the deserialized value to validate it
    pub validate: Option<FunctionReturningError>,

    validate_span: Option<Span>,
    rename_all_span: Option<Span>,
    tag_span: Option<Span>,
    deny_unknown_fields_span: Option<Span>,
}

impl ContainerAttributesInfo {
    /// Merges the other data attributes into `self`.
    ///
    /// This is used to transform a list of attributes, such as the following:
    /// ```ignore
    /// #[deserr(rename_all = lowercase)]
    /// #[deserr(error = MyError)]
    /// ```
    /// into a single equivalent attribute:
    /// ```ignore
    /// #[deserr(rename_all = lowercase, error = MyError)]
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
        if let Some(x) = other.from {
            if let Some(self_from) = &self.from {
                return Err(syn::Error::new(
                    self_from.span,
                    "The `from` attribute is defined twice.",
                ));
            } else if let Some(self_try_from) = &self.try_from {
                return Err(syn::Error::new(
                    self_try_from.span,
                    "The `from` and `try_from` attributes can't be used together.",
                ));
            }
            self.from = Some(x);
        }
        if let Some(x) = other.try_from {
            if let Some(try_self_from) = &self.try_from {
                return Err(syn::Error::new(
                    try_self_from.span,
                    "The `try_from` attribute is defined twice.",
                ));
            } else if let Some(self_from) = &self.from {
                return Err(syn::Error::new(
                    self_from.span,
                    "The `try_from` and `from` attributes can't be used together.",
                ));
            }
            self.try_from = Some(x);
        }
        if let Some(x) = other.validate {
            if let Some(self_validate_span) = &self.validate_span {
                return Err(syn::Error::new(
                    *self_validate_span,
                    "The `validate` attribute is defined twice.",
                ));
            }
            self.validate = Some(x);
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
    // #[deserr( ... rename_all = ident )]
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

fn parse_function_returning_error(
    input: &ParseBuffer,
) -> Result<FunctionReturningError, syn::Error> {
    let function = input.parse::<ExprPath>()?;
    // #[deserr( .. from(from_ty) = function::path::<_> )]
    let _arrow = input.parse::<Token![->]>()?;
    let error_ty = input.parse::<syn::Type>()?;
    Ok(FunctionReturningError { function, error_ty })
}

fn parse_attribute_from(span: Span, input: &ParseBuffer) -> Result<AttributeFrom, syn::Error> {
    let content;
    let _ = parenthesized!(content in input);
    // #[deserr( .. from(..) ..)]
    let is_ref = content.parse::<Token![&]>().is_ok();

    let from_ty = content.parse::<syn::Type>()?;
    // #[deserr( .. from(from_ty) ..)]
    let _eq = input.parse::<Token![=]>()?;
    // #[deserr( .. from(from_ty) = ..)]
    let function = input.parse::<ExprPath>()?;

    Ok(AttributeFrom {
        is_ref,
        from_ty,
        function,
        span,
    })
}

fn parse_attribute_try_from(
    span: Span,
    input: &ParseBuffer,
) -> Result<AttributeTryFrom, syn::Error> {
    let content;
    let _ = parenthesized!(content in input);
    // #[deserr( .. try_from(..) ..)]
    let is_ref = content.parse::<Token![&]>().is_ok();

    let from_ty = content.parse::<syn::Type>()?;
    // #[deserr( .. try_from(from_ty) ..)]
    let _eq = input.parse::<Token![=]>()?;
    // #[deserr( .. try_from(from_ty) = ..)]
    let function = parse_function_returning_error(input)?;

    Ok(AttributeTryFrom {
        is_ref,
        try_from_ty: from_ty,
        function,
        span,
    })
}

impl syn::parse::Parse for ContainerAttributesInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = ContainerAttributesInfo::default();
        // parse starting right after #[deserr .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[deserr( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[deserr( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename_all" => {
                    let rename_all = parse_rename_all(&input)?;
                    this.rename_all = Some(rename_all);
                    this.rename_all_span = Some(attr_name.span());
                }
                "tag" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let lit = input.parse::<LitStr>()?;
                    // #[deserr( ... tag = "lit" )]
                    this.tag = TagType::Internal(lit.value());
                    this.tag_span = Some(attr_name.span());
                }
                "error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let err_ty = input.parse::<syn::Type>()?;
                    // #[deserr( ... error = err_ty )]
                    this.err_ty = Some(err_ty);
                }
                "deny_unknown_fields" => {
                    if input.peek(Token![=]) {
                        let _eq = input.parse::<Token![=]>()?;
                        let func = input.parse::<ExprPath>()?;
                        // #[deserr( ... deny_unknown_fields = func )]
                        this.deny_unknown_fields = Some(DenyUnknownFields::Function(func));
                    } else {
                        this.deny_unknown_fields = Some(DenyUnknownFields::DefaultError);
                    }
                    this.deny_unknown_fields_span = Some(attr_name.span());
                }
                "from" => {
                    let from_attr = parse_attribute_from(attr_name.span(), &input)?;
                    // #[deserr( .. from(from_ty) = function::path::<_>)]
                    this.from = Some(from_attr);
                }
                "try_from" => {
                    let try_from_attr = parse_attribute_try_from(attr_name.span(), &input)?;
                    // #[deserr( .. try_from(from_ty) = function::path::<_> -> to_ty )]
                    this.try_from = Some(try_from_attr);
                }
                "validate" => {
                    // #[deserr( ... validate .. )]
                    let _eq = input.parse::<Token![=]>()?;
                    // #[deserr( ... validate = .. )]
                    let validate_func = parse_function_returning_error(&input)?;
                    // #[deserr( ... validate = some::func<T> )]
                    this.validate = Some(validate_func);
                }
                "generic_param" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let param = input.parse::<GenericParam>()?;
                    // #[deserr( ... generic_params = P )]
                    this.generic_params.push(param);
                }
                "where_predicate" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let pred = input.parse::<WherePredicate>()?;
                    // #[deserr( ... where_predicate = P: Display + Debug )]
                    this.where_predicates.push(pred);
                }
                _ => {
                    let message = format!("Unknown deserr container attribute: {}", attr_name);
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

pub fn validate_container_attributes(
    attributes: &ContainerAttributesInfo,
    container: &DeriveInput,
) -> Result<(), syn::Error> {
    if attributes.try_from.is_some() {
        if let Some(rename_all_span) = attributes.rename_all_span {
            return Err(syn::Error::new(
                rename_all_span,
                "Cannot use the `rename_all` attribute together with the `try_from` attribute",
            ));
        }
        if let Some(tag) = attributes.tag_span {
            return Err(syn::Error::new(
                tag,
                "Cannot use the `tag` attribute together with the `try_from` attribute",
            ));
        }
        if let Some(span) = attributes.deny_unknown_fields_span {
            return Err(syn::Error::new(
                span,
                "Cannot use the `deny_unknown_fields` attribute together with the `try_from` attribute",
            ));
        }
    }
    if matches!(container.data, syn::Data::Struct(..)) {
        if let Some(tag) = attributes.tag_span {
            return Err(syn::Error::new(
                tag,
                "Cannot use the `tag` attribute on structs",
            ));
        }
    }
    Ok(())
}

/// Parses an array of `syn::Attribute` into a single `FieldAttributesInfo` containing the
/// relevant information for the generation of the code deserialising each field.
pub fn read_deserr_container_attributes(
    attributes: &[Attribute],
) -> Result<ContainerAttributesInfo, syn::Error> {
    let mut this = ContainerAttributesInfo::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "deserr" {
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
///     #[deserr(rename = "Apple")]
///     A
///     #[deserr(rename = "Pear", rename_all = camelCase)]
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
    /// #[deserr(rename_all = lowercase)]
    /// #[deserr(error = MyError)]
    /// ```
    /// into a single equivalent attribute:
    /// ```ignore
    /// #[deserr(rename_all = lowercase, error = MyError)]
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
        // parse starting right after #[deserr .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[deserr( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[deserr( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename" => {
                    this.rename = Some(parse_rename(&input)?);
                }
                "rename_all" => {
                    this.rename_all = Some(parse_rename_all(&input)?);
                    this.rename_all_span = Some(attr_name.span());
                }
                _ => {
                    let message = format!("Unknown deserr variant attribute: {}", attr_name);
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
pub fn read_deserr_variant_attributes(
    attributes: &[Attribute],
) -> Result<VariantAttributesInfo, syn::Error> {
    let mut this = VariantAttributesInfo::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "deserr" {
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
