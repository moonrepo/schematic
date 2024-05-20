#![allow(dead_code, clippy::box_collection)]

mod utils;

use schematic::*;
use std::collections::HashMap;

#[derive(Config)]
struct Config {
    field: String,
}

#[derive(ConfigEnum)]
enum ConfigEnum {
    A,
    B,
    C,
}

// VALUES

#[derive(Config)]
struct Single(String);

#[derive(Config)]
struct Multiple(String, usize, bool);

#[derive(Config)]
struct SingleOption(Option<String>);

#[derive(Config)]
struct MultipleOption(Option<String>, usize, Option<bool>);

#[derive(Config)]
struct SingleBox(Box<String>);

#[derive(Config)]
struct MultipleBox(Box<String>, usize, Box<bool>);

#[derive(Config)]
struct SingleOptionBox(Option<Box<String>>);

#[derive(Config)]
struct MultipleOptionBox(Option<Box<String>>, Option<usize>, Box<bool>);

// NESTED

#[derive(Config)]
struct NestedSingle(#[setting(nested)] Config);

#[derive(Config)]
struct NestedMultiple(String, #[setting(nested)] Config);

#[derive(Config)]
struct NestedVec(#[setting(nested)] Vec<Config>);

#[derive(Config)]
struct NestedMap(#[setting(nested)] HashMap<String, Config>);

#[derive(Config)]
struct NestedComplex(
    #[setting(nested)] Option<Vec<Config>>,
    #[setting(nested)] HashMap<String, Option<Box<Config>>>,
);

// DEFAULT

#[derive(Config)]
struct DefaultSingle(#[setting(default = "abc")] String);

#[derive(Config)]
struct DefaultMultiple(
    String,
    #[setting(default = 123)] usize,
    #[setting(default = true)] bool,
);

// ENV

#[derive(Config)]
struct EnvMultiple(
    #[setting(env = "A")] String,
    usize,
    #[setting(env = "C", parse_env = env::parse_bool)] bool,
);

// MERGE

#[derive(Config)]
struct MergeVec(#[setting(merge = merge::append_vec)] Vec<String>);

#[derive(Config)]
struct MergeMapMultiple(
    #[setting(merge = merge::merge_hashmap)] HashMap<String, isize>,
    #[setting(merge = merge::discard)] Option<Vec<String>>,
);

// VALIDATE

#[derive(Config)]
struct ValidateSingle(#[setting(validate = validate::not_empty)] String);

#[derive(Config)]
struct ValidateMultiple(
    #[setting(validate = validate::not_empty)] String,
    #[setting(validate = validate::in_range(0, 100))] usize,
    bool,
);
