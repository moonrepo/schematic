use schematic::*;

#[test]
fn can_create_url_source() {
    let source = Source::new("https://some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::Url {
            url: "https://some/path/config.yml".to_owned(),
        }
    );
}

#[test]
#[should_panic(expected = "HttpsOnly")]
fn errors_on_http() {
    Source::new("http://some/path/config.yml", None).unwrap();
}

#[test]
#[should_panic(expected = "HttpsOnly")]
fn errors_on_www() {
    Source::new("www.domain.com/some/path/config.yml", None).unwrap();
}
