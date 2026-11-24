use serde::de::{Deserialize, Deserializer, Error};
use serde_path_to_error::Segment;

/// Deserialize a flattened map, which collects every key that the other
/// settings don't. Serde deserializes a flattened `Option` by discarding its
/// error, as a flattened type may be absent, so one invalid value left the
/// whole map `None`, as if nothing was configured. A map is never absent, only
/// empty, so its error is returned instead, prefixed with the path to the value
/// that failed, as the path the loader reports ends at the struct flattening it.
pub fn deserialize_map<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    serde_path_to_error::deserialize(deserializer)
        .map(Some)
        .map_err(|error| {
            // Mirrors how `serde_path_to_error` itself displays an error
            if error
                .path()
                .iter()
                .all(|segment| matches!(segment, Segment::Unknown))
            {
                error.into_inner()
            } else {
                D::Error::custom(format!("{}: {}", error.path(), error.inner()))
            }
        })
}
