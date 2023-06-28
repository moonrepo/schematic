use schematic::*;

#[derive(Config)]
enum AllUnit {
    Foo,
    Bar,
    Baz,
}

#[derive(Config)]
enum AllUnnamed {
    Foo(String),
    Bar(bool),
    Baz(usize),
}

#[derive(Config)]
enum OfBothTypes {
    Foo,
    Bar(bool),
}
