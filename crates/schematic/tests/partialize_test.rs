#![allow(dead_code)]

use schematic::internal::partialize_schema;
use schematic::{derive_enum, Config, ConfigEnum, SchemaBuilder, Schematic};
use similar::{ChangeTag, TextDiff};
use starbase_sandbox::assert_snapshot;
use std::collections::HashMap;
use std::fmt::Debug;

fn create_diff<T: Schematic>() -> String {
    let mut schema = SchemaBuilder::build_root::<T>();

    let original = format!("{:#?}", schema);

    partialize_schema(&mut schema, true);

    let partial = format!("{:#?}", schema);

    // println!("ORIGINAL:\n{}\n\n", original);
    // println!("PARTIAL:\n{}\n\n", partial);

    let mut diff = String::new();

    for change in TextDiff::from_lines(&original, &partial).iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "🟥",
            ChangeTag::Insert => "🟩",
            ChangeTag::Equal => "⬛️",
        };

        diff.push_str(&format!("{}{}", sign, change));
    }

    diff
}

#[derive(Config)]
struct Empty {}

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
