#![allow(dead_code, deprecated)]

use schematic::Schematic;
use std::collections::HashMap;

#[derive(Schematic)]
struct Basic<'a> {
    pub name: &'a str,
}

#[derive(Schematic)]
struct Multiple<'a, 'b> {
    pub name: &'a str,
    pub age: &'b u32,
}

#[derive(Schematic)]
struct Collections<'a, 'b> {
    pub map: HashMap<&'a String, &'a bool>,
    pub map_alt: &'a HashMap<String, bool>,
    pub list: Vec<&'b u32>,
    pub list_alt: &'b [u32],
}
