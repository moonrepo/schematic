/// Returns true if the value ends in a supported file extension.
pub fn is_source_format(value: &str) -> bool {
    extract_ext(value).is_some_and(|ext| {
        ext == ".json" || ext == ".pkl" || ext == ".toml" || ext == ".yaml" || ext == ".yml"
    })
}

/// Returns true if the value looks like a file, by checking for `file://`,
/// path separators, or supported file extensions.
pub fn is_file_like(value: &str) -> bool {
    value.starts_with("file://")
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.starts_with('.')
        || value.contains('/')
        || value.contains('\\')
        || value.contains('.')
}

/// Returns true if the value looks like a URL, by checking for `http://`, `https://`, or `www`.
pub fn is_url_like(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("www")
}

/// Returns true if the value is a secure URL, by checking for `https://`. This check can be
/// bypassed for localhost URLs.
pub fn is_secure_url(value: &str) -> bool {
    if value.contains("127.0.0.1") || value.contains("//localhost") {
        return true;
    }

    value.starts_with("https://")
}

/// Strip a leading BOM from the string.
pub fn strip_bom(content: &str) -> &str {
    content.trim_start_matches("\u{feff}")
}

/// Extract a file extension from the provided file path or URL.
pub fn extract_ext(value: &str) -> Option<&str> {
    let value = if is_url_like(value)
        && let Some(index) = value.rfind('?')
    {
        &value[0..index]
    } else {
        value
    };

    value.rfind('.').map(|index| &value[index..])
}
