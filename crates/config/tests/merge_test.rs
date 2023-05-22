#![allow(dead_code)]

use schematic::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Config)]
pub struct MergeNormal {
    string: String,
    vector: Vec<usize>,
    map: HashMap<String, usize>,
}

fn merge_string<C>(prev: String, next: String, _: &C) -> Result<Option<String>, ConfigError> {
    Ok(Some(format!("{}-{}", prev, next)))
}

#[derive(Config)]
pub struct MergeFunc {
    #[setting(merge = merge_string)]
    string: String,
    #[setting(merge = merge::prepend_vec)]
    vector: Vec<usize>,
    #[setting(merge = merge::discard)]
    map: HashMap<String, usize>,
}

#[test]
fn normal_merge_takes_some_next() {
    let mut base = PartialMergeNormal {
        string: Some("foo".into()),
        vector: Some(vec![1, 2, 3]),
        map: Some(HashMap::from_iter([("a".into(), 1)])),
    };

    base.merge(
        &(),
        PartialMergeNormal {
            string: Some("bar".into()),
            vector: Some(vec![4, 5, 6]),
            map: None,
        },
    )
    .unwrap();

    assert_eq!(base.string.unwrap(), "bar");
    assert_eq!(base.vector.unwrap(), vec![4, 5, 6]);
    assert_eq!(base.map.unwrap(), HashMap::from_iter([("a".into(), 1)]));
}

#[test]
fn custom_merge_with_funcs() {
    let mut base = PartialMergeFunc {
        string: Some("foo".into()),
        vector: Some(vec![1, 2, 3]),
        map: Some(HashMap::from_iter([("a".into(), 1)])),
    };

    base.merge(
        &(),
        PartialMergeFunc {
            string: Some("bar".into()),
            vector: Some(vec![4, 5, 6]),
            map: Some(HashMap::from_iter([("b".into(), 2)])),
        },
    )
    .unwrap();

    assert_eq!(base.string.unwrap(), "foo-bar");
    assert_eq!(base.vector.unwrap(), vec![4, 5, 6, 1, 2, 3]);
    assert_eq!(base.map, None);
}

#[derive(Debug, Config)]
pub struct MergeNested {
    #[setting(default = "xyz")]
    string: String,
    #[setting(default = 10)]
    other: usize,
}

#[derive(Debug, Config)]
pub struct MergeBase {
    #[setting(default = "abc")]
    string: String,
    #[setting(default = vec![1,2,3], merge = merge::append_vec)]
    vector: Vec<usize>,
    #[setting(nested)]
    nested: MergeNested,
    #[setting(nested)]
    opt_nested: Option<MergeNested>,
}

#[test]
fn uses_defaults_when_no_layers() {
    let result = ConfigLoader::<MergeBase>::new(SourceFormat::Yaml)
        .load()
        .unwrap();

    assert_eq!(result.config.string, "abc");
    assert_eq!(result.config.vector, vec![1, 2, 3]);
    assert_eq!(result.config.nested.string, "xyz");
    assert_eq!(result.config.nested.other, 10);
    assert!(result.config.opt_nested.is_none());
}

#[test]
fn can_merge_with_defaults() {
    let result = ConfigLoader::<MergeBase>::new(SourceFormat::Yaml)
        .code("string: def")
        .unwrap()
        .code("vector: [4]")
        .unwrap()
        .code(
            r"vector: [5]
nested:
  string: zyx
",
        )
        .unwrap()
        .code(
            r"nested:
  other: 15
",
        )
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "def");
    assert_eq!(result.config.vector, vec![1, 2, 3, 4, 5]);
    assert_eq!(result.config.nested.string, "zyx");
    assert_eq!(result.config.nested.other, 15);
    assert!(result.config.opt_nested.is_none());
}

#[test]
fn loads_defaults_for_optional_nested() {
    let result = ConfigLoader::<MergeBase>::new(SourceFormat::Yaml)
        .code(
            r"
optNested:
    string: hij",
        )
        .unwrap()
        .load()
        .unwrap();

    assert!(result.config.opt_nested.is_some());
    assert_eq!(result.config.opt_nested.as_ref().unwrap().string, "hij");
    assert_eq!(result.config.opt_nested.as_ref().unwrap().other, 10);
}

mod helpers {
    use super::*;

    #[test]
    fn discard() {
        assert_eq!(merge::discard(1, 2, &()).unwrap(), None);
    }

    #[test]
    fn preserve() {
        assert_eq!(merge::preserve(1, 2, &()).unwrap(), Some(1));
    }

    #[test]
    fn replace() {
        assert_eq!(merge::replace(1, 2, &()).unwrap(), Some(2));
    }

    #[test]
    fn append_vec() {
        assert_eq!(
            merge::append_vec(vec![1], vec![2], &()).unwrap(),
            Some(vec![1, 2])
        );
    }

    #[test]
    fn prepend_vec() {
        assert_eq!(
            merge::prepend_vec(vec![1], vec![2], &()).unwrap(),
            Some(vec![2, 1])
        );
    }

    #[test]
    fn merge_btreemap() {
        assert_eq!(
            merge::merge_btreemap(
                BTreeMap::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
                BTreeMap::from_iter([("b".to_string(), 3), ("c".to_string(), 4)]),
                &()
            )
            .unwrap(),
            Some(BTreeMap::from_iter([
                ("a".to_string(), 1),
                ("b".to_string(), 3),
                ("c".to_string(), 4),
            ]))
        );
    }

    #[test]
    fn merge_btreeset() {
        assert_eq!(
            merge::merge_btreeset(
                BTreeSet::from_iter(["a", "b"]),
                BTreeSet::from_iter(["a", "b", "c", "d"]),
                &()
            )
            .unwrap(),
            Some(BTreeSet::from_iter(["a", "b", "c", "d"]))
        );
    }

    #[test]
    fn merge_hashmap() {
        assert_eq!(
            merge::merge_hashmap(
                HashMap::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
                HashMap::from_iter([("b".to_string(), 3), ("c".to_string(), 4)]),
                &()
            )
            .unwrap(),
            Some(HashMap::from_iter([
                ("a".to_string(), 1),
                ("b".to_string(), 3),
                ("c".to_string(), 4),
            ]))
        );
    }

    #[test]
    fn merge_hashset() {
        assert_eq!(
            merge::merge_hashset(
                HashSet::from_iter(["a", "b"]),
                HashSet::from_iter(["a", "b", "c", "d"]),
                &()
            )
            .unwrap(),
            Some(HashSet::from_iter(["a", "b", "c", "d"]))
        );
    }
}
