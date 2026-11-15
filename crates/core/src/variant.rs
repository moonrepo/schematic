#[cfg(feature = "schema")]
use crate::args::SerdeTagFormat;
use crate::args::{
    NestedArg, PartialArg, SerdeContainerArgs, SerdeFieldArgs, SerdeRenameArg, VariantDefaultArg,
};
use crate::container::ContainerArgs;
use crate::field::{Field, VariantScope};
use crate::utils::{
    ImplResult, format_case, format_loose_case, get_meta_path, get_renamed_value,
    is_inheritable_attribute, to_type_string,
};
use crate::value::path_names_a_value;
use crate::variant_value::VariantValue;
use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::rc::Rc;
use syn::ext::IdentExt;
use syn::{
    Attribute, Expr, ExprLit, ExprPath, Fields, FieldsUnnamed, Generics, Ident, Index, Lit, Type,
    Variant as NativeVariant,
};

// #[setting()], #[schema()], #[variant()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, attributes(setting, schema, variant))]
pub struct VariantArgs {
    pub context: Option<ExprPath>,
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
    pub rename_all: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    pub untagged: bool,
}

// #[setting()] on a value of a variant, like `A(#[setting(nested)] Child, bool)`
#[derive(Debug, Default, FromAttributes)]
#[darling(default, attributes(setting))]
pub struct VariantValueArgs {
    pub context: Option<ExprPath>,
    pub nested: Option<NestedArg>,
}

/// The suffix of a binding by position, a letter for the first 26.
fn get_binding_suffix(index: usize) -> String {
    if index < 26 {
        ((b'a' + index as u8) as char).to_string()
    } else {
        index.to_string()
    }
}

/// The bindings of a variant's values or settings, when destructured from
/// this partial (`pa`) and the next (`na`), by position.
pub fn get_binding_idents(index: usize) -> (Ident, Ident) {
    let suffix = get_binding_suffix(index);

    (format_ident!("p{suffix}"), format_ident!("n{suffix}"))
}

/// The binding of a named variant's setting as read from the environment
/// (`ea`), by position, so that a setting's name never shadows it.
fn get_env_binding_ident(index: usize) -> Ident {
    format_ident!("e{}", get_binding_suffix(index))
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
    /// The values of a tuple variant, which aren't optional within the
    /// partial, as the variant is the whole of a setting.
    pub values: Vec<VariantValue>,

    /// The settings of a named variant, which are each optional within the
    /// partial, like the fields of a struct.
    pub settings: Vec<Field>,

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
        self_ty: &Type,
    ) -> Variant {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&variant.attrs).unwrap();

        // Serde names the fields of a variant with its own `rename_all`,
        // otherwise the container's `rename_all_fields`
        let rename_all =
            get_renamed_value(args.rename_all.as_ref(), serde_args.rename_all.as_ref()).or_else(
                || {
                    get_renamed_value(
                        container_args.rename_all_fields.as_ref(),
                        serde_container_args.rename_all_fields.as_ref(),
                    )
                },
            );

        let settings = match &variant.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let (prev, next) = get_binding_idents(index);

                    Field::new(
                        field.to_owned(),
                        container_args.clone(),
                        serde_container_args.clone(),
                        self_ty,
                        Some(VariantScope {
                            rename_all: rename_all.clone(),
                            prev,
                            next,
                        }),
                    )
                })
                .collect(),
            _ => vec![],
        };

        let variant = Self {
            attrs: variant.attrs,
            container_args,
            ident: variant.ident,
            serde_args,
            serde_container_args,
            settings,
            values: match &variant.fields {
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .map(|field| Self::new_value(field, &args, self_ty))
                    .collect(),
                Fields::Named(_) | Fields::Unit => vec![],
            },
            fields: variant.fields,
            args,
        };
        variant.validate_args();
        variant
    }

    /// Analyze a value of this variant. A `#[setting(nested)]` on the value
    /// itself applies to it alone, so a variant can mix nested and plain
    /// values, while one on the variant applies to every value.
    fn new_value(field: &syn::Field, args: &VariantArgs, self_ty: &Type) -> VariantValue {
        let value_args = VariantValueArgs::from_attributes(&field.attrs).unwrap();

        if value_args.nested.is_some() && args.nested.is_some() {
            panic!(
                "Cannot use `nested` on both a variant and its values, choose one or the other."
            );
        }

        VariantValue::new(
            field.ty.clone(),
            value_args.nested.as_ref().or(args.nested.as_ref()),
            self_ty,
            value_args.context.or_else(|| args.context.clone()),
        )
    }

    fn validate_args(&self) {
        if self.is_named_variant() {
            self.validate_named_args();
        } else if self.args.rename_all.is_some() || self.serde_args.rename_all.is_some() {
            panic!("Can only use `rename_all` with named variants, as only they have fields.");
        }

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

    /// Panic on an attribute that a named variant can't honor, as each of its
    /// settings takes its own, like the fields of a struct.
    fn validate_named_args(&self) {
        for (name, used) in [
            ("context", self.args.context.is_some()),
            ("merge", self.args.merge.is_some()),
            ("nested", self.args.nested.is_some()),
            ("required", self.args.required),
            ("transform", self.args.transform.is_some()),
            ("validate", self.args.validate.is_some()),
        ] {
            if used {
                panic!(
                    "Cannot use `{name}` on a named variant, as each of its fields is a setting. Use it on the fields instead."
                );
            }
        }

        if let Some(VariantDefaultArg::Value(expr)) = &self.args.default
            && as_bool_literal(expr).is_none()
        {
            panic!(
                "Cannot use a `default` value with named variants, as each of its fields is a setting. Use `default` on the fields instead."
            );
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

        let name = self.get_name_original();

        match get_renamed_value(
            self.container_args.rename_all.as_ref(),
            self.serde_container_args.rename_all.as_ref(),
        ) {
            Some(format) => format_case(&format, &name, true),
            None => name,
        }
    }

    /// The Rust name of this variant, without the `r#` of a raw identifier,
    /// as serde strips it too.
    pub fn get_name_original(&self) -> String {
        self.ident.unraw().to_string()
    }

    /// Whether this is the default variant, marked with `#[setting(default)]`,
    /// or the standard `#[default]`. A variant holding a value is also marked
    /// by providing that value, while a unit variant takes a boolean.
    pub fn is_default(&self) -> bool {
        let marked = match &self.args.default {
            Some(VariantDefaultArg::Marked) => true,
            Some(VariantDefaultArg::Value(expr)) => {
                self.is_tuple_variant() || as_bool_literal(expr).unwrap_or_default()
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
    /// Only a tuple variant holds a value, while a boolean marks any other.
    pub fn get_default_value(&self) -> Option<&Expr> {
        match &self.args.default {
            Some(VariantDefaultArg::Value(expr)) if self.is_tuple_variant() => Some(expr),
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
        let mut aliases = self.get_aliases().into_iter().cloned().collect::<Vec<_>>();

        // `before_parse` normalizes the input with a conversion that keeps an
        // acronym whole, while the name follows serde and splits it, so the
        // identifier as that conversion sees it is matched as well
        if let Some(format) = &self.container_args.before_parse
            && self.get_name_renamed().is_none()
        {
            let loose = format_loose_case(format, &self.get_name_original());

            if loose != value && !aliases.contains(&loose) {
                aliases.push(loose);
            }
        }

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

    /// Whether any value of this variant is a nested config, from a
    /// `#[setting(nested)]` on the variant, or on the value itself.
    pub fn is_nested(&self) -> bool {
        self.values.iter().any(|value| value.nested)
    }

    pub fn is_required(&self) -> bool {
        self.args.required
    }

    pub fn is_unit_variant(&self) -> bool {
        matches!(self.fields, Fields::Unit)
    }

    /// Whether this variant holds values by position, like `A(String)`.
    pub fn is_tuple_variant(&self) -> bool {
        matches!(self.fields, Fields::Unnamed(_))
    }

    /// Whether this variant holds settings by name, like `A { name: String }`.
    pub fn is_named_variant(&self) -> bool {
        matches!(self.fields, Fields::Named(_))
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

        if let Some(rename_all) = self
            .args
            .rename_all
            .as_ref()
            .or(self.serde_args.rename_all.as_ref())
            .filter(|rename| !rename.is_empty())
        {
            meta.push(rename_all.get_meta("rename_all"));
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
    ///
    /// A named variant is a struct of its settings, which is a partial within
    /// a config's partial when `partial` is true, and accepts unknown fields
    /// when `allows_unknown_fields` is true.
    #[cfg(feature = "schema")]
    pub fn impl_schema_type(
        &self,
        tag_format: &SerdeTagFormat,
        partial: bool,
        allows_unknown_fields: bool,
    ) -> TokenStream {
        use crate::utils::{extract_comment, extract_deprecated, impl_literal_value};

        let name = self.get_name();

        // Untagged variants are represented by their value alone
        let untagged = matches!(tag_format, SerdeTagFormat::Untagged) || self.is_untagged();

        let inner = match &self.fields {
            // A skipped setting is never read or written, so like an excluded
            // one it has no schema, as within a struct
            Fields::Named(_) => {
                let fields = self
                    .settings
                    .iter()
                    .filter(|setting| !setting.is_excluded() && !setting.is_skipped())
                    .map(|setting| setting.impl_schema_type(true))
                    .collect::<Vec<_>>();

                let structure = if fields.is_empty() {
                    quote! { StructType::default() }
                } else {
                    quote! {
                        StructType::new([
                            #(#fields),*
                        ])
                    }
                };

                if allows_unknown_fields {
                    quote! {
                        {
                            let mut structure = #structure;
                            structure.allow_unknown_fields = true;
                            Schema::structure(structure)
                        }
                    }
                } else {
                    quote! { Schema::structure(#structure) }
                }
            }
            Fields::Unnamed(fields) => {
                let items = fields
                    .unnamed
                    .iter()
                    .zip(&self.values)
                    .map(|(field, value)| {
                        let ty = &field.ty;

                        if value.nested {
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
        // literal that is always present. When only some values are nested,
        // each was marked by `infer_as_nested` already, and marking the tuple
        // would mark the others too.
        //
        // A named variant's struct is the partial itself, as its settings are
        // each optional within a config's partial, so it's marked like a
        // nested value.
        let nested = if self.is_named_variant() {
            partial
        } else {
            !self.values.is_empty() && self.values.iter().all(|value| value.nested)
        };
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
            } else if self.is_named_variant() {
                partial_inner
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
                        // The tag makes it a different shape to the type it holds,
                        // so it's no longer that named type, which a renderer
                        // would otherwise reference without the tag
                        wrap(quote! {
                            {
                                let mut item = #inner;

                                if item.ty.add_field(#tag, SchemaField::new(#name_literal)) {
                                    item.name = None;
                                }

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

    /// The fields of a pattern that destructures this named variant, binding
    /// each setting for this partial (`name: pa`), or the next (`name: na`).
    fn get_named_bindings(&self, next: bool) -> Vec<TokenStream> {
        self.settings
            .iter()
            .map(|setting| {
                let key = setting.get_key();
                let binding = if next {
                    setting.get_next_access()
                } else {
                    setting.get_access(quote! { self })
                };

                quote! { #key: #binding }
            })
            .collect()
    }

    /// The default of each of this named variant's settings, in order, like
    /// the default values of a struct, or `None` for a setting without one.
    /// When `full` is true, a bare nested config takes its own default
    /// partial instead, for [`Config::default_partial`]. Also returns whether
    /// building them requires the internal helpers.
    fn get_named_defaults(&self, full: bool) -> (Vec<Option<TokenStream>>, bool) {
        let mut requires_internal = false;

        let values = self
            .settings
            .iter()
            .map(|setting| {
                if full
                    && setting.is_bare_nested()
                    && let Some(ty) = setting.value.get_nested_type()
                {
                    return Some(quote! { Some(<#ty as schematic::Config>::default_partial()) });
                }

                let res = setting.impl_partial_default_value();

                if res.requires_internal {
                    requires_internal = true;
                }

                (!res.no_value).then_some(res.value)
            })
            .collect();

        (values, requires_internal)
    }

    /// Generate this setting's entry for `Config::settings`. A named variant
    /// is the settings of its fields, like a nested config, and a unit variant,
    /// which holds no type, is reported as its name.
    pub fn impl_setting(&self) -> TokenStream {
        let name = self.get_name();

        if self.is_unit_variant() {
            return quote! { (#name.into(), ConfigSetting::new(#name)) };
        }

        if self.is_named_variant() {
            let settings = self.settings.iter().map(|setting| setting.impl_setting());

            return quote! {
                (#name.into(), ConfigSetting::new(#name).nested(
                    std::collections::BTreeMap::from_iter([
                        #(#settings),*
                    ])
                ))
            };
        }

        let type_alias = to_type_string(
            self.values
                .iter()
                .map(|v| v.ty.to_token_stream())
                .collect::<TokenStream>(),
        );

        quote! { (#name.into(), ConfigSetting::new(#type_alias)) }
    }

    pub fn impl_full_from_partial(&self, partial_name: &Ident) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.fields {
            Fields::Named(_) => {
                let bindings = self.get_named_bindings(false);
                let rows = self.settings.iter().map(|setting| {
                    let key = setting.get_key();
                    let value = setting.impl_full_from_partial().value;

                    quote! { #key: #value }
                });

                quote! {
                    #partial_name::#name { #(#bindings),* } => Self::#name {
                        #(#rows),*
                    },
                }
            }
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
            Fields::Named(_) => {
                let keys = self.settings.iter().map(|setting| setting.get_key());

                quote! { #name { #(#keys: Default::default()),* } }
            }
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

    /// Generate this variant as the default partial of the full type, for
    /// [`Config::default_partial`]. A bare nested config holds its own default
    /// partial, which isn't part of `default_values`, as finalizing the
    /// variant applies it. Returns nothing when no value is nested, as the
    /// provided method already matches.
    ///
    /// A named variant holds the defaults of its settings, which finalizing
    /// it applies too. They're built in a closure, as building one can fail.
    pub fn impl_full_default_partial(&self, partial_name: &Ident) -> Option<TokenStream> {
        if self.is_named_variant() {
            let name = &self.ident;
            let (defaults, requires_internal) = self.get_named_defaults(true);

            if defaults.iter().all(Option::is_none) {
                return None;
            }

            let rows = self.settings.iter().zip(defaults).map(|(setting, value)| {
                let key = setting.get_key();
                let value = value.unwrap_or_else(|| quote! { None });

                quote! { #key: #value }
            });
            let internal = ImplResult::impl_use_internal(requires_internal);

            return Some(quote! {
                #internal

                #[allow(unused_variables)]
                let context: &<Self::Partial as schematic::PartialConfig>::Context = &Default::default();

                let partial = (|| -> std::result::Result<Self::Partial, schematic::ConfigError> {
                    Ok(#partial_name::#name { #(#rows),* })
                })();

                partial.expect("Failed to generate default values.")
            });
        }

        if self.get_default_value().is_some() || !self.is_tuple_variant() {
            return None;
        }

        let mut nested = false;
        let name = &self.ident;
        let fields = self
            .values
            .iter()
            .map(|value| match value.get_nested_type() {
                Some(ty) if value.get_partial_layers().is_empty() => {
                    nested = true;

                    quote! { <#ty as schematic::Config>::default_partial() }
                }
                _ => quote! { Default::default() },
            })
            .collect::<Vec<_>>();

        nested.then(|| quote! { #partial_name::#name(#(#fields),*) })
    }

    /// Generate a deserialization attempt for this variant, for use within
    /// untagged enums, where each variant is tried in order.
    ///
    /// A named variant is deserialized through a struct that stands in for
    /// it, with the container's generics, and the serde attributes of
    /// `helper_meta`. A generic one holds a marker, so that every parameter
    /// is used.
    pub fn impl_partial_type_deserialize(
        &self,
        partial_name: &Ident,
        generics: &Generics,
        helper_meta: &[TokenStream],
    ) -> ImplResult {
        let mut res = ImplResult::default();

        if self.is_skipped_deserializing() {
            res.no_value = true;

            return res;
        }

        let name = &self.ident;
        let name_string = self.get_name();

        res.value = match &self.fields {
            Fields::Named(_) => {
                let mut meta = helper_meta.to_vec();

                if let Some(rename_all) = self
                    .settings
                    .first()
                    .and_then(|setting| setting.variant.as_ref())
                    .and_then(|scope| scope.rename_all.as_ref())
                {
                    meta.push(quote! { rename_all = #rename_all });
                }

                let attrs = if meta.is_empty() {
                    quote! {}
                } else {
                    quote! { #[serde(#(#meta),*)] }
                };

                let fields = self.settings.iter().map(|setting| {
                    let attrs = setting.get_partial_deserialize_attributes();
                    let key = setting.get_key();
                    let ty = setting.get_partial_type();

                    quote! {
                        #(#attrs)*
                        #key: #ty,
                    }
                });

                let keys = self
                    .settings
                    .iter()
                    .map(|setting| setting.get_key())
                    .collect::<Vec<_>>();

                // A variant without settings reads none of the value
                let value = if keys.is_empty() {
                    quote! { _ }
                } else {
                    quote! { value }
                };

                let (_, ty_generics, where_clause) = generics.split_for_impl();
                let marker = if generics.params.is_empty() {
                    quote! {}
                } else {
                    quote! {
                        #[serde(skip)]
                        __marker: std::marker::PhantomData<#partial_name #ty_generics>,
                    }
                };

                quote! {
                    {
                        #[derive(serde::Deserialize)]
                        #attrs
                        struct Variant #generics #where_clause {
                            #(#fields)*
                            #marker
                        }

                        match schematic::internal::deserialize_untagged_variant::<Variant #ty_generics>(&content) {
                            Ok(#value) => return Ok(#partial_name::#name {
                                #(#keys: value.#keys),*
                            }),
                            Err(error) => errors.push((#name_string, error)),
                        }
                    }
                }
            }
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
            Fields::Named(_) => {
                return self.impl_partial_finalize_named();
            }
            Fields::Unit => {
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
                                let value = if self.values[i].nested {
                                    // Variant values have no partial `Option`
                                    // wrapper, so walk all layers. An enum's
                                    // environment layer holds no variant, so a
                                    // value is finalized in full.
                                    self.values[i]
                                        .impl_partial_finalize_nested(o, false, true)
                                        .value
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

    /// The value of each of this named variant's settings as read from the
    /// environment, in order, like the environment values of a struct, or
    /// `None` for a setting that reads none.
    fn get_named_env_values(&self) -> Vec<Option<TokenStream>> {
        self.settings
            .iter()
            .map(|setting| {
                let res = setting.impl_partial_env_value();

                (!res.no_value).then_some(res.value)
            })
            .collect()
    }

    /// Generate the finalization of this named variant. No layer holds the
    /// defaults or environment variables of its settings, as a source may
    /// pick it over the default variant, so like the value of a tuple variant,
    /// it's a boundary: its defaults are merged under it and its environment
    /// variables over it, each setting with its own `merge`, as the loader
    /// does for a struct, and then its settings are transformed and finalized
    /// as a struct's are.
    fn impl_partial_finalize_named(&self) -> ImplResult {
        let name = &self.ident;
        let (defaults, mut requires_internal) = self.get_named_defaults(false);
        let env_values = self.get_named_env_values();
        let statements = self
            .settings
            .iter()
            .map(|setting| setting.impl_partial_finalize())
            .collect::<Vec<_>>();

        if defaults.iter().all(Option::is_none)
            && env_values.iter().all(Option::is_none)
            && statements.iter().all(|res| res.no_value)
        {
            return ImplResult::skipped();
        }

        let mut bindings = vec![];
        let mut lets = vec![];
        let mut merges = vec![];
        let mut env_lets = vec![];
        let mut env_merges = vec![];

        // A setting with a default is bound as the next value, and merged into
        // its default, while one without is bound as the value itself
        for (index, ((setting, default), env_value)) in self
            .settings
            .iter()
            .zip(defaults)
            .zip(env_values)
            .enumerate()
        {
            let key = setting.get_key();
            let prev = setting.get_access(quote! { self });
            let next = setting.get_next_access();
            let finalizes = !statements[index].no_value;

            if let Some(value) = default {
                lets.push(quote! { let mut #prev = #value; });
                merges.push(
                    setting
                        .impl_partial_merge_into(&quote! { &mut #prev }, &next)
                        .value,
                );
                bindings.push(quote! { #key: #next });
            } else if finalizes || env_value.is_some() {
                bindings.push(quote! { #key: mut #prev });
            } else {
                bindings.push(quote! { #key: #prev });
            }

            // Merged after every source, as the environment is the last layer
            if let Some(value) = env_value {
                let env_binding = get_env_binding_ident(index);

                env_lets.push(quote! { let #env_binding = #value; });
                env_merges.push(
                    setting
                        .impl_partial_merge_into(&quote! { &mut #prev }, &quote! { #env_binding })
                        .value,
                );
            }
        }

        // The keys a setting derives take the container's own prefix, as a
        // parent's `#[setting(nested, env_prefix)]` never reaches a variant
        let env = if env_lets.is_empty() {
            quote! {}
        } else {
            requires_internal = true;

            let own_prefix = match &self.container_args.env_prefix {
                Some(prefix) => quote! { Some(#prefix) },
                None => quote! { None },
            };

            quote! {
                let mut env = EnvManager::new(None::<&str>, #own_prefix);
                #(#env_lets)*
            }
        };

        merges.extend(env_merges);

        let merge = if merges.is_empty() {
            quote! {}
        } else {
            requires_internal = true;

            quote! {
                MergeManager::new(context)
                    #(#merges)*;
            }
        };

        let statements = statements
            .into_iter()
            .filter(|res| !res.no_value)
            .map(|res| res.value);
        let rebuild = self.get_named_bindings(false);

        ImplResult {
            requires_internal,
            value: quote! {
                Self::#name { #(#bindings),* } => {
                    #(#lets)*
                    #env
                    #merge
                    #(#statements)*

                    Self::#name { #(#rebuild),* }
                }
            },
            ..Default::default()
        }
    }

    pub fn impl_partial_merge(&self) -> ImplResult {
        let mut res = ImplResult::default();

        match &self.fields {
            Fields::Named(_) => {
                // Each setting is merged as it would be within a struct, so
                // the same variant is merged into, and any other replaces it
                if self.settings.is_empty() {
                    return ImplResult::skipped();
                }

                let name = &self.ident;
                let prev = self.get_named_bindings(false);
                let next = self.get_named_bindings(true);
                let merges = self
                    .settings
                    .iter()
                    .map(|setting| setting.impl_partial_merge().value);

                res.requires_internal = true;
                res.value = quote! {
                    Self::#name { #(#prev),* } => {
                        if let Self::#name { #(#next),* } = next {
                            MergeManager::new(context)
                                #(#merges)*;
                        } else {
                            *self = next;
                        }
                    },
                };
            }
            Fields::Unnamed(fields) => {
                let name = &self.ident;

                match &self.args.merge {
                    Some(func) => {
                        if self
                            .values
                            .iter()
                            .any(|value| value.nested && !value.is_collection())
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
                        let mergeable = self.values.iter().any(|value| {
                            (value.nested && !value.is_collection()) || value.is_merge_list()
                        });

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

                                            // A `MergeList` is merged as each
                                            // source directs, through the
                                            // manager when it's optional
                                            if value.is_merge_list() {
                                                if value.is_outer_option_wrapped() {
                                                    requires_internal = true;

                                                    return quote! {
                                                        MergeManager::new(context).apply_with(
                                                            #o,
                                                            #i,
                                                            schematic::merge::merge_list,
                                                        )?;
                                                    };
                                                }

                                                return quote! {
                                                    *#o = schematic::merge::merge_list(
                                                        std::mem::take(#o),
                                                        #i,
                                                        context,
                                                    )?
                                                    .unwrap_or_default();
                                                };
                                            }

                                            // Collections, and values that
                                            // aren't configs, are replaced in place
                                            if !value.nested || value.is_collection() {
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
        if self.is_named_variant() {
            return self.impl_partial_validate_named();
        }

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
                // Each value is checked on its own, as they may differ in type
                statements.push(quote! {
                    if #(#outer_names.is_none())||* {
                        validate.required_variant(#name_string);
                    }
                });
            }

            if self.is_nested() {
                statements.extend(
                    outer_names
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| self.values[*index].nested)
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

    /// Generate the validation of this named variant's settings, as a
    /// struct's are, reported under the variant's name, like `Name.setting`.
    /// Only the settings that are validated are bound.
    fn impl_partial_validate_named(&self) -> ImplResult {
        let mut bindings = vec![];
        let mut statements = vec![];

        for setting in &self.settings {
            let res = setting.impl_partial_validate();

            if res.no_value {
                continue;
            }

            let key = setting.get_key();
            let prev = setting.get_access(quote! { self });

            bindings.push(quote! { #key: #prev });
            statements.push(res.value);
        }

        if statements.is_empty() {
            return ImplResult::skipped();
        }

        let name = &self.ident;
        let name_string = self.get_name();

        ImplResult {
            value: quote! {
                Self::#name { #(#bindings,)* .. } => {
                    validate.named_variant(#name_string, |validate| {
                        #(#statements)*
                    });
                },
            },
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
        let mut outer_names = vec![];
        let mut inner_names = vec![];

        for index in 0..fields.unnamed.len() {
            let (outer_name, inner_name) = get_binding_idents(index);

            outer_names.push(outer_name);
            inner_names.push(inner_name);
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
            Fields::Named(_) => {
                let settings = &self.settings;

                quote! {
                    #(#attrs)*
                    #name {
                        #(#settings)*
                    },
                }
            }
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
