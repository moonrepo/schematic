use std::fmt::{self, Display};

/// Represents all the different forms a path is composed of.
#[derive(Clone, Debug)]
pub enum PathSegment {
    /// List index: `[0]`
    Index(usize),
    /// Map key: `name.`
    Key(String),
    /// Enum variant: `name.`
    Variant(String),
    /// Unknown segment: `?`
    Unknown,
}

/// Represents the path from the struct root to nested a field or field value.
#[derive(Clone, Debug, Default)]
pub struct Path {
    /// List of path segments.
    segments: Vec<PathSegment>,
}

impl Path {
    /// Create a new instance with the provided [`PathSegment`]s.
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }

    /// Create a new instance and append the provided [`PathSegment`]
    /// to the end of the current path.
    pub fn join(&self, segment: PathSegment) -> Self {
        let mut path = self.clone();
        path.segments.push(segment);
        path
    }

    /// Create a new instance and append an `Index` [`PathSegment`]
    /// to the end of the current path.
    pub fn join_index(&self, index: usize) -> Self {
        self.join(PathSegment::Index(index))
    }

    /// Create a new instance and append an `Key` [`PathSegment`]
    /// to the end of the current path.
    pub fn join_key(&self, key: &str) -> Self {
        self.join(PathSegment::Key(key.to_owned()))
    }

    /// Create a new instance and append another [`Path`]
    /// to the end of the current path.
    pub fn join_path(&self, other: &Self) -> Self {
        let mut path = self.clone();
        path.segments.extend(other.segments.clone());
        path
    }

    /// Create a new instance and append an `Variant` [`PathSegment`]
    /// to the end of the current path.
    pub fn join_variant(&self, variant: &str) -> Self {
        self.join(PathSegment::Variant(variant.to_owned()))
    }
}

impl Display for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.segments.is_empty() {
            return formatter.write_str(".");
        }

        let mut separator = "";

        for segment in &self.segments {
            match segment {
                PathSegment::Index(index) => {
                    write!(formatter, "[{}]", index)?;
                }
                PathSegment::Key(key) | PathSegment::Variant(key) => {
                    write!(formatter, "{}{}", separator, key)?;
                }
                PathSegment::Unknown => {
                    write!(formatter, "{}?", separator)?;
                }
            }

            separator = ".";
        }

        Ok(())
    }
}
