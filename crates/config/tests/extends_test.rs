mod utils;

use crate::utils::get_fixture_path;
use schematic::*;

#[derive(Debug, Config)]
struct ExtendsString {
    #[setting(extends)]
    extends: String,
    #[setting(merge = merge::append_vec)]
    value: Vec<usize>,
}

#[derive(Config)]
struct ExtendsList {
    #[setting(extends)]
    extends: Vec<String>,
    #[setting(merge = merge::append_vec)]
    value: Vec<usize>,
}

#[test]
fn extends_from_chain_in_order() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsString>::new(SourceFormat::Yaml)
        .file(root.join("base.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.extends, "");
    assert_eq!(result.config.value, vec![3, 2, 1]);

    assert_eq!(
        result.sources,
        vec![
            Source::File {
                path: root.join("./string2.yml")
            },
            Source::File {
                path: root.join("./string1.yml")
            },
            Source::File {
                path: root.join("./base.yml")
            }
        ]
    );
}

#[test]
fn extends_from_chain_in_order_using_list() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsList>::new(SourceFormat::Yaml)
        .file(root.join("base-list.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.extends, Vec::<String>::new());
    assert_eq!(result.config.value, vec![3, 2, 4, 1]);

    assert_eq!(
        result.sources,
        vec![
            Source::File {
                path: root.join("./string2.yml")
            },
            Source::File {
                path: root.join("./list1.yml")
            },
            Source::File {
                path: root.join("./list2.yml")
            },
            Source::File {
                path: root.join("./base-list.yml")
            }
        ]
    );
}
