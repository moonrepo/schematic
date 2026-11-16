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

    /// Whether a variable naming this variant can pick it, which the enum's
    /// environment layer then holds with the variables of its settings, and
    /// its defaults layer with their defaults. A unit or named variant can be,
    /// as can one whose values are all nested configs, while a plain value has
    /// nothing to provide it. One that serde never deserializes, or that has
    /// no name to match, can't be.
    pub fn is_env_selectable(&self) -> bool {
        if self.is_skipped_deserializing() || self.is_untagged() || self.args.null {
            return false;
        }

        match &self.fields {
            Fields::Unit | Fields::Named(_) => true,
            Fields::Unnamed(_) => self.values.iter().all(|value| value.nested),
        }
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
            Self::#name(schematic::internal::config_enum::parse_fallback(#value)?)
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
        tag_env_var: Option<&str>,
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

        // A tag reports the variable that picks its variant, like a setting
        let tag_field = match tag_env_var {
            Some(key) => quote! {
                {
                    let mut field = SchemaField::new(#name_literal);
                    field.env_var = Some(#key.into());
                    field
                }
            },
            None => quote! { SchemaField::new(#name_literal) },
        };

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
                        quote! { schematic::internal::schema::tagged_unit(#tag, #tag_field) }
                    } else {
                        wrap(quote! {
                            schematic::internal::schema::internally_tagged(#inner, #tag, #tag_field)
                        })
                    }
                }
                // { "tag": "name" } | { "tag": "name", "content": value }
                SerdeTagFormat::Adjacent(tag, content) => {
                    if unit {
                        quote! { schematic::internal::schema::tagged_unit(#tag, #tag_field) }
                    } else {
                        quote! {
                            schematic::internal::schema::adjacently_tagged(
                                #tag,
                                #tag_field,
                                #content,
                                #partial_inner,
                            )
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

    /// Generate this variant's arm of `default_values`, which holds the
    /// variant with its defaults when the partial holds it: those of its
    /// settings, or of the nested configs it holds, and its `default` value
    /// when it's the default variant. A variant with nothing to default has no
    /// arm, unless it's the default variant, which the layer holds as it is.
    pub fn impl_partial_default_values(&self) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.fields {
            Fields::Named(_) => {
                let mut bindings = vec![];
                let mut rows = vec![];
                let mut empty = vec![];
                let mut always_some = false;

                for setting in &self.settings {
                    let key = setting.get_key();
                    let default = setting.impl_partial_default_value();

                    if default.no_value {
                        rows.push(quote! { #key: None });

                        continue;
                    }

                    // Only a nested config's defaults depend on its value
                    if setting.defaults_read_value() {
                        let prev = setting.get_access(quote! { self });

                        bindings.push(quote! { #key: #prev });
                    }

                    if setting.default_may_be_none() {
                        empty.push(quote! { #key: None });
                    } else {
                        always_some = true;
                    }

                    let value = default.value;

                    rows.push(quote! { #key: #value });
                }

                let value = quote! { Self::#name { #(#rows),* } };

                // The default variant is held even without defaults, as it's
                // what the enum defaults to, while another only when it has
                // some, so that the layer doesn't report one that sets nothing
                let value = if always_some || self.is_default() {
                    quote! { Some(#value) }
                } else if empty.is_empty() {
                    return ImplResult::skipped();
                } else {
                    quote! {
                        {
                            let defaults = #value;

                            if matches!(defaults, Self::#name { #(#empty,)* .. }) {
                                None
                            } else {
                                Some(defaults)
                            }
                        }
                    }
                };

                quote! {
                    Self::#name { #(#bindings,)* .. } => #value,
                }
            }
            Fields::Unnamed(_) => {
                let mut patterns = vec![];
                let mut items = vec![];
                let mut nested = vec![];

                for (index, value) in self.values.iter().enumerate() {
                    let (binding, _) = get_binding_idents(index);

                    match value.impl_partial_default_values_nested(&binding) {
                        Some(defaults) => {
                            patterns.push(quote! { #binding });
                            items.push(value.impl_variant_value_from_defaults(&defaults));
                            nested.push(defaults);
                        }
                        None => {
                            patterns.push(quote! { _ });
                            items.push(quote! { Default::default() });
                        }
                    }
                }

                // Only a single value that isn't nested may have one
                if let Some(default) = self.impl_default_value() {
                    items[0] = default;
                }

                let value = if self.is_default() {
                    quote! { Some(Self::#name(#(#items),*)) }
                } else if nested.is_empty() {
                    return ImplResult::skipped();
                } else if self.values.len() == 1 {
                    // Another variant has defaults only when its config does
                    let defaults = &nested[0];

                    if self.values[0].get_partial_layers().is_empty() {
                        quote! { #defaults.map(Self::#name) }
                    } else {
                        let inner = self.values[0].wrap_variant_partial(quote! { inner });

                        quote! { #defaults.map(|inner| Self::#name(#inner)) }
                    }
                } else {
                    quote! { Some(Self::#name(#(#items),*)) }
                };

                quote! {
                    Self::#name(#(#patterns),*) => #value,
                }
            }
            Fields::Unit => {
                if !self.is_default() {
                    return ImplResult::skipped();
                }

                quote! {
                    Self::#name => Some(Self::#name),
                }
            }
        };

        res
    }

    /// Generate the `default` value this tuple variant holds, from
    /// `#[setting(default = value)]`.
    fn impl_default_value(&self) -> Option<TokenStream> {
        let expr = self.get_default_value()?;

        Some(match expr {
            // Handler functions return an optional value,
            // falling back to the default of its type
            Expr::Path(func) if !path_names_a_value(&func.path) => {
                quote! { schematic::internal::defaults::handle_default_result(#func(context))?.unwrap_or_default() }
            }
            // A variant has no partial `Option` of its own, so every layer applies
            expr => {
                let value = &self.values[0];

                value.impl_default_value(expr, &value.get_partial_layers())
            }
        })
    }

    /// Whether the enum's environment layer reads this variant's variables.
    /// A named variant's settings are optional, so the layer holds only those
    /// it read, as it does for a variant holding a single nested config. A
    /// variant holding several values would have to restate the ones it
    /// didn't read, which would replace them, so it reads them when it's
    /// finalized instead, as does one holding a recursive or collected config.
    pub fn has_layered_env(&self) -> bool {
        match &self.fields {
            Fields::Named(_) => true,
            Fields::Unnamed(_) => {
                self.values.len() == 1
                    && self.values[0].nested
                    && !self.values[0].is_collection()
                    && self.values[0].supports_env_value(false)
            }
            Fields::Unit => false,
        }
    }

    /// Generate this variant's arm of `env_values`, which holds the variant
    /// with the variables of its settings, or of the nested config it holds,
    /// when the partial holds it, or the tag's variable picks it. A variant
    /// that reads nothing has no arm.
    #[cfg(feature = "env")]
    pub fn impl_partial_env_values(&self) -> ImplResult {
        if !self.has_layered_env() {
            return ImplResult::skipped();
        }

        let name = &self.ident;

        let value = match &self.fields {
            Fields::Named(_) => {
                let mut bindings = vec![];
                let mut rows = vec![];
                let mut reads = false;

                for setting in &self.settings {
                    let key = setting.get_key();
                    let res = setting.impl_partial_env_value();

                    if res.no_value {
                        rows.push(quote! { #key: None });

                        continue;
                    }

                    // Only a nested config's variables depend on its value
                    if setting.env_reads_value() {
                        let prev = setting.get_access(quote! { self });

                        bindings.push(quote! { #key: #prev });
                    }

                    let value = res.value;

                    reads = true;
                    rows.push(quote! { #key: #value });
                }

                if !reads {
                    return ImplResult::skipped();
                }

                quote! {
                    Self::#name { #(#bindings,)* .. } => Self::#name { #(#rows),* },
                }
            }
            _ => {
                let value = &self.values[0];
                let (binding, _) = get_binding_idents(0);

                // The value is discarded when nothing is read, so its default
                // only stands in
                let read = if value.is_outer_option_wrapped() {
                    quote! { env.nested(schematic::internal::layers::nested_env_values(#binding.as_ref(), None)?)? }
                } else {
                    quote! {
                        env.nested(schematic::internal::layers::nested_env_values(Some(#binding), None)?)?
                            .unwrap_or_default()
                    }
                };

                quote! {
                    Self::#name(#binding) => Self::#name(#read),
                }
            }
        };

        ImplResult {
            value,
            ..Default::default()
        }
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

                let (_, ty_generics, where_clause) = generics.split_for_impl();
                let marker = if generics.params.is_empty() {
                    quote! {}
                } else {
                    quote! {
                        #[serde(skip)]
                        __marker: std::marker::PhantomData<#partial_name #ty_generics>,
                    }
                };

                // A variant without settings reads none of the value
                let attempt = if keys.is_empty() {
                    quote! {
                        if attempts.variant::<Variant #ty_generics>(#name_string).is_some() {
                            return Ok(#partial_name::#name {});
                        }
                    }
                } else {
                    quote! {
                        if let Some(value) = attempts.variant::<Variant #ty_generics>(#name_string) {
                            return Ok(#partial_name::#name {
                                #(#keys: value.#keys),*
                            });
                        }
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

                        #attempt
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
                        if let Some(value) = attempts.variant::<#ty>(#name_string) {
                            return Ok(#partial_name::#name(value));
                        }
                    }
                } else {
                    // Deserialize multiple values as a tuple, then destructure
                    let indexes = (0..types.len()).map(Index::from).collect::<Vec<_>>();

                    quote! {
                        if let Some(value) = attempts.variant::<(#(#types),*)>(#name_string) {
                            return Ok(#partial_name::#name(#(value.#indexes),*));
                        }
                    }
                }
            }
            // Unit variants in untagged enums are represented as null
            Fields::Unit => quote! {
                if attempts.unit(#name_string) {
                    return Ok(#partial_name::#name);
                }
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
            #value #(| #aliases)* => Some(#partial_name::#name),
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
                    let env_boundary = cfg!(feature = "env") && !self.has_layered_env();

                    res.value = self.map_unnamed_match(name, fields, |outer_names, _| {
                        let items = outer_names
                            .iter()
                            .enumerate()
                            .map(|(i, o)| {
                                let value = if self.values[i].nested {
                                    // Variant values have no partial `Option`
                                    // wrapper, so walk all layers. A value
                                    // whose variables the enum's layer can't
                                    // hold reads them here.
                                    self.values[i]
                                        .impl_partial_finalize_nested(o, false, env_boundary)
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

    /// Generate the finalization of this named variant, whose settings are
    /// transformed and finalized as a struct's are. Its defaults and
    /// environment variables are part of the enum's layers, like a struct's.
    fn impl_partial_finalize_named(&self) -> ImplResult {
        let name = &self.ident;
        let statements = self
            .settings
            .iter()
            .map(|setting| setting.impl_partial_finalize())
            .collect::<Vec<_>>();

        if statements.iter().all(|res| res.no_value) {
            return ImplResult::skipped();
        }

        // A setting that's finalized is reassigned
        let bindings = self
            .settings
            .iter()
            .zip(&statements)
            .map(|(setting, res)| {
                let key = setting.get_key();
                let prev = setting.get_access(quote! { self });

                if res.no_value {
                    quote! { #key: #prev }
                } else {
                    quote! { #key: mut #prev }
                }
            })
            .collect::<Vec<_>>();

        let statements = statements
            .into_iter()
            .filter(|res| !res.no_value)
            .map(|res| res.value);
        let rebuild = self.get_named_bindings(false);

        ImplResult {
            value: quote! {
                Self::#name { #(#bindings),* } => {
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

                res.value = quote! {
                    Self::#name { #(#prev),* } => {
                        if let Self::#name { #(#next),* } = next {
                            schematic::internal::merge::MergeManager::new(context)
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
                                                    return quote! {
                                                        schematic::internal::merge::MergeManager::new(context).apply_with(
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
                                            if value.is_outer_option_wrapped() {
                                                let inner = merge.value;

                                                quote! { schematic::internal::merge::MergeManager::new(context)#inner; }
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
