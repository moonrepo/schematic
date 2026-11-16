#![allow(dead_code)]

use schematic::*;
use std::collections::HashMap;

fn test_string<T, C>(value: &str, _: &T, _: &C, _: bool) -> ValidateResult {
    if value.is_empty() {
        return Ok(());
    }

    Err(ValidateError::new("invalid string"))
}

#[derive(Config)]
pub struct NestedValidate {
    #[setting(validate = test_string)]
    string2: String,
}

#[derive(Config)]
pub struct Validate {
    #[setting(validate = test_string)]
    string1: String,
    #[setting(nested)]
    nested: NestedValidate,
}

#[test]
fn errors_for_field() {
    let error = ConfigLoader::<Validate>::new()
        .code(r#"{ "string1": "abc" }"#, "code.json")
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  string1: invalid string"
    )
}

#[test]
fn errors_for_nested_field() {
    let error = ConfigLoader::<Validate>::new()
        .code(
            r#"{ "string1": "abc", "nested": { "string2": "abc" } }"#,
            "code.json",
        )
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  string1: invalid string\n  nested.string2: invalid string"
    )
}

fn test_string_path<T, C>(_: &String, _: &T, _: &C) -> Result<(), ValidateError> {
    Err(ValidateError::with_segments(
        "invalid string",
        [PathSegment::Index(1), PathSegment::Key("foo".to_owned())],
    ))
}

#[derive(Config)]
pub struct ValidateFuncs {
    #[setting(validate = validate::alphanumeric)]
    alnum: String,
    #[setting(validate = validate::ascii)]
    ascii: String,
    #[setting(validate = validate::contains("foo"))]
    contains: String,
    #[setting(validate = validate::email)]
    email: String,
    #[setting(validate = validate::ip)]
    ip: String,
    #[setting(validate = validate::ip_v4)]
    ip_v4: String,
    #[setting(validate = validate::ip_v6)]
    ip_v6: String,
    #[setting(validate = validate::regex("^foo$"))]
    regex: String,
    #[setting(validate = validate::min_length(1))]
    min: String,
    #[setting(validate = validate::max_length(1))]
    max: String,
    #[setting(validate = validate::in_length(1, 5))]
    len: String,
    #[setting(validate = validate::url)]
    url: String,
    #[setting(validate = validate::url_secure)]
    url_secure: String,
    #[setting(validate = validate::uuid)]
    uuid: String,
    #[setting(validate = validate::in_range(1, 5))]
    range: i32,
    #[setting(validate = validate::extends_string)]
    ext_str: String,
    #[setting(validate = validate::extends_list)]
    ext_list: Vec<String>,
    #[setting(validate = validate::extends_from)]
    ext_from: ExtendsFrom,
}

#[test]
fn runs_the_validator_funcs() {
    let error = ConfigLoader::<ValidateFuncs>::new()
        .code(r#"{}"#, "code.json")
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  contains: does not contain \"foo\"\n  email: not a valid email: value is empty\n  ip: not a valid IP address\n  ip_v4: not a valid IPv4 address\n  ip_v6: not a valid IPv6 address\n  regex: does not match pattern /^foo$/\n  min: length is lower than 1\n  len: length is lower than 1\n  url: not a valid url: relative URL without a base\n  url_secure: not a valid url: relative URL without a base\n  uuid: not a valid UUID: expected 36 characters, found 0\n  range: lower than 1\n  ext_str: only file paths and URLs can be extended"
    )
}

#[derive(Config)]
pub struct ValidateOptional {
    #[setting(validate = test_string)]
    string1: Option<String>,
}

#[test]
fn skips_optional_fields() {
    let result = ConfigLoader::<ValidateOptional>::new()
        .code(r#"{}"#, "code.json")
        .unwrap()
        .load();

    assert!(result.is_ok());
}

#[test]
fn errors_for_optional_field() {
    let error = ConfigLoader::<ValidateOptional>::new()
        .code(r#"{ "string1": "abc" }"#, "code.json")
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  string1: invalid string"
    )
}

#[derive(Config)]
pub struct ValidateCollections {
    #[setting(nested)]
    list: Vec<NestedValidate>,
    #[setting(nested)]
    map: HashMap<String, NestedValidate>,
}

#[test]
fn errors_for_nested_field_collections() {
    let error = ConfigLoader::<ValidateCollections>::new()
        .code(
            r#"{ "list": [ {"string2": "abc"} ], "map": { "key": {"string2": "abc"} } }"#,
            "code.json",
        )
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  list[0].string2: invalid string\n  map.key.string2: invalid string"
    )
}

#[derive(Config)]
pub struct ValidateRequired {
    #[setting(required)]
    string: Option<String>,
}

#[derive(Config)]
pub enum ValidateEnumRequired {
    Optional(Option<String>),
    #[setting(required)]
    Required(Option<String>),
}

#[test]
fn doesnt_error_if_required_and_notempty() {
    let result = ConfigLoader::<ValidateRequired>::new()
        .code(r#"{ "string": "abc" }"#, "code.json")
        .unwrap()
        .load();

    assert!(result.is_ok());

    let result = ConfigLoader::<ValidateEnumRequired>::new()
        .code(r#"{ "Required": "abc" }"#, "code.json")
        .unwrap()
        .load();

    assert!(result.is_ok());

    let result = ConfigLoader::<ValidateEnumRequired>::new()
        .code(r#"{ "Optional": null }"#, "code.json")
        .unwrap()
        .load();

    assert!(result.is_ok());
}

#[test]
fn errors_if_required_and_empty() {
    let error = ConfigLoader::<ValidateRequired>::new()
        .code(r#"{}"#, "code.json")
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  string: this setting is required"
    );

    let error = ConfigLoader::<ValidateEnumRequired>::new()
        .code(r#"{ "Required": null }"#, "code.json")
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.json. \n  Required: this setting is required"
    );
}

fn load_error<T: Config>(code: &str) -> String {
    ConfigLoader::<T>::new()
        .code(code, "code.json")
        .unwrap()
        .load()
        .err()
        .map(|error| error.to_full_string())
        .unwrap_or_default()
}

mod bounds {
    use super::*;

    #[derive(Config)]
    pub struct Bounded {
        #[setting(default = 1, validate = validate::min(1))]
        at_least_one: usize,
        #[setting(validate = validate::max(10))]
        at_most_ten: i32,
    }

    #[test]
    fn accepts_values_within_one_side() {
        assert_eq!(
            load_error::<Bounded>(r#"{ "at_least_one": 1000, "at_most_ten": -1000 }"#),
            ""
        );
    }

    #[test]
    fn errors_past_either_bound() {
        let error = load_error::<Bounded>(r#"{ "at_least_one": 0, "at_most_ten": 11 }"#);

        assert!(error.contains("at_least_one: lower than 1"), "{error}");
        assert!(error.contains("at_most_ten: greater than 10"), "{error}");
    }
}

mod several_errors {
    use super::*;

    // Reports every empty item, rather than only the first
    fn no_empty_items<D, C>(
        items: &[String],
        _: &D,
        _: &C,
        _: bool,
    ) -> Result<(), Vec<ValidateError>> {
        let errors = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.is_empty())
            .map(|(index, _)| {
                ValidateError::with_segment("must not be empty", PathSegment::Index(index))
            })
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // A garde report, as from a type that's validated with garde
    fn garde_report<D, C>(_: &String, _: &D, _: &C, _: bool) -> Result<(), garde::Report> {
        let mut report = garde::Report::new();

        report.append(
            garde::Path::new("inner").join(0usize),
            garde::Error::new("bad item"),
        );
        report.append(garde::Path::empty(), garde::Error::new("bad value"));

        Err(report)
    }

    #[derive(Config)]
    pub struct Lists {
        #[setting(validate = no_empty_items)]
        items: Vec<String>,
        #[setting(validate = garde_report)]
        reported: String,
    }

    #[test]
    fn reports_each_error_at_its_path() {
        let error = load_error::<Lists>(r#"{ "items": ["a", "", ""], "reported": "x" }"#);

        assert_eq!(
            error,
            "Failed to validate code.json. \n  items[1]: must not be empty\n  items[2]: must not be empty\n  reported.inner[0]: bad item\n  reported: bad value"
        );
    }
}

mod config_validate {
    use super::*;

    // Settings may be missing from a layer, so only compare what's set
    fn min_below_max<C>(partial: &PartialRange, _: &C, _: bool) -> ValidateResult {
        if let (Some(min), Some(max)) = (partial.min, partial.max)
            && min > max
        {
            return Err(ValidateError::with_segment(
                format!("must be at most max ({max})"),
                PathSegment::Key("min".into()),
            ));
        }

        Ok(())
    }

    #[derive(Config)]
    #[config(validate = min_below_max)]
    pub struct Range {
        min: usize,
        max: usize,
    }

    #[test]
    fn validates_across_settings() {
        assert_eq!(load_error::<Range>(r#"{ "min": 1, "max": 2 }"#), "");
        assert_eq!(
            load_error::<Range>(r#"{ "min": 3, "max": 2 }"#),
            "Failed to validate code.json. \n  min: must be at most max (2)"
        );
    }

    // Only the final pass has every layer, so a check across them waits for it
    fn only_final<C>(partial: &PartialLayered, _: &C, finalizing: bool) -> ValidateResult {
        let max = partial.range.as_ref().and_then(|range| range.max);

        if finalizing && max == Some(0) {
            return Err(ValidateError::new("max can't be 0"));
        }

        Ok(())
    }

    #[derive(Config)]
    #[config(validate = only_final)]
    pub struct Layered {
        #[setting(nested)]
        range: Range,
    }

    #[test]
    fn waits_for_the_final_pass() {
        let load = |codes: &[&str]| {
            let mut loader = ConfigLoader::<Layered>::new();

            for code in codes {
                loader.code(*code, "code.json").unwrap();
            }

            loader
                .load()
                .err()
                .map(|error| error.to_full_string())
                .unwrap_or_default()
        };

        // A layer may hold what a later one replaces
        assert_eq!(
            load(&[
                r#"{ "range": { "max": 0 } }"#,
                r#"{ "range": { "max": 5 } }"#
            ]),
            ""
        );
        assert_eq!(
            load(&[
                r#"{ "range": { "max": 5 } }"#,
                r#"{ "range": { "max": 0 } }"#
            ]),
            "Failed to validate code.json. \n  max can't be 0"
        );
    }

    // A factory, like a setting's validator
    fn at_most<D, C>(limit: usize) -> impl FnOnce(&D, &C, bool) -> Result<(), Vec<ValidateError>>
    where
        D: HasTotal,
    {
        move |partial, _, _| {
            if partial.total() > limit {
                return Err(vec![
                    ValidateError::with_segment("over the limit", PathSegment::Key("a".into())),
                    ValidateError::with_segment("over the limit", PathSegment::Key("b".into())),
                ]);
            }

            Ok(())
        }
    }

    pub trait HasTotal {
        fn total(&self) -> usize;
    }

    impl HasTotal for PartialTotals {
        fn total(&self) -> usize {
            self.a.unwrap_or_default() + self.b.unwrap_or_default()
        }
    }

    #[derive(Config)]
    #[config(validate = at_most(10))]
    pub struct Totals {
        a: usize,
        b: usize,
    }

    #[test]
    fn supports_a_factory_and_several_errors() {
        assert_eq!(
            load_error::<Totals>(r#"{ "a": 6, "b": 6 }"#),
            "Failed to validate code.json. \n  a: over the limit\n  b: over the limit"
        );
    }

    #[derive(Config)]
    pub struct Parent {
        #[setting(nested)]
        range: Range,
    }

    #[test]
    fn reports_a_nested_config_at_its_path() {
        assert_eq!(
            load_error::<Parent>(r#"{ "range": { "min": 3, "max": 2 } }"#),
            "Failed to validate code.json. \n  range.min: must be at most max (2)"
        );
    }

    fn not_off<C>(partial: &PartialMode, _: &C, _: bool) -> ValidateResult {
        if matches!(partial, PartialMode::Off) {
            return Err(ValidateError::new("must not be off"));
        }

        Ok(())
    }

    #[derive(Config)]
    #[config(validate = not_off)]
    pub enum Mode {
        #[setting(default)]
        On,
        Off,
    }

    #[derive(Config)]
    pub struct UsesMode {
        #[setting(nested)]
        mode: Mode,
    }

    #[test]
    fn validates_an_enum() {
        assert_eq!(load_error::<UsesMode>(r#"{ "mode": "On" }"#), "");
        assert_eq!(
            load_error::<UsesMode>(r#"{ "mode": "Off" }"#),
            "Failed to validate code.json. \n  mode: must not be off"
        );
    }
}

// The final pass validates the merged result, which no single source is to
// blame for, so each layer that took part is named
mod merged_location {
    use super::*;
    use serial_test::serial;

    #[derive(Config)]
    pub struct Ports {
        #[setting(
            default = 1,
            env = "VALIDATE_MERGED_PORT",
            validate = validate::in_range(1, 100)
        )]
        port: u16,
        #[setting(required)]
        name: Option<String>,
    }

    #[test]
    #[serial]
    fn names_every_source_and_the_environment() {
        unsafe { std::env::set_var("VALIDATE_MERGED_PORT", "500") };

        let error = ConfigLoader::<Ports>::new()
            .code(r#"{ "name": "a" }"#, "a.json")
            .unwrap()
            .code("{}", "b.json")
            .unwrap()
            .load()
            .err()
            .unwrap();

        unsafe { std::env::remove_var("VALIDATE_MERGED_PORT") };

        assert_eq!(
            error.to_full_string(),
            "Failed to validate a.json, b.json, environment variables. \n  port: greater than 100"
        );
    }

    #[test]
    #[serial]
    fn names_the_config_without_any_source() {
        let error = ConfigLoader::<Ports>::new().load().err().unwrap();

        assert_eq!(
            error.to_full_string(),
            "Failed to validate Ports. \n  name: this setting is required"
        );
    }
}
