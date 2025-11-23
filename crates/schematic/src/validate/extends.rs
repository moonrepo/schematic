use crate::config::{ExtendsFrom, Path, PathSegment, ValidateError, ValidateResult};
use crate::helpers::*;

/// Validate an `extend` value is either a file path or secure URL.
pub fn extends_string<D, C>(
    value: &str,
    _data: &D,
    _context: &C,
    _finalize: bool,
) -> ValidateResult {
    let is_file = is_file_like(value);
    let is_url = is_url_like(value);

    if !is_url && !is_file {
        return Err(ValidateError::new(
            "only file paths and URLs can be extended",
        ));
    }

    if is_url && !is_secure_url(value) {
        return Err(ValidateError::new("only secure URLs can be extended"));
    }

    Ok(())
}

/// Validate a list of `extend` values are either a file path or secure URL.
pub fn extends_list<D, C>(
    values: &[String],
    data: &D,
    context: &C,
    finalize: bool,
) -> ValidateResult {
    for (i, value) in values.iter().enumerate() {
        if let Err(mut error) = extends_string(value, data, context, finalize) {
            error.path = Path::new(vec![PathSegment::Index(i)]);

            return Err(error);
        }
    }

    Ok(())
}

/// Validate an `extend` value is either a file path or secure URL.
pub fn extends_from<D, C>(
    value: &ExtendsFrom,
    data: &D,
    context: &C,
    finalize: bool,
) -> ValidateResult {
    match value {
        ExtendsFrom::String(string) => extends_string(string, data, context, finalize)?,
        ExtendsFrom::List(list) => extends_list(list, data, context, finalize)?,
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_valid() {
        assert!(extends_string("file.yml", &(), &(), false).is_ok());
        assert!(extends_string("./file.json", &(), &(), false).is_ok());
        assert!(extends_string("../file.yaml", &(), &(), false).is_ok());
        assert!(extends_string("/nested/file.toml", &(), &(), false).is_ok());
        assert!(extends_string("https://domain.com/file.yml", &(), &(), false).is_ok());
        assert!(extends_string("https://domain.com/nested/file.toml", &(), &(), false).is_ok());
        assert!(
            extends_string(
                "https://domain.com/nested/file.toml?query=string",
                &(),
                &(),
                false
            )
            .is_ok()
        );
    }

    #[test]
    fn string_invalid() {
        // no http
        assert!(extends_string("http://domain.com/nested/file.json", &(), &(), false).is_err());
    }
}
