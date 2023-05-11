pub trait PartialConfig {
    fn default_values() -> Self;
}

pub trait Config {
    type Partial: PartialConfig;
}
