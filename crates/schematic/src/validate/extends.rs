use crate::config::{
    is_file_like, is_secure_url, is_source_format, is_url_like, ExtendsFrom, Path, PathSegment,
    ValidateError,
};

/// Validate an `extend` value is either a file path or secure URL.
pub fn extends_string<D, C>(value: &str, _data: &D, _context: &C) -> Result<(), ValidateError> {
    let is_file = is_file_like(value);
    let is_url = is_url_like(value);

    if !is_url && !is_file {
        return Err(ValidateError::new(
            "only file paths and URLs can be extended",
        ));
    }

    if !value.is_empty() && !is_source_format(value) {
        return Err(ValidateError::new(
            "invalid format, try a supported extension",
        ));
    }

    if is_url && !is_secure_url(value) {
        return Err(ValidateError::new("only secure URLs can be extended"));
    }

    Ok(())
}

/// Validate a list of `extend` values are either a file path or secure URL.
pub fn extends_list<D, C>(values: &[String], data: &D, context: &C) -> Result<(), ValidateError> {
    for (i, value) in values.iter().enumerate() {
        if let Err(mut error) = extends_string(value, data, context) {
            error.path = Path::new(vec![PathSegment::Index(i)]);

            return Err(error);
        }
    }

    Ok(())
}

/// Validate an `extend` value is either a file path or secure URL.
pub fn extends_from<D, C>(value: &ExtendsFrom, data: &D, context: &C) -> Result<(), ValidateError> {
    match value {
        ExtendsFrom::String(string) => extends_string(string, data, context)?,
        ExtendsFrom::List(list) => extends_list(list, data, context)?,
    };

    Ok(())
}
