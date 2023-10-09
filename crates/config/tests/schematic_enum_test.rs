#![allow(dead_code, deprecated)]

use schematic::Schematic;
use std::collections::HashMap;

#[derive(Default, Schematic)]
pub enum SomeEnum {
    #[default]
    A,
    B,
    C,
}

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
}
