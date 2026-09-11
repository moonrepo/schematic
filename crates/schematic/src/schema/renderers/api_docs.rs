use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::miette;
use schematic_types::*;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::fmt;

/// Options to control the rendered API documentation.
pub struct ApiDocsOptions {
    /// Exclude field aliases from being rendered.
    pub exclude_aliases: bool,

    /// File extension appended to the name of a referenced type when linking
    /// to its page, such as `.md`. Set to an empty string to link to the bare
    /// name, for documentation tools that route on the file name.
    pub link_extension: String,

    /// Tag all non-optional struct fields as required.
    pub mark_struct_fields_required: bool,
}

impl Default for ApiDocsOptions {
    fn default() -> Self {
        Self {
            exclude_aliases: false,
            link_extension: ".md".into(),
            mark_struct_fields_required: true,
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

/// Keep a description as written, so that paragraphs and lists survive.
fn clean_comment(comment: &str) -> String {
    comment
        .trim()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
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

    let variants_types = uni
        .variants_types
        .iter()
        .filter(|variant| !variant.is_null())
        .cloned()
        .collect::<Vec<_>>();

    match variants_types.len() {
        0 => Cow::Owned(Schema::null()),
        1 => Cow::Owned(*variants_types.into_iter().next().unwrap()),
        _ => {
            let mut stripped = schema.clone();
            stripped.ty = SchemaType::Union(Box::new(UnionType {
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

    fn create_link(&self, name: &str) -> String {
        format!("./{name}{}", self.options.link_extension)
    }

    fn render_type_expression(&self, expr: &TypeExpression) -> String {
        let mut out = String::new();

        for part in &expr.parts {
            match part {
                TypePart::Code(value) => out.push_str(&code(value)),
                TypePart::Reference(name) => {
                    out.push_str(&format!("[{}]({})", code(name), self.create_link(name)));
                }
            }
        }

        out
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

    fn render_tags(&self, tags: Vec<String>) -> Option<String> {
        if tags.is_empty() {
            return None;
        }

        Some(format!("> {}", tags.join(" · ")))
    }

    fn deprecated_tag(&self, message: &str) -> String {
        if message.is_empty() {
            "**Deprecated**".to_owned()
        } else {
            format!("**Deprecated** ({})", message.trim())
        }
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

    /// Render the table describing a field's type: its shape, default,
    /// accepted values, and constraints.
    fn render_field_table(&mut self, field: &SchemaField) -> RenderResult {
        let schema = strip_null(&field.schema);
        let expr = self.render_schema(&schema)?;

        let mut rows = vec![("Type", self.render_type_expression(&expr))];

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
            tags.push("**Optional**".to_owned());
        }
        // A missing `Option` deserializes as `None`, and a flattened field has
        // no key of its own, so neither can be required
        else if self.options.mark_struct_fields_required && !nullable && !field.flatten {
            tags.push("**Required**".to_owned());
        }

        if nullable {
            tags.push("**Nullable**".to_owned());
        }

        if let Some(deprecated) = &field.deprecated {
            tags.push(self.deprecated_tag(deprecated));
        }

        if field.read_only {
            tags.push("**Read only**".to_owned());
        }

        if field.write_only {
            tags.push("**Write only**".to_owned());
        }

        if field.flatten {
            tags.push("**Flattened**".to_owned());
        }

        if let Some(tags) = self.render_tags(tags) {
            out.push(tags);
        }

        if let Some(comment) = &field.comment {
            out.push(clean_comment(comment));
        }

        out.push(self.render_field_table(field)?);

        Ok(out.join("\n\n"))
    }

    fn render_struct_page(&mut self, structure: &StructType) -> RenderResult {
        let mut out = vec![];

        for (name, field) in structure.sorted_fields() {
            if field.hidden {
                continue;
            }

            out.push(self.render_property(name, field)?);
        }

        if out.is_empty() {
            return Ok(String::new());
        }

        out.insert(0, "## Properties".to_owned());

        Ok(out.join("\n\n"))
    }

    fn render_variant(
        &mut self,
        name: &str,
        variant: &SchemaField,
        is_default: bool,
    ) -> RenderResult {
        let mut out = vec![format!("### {}", code(name))];
        let mut tags = vec![];

        if is_default {
            tags.push("**Default**".to_owned());
        }

        if let Some(deprecated) = variant
            .deprecated
            .as_ref()
            .or(variant.schema.deprecated.as_ref())
        {
            tags.push(self.deprecated_tag(deprecated));
        }

        if let Some(tags) = self.render_tags(tags) {
            out.push(tags);
        }

        if let Some(comment) = variant
            .comment
            .as_ref()
            .or(variant.schema.description.as_ref())
        {
            out.push(clean_comment(comment));
        }

        // A unit variant is its value, while a fallback variant accepts a type
        let key = if matches!(variant.schema.ty, SchemaType::Literal(_)) {
            "Value"
        } else {
            "Type"
        };

        let expr = self.render_schema(&variant.schema)?;
        let mut rows = vec![(key, self.render_type_expression(&expr))];

        rows.extend(self.create_constraint_rows(&variant.schema));

        out.push(self.render_table(rows));

        Ok(out.join("\n\n"))
    }

    fn render_enum_page(&mut self, enu: &EnumType) -> RenderResult {
        let mut out = vec![];

        match &enu.variants {
            Some(variants) => {
                for (index, (name, variant)) in variants.iter().enumerate() {
                    if variant.hidden {
                        continue;
                    }

                    out.push(self.render_variant(
                        name,
                        variant,
                        enu.default_index == Some(index),
                    )?);
                }
            }
            None => {
                for (index, value) in enu.values.iter().enumerate() {
                    let name = match value {
                        LiteralValue::String(inner) => inner.to_owned(),
                        other => other.to_string(),
                    };

                    out.push(self.render_variant(
                        &name,
                        &SchemaField::new(Schema::literal_value(value.clone())),
                        enu.default_index == Some(index),
                    )?);
                }
            }
        };

        if out.is_empty() {
            return Ok(String::new());
        }

        out.insert(0, "## Variants".to_owned());

        Ok(out.join("\n\n"))
    }

    /// A page for a union, such as an untagged enum, which lists each variant.
    /// Variants have no names of their own, so each is headed by its type.
    fn render_union_page(&mut self, uni: &UnionType) -> RenderResult {
        let mut out = vec![];

        for (index, variant) in uni.variants_types.iter().enumerate() {
            // Reported through the page's nullable tag instead
            if variant.is_null() {
                continue;
            }

            let expr = self.render_schema(variant)?;
            let mut section = vec![format!("### {}", code(expr.to_string()))];
            let mut tags = vec![];

            if uni.default_index == Some(index) {
                tags.push("**Default**".to_owned());
            }

            if let Some(deprecated) = &variant.deprecated {
                tags.push(self.deprecated_tag(deprecated));
            }

            if let Some(tags) = self.render_tags(tags) {
                section.push(tags);
            }

            if let Some(description) = &variant.description {
                section.push(clean_comment(description));
            }

            let mut rows = vec![("Type", self.render_type_expression(&expr))];

            if let Some(default) = variant.get_default() {
                rows.push(("Default", code(lit_to_string(default))));
            }

            if let Some(values) = self.render_enum_values(variant) {
                rows.push(("Values", values));
            }

            rows.extend(self.create_constraint_rows(variant));

            section.push(self.render_table(rows));

            out.push(section.join("\n\n"));
        }

        if out.is_empty() {
            return Ok(String::new());
        }

        out.insert(0, "## Variants".to_owned());

        Ok(out.join("\n\n"))
    }

    /// A page for a type that is neither a struct, an enum, nor a union, such
    /// as an aliased scalar, which describes the type as a whole.
    fn render_type_page(&mut self, schema: &Schema) -> RenderResult {
        let stripped = strip_null(schema);
        let expr = self.render_schema_without_reference(&stripped)?;

        let mut rows = vec![("Type", self.render_type_expression(&expr))];

        if let Some(default) = schema.get_default() {
            rows.push(("Default", code(lit_to_string(default))));
        }

        if let Some(values) = self.render_enum_values(&stripped) {
            rows.push(("Values", values));
        }

        rows.extend(self.create_constraint_rows(&stripped));

        Ok(format!("## Type\n\n{}", self.render_table(rows)))
    }

    fn render_page(&mut self, name: &str, schema: &Schema) -> RenderResult {
        let mut out = vec![format!("---\ntitle: {name}\n---")];
        let mut tags = vec![];

        if let Some(deprecated) = &schema.deprecated {
            tags.push(self.deprecated_tag(deprecated));
        }

        if schema.nullable || schema.is_nullable() {
            tags.push("**Nullable**".to_owned());
        }

        if let Some(tags) = self.render_tags(tags) {
            out.push(tags);
        }

        if let Some(description) = &schema.description {
            out.push(clean_comment(description));
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
                .map(|name| format!("- [{name}]({})", self.create_link(name)))
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
        let Some((name, schema)) = schemas.last() else {
            return Err(miette!(
                "At least one type must be added to the generator to render API docs."
            ));
        };

        self.references = HashSet::from_iter(schemas.keys().cloned());
        self.linked = BTreeSet::default();

        self.render_page(name, schema)
    }
}
