use schematic::helpers::extract_ext;

mod ext {
    use super::*;

    #[test]
    fn works_on_files() {
        assert!(extract_ext("file").is_none());
        assert_eq!(extract_ext("file.json").unwrap(), ".json");
        assert_eq!(extract_ext("dir/file.yaml").unwrap(), ".yaml");
        assert_eq!(extract_ext("../file.toml").unwrap(), ".toml");
        assert_eq!(extract_ext("/root/file.other.json").unwrap(), ".json");
    }

    #[test]
    fn works_on_urls() {
        assert!(extract_ext("https://domain.com/file").is_none());
        assert_eq!(
            extract_ext("https://domain.com/file.json").unwrap(),
            ".json"
        );
        assert_eq!(
            extract_ext("http://domain.com/dir/file.yaml").unwrap(),
            ".yaml"
        );
        assert_eq!(
            extract_ext("https://domain.com/file.toml?query").unwrap(),
            ".toml"
        );
        assert_eq!(
            extract_ext("http://domain.com/root/file.other.json").unwrap(),
            ".json"
        );
        assert_eq!(
            extract_ext("https://domain.com/other.segment/file.toml?query").unwrap(),
            ".toml"
        );
    }
}
