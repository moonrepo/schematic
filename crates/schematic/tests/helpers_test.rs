use schematic::helpers::extract_file_ext;

mod ext {
    use super::*;

    #[test]
    fn works_on_files() {
        assert!(extract_file_ext("file").is_none());
        assert_eq!(extract_file_ext("file.json").unwrap(), "json");
        assert_eq!(extract_file_ext("dir/file.yaml").unwrap(), "yaml");
        assert_eq!(extract_file_ext("../file.toml").unwrap(), "toml");
        assert_eq!(extract_file_ext("/root/file.other.json").unwrap(), "json");
    }

    #[test]
    fn works_on_urls() {
        assert!(extract_file_ext("https://domain.com/file").is_none());
        assert_eq!(
            extract_file_ext("https://domain.com/file.json").unwrap(),
            "json"
        );
        assert_eq!(
            extract_file_ext("http://domain.com/dir/file.yaml").unwrap(),
            "yaml"
        );
        assert_eq!(
            extract_file_ext("https://domain.com/file.toml?query").unwrap(),
            "toml"
        );
        assert_eq!(
            extract_file_ext("http://domain.com/root/file.other.json").unwrap(),
            "json"
        );
        assert_eq!(
            extract_file_ext("https://domain.com/other.segment/file.toml?query").unwrap(),
            "toml"
        );
    }
}

mod urls {
    use schematic::helpers::{is_secure_url, is_url_like};

    #[test]
    fn secure_for_https() {
        assert!(is_secure_url("https://domain.com/config.yml"));
        assert!(is_secure_url("HTTPS://domain.com/config.yml"));
    }

    #[test]
    fn secure_for_local_http() {
        assert!(is_secure_url("http://localhost/config.yml"));
        assert!(is_secure_url("http://localhost:3000/config.yml"));
        assert!(is_secure_url("http://127.0.0.1:3000/config.yml"));
        assert!(is_secure_url("http://[::1]:3000/config.yml"));
        assert!(is_secure_url("http://user@localhost/config.yml"));
    }

    #[test]
    fn insecure_for_remote_http() {
        assert!(!is_secure_url("http://domain.com/config.yml"));
        assert!(!is_secure_url("http://evil.example/c.yml?h=127.0.0.1"));
        assert!(!is_secure_url("http://evil.example/localhost/c.yml"));
        assert!(!is_secure_url("http://localhost.evil.example/c.yml"));
        assert!(!is_secure_url("http://127.0.0.1.nip.io/c.yml"));
        assert!(!is_secure_url("http://localhost@evil.example/c.yml"));
    }

    #[test]
    fn insecure_without_scheme() {
        assert!(!is_secure_url("domain.com/config.yml"));
        assert!(!is_secure_url("127.0.0.1/config.yml"));
        assert!(!is_secure_url("ftp://127.0.0.1/config.yml"));
    }

    #[test]
    fn url_like_requires_www_dot() {
        assert!(is_url_like("www.domain.com/config.yml"));
        assert!(!is_url_like("www/config.yml"));
        assert!(!is_url_like("wwwroot/config.yml"));
    }
}
