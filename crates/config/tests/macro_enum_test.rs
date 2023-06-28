use schematic::*;

#[derive(Config)]
enum AllUnit {
    Foo,
    #[variant(default)]
    Bar,
    Baz,
}

#[derive(Config)]
enum AllUnnamed {
    Foo(String),
    Bar(bool),
    Baz(usize),
}

impl Default for PartialAllUnnamed {
    fn default() -> Self {
        Self::Foo(String::from("default"))
    }
}

#[derive(Config)]
enum OfBothTypes {
    Foo,
    Bar(bool),
}
