#![allow(dead_code, deprecated)]

use schematic::Schematic;
use std::collections::HashMap;

/// Some comment.
#[derive(Default, Schematic)]
pub enum SomeEnum {
    #[default]
    A,
    B,
    C,
    #[schema(exclude)]
    D,
}

pub struct NonSchematic {
    string: String,
}

/** A comment. */
#[derive(Schematic)]
#[schematic(rename_all = "snake_case")]
pub struct ValueTypes {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
    map: HashMap<String, u64>,
    enums: SomeEnum,
    s3_value: String,

    #[schema(exclude)]
    other: NonSchematic,
}
