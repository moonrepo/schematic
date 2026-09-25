//! Helpers shared by the Pkl renderers.

use schematic_types::*;
use std::fmt;

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

/// Render a float so that Pkl reads it as a `Float`, which requires a
/// fraction or an exponent. `1` is an `Int`, and fails a `Float` type check.
pub fn format_float(value: impl fmt::Debug) -> String {
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
pub fn quote_string(value: &str) -> String {
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

/// Quote an identifier with backticks when it is not a legal one, such as a
/// keyword, or a name that contains a hyphen.
pub fn quote_identifier(name: &str) -> String {
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

/// Remove `null` from a nullable schema that has a single other variant.
pub fn unwrap_nullable(schema: &Schema) -> &Schema {
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
pub fn is_duration(structure: &StructType) -> bool {
    let has_field = |name: &str, kind: IntegerKind| {
        structure.fields.get(name).is_some_and(|field| {
            matches!(&unwrap_nullable(&field.schema).ty, SchemaType::Integer(integer) if integer.kind == kind)
        })
    };

    structure.fields.len() == 2
        && has_field("secs", IntegerKind::U64)
        && has_field("nanos", IntegerKind::U32)
}
