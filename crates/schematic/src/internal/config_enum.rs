use crate::config::ConfigError;

/// Normalize a string into the provided case, using the same format names as
/// serde's `rename_all`. Backs `#[config(before_parse)]`, where the incoming
/// value is reshaped before it is matched against a variant.
///
/// The format is validated at derive time, so an unknown one is unreachable
/// from generated code.
pub fn format_case(value: &str, format: &str) -> String {
    use convert_case::{Boundary, Case, Casing};

    let case = match format {
        // `Case::Lower`/`Case::Upper` are space delimited, so these two are
        // handled directly rather than through a case conversion
        "lowercase" => return value.to_lowercase(),
        "UPPERCASE" => return value.to_uppercase(),
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::UpperSnake,
        "kebab-case" => Case::Kebab,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        other => {
            panic!("Unknown `before_parse` value `{other}`.");
        }
    };

    value
        // Unlike the derive-time equivalent this does not hint at an incoming
        // case, since the value came from a user rather than a Rust ident
        .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(case)
}

/// Convert a value that no other variant of a `ConfigEnum` matched into the
/// type its fallback variant holds.
pub fn parse_fallback<'a, T: TryFrom<&'a str>>(value: &'a str) -> Result<T, ConfigError> {
    value
        .try_into()
        .map_err(|_| ConfigError::EnumInvalidFallback(value.to_string()))
}
