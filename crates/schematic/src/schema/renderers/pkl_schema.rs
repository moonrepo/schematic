use crate::schema::{RenderResult, SchemaGenerator, SchemaRenderer};
use convert_case::{Case, Casing};
use indexmap::IndexMap;
use miette::{IntoDiagnostic, miette};
use schematic_types::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::mem;
use std::path::Path;

/// Words that Pkl reserves, which have to be quoted with backticks to be used
/// as an identifier.
const KEYWORDS: [&str; 42] = [
    "abstract",
    "amends",
    "as",
    "case",
    "class",
    "const",
    "delete",
    "else",
    "extends",
    "external",
    "false",
    "fixed",
    "for",
    "function",
    "hidden",
    "if",
    "import",
    "in",
    "is",
    "let",
    "local",
    "module",
    "new",
    "nothing",
    "null",
    "open",
    "out",
    "outer",
    "override",
    "protected",
    "read",
    "record",
    "super",
    "switch",
    "this",
    "throw",
    "trace",
    "true",
    "typealias",
    "unknown",
    "vararg",
    "when",
];

/// Types from Pkl's base module that rendered types refer to. An import or a
/// class with one of these names would shadow it for the whole module.
const BASE_TYPES: [&str; 27] = [
    "Any", "Boolean", "Char", "DataSize", "Duration", "Dynamic", "Float", "Int", "Int8", "Int16",
    "Int32", "List", "Listing", "Map", "Mapping", "Module", "Null", "Number", "Object", "Pair",
    "Regex", "Set", "String", "UInt", "UInt8", "UInt16", "UInt32",
];

/// Options to control the rendered Pkl modules.
pub struct PklSchemaOptions {
    /// Character(s) to use for indentation.
    pub indent_char: String,

    /// Render struct fields as they are declared: a field that is not optional
    /// must be amended, and a default becomes the property's default value.
    /// When disabled, every property of a struct's module is nullable and has
    /// no default, so that a module amending it only outputs the settings it
    /// sets. Classes, such as the payload of an enum variant, are unaffected,
    /// as their value is only valid in full.
    pub mark_struct_fields_required: bool,
}

impl Default for PklSchemaOptions {
    fn default() -> Self {
        Self {
            indent_char: "  ".into(),
            mark_struct_fields_required: true,
        }
    }
}

/// A rendered Pkl type. Nullability is kept apart from the type itself, so
/// that it can be added once, around a whole union, instead of per variant.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PklType {
    value: String,
    nullable: bool,
    union: bool,
}

impl PklType {
    fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            ..Self::default()
        }
    }

    /// Return true if the type accepts `null`.
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Return true if the type is a union of types.
    pub fn is_union(&self) -> bool {
        self.union
    }
}

impl fmt::Display for PklType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.nullable, self.union) {
            (true, true) => write!(f, "({})?", self.value),
            (true, false) => write!(f, "{}?", self.value),
            _ => write!(f, "{}", self.value),
        }
    }
}

/// Join types into a union, marking the default variant with `*`. A single
/// type is returned as is, as only a union can carry a default marker.
fn join_union(members: Vec<(PklType, bool)>, nullable: bool) -> PklType {
    let mut nullable = nullable;
    let mut variants: Vec<(PklType, bool)> = vec![];

    for (ty, is_default) in members {
        nullable = nullable || ty.nullable;

        match variants
            .iter_mut()
            .find(|(other, _)| other.value == ty.value)
        {
            Some((_, other_default)) => *other_default = *other_default || is_default,
            None => variants.push((ty, is_default)),
        };
    }

    match variants.len() {
        0 => PklType {
            value: if nullable { "Null" } else { "nothing" }.into(),
            ..PklType::default()
        },
        1 => {
            let (ty, _) = variants.remove(0);

            PklType { nullable, ..ty }
        }
        _ => PklType {
            value: variants
                .into_iter()
                .map(|(ty, is_default)| {
                    // A default marker applies to a single type, not a union
                    if is_default && !ty.union {
                        format!("*{}", ty.value)
                    } else {
                        ty.value
                    }
                })
                .collect::<Vec<_>>()
                .join("|"),
            nullable,
            union: true,
        },
    }
}

/// Add constraints to a type, such as `String(length <= 10)`.
fn constrain(base: impl Into<String>, constraints: Vec<String>) -> PklType {
    let base = base.into();

    if constraints.is_empty() {
        PklType::new(base)
    } else {
        PklType::new(format!("{base}({})", constraints.join(", ")))
    }
}

fn length_constraint(min: Option<usize>, max: Option<usize>) -> Option<String> {
    match (min, max) {
        (Some(min), Some(max)) if min == max => Some(format!("length == {min}")),
        (Some(min), Some(max)) => Some(format!("length.isBetween({min}, {max})")),
        (Some(min), None) => Some(format!("length >= {min}")),
        (None, Some(max)) => Some(format!("length <= {max}")),
        (None, None) => None,
    }
}

fn bound_constraints(
    min: Option<String>,
    max: Option<String>,
    min_exclusive: Option<String>,
    max_exclusive: Option<String>,
) -> Vec<String> {
    let mut constraints = vec![];

    match (min, max) {
        (Some(min), Some(max)) => constraints.push(format!("isBetween({min}, {max})")),
        (Some(min), None) => constraints.push(format!("this >= {min}")),
        (None, Some(max)) => constraints.push(format!("this <= {max}")),
        (None, None) => {}
    };

    if let Some(min) = min_exclusive {
        constraints.push(format!("this > {min}"));
    }

    if let Some(max) = max_exclusive {
        constraints.push(format!("this < {max}"));
    }

    constraints
}

/// Render a float so that Pkl reads it as a `Float`, which requires a
/// fraction or an exponent. `1` is an `Int`, and fails a `Float` type check.
fn format_float(value: impl fmt::Debug) -> String {
    let value = format!("{value:?}");

    match value.as_str() {
        "NaN" => value,
        "inf" => "Infinity".into(),
        "-inf" => "-Infinity".into(),
        _ if value.contains(['.', 'e', 'E']) => value,
        _ => format!("{value}.0"),
    }
}

/// Render a string literal, escaping everything that would otherwise end it,
/// or begin an escape or an interpolation.
fn quote_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');

    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{{{:x}}}", ch as u32)),
            ch => out.push(ch),
        };
    }

    out.push('"');
    out
}

/// Render a regex pattern as a custom delimited string, in which backslashes
/// are literal, so that the pattern reads as written.
fn quote_pattern(pattern: &str) -> String {
    if pattern.contains(['\n', '\r']) {
        return quote_string(pattern);
    }

    let mut pounds = "#".to_owned();

    // The delimiter must not appear in the pattern, and neither may an escape
    // or an interpolation that uses it
    while pattern.contains(&format!("\"{pounds}")) || pattern.contains(&format!("\\{pounds}")) {
        pounds.push('#');
    }

    format!("{pounds}\"{pattern}\"{pounds}")
}

/// Quote an identifier with backticks when it is not a legal one, such as a
/// keyword, or a name that contains a hyphen.
fn quote_identifier(name: &str) -> String {
    let mut chars = name.chars();

    let legal = chars
        .next()
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_' || ch == '$')
        && chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$')
        && name != "_"
        && !KEYWORDS.contains(&name);

    if legal {
        name.to_owned()
    } else {
        format!("`{name}`")
    }
}

/// Convert a field or variant name into a part of a class name.
fn to_class_part(name: &str) -> String {
    name.to_case(Case::Pascal)
        .chars()
        .filter(|ch| ch.is_alphanumeric() || *ch == '_')
        .collect()
}

/// Remove `null` from a nullable schema that has a single other variant.
fn unwrap_nullable(schema: &Schema) -> &Schema {
    if let SchemaType::Union(uni) = &schema.ty
        && uni.has_null()
    {
        let mut variants = uni
            .variants_types
            .iter()
            .filter(|variant| !variant.is_null());

        if let (Some(variant), None) = (variants.next(), variants.next()) {
            return unwrap_nullable(variant);
        }
    }

    schema
}

/// Return true if the struct is the shape of a `std::time::Duration`, which
/// serde encodes as seconds and nanoseconds. A Pkl `Duration` is decoded into
/// that same shape, so the native type can be used instead.
fn is_duration(structure: &StructType) -> bool {
    let has_field = |name: &str, kind: IntegerKind| {
        structure.fields.get(name).is_some_and(|field| {
            matches!(&unwrap_nullable(&field.schema).ty, SchemaType::Integer(integer) if integer.kind == kind)
        })
    };

    structure.fields.len() == 2
        && has_field("secs", IntegerKind::U64)
        && has_field("nanos", IntegerKind::U32)
}

/// How a struct's module inherits the struct it flattens.
#[derive(Clone, Debug)]
struct Inheritance {
    // Amending keeps the base's type, which a module that declares no
    // properties of its own, and that nothing uses as a type, can do.
    amends: bool,
    base: String,
    field: String,
}

/// State for the module being rendered.
#[derive(Default)]
struct ModuleContext {
    name: String,

    // Whether the module declares a type alias, rather than properties.
    alias: bool,

    // The identifier each imported schema is known by, keyed by its name.
    imports: BTreeMap<String, String>,

    // Classes declared for anonymous structs, in order of discovery.
    classes: Vec<String>,

    // Identifiers that a class cannot take, as something else claims them.
    taken: HashSet<String>,

    // Properties the module declares, which an import cannot share a name with.
    properties: HashSet<String>,

    // Parts of the name given to a class declared at the current position.
    hints: Vec<String>,

    depth: usize,
}

/// Renders a Pkl module for every struct and enum in a schema. A struct
/// becomes a module that declares its fields as properties, and anything else
/// becomes a module that declares a type alias. Modules refer to each other
/// through imports, and a struct that flattens another extends its module.
#[derive(Default)]
pub struct PklSchemaRenderer {
    options: PklSchemaOptions,
    schemas: IndexMap<String, Schema>,

    // How each struct inherits from the struct it flattens, keyed by name.
    inheritance: HashMap<String, Inheritance>,

    // Structs that another module extends, which must be declared `open`.
    extended: HashSet<String>,

    module: ModuleContext,
}

impl PklSchemaRenderer {
    pub fn new(options: PklSchemaOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    /// Render a module for every schema in the generator into the directory,
    /// each named after its type with a `.pkl` extension. Modules import each
    /// other by relative path, so they must stay together.
    pub fn generate_all<P: AsRef<Path>>(
        &mut self,
        generator: &SchemaGenerator,
        output_dir: P,
    ) -> miette::Result<()> {
        let output_dir = output_dir.as_ref();

        if generator.schemas.is_empty() {
            return Err(miette!(
                "At least one type must be added to the generator to render Pkl modules."
            ));
        }

        self.prepare(generator.schemas.clone());

        fs::create_dir_all(output_dir).into_diagnostic()?;

        for name in generator.schemas.keys() {
            let mut output = self.render_module(name)?;
            output.push('\n');

            fs::write(output_dir.join(format!("{name}.pkl")), output).into_diagnostic()?;
        }

        Ok(())
    }

    /// Learn every schema, and decide how each struct inherits the struct it
    /// flattens, which depends on how the other schemas use it.
    fn prepare(&mut self, schemas: IndexMap<String, Schema>) {
        self.schemas = schemas;
        self.inheritance.clear();
        self.extended.clear();

        let mut referenced = HashSet::new();

        for (name, schema) in &self.schemas {
            for reference in self.collect_references(schema) {
                if reference != *name {
                    referenced.insert(reference);
                }
            }
        }

        let mut inheritance = vec![];

        for (name, schema) in &self.schemas {
            let SchemaType::Struct(structure) = &schema.ty else {
                continue;
            };

            let Some((field, base)) = self.find_base(structure) else {
                continue;
            };

            // A module that amends another cannot be used as a type, and
            // cannot declare properties the other module lacks
            let amends = !referenced.contains(name)
                && self.collect_properties(structure, Some(&field)).is_empty();

            inheritance.push((
                name.to_owned(),
                Inheritance {
                    amends,
                    base,
                    field,
                },
            ));
        }

        for (name, inherit) in inheritance {
            if !inherit.amends {
                self.extended.insert(inherit.base.clone());
            }

            self.inheritance.insert(name, inherit);
        }
    }

    /// Names of the schemas a schema refers to, without following them.
    fn collect_references(&self, schema: &Schema) -> Vec<String> {
        let mut references = vec![];
        let mut visit = |child: &Schema| self.collect_reference(child, &mut references);

        match &schema.ty {
            SchemaType::Array(inner) => visit(&inner.items_type),
            SchemaType::Enum(inner) => {
                for variant in inner.variants.iter().flat_map(|variants| variants.values()) {
                    if !variant.hidden {
                        visit(&variant.schema);
                    }
                }
            }
            SchemaType::Object(inner) => {
                visit(&inner.key_type);
                visit(&inner.value_type);
            }
            SchemaType::Struct(inner) => {
                for field in inner.fields.values() {
                    if !field.hidden {
                        visit(&field.schema);
                    }
                }
            }
            SchemaType::Tuple(inner) => {
                for item in &inner.items_types {
                    visit(item);
                }
            }
            SchemaType::Union(inner) => {
                for variant in &inner.variants_types {
                    visit(variant);
                }
            }
            _ => {}
        };

        references
    }

    fn collect_reference(&self, schema: &Schema, references: &mut Vec<String>) {
        match (&schema.name, &schema.ty) {
            (Some(name), _) if self.is_reference(name) => references.push(name.to_owned()),
            (_, SchemaType::Reference { name, .. }) => references.push(name.to_owned()),
            _ => references.extend(self.collect_references(schema)),
        };
    }

    /// Return true if the type alias `from` refers to the type alias `to`,
    /// directly or through other type aliases, or through the classes they
    /// declare. A struct's module is a class of its own, which Pkl resolves
    /// lazily, so a reference to one never forms a cycle.
    fn reaches(&self, from: &str, to: &str, visited: &mut HashSet<String>) -> bool {
        if !visited.insert(from.to_owned()) {
            return false;
        }

        let Some(schema) = self.schemas.get(from) else {
            return false;
        };

        self.collect_references(schema)
            .into_iter()
            .any(|reference| {
                !self.is_struct(&reference)
                    && (reference == to || self.reaches(&reference, to, visited))
            })
    }

    fn is_struct(&self, name: &str) -> bool {
        self.schemas
            .get(name)
            .is_some_and(|schema| schema.is_struct())
    }

    /// The first flattened field that refers to another struct, which the
    /// module inherits from, as `(field, struct)`.
    fn find_base(&self, structure: &StructType) -> Option<(String, String)> {
        structure
            .sorted_fields()
            .into_iter()
            .find_map(|(name, field)| {
                if !field.flatten || field.hidden {
                    return None;
                }

                let base = unwrap_nullable(&field.schema).name.as_ref()?;

                (self.is_reference(base) && self.is_struct(base))
                    .then(|| (name.to_owned(), base.to_owned()))
            })
    }

    /// The properties a struct declares, keyed by name, with the fields of
    /// the structs it flattens folded in. The `inherited` field is left out,
    /// as its properties come from the module that is extended instead.
    fn collect_properties(
        &self,
        structure: &StructType,
        inherited: Option<&str>,
    ) -> BTreeMap<String, SchemaField> {
        let mut properties = BTreeMap::new();

        for (name, field) in structure.sorted_fields() {
            if field.hidden || inherited == Some(name.as_str()) {
                continue;
            }

            if field.flatten {
                let schema = unwrap_nullable(&field.schema);
                let schema = schema
                    .name
                    .as_ref()
                    .and_then(|name| self.schemas.get(name))
                    .unwrap_or(schema);

                // A flattened map collects unknown keys, which a typed object
                // cannot declare, so only a struct's fields are folded in
                if let SchemaType::Struct(inner) = &schema.ty {
                    properties.extend(self.collect_properties(inner, None));
                }

                continue;
            }

            properties.insert(name.to_owned(), field.clone());
        }

        properties
    }

    fn indent(&self) -> String {
        self.options.indent_char.repeat(self.module.depth)
    }

    /// Render the module of the named schema.
    fn render_module(&mut self, name: &str) -> RenderResult {
        let Some(schema) = self.schemas.get(name).cloned() else {
            return Err(miette!(
                "No schema named `{name}` has been added to the generator."
            ));
        };

        let mut taken = HashSet::from_iter(self.schemas.keys().cloned());
        taken.extend(BASE_TYPES.iter().map(|ty| ty.to_string()));

        self.module = ModuleContext {
            name: name.to_owned(),
            alias: !schema.is_struct(),
            taken,
            // Already a type name, so kept as written
            hints: vec![
                name.chars()
                    .filter(|ch| ch.is_alphanumeric() || *ch == '_')
                    .collect(),
            ],
            ..ModuleContext::default()
        };

        let inheritance = self.inheritance.get(name).cloned();
        let mut members = vec![];

        // The module clause, preceded by its documentation. A type alias
        // documents the alias instead, as that is what other modules refer to.
        let mut clause = vec![];

        if let SchemaType::Struct(structure) = &schema.ty {
            clause.extend(self.render_description(schema.description.as_deref()));
            clause.extend(schema.deprecated.as_deref().map(render_deprecated));

            let properties = self.collect_properties(
                structure,
                inheritance.as_ref().map(|inherit| inherit.field.as_str()),
            );

            self.module.properties = properties.keys().cloned().collect();
            self.module.taken.extend(properties.keys().cloned());

            for (name, field) in &properties {
                members.push(self.render_property(name, field, true)?);
            }
        } else {
            let ty = self.render_schema_without_reference(&schema)?;
            let mut alias = self.render_description(schema.description.as_deref());

            alias.extend(schema.deprecated.as_deref().map(render_deprecated));
            alias.push(format!("typealias {} = {ty}", quote_identifier(name)));

            members.push(alias.join("\n"));
        }

        clause.push(format!(
            "{}module {}",
            if self.extended.contains(name) {
                "open "
            } else {
                ""
            },
            quote_identifier(name)
        ));

        let mut sections = vec![
            "// Automatically generated by schematic. DO NOT MODIFY!".to_owned(),
            clause.join("\n"),
        ];

        if let Some(inherit) = inheritance {
            sections.push(format!(
                "{} \"{}.pkl\"",
                if inherit.amends { "amends" } else { "extends" },
                inherit.base
            ));
        }

        if !self.module.imports.is_empty() {
            sections.push(
                self.module
                    .imports
                    .iter()
                    .map(|(name, ident)| {
                        if name == ident {
                            format!("import \"{name}.pkl\"")
                        } else {
                            format!("import \"{name}.pkl\" as {ident}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }

        sections.extend(members);
        sections.extend(mem::take(&mut self.module.classes));

        Ok(sections.join("\n\n"))
    }

    /// Render a doc comment, keeping the lines as written.
    fn render_description(&self, description: Option<&str>) -> Vec<String> {
        let indent = self.indent();

        description
            .map(str::trim)
            .filter(|description| !description.is_empty())
            .map(|description| {
                description
                    .lines()
                    .map(|line| match line.trim() {
                        "" => format!("{indent}///"),
                        line => format!("{indent}/// {line}"),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Render a property, with its documentation. A property of a module has
    /// the module's own members to contend with, which a class property
    /// doesn't.
    fn render_property(
        &mut self,
        name: &str,
        field: &SchemaField,
        in_module: bool,
    ) -> RenderResult {
        let indent = self.indent();

        // Every module already has an `output` property, which configures how
        // it renders, and Pkl fails outright when a module declares another
        if in_module && name == "output" {
            return Ok(format!(
                "{indent}// The `output` setting cannot be declared, as every Pkl module reserves it."
            ));
        }

        self.module.hints.push(to_class_part(name));

        let mut ty = self.render_schema(&field.schema)?;

        self.module.hints.pop();

        let mut default = None;

        // A literal is the tag of an enum variant, which serde requires, and
        // which defaults to the only value it accepts, so it is never null
        if matches!(unwrap_nullable(&field.schema).ty, SchemaType::Literal(_)) {
            ty.nullable = false;
        }
        // Only a config's own settings can be left out. A class holds a value,
        // such as the payload of an enum variant, which serde needs in full.
        else if in_module && !self.options.mark_struct_fields_required {
            ty.nullable = true;
        } else {
            ty.nullable = ty.nullable || field.nullable;

            // A nullable property defaults to null, which leaves the setting
            // to the value the config would have without it
            if !ty.nullable {
                default = self.render_default(field);

                // Pkl outputs every property, so one that may be omitted, but
                // that has no value to fall back on, has to accept null
                if default.is_none() && field.optional {
                    ty.nullable = true;
                }
            }
        }

        let mut lines = self.render_description(field.comment.as_deref());

        if let Some(deprecated) = &field.deprecated {
            lines.push(format!("{indent}{}", render_deprecated(deprecated)));
        }

        lines.push(format!(
            "{indent}{}: {ty}{}",
            quote_identifier(name),
            default
                .map(|default| format!(" = {default}"))
                .unwrap_or_default()
        ));

        Ok(lines.join("\n"))
    }

    /// Render the default value of a field, unless its type already defaults
    /// to that value, such as an enum marking its default variant.
    fn render_default(&self, field: &SchemaField) -> Option<String> {
        let default = field.schema.get_default()?;

        if self.get_marked_default(&field.schema) == Some(default) {
            return None;
        }

        let is_float = matches!(unwrap_nullable(&field.schema).ty, SchemaType::Float(_));

        Some(match default {
            LiteralValue::Bool(inner) => inner.to_string(),
            LiteralValue::F32(inner) => format_float(inner),
            LiteralValue::F64(inner) => format_float(inner),
            LiteralValue::Int(inner) if is_float => format!("{inner}.0"),
            LiteralValue::Int(inner) => inner.to_string(),
            LiteralValue::UInt(inner) if is_float => format!("{inner}.0"),
            LiteralValue::UInt(inner) => inner.to_string(),
            LiteralValue::String(inner) => quote_string(inner),
        })
    }

    /// The default value that a rendered type carries itself, which Pkl
    /// falls back to when a property has no default of its own.
    fn get_marked_default<'a>(&'a self, schema: &'a Schema) -> Option<&'a LiteralValue> {
        let schema = schema
            .name
            .as_ref()
            .and_then(|name| self.schemas.get(name))
            .unwrap_or(schema);

        match &schema.ty {
            SchemaType::Enum(enu) => enu.get_default(),
            SchemaType::Union(uni) => uni
                .default_index
                .and_then(|index| uni.variants_types.get(index))
                .and_then(|variant| variant.get_default()),
            _ => None,
        }
    }

    /// Declare a class for an anonymous struct, named after the position it
    /// was found at, such as `ConfigServer` for the `server` field of
    /// `Config`, and return that name.
    fn render_class(&mut self, structure: &StructType, schema: &Schema) -> RenderResult<String> {
        let base = self.module.hints.concat();
        let mut name = base.clone();
        let mut suffix = 2;

        while self.module.taken.contains(&name) {
            name = format!("{base}{suffix}");
            suffix += 1;
        }

        self.module.taken.insert(name.clone());

        // Reserve the position first, so that a class is declared before the
        // classes its properties declare
        let index = self.module.classes.len();
        self.module.classes.push(String::new());

        let hints = mem::replace(&mut self.module.hints, vec![name.clone()]);
        let depth = mem::replace(&mut self.module.depth, 1);

        let mut members = vec![];

        for (property, field) in &self.collect_properties(structure, None) {
            members.push(self.render_property(property, field, false)?);
        }

        self.module.hints = hints;
        self.module.depth = 0;

        let mut class = self.render_description(schema.description.as_deref());
        class.extend(schema.deprecated.as_deref().map(render_deprecated));

        class.push(if members.is_empty() {
            format!("class {name}")
        } else {
            format!("class {name} {{\n{}\n}}", members.join("\n\n"))
        });

        self.module.depth = depth;
        self.module.classes[index] = class.join("\n");

        Ok(name)
    }

    /// Import the named schema's module, and return the identifier it is
    /// known by. Pkl names an import after its file, unless that name is
    /// already claimed in the module.
    fn import(&mut self, name: &str) -> String {
        if let Some(ident) = self.module.imports.get(name) {
            return ident.to_owned();
        }

        let ident = if BASE_TYPES.contains(&name) || self.module.properties.contains(name) {
            format!("{name}Module")
        } else {
            name.to_owned()
        };

        self.module.imports.insert(name.to_owned(), ident.clone());

        ident
    }
}

fn render_deprecated(message: &str) -> String {
    let message = message.trim();

    if message.is_empty() {
        "@Deprecated".into()
    } else {
        format!("@Deprecated {{ message = {} }}", quote_string(message))
    }
}

impl SchemaRenderer<PklType> for PklSchemaRenderer {
    fn is_reference(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    fn render_schema(&mut self, schema: &Schema) -> RenderResult<PklType> {
        if let Some(name) = &schema.name
            && self.is_reference(name)
        {
            // An internally tagged variant adds its tag to the struct it
            // holds, which is then a different shape to the struct's module,
            // so it is declared as a class of its own instead
            let is_extended = match (&schema.ty, self.schemas.get(name).map(|other| &other.ty)) {
                (SchemaType::Struct(inner), Some(SchemaType::Struct(other))) => inner
                    .fields
                    .keys()
                    .any(|key| !other.fields.contains_key(key)),
                _ => false,
            };

            if !is_extended {
                return self.render_reference(name, schema);
            }
        }

        self.render_schema_without_reference(schema)
    }

    fn render_array(&mut self, array: &ArrayType, _schema: &Schema) -> RenderResult<PklType> {
        let items = self.render_schema(&array.items_type)?;
        let mut constraints = vec![];

        if array.unique == Some(true) {
            constraints.push("isDistinct".to_owned());
        }

        constraints.extend(length_constraint(array.min_length, array.max_length));

        Ok(constrain(format!("Listing<{items}>"), constraints))
    }

    fn render_boolean(
        &mut self,
        _boolean: &BooleanType,
        _schema: &Schema,
    ) -> RenderResult<PklType> {
        Ok(PklType::new("Boolean"))
    }

    fn render_enum(&mut self, enu: &EnumType, _schema: &Schema) -> RenderResult<PklType> {
        let mut members = vec![];
        let mut nullable = false;

        match &enu.variants {
            // Map using variants instead of values (when available),
            // so that the fallback variant is included
            Some(variants) => {
                for (index, variant) in variants.values().enumerate() {
                    if variant.hidden {
                        continue;
                    }

                    if variant.schema.is_null() {
                        nullable = true;
                        continue;
                    }

                    members.push((
                        self.render_schema(&variant.schema)?,
                        enu.default_index == Some(index),
                    ));
                }
            }
            None => {
                for (index, value) in enu.values.iter().enumerate() {
                    members.push((
                        self.render_literal(&LiteralType::new(value.clone()), &Schema::default())?,
                        enu.default_index == Some(index),
                    ));
                }
            }
        };

        Ok(join_union(members, nullable))
    }

    fn render_float(&mut self, float: &FloatType, _schema: &Schema) -> RenderResult<PklType> {
        let mut constraints = vec![];

        if let Some(values) = &float.enum_values {
            constraints.push(format!(
                "List({}).contains(this)",
                values
                    .iter()
                    .map(format_float)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        constraints.extend(bound_constraints(
            float.min.map(format_float),
            float.max.map(format_float),
            float.min_exclusive.map(format_float),
            float.max_exclusive.map(format_float),
        ));

        if let Some(multiple) = float.multiple_of {
            constraints.push(format!("this % {} == 0.0", format_float(multiple)));
        }

        Ok(constrain("Float", constraints))
    }

    fn render_integer(&mut self, integer: &IntegerType, _schema: &Schema) -> RenderResult<PklType> {
        // Pkl's `Int` is 64-bit, so wider integers are narrowed to it
        let base = match integer.kind {
            IntegerKind::I8 => "Int8",
            IntegerKind::I16 => "Int16",
            IntegerKind::I32 => "Int32",
            IntegerKind::I64 | IntegerKind::I128 | IntegerKind::Isize => "Int",
            IntegerKind::U8 => "UInt8",
            IntegerKind::U16 => "UInt16",
            IntegerKind::U32 => "UInt32",
            IntegerKind::U64 | IntegerKind::U128 | IntegerKind::Usize => "UInt",
        };

        let mut constraints = vec![];

        if let Some(values) = &integer.enum_values {
            constraints.push(format!(
                "List({}).contains(this)",
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        constraints.extend(bound_constraints(
            integer.min.map(|value| value.to_string()),
            integer.max.map(|value| value.to_string()),
            integer.min_exclusive.map(|value| value.to_string()),
            integer.max_exclusive.map(|value| value.to_string()),
        ));

        if let Some(multiple) = integer.multiple_of {
            constraints.push(format!("this % {multiple} == 0"));
        }

        Ok(constrain(base, constraints))
    }

    fn render_literal(&mut self, literal: &LiteralType, _schema: &Schema) -> RenderResult<PklType> {
        // Only strings have literal types, so any other value is a constraint
        Ok(PklType::new(match &literal.value {
            LiteralValue::String(inner) => quote_string(inner),
            LiteralValue::Bool(inner) => format!("Boolean(this == {inner})"),
            LiteralValue::F32(inner) => format!("Float(this == {})", format_float(inner)),
            LiteralValue::F64(inner) => format!("Float(this == {})", format_float(inner)),
            LiteralValue::Int(inner) => format!("Int(this == {inner})"),
            LiteralValue::UInt(inner) => format!("Int(this == {inner})"),
        }))
    }

    fn render_null(&mut self, _schema: &Schema) -> RenderResult<PklType> {
        Ok(PklType::new("Null"))
    }

    fn render_object(&mut self, object: &ObjectType, _schema: &Schema) -> RenderResult<PklType> {
        let key = self.render_schema(&object.key_type)?;
        let value = self.render_schema(&object.value_type)?;

        Ok(constrain(
            format!("Mapping<{key}, {value}>"),
            length_constraint(object.min_length, object.max_length)
                .into_iter()
                .collect(),
        ))
    }

    fn render_reference(&mut self, reference: &str, _schema: &Schema) -> RenderResult<PklType> {
        let is_struct = self.is_struct(reference);

        // Pkl rejects a type alias that refers back to itself, even through a
        // class it declares, so the cycle is broken with the widest type
        if self.module.alias
            && !is_struct
            && (reference == self.module.name
                || self.reaches(reference, &self.module.name, &mut HashSet::new()))
        {
            return Ok(PklType::new("Any"));
        }

        let ident = self.import(reference);

        // A struct's module is its type, while any other module declares a
        // type alias of the same name
        Ok(PklType::new(if is_struct {
            ident
        } else {
            format!("{ident}.{}", quote_identifier(reference))
        }))
    }

    fn render_string(&mut self, string: &StringType, _schema: &Schema) -> RenderResult<PklType> {
        if let Some(values) = &string.enum_values {
            return Ok(join_union(
                values
                    .iter()
                    .map(|value| (PklType::new(quote_string(value)), false))
                    .collect(),
                false,
            ));
        }

        let is_char = string.min_length == Some(1) && string.max_length == Some(1);
        let mut constraints = vec![];

        if !is_char {
            constraints.extend(length_constraint(string.min_length, string.max_length));
        }

        if let Some(pattern) = &string.pattern {
            constraints.push(format!("matches(Regex({}))", quote_pattern(pattern)));
        }

        Ok(constrain(
            if is_char { "Char" } else { "String" },
            constraints,
        ))
    }

    fn render_struct(&mut self, structure: &StructType, schema: &Schema) -> RenderResult<PklType> {
        if is_duration(structure) {
            return Ok(PklType::new("Duration"));
        }

        Ok(PklType::new(self.render_class(structure, schema)?))
    }

    fn render_tuple(&mut self, tuple: &TupleType, _schema: &Schema) -> RenderResult<PklType> {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        // A pair is decoded into a sequence of its two values, the same as a
        // tuple, while a longer tuple can only be typed as a list of its items
        if let [first, second] = items.as_slice() {
            return Ok(PklType::new(format!("Pair<{first}, {second}>")));
        }

        let length = items.len();
        let items = join_union(items.into_iter().map(|item| (item, false)).collect(), false);

        Ok(constrain(
            format!("Listing<{items}>"),
            vec![format!("length == {length}")],
        ))
    }

    fn render_union(&mut self, uni: &UnionType, _schema: &Schema) -> RenderResult<PklType> {
        let mut members = vec![];
        let mut nullable = false;

        for (index, variant) in uni.variants_types.iter().enumerate() {
            if variant.is_null() {
                nullable = true;
                continue;
            }

            // A class declared for a variant is named after it
            let hint = uni.get_variant_name(index).map(|name| to_class_part(name));

            if let Some(hint) = &hint {
                self.module.hints.push(hint.to_owned());
            }

            let ty = self.render_schema(variant)?;

            if hint.is_some() {
                self.module.hints.pop();
            }

            members.push((ty, uni.default_index == Some(index)));
        }

        Ok(join_union(members, nullable))
    }

    fn render_unknown(&mut self, _schema: &Schema) -> RenderResult<PklType> {
        Ok(PklType::new("Any"))
    }

    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
        // The last schema in the generator is the module to render, and every
        // other schema is a module it may import
        let Some(name) = schemas.keys().last().cloned() else {
            return Err(miette!(
                "At least one type must be added to the generator to render Pkl modules."
            ));
        };

        self.prepare(schemas);
        self.render_module(&name)
    }
}
