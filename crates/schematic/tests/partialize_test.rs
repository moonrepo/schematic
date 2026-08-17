#![allow(dead_code)]

use schematic::internal::partialize_schema;
use schematic::{Config, ConfigEnum, Schema, SchemaBuilder, SchemaType, Schematic, derive_enum};
use similar::{ChangeTag, TextDiff};
use starbase_sandbox::assert_snapshot;
use std::collections::HashMap;
use std::fmt::Debug;

fn create_diff<T: Schematic>() -> String {
    let mut schema = SchemaBuilder::build_root::<T>();

    let original = format!("{schema:#?}");

    partialize_schema(&mut schema, true);

    let partial = format!("{schema:#?}");

    // println!("ORIGINAL:\n{}\n\n", original);
    // println!("PARTIAL:\n{}\n\n", partial);

    let mut diff = String::new();

    for change in TextDiff::from_lines(&original, &partial).iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "🟥",
            ChangeTag::Insert => "🟩",
            ChangeTag::Equal => "⬛️",
        };

        diff.push_str(&format!("{sign}{change}"));
    }

    diff
}

#[derive(Config)]
struct Empty {}

#[derive(Config)]
#[config(partial(derive(derive_more::AsRef)))]
struct Basic {
    field: String,
}

#[derive(Config)]
struct Primitives {
    string: String,
    number: usize,
    boolean: bool,
    float: f32,
    string_opt: Option<String>,
    number_opt: Option<isize>,
    boolean_opt: Option<bool>,
    float_opt: Option<f64>,
}

#[test]
fn primitives() {
    assert_snapshot!(create_diff::<Primitives>());
}

#[derive(Config)]
struct Compounds {
    list: Vec<String>,
    list_opt: Option<Vec<usize>>,
    map: HashMap<String, bool>,
    map_opt: Option<HashMap<isize, f32>>,
}

#[test]
fn compounds() {
    assert_snapshot!(create_diff::<Compounds>());
}

derive_enum!(
    #[derive(Default, ConfigEnum)]
    enum UnitEnum {
        #[default]
        A,
        B,
        C,
    }
);

derive_enum!(
    #[derive(Default, ConfigEnum)]
    enum UnitFallbackEnum {
        #[default]
        A,
        B,
        C,
        #[variant(fallback)]
        Other(String),
    }
);

#[derive(Config)]
enum TupleEnum {
    #[setting(nested)]
    A(Empty),
    #[setting(nested)]
    B(Empty),
}

#[derive(Config)]
struct Enums {
    unit: UnitEnum,
    unit_opt: Option<UnitEnum>,
    fallback: UnitFallbackEnum,
    fallback_opt: Option<UnitFallbackEnum>,
    #[setting(nested)]
    tuple: TupleEnum,
    #[setting(nested)]
    tuple_opt: Option<TupleEnum>,
}

#[test]
fn enums() {
    assert_snapshot!(create_diff::<Enums>());
}

#[derive(Config)]
struct Nested {
    #[setting(nested)]
    field: Basic,
    #[setting(nested)]
    field_opt: Option<Basic>,
}

#[test]
fn nested() {
    assert_snapshot!(create_diff::<Nested>());
}

#[derive(Config)]
struct NestedList {
    #[setting(nested)]
    field: Vec<Basic>,
    #[setting(nested)]
    field_opt: Option<Vec<Basic>>,
}

#[test]
fn nested_list() {
    assert_snapshot!(create_diff::<NestedList>());
}

#[derive(Config)]
struct NestedMap {
    #[setting(nested)]
    field: HashMap<String, Basic>,
    #[setting(nested)]
    field_opt: Option<HashMap<usize, Basic>>,
}

#[test]
fn nested_map() {
    assert_snapshot!(create_diff::<NestedMap>());
}

#[derive(Config)]
#[config(serde(untagged))]
enum Untagged {
    Foo,
    Bar(bool),
    Baz(usize, String),
    #[setting(nested)]
    Qux(Basic),
}

#[test]
fn enum_untagged() {
    assert_snapshot!(create_diff::<Untagged>());
}

#[derive(Config)]
enum ExternalTagged {
    Foo,
    Bar(bool),
    Baz(usize),
    #[setting(nested)]
    Qux(Basic),
}

#[test]
fn enum_external() {
    assert_snapshot!(create_diff::<ExternalTagged>());
}

#[derive(Config)]
#[config(serde(tag = "type"))]
enum InternalTagged {
    Foo,
    Bar(bool),
    Baz(usize),
    #[setting(nested)]
    Qux(Basic),
}

#[test]
fn enum_internal() {
    assert_snapshot!(create_diff::<InternalTagged>());
}

#[derive(Config)]
#[config(
    serde(tag = "type", content = "content"),
    partial(derive(derive_more::TryInto))
)]
enum AdjacentTagged {
    Foo,
    Bar(bool),
    Baz(usize),
    #[setting(nested)]
    Qux(Basic),
}

#[test]
fn enum_adjacent() {
    assert_snapshot!(create_diff::<AdjacentTagged>());
}

#[derive(Config)]
#[config(partial(derive(derive_more::AsRef)))]
struct Basic2 {
    #[setting(partial(as_ref))]
    field1: String,
    field2: String,
}

#[test]
fn partial_derive() {
    // Ensure the attributes we applied to the partial struct were applied correctly

    let bar: bool = PartialAdjacentTagged::Bar(true).try_into().unwrap();
    assert!(bar);

    let basic = PartialBasic {
        field: Some("test".to_string()),
    };
    assert_eq!(&Some("test".to_string()), basic.as_ref());

    let basic2 = PartialBasic2 {
        field1: Some("test1".to_string()),
        field2: Some("test2".to_string()),
    };
    assert_eq!(&Some("test1".to_string()), basic2.as_ref());
}

#[derive(Config)]
struct Recursive {
    name: String,
    #[setting(nested)]
    children: Vec<Recursive>,
}

fn collect_references(schema: &Schema, out: &mut Vec<String>) {
    match &schema.ty {
        SchemaType::Reference { name, .. } => out.push(name.to_owned()),
        SchemaType::Array(inner) => collect_references(&inner.items_type, out),
        SchemaType::Object(inner) => collect_references(&inner.value_type, out),
        SchemaType::Struct(inner) => {
            for field in inner.fields.values() {
                collect_references(&field.schema, out);
            }
        }
        SchemaType::Tuple(inner) => {
            for item in &inner.items_types {
                collect_references(item, out);
            }
        }
        SchemaType::Union(inner) => {
            for variant in &inner.variants_types {
                collect_references(variant, out);
            }
        }
        _ => {}
    };
}

// A cycle resolves to a reference, which has to be renamed alongside the type
// it points at, otherwise the partial schema references a type that was never
// rendered.
#[test]
fn recursive_references_follow_the_partial_name() {
    let schema = SchemaBuilder::build_root::<PartialRecursive>();

    assert_eq!(schema.name.as_deref(), Some("PartialRecursive"));

    let mut references = vec![];
    collect_references(&schema, &mut references);

    assert_eq!(references, vec!["PartialRecursive"]);
}

#[test]
fn recursive_references_are_untouched_when_not_partial() {
    let schema = SchemaBuilder::build_root::<Recursive>();

    assert_eq!(schema.name.as_deref(), Some("Recursive"));

    let mut references = vec![];
    collect_references(&schema, &mut references);

    assert_eq!(references, vec!["Recursive"]);
}
