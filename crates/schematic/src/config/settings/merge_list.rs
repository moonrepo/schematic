use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

/// A list that a configuration source can merge however it chooses, rather
/// than only how the setting's `merge` function does.
///
/// A source writes either a list, which replaces the previous items, or a map
/// of how to merge with them, with any of these keys:
///
/// - `prepend` - Items to add before the previous items.
/// - `append` - Items to add after the previous items.
/// - `replace` - Items to use instead of the previous items.
///
/// ```yaml
/// # Replaces the previous items
/// items: [a, b]
///
/// # Adds to the previous items
/// items:
///   prepend: [a]
///   append: [b]
/// ```
///
/// The derive merges a setting written as `MergeList<..>` with
/// [`merge_list`](crate::merge::merge_list) without being told to, as it knows
/// the type by name, and an explicit `merge` still takes precedence. Through
/// an alias it can't tell, so the setting needs
/// `#[setting(merge = schematic::merge::merge_list)]`, or it's replaced like any
/// other and the previous items are lost.
///
/// Derefs to a slice of its items, which are complete once every layer has been
/// merged over the defaults. A list that only added items, and that no layer or
/// default came before, holds just its own.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MergeList<T> {
    items: Vec<T>,

    // Where the previous items go when merged into, or `None` when they're
    // replaced. Items before it were prepended, and items after it appended.
    base: Option<usize>,
}

impl<T> MergeList<T> {
    /// Create a list that replaces the previous items.
    pub fn new(items: Vec<T>) -> Self {
        Self { items, base: None }
    }

    /// Create a list that adds the items after the previous items.
    pub fn append(items: Vec<T>) -> Self {
        Self {
            items,
            base: Some(0),
        }
    }

    /// Create a list that adds the items before the previous items.
    pub fn prepend(items: Vec<T>) -> Self {
        Self {
            base: Some(items.len()),
            items,
        }
    }

    /// Return true if the list adds to the previous items, rather than
    /// replacing them.
    pub fn is_relative(&self) -> bool {
        self.base.is_some()
    }

    /// Merge the next list into this one, as it directs. The items of this
    /// list are kept between the items the next list prepends and appends,
    /// unless it replaces them.
    ///
    /// Merging is associative, so a merged result still adds to whatever
    /// comes before it, like the defaults.
    pub fn merge(self, next: Self) -> Self {
        let Some(at) = next.base else {
            return next;
        };

        let mut next_items = next.items;
        let appended = next_items.split_off(at);
        let base = self.base.map(|index| index + at);

        next_items.extend(self.items);
        next_items.extend(appended);

        Self {
            items: next_items,
            base,
        }
    }

    /// Return the items as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Return the items, consuming the list.
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }
}

/// Defaults to an empty list that replaces the previous items.
impl<T> Default for MergeList<T> {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl<T> Deref for MergeList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> AsRef<[T]> for MergeList<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

impl<T> From<Vec<T>> for MergeList<T> {
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<T> From<MergeList<T>> for Vec<T> {
    fn from(list: MergeList<T>) -> Self {
        list.items
    }
}

impl<T> FromIterator<T> for MergeList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for MergeList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a MergeList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

// So that the length validators accept it, like a `Vec`, as they're generic
// over the value and wouldn't deref coerce it to a slice
#[cfg(feature = "validate")]
impl<T> garde::rules::length::HasSimpleLength for MergeList<T> {
    fn length(&self) -> usize {
        self.items.len()
    }
}

/// Serializes a list that replaces as a list, and one that adds to the
/// previous items as a map of the items it prepends and appends.
impl<T: Serialize> Serialize for MergeList<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(at) = self.base else {
            return self.items.serialize(serializer);
        };

        let (prepended, appended) = self.items.split_at(at);
        let len = usize::from(!prepended.is_empty()) + usize::from(!appended.is_empty());
        let mut map = serializer.serialize_map(Some(len))?;

        if !prepended.is_empty() {
            map.serialize_entry("prepend", prepended)?;
        }

        if !appended.is_empty() {
            map.serialize_entry("append", appended)?;
        }

        map.end()
    }
}

const KEYS: &[&str] = &["append", "prepend", "replace"];

struct MergeListVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for MergeListVisitor<T> {
    type Value = MergeList<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list, or a map of items to append, prepend, or replace")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        Vec::deserialize(de::value::SeqAccessDeserializer::new(seq)).map(MergeList::new)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut append: Option<Vec<T>> = None;
        let mut prepend: Option<Vec<T>> = None;
        let mut replace: Option<Vec<T>> = None;

        while let Some(key) = map.next_key::<String>()? {
            let slot = match key.as_str() {
                "append" => &mut append,
                "prepend" => &mut prepend,
                "replace" => &mut replace,
                other => return Err(de::Error::unknown_field(other, KEYS)),
            };

            if slot.is_some() {
                return Err(de::Error::custom(format_args!("duplicate field `{key}`")));
            }

            *slot = Some(map.next_value()?);
        }

        let mut items = prepend.unwrap_or_default();
        let at = items.len();
        let base = if let Some(replace) = replace {
            items.extend(replace);
            None
        } else {
            Some(at)
        };

        items.extend(append.unwrap_or_default());

        Ok(MergeList { items, base })
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for MergeList<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(MergeListVisitor(PhantomData))
    }
}

#[cfg(feature = "schema")]
mod schema {
    use super::MergeList;
    use crate::schema::{
        ArrayType, Schema, SchemaBuilder, SchemaField, Schematic, StructType, UnionType,
    };

    /// A list, or a struct of the lists to merge with the previous items.
    impl<T: Schematic> Schematic for MergeList<T> {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            let list = || Schema::array(ArrayType::new(schema.infer::<T>()));

            let directives = StructType::new(["append", "prepend", "replace"].map(|key| {
                (
                    key.to_owned(),
                    SchemaField {
                        optional: true,
                        ..SchemaField::new(list())
                    },
                )
            }));

            schema.union(UnionType::new_any([list(), Schema::structure(directives)]))
        }
    }
}
