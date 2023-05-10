use schematic::*;

#[derive(Config)]
struct TestConfig3 {
    required_field: String,
    optional_field: Option<String>,
}

#[derive(Config)]
struct TestConfigA {
    required_field: String,
    optional_field: Option<String>,
}
