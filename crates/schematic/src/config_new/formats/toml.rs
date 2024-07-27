use serde::de::DeserializeOwned;

pub fn parse<D>(content: String) -> Result<D, ParserError>
where
    D: DeserializeOwned,
{
    let de = toml::Deserializer::new(&content);

    serde_path_to_error::deserialize(de).map_err(|error| ParserError {
        // content: NamedSource::new(location, content.to_owned()),
        content: content.to_owned(),
        path: error.path().to_string(),
        span: error.inner().span().map(|s| s.into()),
        message: error.inner().message().to_owned(),
    })?
}
