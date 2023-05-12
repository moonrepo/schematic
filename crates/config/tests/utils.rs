use std::env;
use std::path::PathBuf;

pub fn get_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests/__fixtures__")
        .join(name)
}
