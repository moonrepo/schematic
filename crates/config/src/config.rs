pub trait PartialConfig {}

pub trait Config {
    type Partial: PartialConfig;
}
