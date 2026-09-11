use crate::schema::SchemaGenerator;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::{IntoDiagnostic, miette};
use schematic_types::*;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::fs;
use std::path::Path;

/// A tag describing how a type, property, or variant may be provided.
#[derive(Clone, Debug, PartialEq)]
pub enum ApiDocsTag {
    /// The default variant of an enum or union.
    Default,
    /// Deprecated, with the deprecation message when one was given.
    Deprecated(Option<String>),
    /// A property whose keys are flattened into its parent.
    Flattened,
    /// Accepts `null`.
    Nullable,
    /// A property that may be omitted.
    Optional,
    /// A property that must be provided.
    Required,
    /// A property that is only serialized, never deserialized.
    ReadOnly,
    /// A property that is only deserialized, never serialized.
    WriteOnly,
}

impl ApiDocsTag {
    /// The human readable label of the tag.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Deprecated(_) => "Deprecated",
            Self::Flattened => "Flattened",
            Self::Nullable => "Nullable",
            Self::Optional => "Optional",
            Self::Required => "Required",
            Self::ReadOnly => "Read only",
            Self::WriteOnly => "Write only",
        }
    }
}

impl fmt::Display for ApiDocsTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Renders the tags of a type, property, or variant to markdown. Only called
/// with at least one tag, and an empty result omits the tags entirely.
pub type ApiDocsTagsRenderer = Box<dyn Fn(&[ApiDocsTag]) -> String>;

/// Renders the description of a type, property, or variant to markdown,
/// from the doc comment as written. An empty result omits the description.
pub type ApiDocsDescriptionRenderer = Box<dyn Fn(&str) -> String>;

/// Renders a link to a referenced type's page, from the type name, as a
/// complete markdown link including its label. Used wherever a type is
/// linked: inline in a type expression, and in the references list.
pub type ApiDocsLinkRenderer = Box<dyn Fn(&str) -> String>;

/// The default link renderer, which labels the link with the type name as
/// inline code, and links to a markdown file named after the type in the
/// same directory, such as `` [`Name`](./Name.md) ``.
pub fn default_link_renderer(name: &str) -> String {
    format!("[{}](./{name}.md)", code(name))
}

/// Renders the fragment a section is linked by, from the text of its
/// heading, such as the property or variant name. The result follows the
/// `#` in an index link, so it must match the id the documentation tool
/// gives the heading.
pub type ApiDocsAnchorRenderer = Box<dyn Fn(&str) -> String>;

/// The default anchor renderer, which follows the GitHub convention that
/// most documentation tools share: lowercased, punctuation removed, and
/// spaces hyphenated, so `` `expand_array` `` becomes `expand_array`.
pub fn default_anchor_renderer(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_alphanumeric() || *ch == ' ' || *ch == '-' || *ch == '_')
        .map(|ch| if ch == ' ' { '-' } else { ch })
        .collect()
}

/// The default tags renderer, which renders a block quote of bold labels
/// separated by a middle dot, such as `> **Required** · **Nullable**`. A
/// deprecation message follows its label in parentheses.
pub fn default_tags_renderer(tags: &[ApiDocsTag]) -> String {
    let tags = tags
        .iter()
        .map(|tag| match tag {
            ApiDocsTag::Deprecated(Some(message)) => {
                format!("**{}** ({})", tag.label(), message.trim())
            }
            _ => format!("**{}**", tag.label()),
        })
        .collect::<Vec<_>>();

    format!("> {}", tags.join(" · "))
}

/// The default description renderer, which keeps a description as written,
/// so that paragraphs and lists survive, and only trims each line.
pub fn default_description_renderer(description: &str) -> String {
    description
        .trim()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

/// How the variants of a unit enum are rendered.
#[derive(Debug, Default, PartialEq)]
pub enum ApiDocsEnumFormat {
    /// A section per variant, with its tags, description, and value.
    #[default]
    Sections,
    /// A single table, one row per variant. The index is omitted, as the
    /// table already summarizes every variant.
    Table,
}

/// Options to control the rendered API documentation.
pub struct ApiDocsOptions {
    /// How the variants of a unit enum are rendered. Unions, whose variants
    /// carry values, are always rendered as sections.
    pub enum_format: ApiDocsEnumFormat,

    /// Exclude field aliases from being rendered.
    pub exclude_aliases: bool,

    /// Additional frontmatter entries, written as `key: value` lines after
    /// the title, in key order. Values are written verbatim, so quote them
    /// as the consuming tool expects. A `title` entry replaces the type name.
    pub frontmatter: BTreeMap<String, String>,

    /// Render an index of all properties or variants, linking to each
    /// section, ahead of the sections themselves.
    pub include_index: bool,

    /// File name of the index page listing every type, written by
    /// [`ApiDocsRenderer::generate_all`]. `None` skips the page.
    pub index_page: Option<String>,

    /// Tag all non-optional struct fields as required.
    pub mark_struct_fields_required: bool,

    /// Renders the fragment an index entry links its section by, from the
    /// heading text. Defaults to [`default_anchor_renderer`].
    pub render_anchor: ApiDocsAnchorRenderer,

    /// Renders a link to a referenced type's page, label included, from the
    /// type name. Defaults to [`default_link_renderer`].
    pub render_link: ApiDocsLinkRenderer,

    /// Renders the description of a type, property, or variant. Receives the
    /// doc comment as written. Defaults to [`default_description_renderer`].
    pub render_description: ApiDocsDescriptionRenderer,

    /// Renders the tags of a type, property, or variant. Receives the tags in
    /// a fixed order, and is only called when there is at least one. Defaults
    /// to [`default_tags_renderer`].
    pub render_tags: ApiDocsTagsRenderer,
}

impl Default for ApiDocsOptions {
    fn default() -> Self {
        Self {
            enum_format: ApiDocsEnumFormat::default(),
            exclude_aliases: false,
            frontmatter: BTreeMap::new(),
            include_index: true,
            index_page: Some("index.md".into()),
            mark_struct_fields_required: true,
            render_anchor: Box::new(default_anchor_renderer),
            render_link: Box::new(default_link_renderer),
            render_description: Box::new(default_description_renderer),
            render_tags: Box::new(default_tags_renderer),
        }
    }
}

/// A rendered type expression. Kept in pieces rather than as a string so that
/// references to other pages can become links, while everything around them
/// is rendered as inline code.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeExpression {
    parts: Vec<TypePart>,
    is_union: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum TypePart {
    Code(String),
    Reference(String),
}

impl TypeExpression {
    /// Create an expression from a piece of code.
    pub fn code(value: impl Into<String>) -> Self {
        let mut expr = Self::default();
        expr.push_code(value);
        expr
    }

    /// Create an expression that refers to another type by name.
    pub fn reference(name: impl Into<String>) -> Self {
        Self {
            parts: vec![TypePart::Reference(name.into())],
            is_union: false,
        }
    }

    /// Join expressions with a separator.
    pub fn join<I>(exprs: I, separator: &str) -> Self
    where
        I: IntoIterator<Item = TypeExpression>,
    {
        let mut out = Self::default();

        for (index, expr) in exprs.into_iter().enumerate() {
            if index > 0 {
                out.push_code(separator);
            }

            out.extend(expr);
        }

        out
    }

    /// Append a piece of code.
    pub fn push_code(&mut self, value: impl Into<String>) {
        let value = value.into();

        if value.is_empty() {
            return;
        }

        // Merge onto the previous piece of code so that the rendered output is
        // a single code span, instead of one per piece
        if let Some(TypePart::Code(last)) = self.parts.last_mut() {
            last.push_str(&value);
        } else {
            self.parts.push(TypePart::Code(value));
        }
    }

    /// Append another expression.
    pub fn extend(&mut self, other: TypeExpression) {
        for part in other.parts {
            match part {
                TypePart::Code(code) => self.push_code(code),
                TypePart::Reference(name) => self.parts.push(TypePart::Reference(name)),
            }
        }
    }

    /// Wrap the expression in parentheses.
    pub fn parenthesize(self) -> Self {
        let mut out = Self::code("(");
        out.extend(self);
        out.push_code(")");
        out
    }

    /// Return true if the expression is a union of types.
    pub fn is_union(&self) -> bool {
        self.is_union
    }

    /// Return true if the expression is nothing but a reference to a type.
    pub fn is_reference(&self) -> bool {
        matches!(self.parts.as_slice(), [TypePart::Reference(_)])
    }

    /// The types the expression refers to, in order of first appearance.
    pub fn references(&self) -> Vec<&str> {
        let mut names: Vec<&str> = vec![];

        for part in &self.parts {
            if let TypePart::Reference(name) = part
                && !names.contains(&name.as_str())
            {
                names.push(name);
            }
        }

        names
    }
}

/// The expression as plain text, with references reduced to their names.
impl fmt::Display for TypeExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in &self.parts {
            match part {
                TypePart::Code(value) => write!(f, "{value}")?,
                TypePart::Reference(name) => write!(f, "{name}")?,
            }
        }

        Ok(())
    }
}

fn lit_to_string(lit: &LiteralValue) -> String {
    match lit {
        LiteralValue::Bool(inner) => inner.to_string(),
        LiteralValue::F32(inner) => inner.to_string(),
        LiteralValue::F64(inner) => inner.to_string(),
        LiteralValue::Int(inner) => inner.to_string(),
        LiteralValue::UInt(inner) => inner.to_string(),
        LiteralValue::String(inner) => format!("\"{inner}\""),
    }
}

/// Render a value as inline code. A pipe inside a table cell ends the cell,
/// even within a code span, so it has to be escaped.
fn code(value: impl AsRef<str>) -> String {
    format!("`{}`", value.as_ref().replace('|', "\\|"))
}

/// Render a list of values as inline code, separated by commas.
fn code_list<I, V>(values: I) -> String
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    values.into_iter().map(code).collect::<Vec<_>>().join(", ")
}

/// The first paragraph of a description on a single line, for a table cell.
fn summarize(description: &str) -> String {
    description
        .trim()
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

/// A row of the index that heads a page.
struct IndexRow {
    name: String,
    ty: String,
    description: Option<String>,
}

/// Remove `null` from a nullable schema, so that nullability is reported once
/// through a tag instead of repeated in every type expression. A union that
/// is left with a single variant collapses to that variant.
fn strip_null(schema: &Schema) -> Cow<'_, Schema> {
    let SchemaType::Union(uni) = &schema.ty else {
        return Cow::Borrowed(schema);
    };

    if !uni.has_null() {
        return Cow::Borrowed(schema);
    }

    let mut variants_types = vec![];
    let mut variants_names = vec![];

    for (index, variant) in uni.variants_types.iter().enumerate() {
        if variant.is_null() {
            continue;
        }

        variants_types.push(variant.clone());

        if let Some(name) = uni.get_variant_name(index) {
            variants_names.push(name.to_owned());
        }
    }

    match variants_types.len() {
        0 => Cow::Owned(Schema::null()),
        1 => Cow::Owned(*variants_types.into_iter().next().unwrap()),
        _ => {
            let mut stripped = schema.clone();
            stripped.ty = SchemaType::Union(Box::new(UnionType {
                // Names travel by position, so only keep them if every kept
                // variant still has one
                variants_names: (variants_names.len() == variants_types.len())
                    .then_some(variants_names),
                variants_types,
                ..(**uni).clone()
            }));
            Cow::Owned(stripped)
        }
    }
}

/// Renders markdown API documentation from a schema, one page per type.
#[derive(Default)]
pub struct ApiDocsRenderer {
    options: ApiDocsOptions,
    references: HashSet<String>,

    // Types the current page linked to, listed under "References"
    linked: BTreeSet<String>,
}

impl ApiDocsRenderer {
    pub fn new(options: ApiDocsOptions) -> Self {
        Self {
            options,
            references: HashSet::default(),
            linked: BTreeSet::default(),
        }
    }

    /// Render a page for every schema in the generator into the directory,
    /// each named after its type with a `.md` extension, along with the
    /// index page. Every schema is known to every page, so references link
    /// to each other the same as they do through [`SchemaGenerator::generate`].
    pub fn generate_all<P: AsRef<Path>>(
        &mut self,
        generator: &SchemaGenerator,
        output_dir: P,
    ) -> miette::Result<()> {
        let output_dir = output_dir.as_ref();
        let schemas = &generator.schemas;

        if schemas.is_empty() {
            return Err(miette!(
                "At least one type must be added to the generator to render API docs."
            ));
        }

        fs::create_dir_all(output_dir).into_diagnostic()?;

        let write = |file_name: String, mut output: String| -> miette::Result<()> {
            output.push('\n');
            fs::write(output_dir.join(file_name), output).into_diagnostic()
        };

        for name in schemas.keys() {
            write(format!("{name}.md"), self.render_page_of(schemas, name)?)?;
        }

        if let Some(index_page) = self.options.index_page.clone() {
            write(index_page, self.render_index_page(schemas)?)?;
        }

        Ok(())
    }

    /// Render the index page, a table of every schema in name order that
    /// links to its page, with its kind and the first paragraph of its
    /// description.
    pub fn render_index_page(&mut self, schemas: &IndexMap<String, Schema>) -> RenderResult {
        self.references = HashSet::from_iter(schemas.keys().cloned());
        self.linked = BTreeSet::default();

        let mut out = vec![
            self.render_frontmatter("Index"),
            "## Types".to_owned(),
            "| Type | Kind | Description |\n| --- | --- | --- |".to_owned(),
        ];

        let mut rows = vec![];

        for (name, schema) in BTreeMap::from_iter(schemas) {
            let kind = match &schema.ty {
                SchemaType::Struct(_) => "Struct".to_owned(),
                SchemaType::Enum(_) => "Enum".to_owned(),
                SchemaType::Union(_) => "Union".to_owned(),
                _ => {
                    let expr = self.render_schema_without_reference(schema)?;

                    self.render_type_expression(&expr)
                }
            };

            rows.push(format!(
                "| {} | {kind} | {} |",
                self.create_link(name),
                schema
                    .description
                    .as_deref()
                    .map(summarize)
                    .unwrap_or_default(),
            ));
        }

        // Rows follow the header directly, with no blank line between
        let table = out.pop().unwrap();
        out.push(format!("{table}\n{}", rows.join("\n")));

        Ok(out.join("\n\n"))
    }

    /// Render the page of one schema, with every schema known as a reference.
    fn render_page_of(&mut self, schemas: &IndexMap<String, Schema>, name: &str) -> RenderResult {
        let Some(schema) = schemas.get(name) else {
            return Err(miette!(
                "No schema named `{name}` has been added to the generator."
            ));
        };

        self.references = HashSet::from_iter(schemas.keys().cloned());
        self.linked = BTreeSet::default();

        self.render_page(name, schema)
    }

    fn create_link(&self, name: &str) -> String {
        (self.options.render_link)(name)
    }

    /// Render a type expression as a single code span, or as a link when it
    /// is nothing but a reference. A link cannot sit inside a code span, so
    /// the types a composite expression refers to are linked separately by
    /// [`Self::render_type_references`].
    fn render_type_expression(&self, expr: &TypeExpression) -> String {
        match expr.references().as_slice() {
            [name] if expr.is_reference() => self.create_link(name),
            _ => code(expr.to_string()),
        }
    }

    /// Links to the types a composite expression refers to, or `None` when
    /// it refers to nothing, or is itself rendered as a link.
    fn render_type_references(&self, expr: &TypeExpression) -> Option<String> {
        if expr.is_reference() {
            return None;
        }

        let links = expr
            .references()
            .into_iter()
            .map(|name| self.create_link(name))
            .collect::<Vec<_>>();

        if links.is_empty() {
            None
        } else {
            Some(links.join(", "))
        }
    }

    /// The rows describing a type in a table: the type itself, and the
    /// types it refers to when they cannot be linked from the type.
    fn type_rows(&self, label: &'static str, expr: &TypeExpression) -> Vec<(&'static str, String)> {
        let mut rows = vec![(label, self.render_type_expression(expr))];

        if let Some(references) = self.render_type_references(expr) {
            rows.push(("References", references));
        }

        rows
    }

    fn render_table(&self, rows: Vec<(&str, String)>) -> String {
        let mut out = vec![
            "| Attribute | Value |".to_owned(),
            "| --- | --- |".to_owned(),
        ];

        for (key, value) in rows {
            out.push(format!("| {key} | {value} |"));
        }

        out.join("\n")
    }

    fn render_tags(&self, tags: Vec<ApiDocsTag>) -> Option<String> {
        if tags.is_empty() {
            return None;
        }

        Some((self.options.render_tags)(&tags)).filter(|out| !out.is_empty())
    }

    fn render_description(&self, description: &str) -> Option<String> {
        Some((self.options.render_description)(description)).filter(|out| !out.is_empty())
    }

    /// The tags of an enum variant.
    fn variant_tags(&self, variant: &SchemaField, is_default: bool) -> Vec<ApiDocsTag> {
        let mut tags = vec![];

        if is_default {
            tags.push(ApiDocsTag::Default);
        }

        if let Some(deprecated) = variant
            .deprecated
            .as_deref()
            .or(variant.schema.deprecated.as_deref())
        {
            tags.push(self.deprecated_tag(deprecated));
        }

        tags
    }

    /// Tags as a comma separated list of labels, for a table cell. A block
    /// quote from `render_tags` cannot sit inside a cell, so it isn't used.
    fn render_inline_tags(&self, tags: &[ApiDocsTag]) -> String {
        tags.iter()
            .map(|tag| match tag {
                ApiDocsTag::Deprecated(Some(message)) => {
                    format!("**{}** ({})", tag.label(), message.trim())
                }
                _ => format!("**{}**", tag.label()),
            })
            .collect::<Vec<_>>()
            .join(", ")
            .replace('|', "\\|")
    }

    fn deprecated_tag(&self, message: &str) -> ApiDocsTag {
        let message = message.trim();

        ApiDocsTag::Deprecated((!message.is_empty()).then(|| message.to_owned()))
    }

    /// Render the literal values an enumerable type accepts, for both
    /// [`EnumType`]s and scalars constrained with `enum_values`.
    fn render_enum_values(&self, schema: &Schema) -> Option<String> {
        let values = match &schema.ty {
            SchemaType::Enum(enu) => match &enu.variants {
                Some(variants) => variants
                    .values()
                    .filter(|variant| !variant.hidden)
                    .filter_map(|variant| match &variant.schema.ty {
                        SchemaType::Literal(lit) => Some(lit_to_string(&lit.value)),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                None => enu.values.iter().map(lit_to_string).collect(),
            },
            SchemaType::Float(float) => float
                .enum_values
                .as_ref()?
                .iter()
                .map(|value| value.to_string())
                .collect(),
            SchemaType::Integer(integer) => integer
                .enum_values
                .as_ref()?
                .iter()
                .map(|value| value.to_string())
                .collect(),
            SchemaType::String(string) => string
                .enum_values
                .as_ref()?
                .iter()
                .map(|value| format!("\"{value}\""))
                .collect(),
            _ => return None,
        };

        if values.is_empty() {
            None
        } else {
            Some(code_list(values))
        }
    }

    /// Rows describing the constraints a type carries, beyond its shape.
    fn create_constraint_rows(&self, schema: &Schema) -> Vec<(&'static str, String)> {
        let mut rows = vec![];

        let mut push_some = |key, value: Option<String>| {
            if let Some(value) = value {
                rows.push((key, value));
            }
        };

        match &schema.ty {
            SchemaType::Array(array) => {
                push_some("Min items", array.min_length.map(|v| code(v.to_string())));
                push_some("Max items", array.max_length.map(|v| code(v.to_string())));
                push_some("Unique items", array.unique.map(|v| code(v.to_string())));
            }
            SchemaType::Float(float) => {
                push_some("Format", float.format.as_ref().map(code));
                push_some("Minimum", float.min.map(|v| code(v.to_string())));
                push_some(
                    "Exclusive minimum",
                    float.min_exclusive.map(|v| code(v.to_string())),
                );
                push_some("Maximum", float.max.map(|v| code(v.to_string())));
                push_some(
                    "Exclusive maximum",
                    float.max_exclusive.map(|v| code(v.to_string())),
                );
                push_some(
                    "Multiple of",
                    float.multiple_of.map(|v| code(v.to_string())),
                );
            }
            SchemaType::Integer(integer) => {
                push_some("Format", integer.format.as_ref().map(code));
                push_some("Minimum", integer.min.map(|v| code(v.to_string())));
                push_some(
                    "Exclusive minimum",
                    integer.min_exclusive.map(|v| code(v.to_string())),
                );
                push_some("Maximum", integer.max.map(|v| code(v.to_string())));
                push_some(
                    "Exclusive maximum",
                    integer.max_exclusive.map(|v| code(v.to_string())),
                );
                push_some(
                    "Multiple of",
                    integer.multiple_of.map(|v| code(v.to_string())),
                );
            }
            SchemaType::Object(object) => {
                push_some(
                    "Min properties",
                    object.min_length.map(|v| code(v.to_string())),
                );
                push_some(
                    "Max properties",
                    object.max_length.map(|v| code(v.to_string())),
                );
            }
            SchemaType::String(string) => {
                push_some("Format", string.format.as_ref().map(code));
                push_some("Pattern", string.pattern.as_ref().map(code));
                push_some("Min length", string.min_length.map(|v| code(v.to_string())));
                push_some("Max length", string.max_length.map(|v| code(v.to_string())));
            }
            _ => {}
        };

        rows
    }

    /// Render the index heading a page, one row per section, each linking to
    /// the section it summarizes.
    fn render_index(&self, kind: &str, rows: &[IndexRow]) -> Option<String> {
        if !self.options.include_index || rows.is_empty() {
            return None;
        }

        let mut out = vec![
            "## Index".to_owned(),
            String::new(),
            format!("| {kind} | Type | Description |"),
            "| --- | --- | --- |".to_owned(),
        ];

        for row in rows {
            out.push(format!(
                "| [{}](#{}) | {} | {} |",
                code(&row.name),
                (self.options.render_anchor)(&row.name),
                row.ty,
                row.description
                    .as_deref()
                    .map(summarize)
                    .unwrap_or_default(),
            ));
        }

        Some(out.join("\n"))
    }

    /// Render a field's type, without the `null` its nullable tag reports.
    fn render_field_type(&mut self, field: &SchemaField) -> RenderResult {
        let schema = strip_null(&field.schema);
        let expr = self.render_schema(&schema)?;

        Ok(self.render_type_expression(&expr))
    }

    /// Render the table describing a field's type: its shape, default,
    /// accepted values, and constraints.
    fn render_field_table(&mut self, field: &SchemaField) -> RenderResult {
        let schema = strip_null(&field.schema);
        let expr = self.render_schema(&schema)?;
        let mut rows = self.type_rows("Type", &expr);

        if let Some(default) = field.schema.get_default() {
            rows.push(("Default", code(lit_to_string(default))));
        }

        if let Some(values) = self.render_enum_values(&schema) {
            rows.push(("Values", values));
        }

        rows.extend(self.create_constraint_rows(&schema));

        if !self.options.exclude_aliases && !field.aliases.is_empty() {
            rows.push(("Aliases", code_list(&field.aliases)));
        }

        if let Some(env_var) = &field.env_var
            && !env_var.is_empty()
        {
            rows.push(("Environment variable", code(env_var)));
        }

        Ok(self.render_table(rows))
    }

    fn render_property(&mut self, name: &str, field: &SchemaField) -> RenderResult {
        let mut out = vec![format!("### {}", code(name))];
        let mut tags = vec![];

        let nullable = field.nullable || field.schema.is_nullable();

        if field.optional {
            tags.push(ApiDocsTag::Optional);
        }
        // A missing `Option` deserializes as `None`, and a flattened field has
        // no key of its own, so neither can be required
        else if self.options.mark_struct_fields_required && !nullable && !field.flatten {
            tags.push(ApiDocsTag::Required);
        }

        if nullable {
            tags.push(ApiDocsTag::Nullable);
        }

        if let Some(deprecated) = &field.deprecated {
            tags.push(self.deprecated_tag(deprecated));
        }

        if field.read_only {
            tags.push(ApiDocsTag::ReadOnly);
        }

        if field.write_only {
            tags.push(ApiDocsTag::WriteOnly);
        }

        if field.flatten {
            tags.push(ApiDocsTag::Flattened);
        }

        if let Some(tags) = self.render_tags(tags) {
            out.push(tags);
        }

        if let Some(comment) = field
            .comment
            .as_deref()
            .and_then(|comment| self.render_description(comment))
        {
            out.push(comment);
        }

        out.push(self.render_field_table(field)?);

        Ok(out.join("\n\n"))
    }

    fn render_struct_page(&mut self, structure: &StructType) -> RenderResult {
        let mut index = vec![];
        let mut sections = vec![];

        for (name, field) in structure.sorted_fields() {
            if field.hidden {
                continue;
            }

            index.push(IndexRow {
                name: name.to_owned(),
                ty: self.render_field_type(field)?,
                description: field.comment.clone(),
            });

            sections.push(self.render_property(name, field)?);
        }

        Ok(self.assemble_sections("Property", "## Properties", index, sections))
    }

    /// Assemble a page body from its index and sections, which is empty when
    /// there are no sections to list.
    fn assemble_sections(
        &self,
        kind: &str,
        heading: &str,
        index: Vec<IndexRow>,
        sections: Vec<String>,
    ) -> String {
        if sections.is_empty() {
            return String::new();
        }

        let mut out = vec![];

        if let Some(index) = self.render_index(kind, &index) {
            out.push(index);
        }

        out.push(heading.to_owned());
        out.extend(sections);

        out.join("\n\n")
    }

    fn render_variant(
        &mut self,
        name: &str,
        variant: &SchemaField,
        is_default: bool,
    ) -> RenderResult {
        let mut out = vec![format!("### {}", code(name))];

        if let Some(tags) = self.render_tags(self.variant_tags(variant, is_default)) {
            out.push(tags);
        }

        if let Some(comment) = variant
            .comment
            .as_deref()
            .or(variant.schema.description.as_deref())
            .and_then(|comment| self.render_description(comment))
        {
            out.push(comment);
        }

        // A unit variant is its value, while a fallback variant accepts a type
        let key = if matches!(variant.schema.ty, SchemaType::Literal(_)) {
            "Value"
        } else {
            "Type"
        };

        let expr = self.render_schema(&variant.schema)?;
        let mut rows = self.type_rows(key, &expr);

        rows.extend(self.create_constraint_rows(&variant.schema));

        out.push(self.render_table(rows));

        Ok(out.join("\n\n"))
    }

    fn render_enum_page(&mut self, enu: &EnumType) -> RenderResult {
        // Variants without names are their values
        let variants = match &enu.variants {
            Some(variants) => variants
                .iter()
                .map(|(name, variant)| (name.to_owned(), (**variant).clone()))
                .collect::<Vec<_>>(),
            None => enu
                .values
                .iter()
                .map(|value| {
                    let name = match value {
                        LiteralValue::String(inner) => inner.to_owned(),
                        other => other.to_string(),
                    };

                    (name, SchemaField::new(Schema::literal_value(value.clone())))
                })
                .collect(),
        };

        if self.options.enum_format == ApiDocsEnumFormat::Table {
            return self.render_enum_table(enu, &variants);
        }

        let mut index = vec![];
        let mut sections = vec![];

        for (position, (name, variant)) in variants.iter().enumerate() {
            if variant.hidden {
                continue;
            }

            let expr = self.render_schema(&variant.schema)?;

            index.push(IndexRow {
                name: name.to_owned(),
                ty: self.render_type_expression(&expr),
                description: variant
                    .comment
                    .clone()
                    .or_else(|| variant.schema.description.clone()),
            });

            sections.push(self.render_variant(
                name,
                variant,
                enu.default_index == Some(position),
            )?);
        }

        Ok(self.assemble_sections("Variant", "## Variants", index, sections))
    }

    /// The variants of a unit enum as one table, a row per variant. A
    /// fallback variant, which accepts a type rather than a value, shows
    /// that type in the value column.
    fn render_enum_table(
        &mut self,
        enu: &EnumType,
        variants: &[(String, SchemaField)],
    ) -> RenderResult {
        let mut rows = vec![];

        for (position, (name, variant)) in variants.iter().enumerate() {
            if variant.hidden {
                continue;
            }

            let expr = self.render_schema(&variant.schema)?;
            let tags = self.variant_tags(variant, enu.default_index == Some(position));
            let description = variant
                .comment
                .as_deref()
                .or(variant.schema.description.as_deref())
                .map(summarize)
                .unwrap_or_default();

            rows.push(format!(
                "| {} | {} | {} | {} |",
                code(name),
                self.render_type_expression(&expr),
                self.render_inline_tags(&tags),
                description,
            ));
        }

        if rows.is_empty() {
            return Ok(String::new());
        }

        Ok(format!(
            "## Variants\n\n| Variant | Value | Tags | Description |\n| --- | --- | --- | --- |\n{}",
            rows.join("\n")
        ))
    }

    /// A page for a union, such as an untagged enum, which lists each variant.
    /// A variant is headed by its name when the union was derived from an
    /// enum, and by its type otherwise.
    fn render_union_page(&mut self, uni: &UnionType) -> RenderResult {
        let mut index = vec![];
        let mut sections = vec![];

        for (position, variant) in uni.variants_types.iter().enumerate() {
            // Reported through the page's nullable tag instead
            if variant.is_null() {
                continue;
            }

            let expr = self.render_schema(variant)?;
            let heading = match uni.get_variant_name(position) {
                Some(name) => name.to_owned(),
                None => expr.to_string(),
            };

            index.push(IndexRow {
                name: heading.clone(),
                ty: self.render_type_expression(&expr),
                description: variant.description.clone(),
            });

            let mut section = vec![format!("### {}", code(heading))];
            let mut tags = vec![];

            if uni.default_index == Some(position) {
                tags.push(ApiDocsTag::Default);
            }

            if let Some(deprecated) = &variant.deprecated {
                tags.push(self.deprecated_tag(deprecated));
            }

            if let Some(tags) = self.render_tags(tags) {
                section.push(tags);
            }

            if let Some(description) = variant
                .description
                .as_deref()
                .and_then(|description| self.render_description(description))
            {
                section.push(description);
            }

            let mut rows = self.type_rows("Type", &expr);

            if let Some(default) = variant.get_default() {
                rows.push(("Default", code(lit_to_string(default))));
            }

            if let Some(values) = self.render_enum_values(variant) {
                rows.push(("Values", values));
            }

            rows.extend(self.create_constraint_rows(variant));

            section.push(self.render_table(rows));

            sections.push(section.join("\n\n"));
        }

        Ok(self.assemble_sections("Variant", "## Variants", index, sections))
    }

    /// A page for a type that is neither a struct, an enum, nor a union, such
    /// as an aliased scalar, which describes the type as a whole.
    fn render_type_page(&mut self, schema: &Schema) -> RenderResult {
        let stripped = strip_null(schema);
        let expr = self.render_schema_without_reference(&stripped)?;

        let mut rows = self.type_rows("Type", &expr);

        if let Some(default) = schema.get_default() {
            rows.push(("Default", code(lit_to_string(default))));
        }

        if let Some(values) = self.render_enum_values(&stripped) {
            rows.push(("Values", values));
        }

        rows.extend(self.create_constraint_rows(&stripped));

        Ok(format!("## Type\n\n{}", self.render_table(rows)))
    }

    fn render_frontmatter(&self, name: &str) -> String {
        let title = self
            .options
            .frontmatter
            .get("title")
            .map(String::as_str)
            .unwrap_or(name);
        let mut lines = vec!["---".to_owned(), format!("title: {title}")];

        // Sorted so that the output is stable no matter how the map iterates
        for (key, value) in &self.options.frontmatter {
            if key != "title" {
                lines.push(format!("{key}: {value}"));
            }
        }

        lines.push("---".to_owned());
        lines.join("\n")
    }

    fn render_page(&mut self, name: &str, schema: &Schema) -> RenderResult {
        let mut out = vec![self.render_frontmatter(name)];
        let mut tags = vec![];

        if let Some(deprecated) = &schema.deprecated {
            tags.push(self.deprecated_tag(deprecated));
        }

        if schema.nullable || schema.is_nullable() {
            tags.push(ApiDocsTag::Nullable);
        }

        if let Some(tags) = self.render_tags(tags) {
            out.push(tags);
        }

        if let Some(description) = schema
            .description
            .as_deref()
            .and_then(|description| self.render_description(description))
        {
            out.push(description);
        }

        let body = match &schema.ty {
            SchemaType::Struct(inner) => self.render_struct_page(inner)?,
            SchemaType::Enum(inner) => self.render_enum_page(inner)?,
            SchemaType::Union(inner) => self.render_union_page(inner)?,
            _ => self.render_type_page(schema)?,
        };

        if !body.is_empty() {
            out.push(body);
        }

        // A recursive type links to itself, which is not a reference to
        // another page
        self.linked.remove(name);

        if !self.linked.is_empty() {
            let links = self
                .linked
                .iter()
                .map(|name| format!("- {}", self.create_link(name)))
                .collect::<Vec<_>>();

            out.push(format!("## References\n\n{}", links.join("\n")));
        }

        Ok(out.join("\n\n"))
    }
}

impl SchemaRenderer<TypeExpression> for ApiDocsRenderer {
    fn is_reference(&self, name: &str) -> bool {
        self.references.contains(name)
    }

    fn render_array(
        &mut self,
        array: &ArrayType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        let mut expr = self.render_schema(&array.items_type)?;

        if expr.is_union() {
            expr = expr.parenthesize();
        }

        expr.push_code("[]");

        Ok(expr)
    }

    fn render_boolean(
        &mut self,
        _boolean: &BooleanType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code("boolean"))
    }

    fn render_enum(&mut self, enu: &EnumType, schema: &Schema) -> RenderResult<TypeExpression> {
        // Map using variants instead of values (when available),
        // so that the fallback variant is included
        let variants_types = match &enu.variants {
            Some(variants) => variants
                .values()
                .filter(|variant| !variant.hidden)
                .map(|variant| variant.schema.clone())
                .collect::<Vec<_>>(),
            None => enu
                .values
                .iter()
                .map(|value| Schema::literal_value(value.clone()))
                .collect(),
        };

        self.render_union(&UnionType::new_any(variants_types), schema)
    }

    fn render_float(
        &mut self,
        _float: &FloatType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code("number"))
    }

    fn render_integer(
        &mut self,
        _integer: &IntegerType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code("number"))
    }

    fn render_literal(
        &mut self,
        literal: &LiteralType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code(lit_to_string(&literal.value)))
    }

    fn render_null(&mut self, _schema: &Schema) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code("null"))
    }

    fn render_object(
        &mut self,
        object: &ObjectType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        let mut expr = TypeExpression::code("Record<");
        expr.extend(self.render_schema(&object.key_type)?);
        expr.push_code(", ");
        expr.extend(self.render_schema(&object.value_type)?);
        expr.push_code(">");

        Ok(expr)
    }

    fn render_reference(
        &mut self,
        reference: &str,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        self.linked.insert(reference.to_owned());

        Ok(TypeExpression::reference(reference))
    }

    fn render_string(
        &mut self,
        _string: &StringType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code("string"))
    }

    fn render_struct(
        &mut self,
        structure: &StructType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        let mut fields = vec![];

        for (name, field) in structure.sorted_fields() {
            if field.hidden {
                continue;
            }

            let mut expr =
                TypeExpression::code(format!("{name}{}: ", if field.optional { "?" } else { "" }));
            expr.extend(self.render_schema(&field.schema)?);

            fields.push(expr);
        }

        if fields.is_empty() {
            return Ok(TypeExpression::code("{}"));
        }

        let mut expr = TypeExpression::code("{ ");
        expr.extend(TypeExpression::join(fields, ", "));
        expr.push_code(" }");

        Ok(expr)
    }

    fn render_tuple(
        &mut self,
        tuple: &TupleType,
        _schema: &Schema,
    ) -> RenderResult<TypeExpression> {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        let mut expr = TypeExpression::code("[");
        expr.extend(TypeExpression::join(items, ", "));
        expr.push_code("]");

        Ok(expr)
    }

    fn render_union(&mut self, uni: &UnionType, _schema: &Schema) -> RenderResult<TypeExpression> {
        let mut items = vec![];

        for item in &uni.variants_types {
            items.push(self.render_schema(item)?);
        }

        let mut expr = TypeExpression::join(items, " | ");
        expr.is_union = uni.variants_types.len() > 1;

        Ok(expr)
    }

    fn render_unknown(&mut self, _schema: &Schema) -> RenderResult<TypeExpression> {
        Ok(TypeExpression::code("unknown"))
    }

    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
        // The last schema in the generator is the page to render, and every
        // other schema is a type it may link to
        let Some(name) = schemas.keys().last().cloned() else {
            return Err(miette!(
                "At least one type must be added to the generator to render API docs."
            ));
        };

        self.render_page_of(&schemas, &name)
    }
}
