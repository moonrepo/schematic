use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    Boolean,
    Number,
    String,
    Array(Box<Type>),
    Object(Box<Type>, Box<Type>),
    Tuple(Vec<Box<Type>>),
    Union(Vec<Box<Type>>),
    Reference(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Boolean => write!(f, "boolean"),
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Array(item) => write!(f, "{}[]", item),
            Type::Object(key, value) => write!(f, "Record<{}, {}>", key, value),
            Type::Tuple(items) => write!(
                f,
                "[{}]",
                items
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Union(items) => write!(
                f,
                "{}",
                items
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
            Type::Reference(item) => write!(f, "{}", item),
        }
    }
}

impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Schematic
        if s == "schematic::ExtendsFrom" || s == "ExtendsFrom" {
            return Ok(Type::Union(vec![
                Box::new(Type::String),
                Box::new(Type::Array(Box::new(Type::String))),
            ]));
        }

        // Objects
        for object_prefix in [
            "HashMap",
            "BTreeMap",
            "FxHashMap",
            "std::collections::HashMap",
            "std::collections::hash_map::HashMap",
            "std::collections::BTreeMap",
            "std::collections::btree_map::BTreeMap",
            "rustc_hash::FxHashMap",
        ] {
            if s.starts_with(object_prefix) {
                let inner = &s[(object_prefix.len() + 1)..(s.len() - 1)];
                let comma_index = inner.find(',').unwrap();

                return Ok(Type::Object(
                    Box::new(Type::from_str(inner[0..comma_index].trim())?),
                    Box::new(Type::from_str(inner[(comma_index + 1)..].trim())?),
                ));
            }
        }

        // Arrays
        for array_prefix in [
            "Vec",
            "VecDeque",
            "HashSet",
            "BTreeSet",
            "FxHashSet",
            "alloc::vec::Vec",
            "std::vec::Vec",
            "std::collections::VecDeque",
            "std::collections::vec_deque::VecDeque",
            "std::collections::HashSet",
            "std::collections::hash_set::HashSet",
            "std::collections::BTreeSet",
            "std::collections::btree_set::BTreeSet",
            "rustc_hash::FxHashSet",
        ] {
            if s.starts_with(array_prefix) {
                let inner = &s[(array_prefix.len() + 1)..(s.len() - 1)];

                return Ok(Type::Array(Box::new(Type::from_str(inner)?)));
            }
        }

        // Arrays (native)
        if s.starts_with('[') && s.ends_with(']') {
            let mut inner = s[1..(s.len() - 1)].split(';'); // Remove the fixed length

            return Ok(Type::Array(Box::new(Type::from_str(
                inner.next().unwrap().trim(),
            )?)));
        }

        // Tuples
        if s.starts_with('(') && s.ends_with(')') {
            let mut items = vec![];

            for item in s[1..(s.len() - 1)].split(',') {
                items.push(Box::new(Type::from_str(item.trim())?));
            }

            return Ok(Type::Tuple(items));
        }

        // Strings
        for string_name in [
            "str",
            "String",
            "Path",
            "PathBuf",
            "RelativePath",
            "RelativePathBuf",
            "alloc::str",
            "std::str",
            "std::primitive::str",
            "alloc::string::String",
            "std::string::String",
            "std::path::Path",
            "std::path::PathBuf",
            "relative_path::RelativePath",
            "relative_path::RelativePathBuf",
        ] {
            if s == string_name {
                return Ok(Type::String);
            }
        }

        Ok(match s {
            "bool" | "boolean" => Type::Boolean,
            "usize" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32"
            | "f64" => Type::Number,
            other => Type::Reference(other.to_owned()),
            // unknown => panic!(
            //     "Unknown or unsupported type \"{}\", unable to generate TypeScript declarations. Has this type been registered?",
            //     unknown
            // ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives() {
        assert_eq!(Type::from_str("bool").unwrap(), Type::Boolean);
        assert_eq!(Type::from_str("boolean").unwrap(), Type::Boolean);

        assert_eq!(Type::from_str("usize").unwrap(), Type::Number);
        assert_eq!(Type::from_str("u32").unwrap(), Type::Number);
        assert_eq!(Type::from_str("i16").unwrap(), Type::Number);
        assert_eq!(Type::from_str("f64").unwrap(), Type::Number);
    }

    #[test]
    fn strings() {
        assert_eq!(Type::from_str("str").unwrap(), Type::String);
        assert_eq!(Type::from_str("String").unwrap(), Type::String);
        assert_eq!(Type::from_str("Path").unwrap(), Type::String);
        assert_eq!(Type::from_str("std::string::String").unwrap(), Type::String);
        assert_eq!(Type::from_str("std::path::PathBuf").unwrap(), Type::String);
        assert_eq!(
            Type::from_str("relative_path::RelativePath").unwrap(),
            Type::String
        );
    }

    #[test]
    fn extends() {
        let shape = Type::Union(vec![
            Box::new(Type::String),
            Box::new(Type::Array(Box::new(Type::String))),
        ]);

        assert_eq!(Type::from_str("ExtendsFrom").unwrap(), shape);
        assert_eq!(Type::from_str("schematic::ExtendsFrom").unwrap(), shape);
    }

    #[test]
    fn arrays() {
        assert_eq!(
            Type::from_str("Vec<String>").unwrap(),
            Type::Array(Box::new(Type::String))
        );
        assert_eq!(
            Type::from_str("std::vec::Vec<usize>").unwrap(),
            Type::Array(Box::new(Type::Number))
        );
        assert_eq!(
            Type::from_str("Vec<std::collections::HashSet<bool>>").unwrap(),
            Type::Array(Box::new(Type::Array(Box::new(Type::Boolean))))
        );
        assert_eq!(
            Type::from_str("std::collections::BTreeSet<rustc_hash::FxHashSet<Vec<i32>>>").unwrap(),
            Type::Array(Box::new(Type::Array(Box::new(Type::Array(Box::new(
                Type::Number
            ))))))
        );

        assert_eq!(
            Type::from_str("[String;4]").unwrap(),
            Type::Array(Box::new(Type::String))
        );
        assert_eq!(
            Type::from_str("[f32;4]").unwrap(),
            Type::Array(Box::new(Type::Number))
        );
    }

    #[test]
    fn tuples() {
        assert_eq!(
            Type::from_str("(String)").unwrap(),
            Type::Tuple(vec![Box::new(Type::String)])
        );
        assert_eq!(
            Type::from_str("(String, u8)").unwrap(),
            Type::Tuple(vec![Box::new(Type::String), Box::new(Type::Number)])
        );
        assert_eq!(
            Type::from_str("(String, u8, bool)").unwrap(),
            Type::Tuple(vec![
                Box::new(Type::String),
                Box::new(Type::Number),
                Box::new(Type::Boolean)
            ])
        );
        assert_eq!(
            Type::from_str("(String, u8, bool, i64)").unwrap(),
            Type::Tuple(vec![
                Box::new(Type::String),
                Box::new(Type::Number),
                Box::new(Type::Boolean),
                Box::new(Type::Number),
            ])
        );

        assert_eq!(
            Type::from_str("Vec<(String, String)>").unwrap(),
            Type::Array(Box::new(Type::Tuple(vec![
                Box::new(Type::String),
                Box::new(Type::String)
            ])))
        );
    }

    #[test]
    fn objects() {
        assert_eq!(
            Type::from_str("HashMap<String, String>").unwrap(),
            Type::Object(Box::new(Type::String), Box::new(Type::String))
        );
        assert_eq!(
            Type::from_str("BTreeMap<usize, i32>").unwrap(),
            Type::Object(Box::new(Type::Number), Box::new(Type::Number))
        );
        assert_eq!(
            Type::from_str("std::collections::HashMap<String, Vec<u8>>").unwrap(),
            Type::Object(
                Box::new(Type::String),
                Box::new(Type::Array(Box::new(Type::Number)))
            )
        );
    }
}
