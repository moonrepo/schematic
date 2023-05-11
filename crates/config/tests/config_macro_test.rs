use schematic::*;

#[derive(Config)]
struct NestedConfig {
    enabled: bool,
}

#[derive(Config)]
struct TestConfig {
    // test comment
    required_field: String,

    /// and another
    optional_field: Option<String>,

    /* what about */
    #[setting(nested)]
    nested_config: NestedConfig,
}

fn test() {
    let mut a = PartialTestConfig::default();

    dbg!(a);
}
