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

/// Returns true if the value looks like a URL, by checking for `http://`, `https://`, or `www.`.
pub fn is_url_like(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("www.")
}

/// Returns true if the value is a secure URL, by checking for an `https://` scheme. A plain
/// `http://` URL is only allowed when its host is the local machine, either `localhost`,
/// `127.0.0.1`, or `[::1]`.
pub fn is_secure_url(value: &str) -> bool {
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };

    if scheme.eq_ignore_ascii_case("https") {
        return true;
    }

    if !scheme.eq_ignore_ascii_case("http") {
        return false;
    }

    // The authority ends at the path, query, or fragment, and may include credentials
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_and_port = authority.rsplit('@').next().unwrap_or_default();

    let host = if let Some(ipv6) = host_and_port.strip_prefix('[') {
        ipv6.split(']').next().unwrap_or_default()
    } else {
        host_and_port.split(':').next().unwrap_or_default()
    };

    host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1"
}

/// Strip a leading BOM from the string.
pub fn strip_bom(content: &str) -> &str {
    content.trim_start_matches("\u{feff}")
}

/// Extract a file name from the provided file path or URL.
pub fn extract_file_name(value: &str) -> &str {
    // Remove any query string
    let value = if let Some(index) = value.rfind('?') {
        &value[0..index]
    } else {
        value
    };

    // And only check the last segment
    if let Some(index) = value.rfind('/') {
        &value[index + 1..]
    } else {
        value
    }
}

/// Extract a file extension (without period) from the provided file path or URL.
pub fn extract_file_ext(value: &str) -> Option<&str> {
    let name = extract_file_name(value);

    name.rfind('.').map(|index| &name[index + 1..])
}

/// The name of a type without its module paths, like `Vec<Config>` rather
/// than `alloc::vec::Vec<app::Config>`, for use within error messages.
#[cfg(feature = "config")]
pub(crate) fn short_type_name<T: ?Sized>() -> String {
    let name = std::any::type_name::<T>();
    let mut short = String::with_capacity(name.len());
    let mut path = String::new();

    // Each path keeps only its last segment, while everything
    // between them, like `<`, `, `, or `&`, is kept as is
    for char in name.chars().chain(std::iter::once('\0')) {
        if char.is_alphanumeric() || char == '_' || char == ':' {
            path.push(char);
            continue;
        }

        short.push_str(path.rsplit("::").next().unwrap_or_default());
        path.clear();

        if char != '\0' {
            short.push(char);
        }
    }

    short
}

#[cfg(all(test, feature = "config"))]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    struct Wrapper<T>(T);

    #[test]
    fn shortens_every_path_of_a_type() {
        assert_eq!(short_type_name::<u16>(), "u16");
        assert_eq!(short_type_name::<PathBuf>(), "PathBuf");
        assert_eq!(
            short_type_name::<Wrapper<Option<String>>>(),
            "Wrapper<Option<String>>"
        );
        assert_eq!(
            short_type_name::<HashMap<String, Vec<(u8, &str)>>>(),
            "HashMap<String, Vec<(u8, &str)>>"
        );
        assert_eq!(short_type_name::<dyn std::fmt::Debug>(), "dyn Debug");
    }
}
