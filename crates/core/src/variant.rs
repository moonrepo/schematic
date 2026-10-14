#[cfg(feature = "schema")]
use crate::args::SerdeTagFormat;
use crate::args::{
    NestedArg, PartialArg, SerdeContainerArgs, SerdeFieldArgs, SerdeRenameArg, VariantDefaultArg,
};
use crate::container::ContainerArgs;
use crate::utils::{
    ImplResult, format_case, get_meta_path, get_renamed_value, is_inheritable_attribute,
};
use crate::value::path_names_a_value;
use crate::variant_value::VariantValue;
use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::rc::Rc;
use syn::{
    Attribute, Expr, ExprLit, ExprPath, Fields, FieldsUnnamed, Ident, Index, Lit,
    Variant as NativeVariant,
};

// #[setting()], #[schema()], #[variant()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, attributes(setting, schema, variant))]
pub struct VariantArgs {
    pub default: Option<VariantDefaultArg>,
    pub exclude: bool,
    pub fallback: bool,
    pub merge: Option<ExprPath>,
    pub nested: Option<NestedArg>,
    pub null: bool,
    #[darling(multiple)]
    pub partial: Vec<PartialArg>,
    pub required: bool,
    pub transform: Option<ExprPath>,
    pub validate: Option<crate::args::ValidateArg>,

    // serde
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    pub untagged: bool,
}

/// Return the value of a boolean literal, which is all
/// the `default` of a unit variant accepts.
fn as_bool_literal(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) => Some(value.value),
        _ => None,
    }
}

#[derive(Debug)]
pub struct Variant {
    pub values: Vec<VariantValue>,

    // args
    pub args: VariantArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub fields: Fields,
}

impl Variant {
    pub fn new(
        variant: NativeVariant,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Variant {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&variant.attrs).unwrap();

        let variant = Self {
            attrs: variant.attrs,
            container_args,
            ident: variant.ident,
            serde_args,
            serde_container_args,
            values: match &variant.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|field| VariantValue::new(field.ty.clone(), args.nested.as_ref()))
                    .collect(),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .map(|field| VariantValue::new(field.ty.clone(), args.nested.as_ref()))
                    .collect(),
                Fields::Unit => vec![],
            },
            fields: variant.fields,
            args,
        };
        variant.validate_args();
        variant
    }

    fn validate_args(&self) {
        if self.is_required()
            && self
                .values
                .iter()
                .any(|value| !value.is_outer_option_wrapped())
        {
            panic!("Cannot use `required` with non-optional settings.");
        }

        #[allow(clippy::collapsible_else_if)]
        if self.is_unit_variant() {
            if self.args.merge.is_some() {
                panic!("Cannot use `merge` with unit variants.");
            }

            if self.args.nested.is_some() {
                panic!("Cannot use `nested` with unit variants.");
            }

            if self.args.required {
                panic!("Cannot use `required` with unit variants.");
            }

            if let Some(VariantDefaultArg::Value(expr)) = &self.args.default
                && as_bool_literal(expr).is_none()
            {
                panic!("Cannot use a `default` value with unit variants, as they hold no value.");
            }

            #[cfg(feature = "validate")]
            if self.args.validate.is_some() {
                panic!("Cannot use `validate` with unit variants.");
            }
        } else {
            if self.args.null {
                panic!("Can only use `null` with unit variants.");
            }

            if self.get_default_value().is_some() {
                if self.values.len() > 1 {
                    panic!(
                        "Can only use a `default` value with variants that hold a single value."
                    );
                }

                if self.is_nested() {
                    panic!("Cannot use a `default` value with `nested`.");
                }
            }
        }
    }

    /// Return an explicit `rename`, if one was provided. Casing is not
    /// applied, as a rename is used exactly as written.
    fn get_name_renamed(&self) -> Option<String> {
        get_renamed_value(self.args.rename.as_ref(), self.serde_args.rename.as_ref())
    }

    /// Return the name this variant serializes as. A container `rename_all`
    /// is applied here, and not only forwarded to serde, so that schemas,
    /// `settings()`, and validation paths describe the key that is actually
    /// accepted. There is no default case.
    pub fn get_name(&self) -> String {
        if let Some(name) = self.get_name_renamed() {
            return name;
        }

        let name = self.ident.to_string();

        match get_renamed_value(
            self.container_args.rename_all.as_ref(),
            self.serde_container_args.rename_all.as_ref(),
        ) {
            Some(format) => format_case(&format, &name, true),
            None => name,
        }
    }

    /// Whether this is the default variant, marked with `#[setting(default)]`,
    /// or the standard `#[default]`. A variant holding a value is also marked
    /// by providing that value, while a unit variant takes a boolean.
    pub fn is_default(&self) -> bool {
        let marked = match &self.args.default {
            Some(VariantDefaultArg::Marked) => true,
            Some(VariantDefaultArg::Value(expr)) => {
                !self.is_unit_variant() || as_bool_literal(expr).unwrap_or_default()
            }
            None => false,
        };

        marked
            || self
                .attrs
                .iter()
                .any(|attr| get_meta_path(&attr.meta).is_ident("default"))
    }

    /// The value this variant holds by default, from `#[setting(default = value)]`.
    pub fn get_default_value(&self) -> Option<&Expr> {
        match &self.args.default {
            Some(VariantDefaultArg::Value(expr)) if !self.is_unit_variant() => Some(expr),
            _ => None,
        }
    }

    /// Whether this variant absorbs any value that no other variant matches.
    pub fn is_fallback(&self) -> bool {
        self.args.fallback
    }

    /// Whether this variant is represented by its value alone, even when
    /// the enum is tagged.
    pub fn is_untagged(&self) -> bool {
        self.args.untagged || self.serde_args.untagged
    }

    /// Whether serde never deserializes into this variant.
    pub fn is_skipped_deserializing(&self) -> bool {
        self.args.skip
            || self.serde_args.skip
            || self.args.skip_deserializing
            || self.serde_args.skip_deserializing
    }

    /// Aliases can be provided by both attributes, so combine them.
    pub fn get_aliases(&self) -> Vec<&String> {
        let mut aliases = vec![];

        for alias in self.args.alias.iter().chain(self.serde_args.alias.iter()) {
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }

        aliases
    }

    /// Validate this variant within a `ConfigEnum`, which models a list of
    /// named string values, optionally widened by variants that hold a value.
    pub fn validate_config_enum(&self) {
        let kind = if self.is_fallback() {
            "fallback"
        } else if self.is_untagged() {
            "untagged"
        } else {
            if !self.is_unit_variant() {
                panic!(
                    "Only unit variants are supported, unless marked as `fallback` or `untagged`."
                );
            }

            return;
        };

        match &self.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {}
            Fields::Unnamed(_) => {
                panic!("Only 1 unnamed field is supported for `{kind}`.");
            }
            _ => {
                panic!("Only unnamed tuple variants are supported for `{kind}`.");
            }
        };
    }

    /// Whether this variant holds a value within a `ConfigEnum`, rather
    /// than being one of its named values.
    fn is_config_enum_value_variant(&self) -> bool {
        self.is_fallback() || self.is_untagged()
    }

    /// An expression that constructs this variant, for `ConfigEnum::variants`.
    pub fn impl_config_enum_value(&self) -> TokenStream {
        let name = &self.ident;

        if self.is_config_enum_value_variant() {
            quote! { Self::#name(Default::default()) }
        } else {
            quote! { Self::#name }
        }
    }

    /// A match arm that formats this variant back into its string value.
    /// Each arm writes for itself, so that a variant holding a generic type
    /// can go through `Display` rather than having to be a `&str`.
    pub fn impl_config_enum_display(&self) -> TokenStream {
        let name = &self.ident;

        if self.is_config_enum_value_variant() {
            return quote! { Self::#name(inner) => write!(f, "{inner}"), };
        }

        let value = self.get_name();

        quote! { Self::#name => f.write_str(#value), }
    }

    /// A match arm that parses a string into this named variant.
    pub fn impl_config_enum_from_str(&self) -> TokenStream {
        let name = &self.ident;
        let value = self.get_name();
        let aliases = self.get_aliases();

        quote! { #value #(| #aliases)* => Self::#name, }
    }

    /// An expression that converts a value no other variant matched into
    /// this fallback.
    pub fn impl_config_enum_fallback(&self, value: &Ident) -> TokenStream {
        let name = &self.ident;

        quote! {
            Self::#name(
                #value.try_into().map_err(|_| {
                    schematic::ConfigError::EnumInvalidFallback(#value.to_string())
                })?
            )
        }
    }

    /// A statement that attempts to parse the input into this untagged
    /// variant, returning it on success. The input is parsed as written,
    /// since `before_parse` only exists to normalize named values.
    pub fn impl_config_enum_parse_untagged(&self, input: &Ident) -> TokenStream {
        let name = &self.ident;

        let Fields::Unnamed(fields) = &self.fields else {
            unreachable!("Validated by `validate_config_enum`.");
        };

        let ty = &fields.unnamed[0].ty;

        quote! {
            if let Ok(inner) = <#ty as std::str::FromStr>::from_str(#input) {
                return Ok(Self::#name(inner));
            }
        }
    }

    pub fn is_excluded(&self) -> bool {
        #[cfg(feature = "schema")]
        {
            self.args.exclude
        }

        #[cfg(not(feature = "schema"))]
        {
            false
        }
    }

    pub fn is_nested(&self) -> bool {
        self.args
            .nested
            .as_ref()
            .is_some_and(|nested| nested.is_nested())
    }

    pub fn is_required(&self) -> bool {
        self.args.required
    }

    pub fn is_unit_variant(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get_partial_attributes(&self) -> Vec<TokenStream> {
        let mut attrs = vec![];

        // Serde attributes come first, so that they take precedence
        // over any provided by the user via `partial(serde(...))`
        let serde_args = self.get_partial_serde_attribute_args();

        if !serde_args.is_empty() {
            attrs.push(quote! { #[serde(#serde_args)] });
        }

        // Inherit non-schematic attributes from the variant,
        // like `doc`, `allow`, and `deprecated`
        for attr in &self.attrs {
            if is_inheritable_attribute(attr) {
                attrs.push(quote! { #attr });
            }
        }

        // Then apply any user-provided partial attributes, in the order
        // declared, as `partial(...)` may be repeated
        for partial in &self.args.partial {
            attrs.extend(partial.get_attributes());
        }

        attrs
    }

    pub fn get_partial_serde_attribute_args(&self) -> TokenStream {
        let mut meta = vec![];

        for alias in self.get_aliases() {
            meta.push(quote! { alias = #alias });
        }

        // Setting attributes take precedence over serde attributes
        if let Some(rename) = self
            .args
            .rename
            .as_ref()
            .or(self.serde_args.rename.as_ref())
            .filter(|rename| !rename.is_empty())
        {
            meta.push(rename.get_meta("rename"));
        }

        if self.args.skip || self.serde_args.skip {
            meta.push(quote! { skip });
        } else {
            if self.args.skip_serializing || self.serde_args.skip_serializing {
                meta.push(quote! { skip_serializing });
            }

            if self.args.skip_deserializing || self.serde_args.skip_deserializing {
                meta.push(quote! { skip_deserializing });
            }
        }

        if self.is_untagged() {
            meta.push(quote! { untagged });
        }

        if self.serde_args.other {
            meta.push(quote! { other });
        }

        quote! {
            #(#meta),*
        }
    }

    /// Generate the schema for this variant, shaped by how the
    /// enum is tagged.
    #[cfg(feature = "schema")]
    pub fn impl_schema_type(&self, tag_format: &SerdeTagFormat) -> TokenStream {
        use crate::utils::{extract_comment, extract_deprecated, impl_literal_value};

        let name = self.get_name();
        let nested = self.is_nested();

        // Untagged variants are represented by their value alone
        let untagged = matches!(tag_format, SerdeTagFormat::Untagged) || self.is_untagged();

        let inner = match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(fields) => {
                let items = fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        let ty = &field.ty;

                        if nested {
                            return quote! { schema.infer_as_nested::<#ty>() };
                        }

                        // Literal defaults are rendered within the schema, and
                        // only a variant holding a single value has one
                        if let Some(Expr::Lit(lit)) = self.get_default_value()
                            && let Some(value) = impl_literal_value(&lit.lit)
                        {
                            return quote! { schema.infer_with_default::<#ty>(#value) };
                        }

                        quote! { schema.infer::<#ty>() }
                    })
                    .collect::<Vec<_>>();

                if items.len() == 1 {
                    let item = &items[0];

                    quote! { #item }
                } else {
                    quote! {
                        Schema::tuple(TupleType::new([
                            #(#items),*
                        ]))
                    }
                }
            }
            // Unit variants carry no value, so they're the name itself
            Fields::Unit => quote! { Schema::literal_value(LiteralValue::String(#name.into())) },
        };

        // Mark a nested value so that it resolves to its partial. Only the
        // value itself is marked, never a struct synthesized around it for
        // tagging, as that struct is not a partial config -- its tag is a
        // literal that is always present.
        let wrap = |value: TokenStream| {
            if nested {
                quote! {
                    {
                        let mut item = #value;
                        item.partialize();
                        item
                    }
                }
            } else {
                value
            }
        };
        let partial_inner = wrap(inner.clone());

        let unit = self.is_unit_variant();
        let name_literal = quote! { Schema::literal_value(LiteralValue::String(#name.into())) };

        let mut schema = if let SerdeTagFormat::Unit = tag_format {
            // Every variant is a unit, so the enum is a list of named values
            let ty = if self.args.null {
                quote! { Schema::null() }
            } else {
                inner
            };

            quote! {
                Schema {
                    name: Some(#name.into()),
                    ty: #ty.ty,
                    ..Default::default()
                }
            }
        } else if self.args.null {
            // Explicitly represents null, so it's never tagged
            quote! { Schema::null() }
        } else if untagged {
            // Untagged variants are represented by their value alone,
            // and units have no value
            if unit {
                quote! { Schema::null() }
            } else {
                inner
            }
        } else {
            match tag_format {
                SerdeTagFormat::Unit | SerdeTagFormat::Untagged => unreachable!(),
                // "name" | { "name": value }
                SerdeTagFormat::External => {
                    if unit {
                        inner
                    } else {
                        quote! {
                            Schema::structure(StructType::new([
                                (#name.into(), #partial_inner),
                            ]))
                        }
                    }
                }
                // { "tag": "name", ...value }
                SerdeTagFormat::Internal(tag) => {
                    if unit {
                        quote! {
                            Schema::structure(StructType::new([
                                (#tag.into(), #name_literal),
                            ]))
                        }
                    } else {
                        wrap(quote! {
                            {
                                let mut item = #inner;
                                item.ty.add_field(#tag, SchemaField::new(#name_literal));
                                item
                            }
                        })
                    }
                }
                // { "tag": "name" } | { "tag": "name", "content": value }
                SerdeTagFormat::Adjacent(tag, content) => {
                    if unit {
                        quote! {
                            Schema::structure(StructType::new([
                                (#tag.into(), #name_literal),
                            ]))
                        }
                    } else {
                        quote! {
                            Schema::structure(StructType::new([
                                (#tag.into(), #name_literal),
                                (#content.into(), #partial_inner),
                            ]))
                        }
                    }
                }
            }
        };

        let comment = extract_comment(&self.attrs);
        let deprecated = extract_deprecated(&self.attrs);

        if comment.is_some() || deprecated.is_some() {
            let comment = comment.map(|value| quote! { item.description = Some(#value.into()); });
            let deprecated =
                deprecated.map(|value| quote! { item.deprecated = Some(#value.into()); });

            schema = quote! {
                {
                    let mut item = #schema;
                    #comment
                    #deprecated
                    item
                }
            };
        }

        schema
    }

    pub fn impl_full_from_partial(&self, partial_name: &Ident) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(fields) => {
                self.map_unnamed_match_custom(name, partial_name, fields, |outer_names, _| {
                    let items = outer_names
                        .iter()
                        .enumerate()
                        .map(|(index, o)| {
                            if self.values[index].requires_from_partial_mapping() {
                                self.values[index].impl_full_from_partial_value(o).value
                            } else {
                                quote! { #o }
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        Self::#name(#(#items),*)
                    }
                })
            }
            Fields::Unit => quote! {
                #partial_name::#name => Self::#name,
            },
        };

        res
    }

    /// Generate this variant holding the default of each of its types, for
    /// the partial's `Default`. Like the empty partial of a struct, this
    /// leaves out any `default` value, which `default_values` applies.
    pub fn impl_partial_default_value(&self) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .map(|_| {
                        quote! { Default::default() }
                    })
                    .collect::<Vec<_>>();

                quote! { #name(#(#fields),*) }
            }
            Fields::Unit => quote! { #name },
        };

        res
    }

    /// Generate this variant holding its `default` value, for the partial's
    /// `default_values`. Without one, it holds the default of its type.
    pub fn impl_partial_default_values(&self) -> ImplResult {
        let Some(expr) = self.get_default_value() else {
            return self.impl_partial_default_value();
        };

        let mut res = ImplResult::default();
        let name = &self.ident;

        let value = match expr {
            // Handler functions return an optional value,
            // falling back to the default of its type
            Expr::Path(func) if !path_names_a_value(&func.path) => {
                res.requires_internal = true;

                quote! { handle_default_result(#func(context))?.unwrap_or_default() }
            }
            // A variant has no partial `Option` of its own, so every layer applies
            expr => {
                let value = &self.values[0];
                let default = value.impl_default_value(expr, &value.get_partial_layers());

                res.requires_internal = default.requires_internal;
                default.value
            }
        };

        res.value = quote! { #name(#value) };
        res
    }

    /// Generate a deserialization attempt for this variant, for use within
    /// untagged enums, where each variant is tried in order.
    pub fn impl_partial_type_deserialize(&self, partial_name: &Ident) -> ImplResult {
        let mut res = ImplResult::default();

        if self.is_skipped_deserializing() {
            res.no_value = true;

            return res;
        }

        let name = &self.ident;
        let name_string = self.get_name();

        res.value = match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(_) => {
                let types = self
                    .values
                    .iter()
                    .map(|value| value.get_partial_type())
                    .collect::<Vec<_>>();

                if types.len() == 1 {
                    let ty = &types[0];

                    quote! {
                        match schematic::internal::deserialize_untagged_variant::<#ty>(&content) {
                            Ok(value) => return Ok(#partial_name::#name(value)),
                            Err(error) => errors.push((#name_string, error)),
                        }
                    }
                } else {
                    // Deserialize multiple values as a tuple, then destructure
                    let indexes = (0..types.len()).map(Index::from).collect::<Vec<_>>();

                    quote! {
                        match schematic::internal::deserialize_untagged_variant::<(#(#types),*)>(&content) {
                            Ok(value) => return Ok(#partial_name::#name(#(value.#indexes),*)),
                            Err(error) => errors.push((#name_string, error)),
                        }
                    }
                }
            }
            // Unit variants in untagged enums are represented as null
            Fields::Unit => quote! {
                if schematic::internal::is_null_content(&content) {
                    return Ok(#partial_name::#name);
                }

                errors.push((#name_string, "expected null".to_owned()));
            },
        };

        res
    }

    /// Generate a match arm for this unit variant, for use within literal
    /// unions, where a tagged unit is written as its name.
    pub fn impl_partial_type_deserialize_literal(&self, partial_name: &Ident) -> ImplResult {
        let mut res = ImplResult::default();

        if self.is_skipped_deserializing() {
            res.no_value = true;

            return res;
        }

        let name = &self.ident;
        let value = self.get_name();
        let aliases = self.get_aliases();

        res.value = quote! {
            Some(#value #(| #aliases)*) => return Ok(#partial_name::#name),
        };

        res
    }

    pub fn impl_partial_finalize(&self) -> ImplResult {
        let mut res = ImplResult::default();

        match &self.fields {
            Fields::Named(_) | Fields::Unit => {
                res.no_value = true;
            }
            Fields::Unnamed(fields) => {
                if !self.is_nested() && self.args.transform.is_none() {
                    res.no_value = true;
                } else {
                    let name = &self.ident;

                    res.value = self.map_unnamed_match(name, fields, |outer_names, _| {
                        let items = outer_names
                            .iter()
                            .enumerate()
                            .map(|(i, o)| {
                                let value = if self.is_nested() {
                                    // Variant values have no partial `Option`
                                    // wrapper, so walk all layers
                                    self.values[i].impl_partial_finalize_nested(o, false).value
                                } else {
                                    quote! { #o }
                                };

                                if let Some(func) = &self.args.transform {
                                    quote! { #func(#value, context)? }
                                } else {
                                    value
                                }
                            })
                            .collect::<Vec<_>>();

                        quote! {
                            Self::#name(#(#items),*)
                        }
                    });
                }
            }
        }

        res
    }

    pub fn impl_partial_merge(&self) -> ImplResult {
        let mut res = ImplResult::default();

        match &self.fields {
            Fields::Named(_) => {
                res.no_value = true;
            }
            Fields::Unnamed(fields) => {
                let name = &self.ident;

                match &self.args.merge {
                    Some(func) => {
                        if self.is_nested()
                            && self
                                .values
                                .first()
                                .is_none_or(|value| !value.is_collection())
                        {
                            panic!(
                                "Nested configs do not support `merge` unless wrapped in a collection."
                            );
                        }

                        res.value = self.map_unnamed_match(&self.ident, fields, |outer_names, inner_names| {
                            if outer_names.len() == 1 {
                                quote! {
                                    if let Self::#name(na) = next {
                                        *self = Self::#name(
                                            #func(pa.to_owned(), na, context)?.unwrap_or_default(),
                                        );
                                    } else {
                                        *self = next;
                                    }
                                }
                            } else {
                                let defaults = outer_names
                                    .iter()
                                    .map(|_| {
                                        quote! { Default::default() }
                                    })
                                    .collect::<Vec<_>>();

                                quote! {
                                    if let Self::#name(#(#inner_names),*) = next {
                                        if let Some((#(#outer_names),*)) = #func(
                                            (#(#outer_names.to_owned()),*),
                                            (#(#inner_names),*),
                                            context,
                                        )? {
                                            *self = Self::#name(#(#outer_names),*);
                                        } else {
                                            *self = Self::#name(#(#defaults),*);
                                        }
                                    } else {
                                        *self = next;
                                    }
                                }
                            }
                        });
                    }
                    None => {
                        // Nested configs are merged recursively, but collections
                        // of them are replaced, as there's no way to know how to
                        // pair up their items. Define `merge` to customize this.
                        let mergeable = self.is_nested()
                            && self.values.iter().any(|value| !value.is_collection());

                        if mergeable {
                            let mut requires_internal = false;

                            res.value = self.map_unnamed_match(
                                &self.ident,
                                fields,
                                |outer_names, inner_names| {
                                    let statements = outer_names
                                        .iter()
                                        .enumerate()
                                        .map(|(index, o)| {
                                            let i = &inner_names[index];
                                            let value = &self.values[index];

                                            // Collections are replaced in place
                                            if value.is_collection() {
                                                return quote! { *#o = #i; };
                                            }

                                            // Variant values are not wrapped by the
                                            // partial, so all layers must be handled
                                            let merge = value.impl_partial_merge_nested(
                                                &quote! { #o },
                                                &quote! { #i },
                                                false,
                                            );

                                            // Wrap with the manager when the value is optional
                                            if merge.requires_internal {
                                                requires_internal = true;

                                                let inner = merge.value;

                                                quote! { MergeManager::new(context)#inner; }
                                            } else {
                                                merge.value
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    quote! {
                                        if let Self::#name(#(#inner_names),*) = next {
                                            #(#statements)*
                                        } else {
                                            *self = next;
                                        }
                                    }
                                },
                            );

                            res.requires_internal = requires_internal;
                        } else {
                            res.no_value = true;
                        }
                    }
                };
            }
            Fields::Unit => {
                res.no_value = true;
            }
        };

        res
    }

    pub fn impl_partial_validate(&self) -> ImplResult {
        let Fields::Unnamed(fields) = &self.fields else {
            return ImplResult::skipped();
        };

        let value = self.map_unnamed_match(&self.ident, fields, |outer_names, _| {
            let mut statements = vec![];
            let name_string = self.get_name();

            #[cfg(feature = "validate")]
            if let Some(expr) = self.args.validate.as_deref() {
                use syn::Expr;

                let func = match expr {
                    // func(arg)() - returns a validator
                    // func() - is the validator itself
                    Expr::Call(_) | Expr::Path(_) => quote! { #expr },
                    _ => {
                        panic!("Unsupported `validate` syntax.");
                    }
                };

                // See `FieldValue::impl_partial_validate` for why this is
                // called through a closure instead of passed by value
                statements.push(quote! {
                    validate.check_variant(
                        #name_string,
                        (#(#outer_names),*),
                        self,
                        |v, d, c, f| #func(v, d, c, f),
                    );
                });
            }

            if self.is_required() {
                statements.push(quote! {
                    if [#(#outer_names),*].iter().any(|v| v.is_none()) {
                        validate.required_variant(#name_string);
                    }
                });
            }

            if self.is_nested() {
                statements.extend(
                    outer_names
                        .iter()
                        .enumerate()
                        .map(|(index, o)| {
                            // Variant values are not wrapped by the partial,
                            // so all layers must be handled
                            self.values[index]
                                .impl_partial_validate_nested(
                                    &quote! { #name_string, #index },
                                    o,
                                    true,
                                    false,
                                )
                                .value
                        })
                        .collect::<Vec<_>>(),
                );
            }

            quote! {
                #(#statements)*
            }
        });

        ImplResult {
            value,
            ..Default::default()
        }
    }

    fn map_unnamed_match<F>(&self, name: &Ident, fields: &FieldsUnnamed, factory: F) -> TokenStream
    where
        F: FnOnce(&[Ident], &[Ident]) -> TokenStream,
    {
        let self_name = format_ident!("Self");

        self.map_unnamed_match_custom(name, &self_name, fields, factory)
    }

    #[allow(clippy::explicit_counter_loop)]
    fn map_unnamed_match_custom<F>(
        &self,
        name: &Ident,
        self_name: &Ident,
        fields: &FieldsUnnamed,
        factory: F,
    ) -> TokenStream
    where
        F: FnOnce(&[Ident], &[Ident]) -> TokenStream,
    {
        let mut count: u8 = 97; // a
        let mut outer_names = vec![];
        let mut inner_names = vec![];

        for _ in &fields.unnamed {
            let outer_name = format_ident!("p{}", count as char);
            let inner_name = format_ident!("n{}", count as char);

            outer_names.push(outer_name);
            inner_names.push(inner_name);

            count += 1;
        }

        let inner = factory(&outer_names, &inner_names);

        quote! {
            #self_name::#name(#(#outer_names),*) => {
                #inner
            },
        }
    }
}

// Only used for partials!
impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = self.get_partial_attributes();
        let name = &self.ident;

        tokens.extend(match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(_) => {
                let types = self
                    .values
                    .iter()
                    .map(|value| value.get_partial_type())
                    .collect::<Vec<_>>();

                quote! {
                    #(#attrs)*
                    #name(#(#types),*),
                }
            }
            Fields::Unit => quote! {
                #(#attrs)*
                #name,
            },
        });
    }
}
