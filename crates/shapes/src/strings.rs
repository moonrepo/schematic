#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StringType {
    pub format: Option<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
}
