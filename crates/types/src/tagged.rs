use crate::*;
use std::fmt;

/// How serde writes the name of an enum variant, from the enum's
/// `#[serde(tag)]` and `#[serde(content)]` attributes. An untagged variant is
/// written as its value alone, so it has no format, and is no [`TaggedType`].
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum TagFormat {
    /// `{ "name": value }`, the default.
    #[default]
    External,

    /// `{ "tag": "name", ...value }`, from `#[serde(tag = "tag")]`.
    Internal { tag: String },

    /// `{ "tag": "name", "content": value }`, from
    /// `#[serde(tag = "tag", content = "content")]`.
    Adjacent { tag: String, content: String },
}

/// An enum variant that serde writes with its name, as a tag around or within
/// the value it holds, which renderers can render as they see fit.
///
/// The value is kept apart from the tag, rather than within a struct of
/// them, as a named type's schema is shared by every type that holds it. A
/// renderer that only renders the struct serde writes, as most do, renders
/// [`TaggedType::to_schema`].
///
/// An externally tagged unit variant is written as its name alone, so it's a
/// literal rather than a tagged type.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct TaggedType {
    pub format: TagFormat,

    /// The name of the variant, as serde writes it.
    pub name: String,

    /// The environment variable that picks the variant through its tag.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub env_var: Option<String>,

    /// The value the variant holds, which a unit variant has none of.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<Box<Schema>>,
}

impl TaggedType {
    /// Create a tagged variant with the provided format, name, and value.
    pub fn new(format: TagFormat, name: impl AsRef<str>, value: Option<Schema>) -> Self {
        TaggedType {
            format,
            name: name.as_ref().to_owned(),
            env_var: None,
            value: value.map(Box::new),
        }
    }

    /// Return the tag's key, which an externally tagged variant has none of,
    /// as its name is the key.
    pub fn get_tag(&self) -> Option<&str> {
        match &self.format {
            TagFormat::External => None,
            TagFormat::Internal { tag } | TagFormat::Adjacent { tag, .. } => Some(tag),
        }
    }

    /// Return the schema of what serde writes for the variant, a struct of
    /// its tag and its value:
    ///
    /// - External: `{ "name": value }`, or the name for a unit.
    /// - Internal: `{ "tag": "name" }` with the value's fields. A struct of
    ///   its own, such as a named variant's fields, holds the tag itself. A
    ///   named struct is flattened into a struct of the tag instead, which
    ///   renderers reference by name, like `{ type: 'git' } & GitSource`, as
    ///   adding the tag would claim it for every type that holds the struct.
    ///   Anything else is the value as it is, as serde can only add a tag to
    ///   a map.
    /// - Adjacent: `{ "tag": "name", "content": value }`.
    ///
    /// A value keeps its description, as a field holding it is created with
    /// [`SchemaField::new`].
    pub fn to_schema(&self) -> Schema {
        let name = Schema::literal_value(LiteralValue::String(self.name.clone()));

        let Some(value) = &self.value else {
            return match self.get_tag() {
                Some(tag) => Schema::structure(StructType::new([(
                    tag.to_owned(),
                    self.create_tag_field(name),
                )])),
                None => name,
            };
        };

        let value = value.as_ref().clone();

        match &self.format {
            TagFormat::External => Schema::structure(StructType::new([(
                self.name.clone(),
                SchemaField::new(value),
            )])),
            TagFormat::Internal { tag } => {
                let mut value = value;
                let tag_field = self.create_tag_field(name);

                let SchemaType::Struct(inner) = &value.ty else {
                    return value;
                };

                let Some(key) = value.name.clone() else {
                    value.add_field(tag, tag_field);

                    return value;
                };

                // Serde reads the keys besides the tag as the value, so the
                // variant accepts unknown keys when the value does
                let allow_unknown_fields = inner.allow_unknown_fields;

                // A flattened value has no key of its own, so it's keyed by
                // its type, which the tag can't share
                let mut key = key;

                while &key == tag {
                    key.insert(0, '_');
                }

                let mut field = SchemaField::new(value);
                field.flatten = true;

                let mut structure = StructType::new([(tag.to_owned(), tag_field), (key, field)]);
                structure.allow_unknown_fields = allow_unknown_fields;

                Schema::structure(structure)
            }
            TagFormat::Adjacent { tag, content } => Schema::structure(StructType::new([
                (tag.to_owned(), self.create_tag_field(name)),
                (content.to_owned(), SchemaField::new(value)),
            ])),
        }
    }

    fn create_tag_field(&self, name: Schema) -> SchemaField {
        let mut field = SchemaField::new(name);
        field.env_var.clone_from(&self.env_var);
        field
    }
}

impl fmt::Display for TaggedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_schema())
    }
}
