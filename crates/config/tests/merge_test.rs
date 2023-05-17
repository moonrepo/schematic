#![allow(dead_code)]

use schematic::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Config)]
pub struct MergeNormal {
    string: String,
    vector: Vec<usize>,
    map: HashMap<String, usize>,
}

fn merge_string(prev: String, next: String) -> Option<String> {
    Some(format!("{}-{}", prev, next))
}

#[derive(Config)]
pub struct MergeFunc {
    #[setting(merge = merge_string)]
    string: String,
    #[setting(merge = merge::prepend_vec)]
    vector: Vec<usize>,
    #[setting(merge = merge::discard)]
    map: HashMap<String, usize>,
    #[setting(nested, merge = merge::merge_partial)]
    nested: MergeNormal,
}

#[test]
fn normal_merge_takes_some_next() {
    let mut base = PartialMergeNormal {
        string: Some("foo".into()),
        vector: Some(vec![1, 2, 3]),
        map: Some(HashMap::from_iter([("a".into(), 1)])),
    };

    base.merge(PartialMergeNormal {
        string: Some("bar".into()),
        vector: Some(vec![4, 5, 6]),
        map: None,
    });

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
        nested: None,
    };

    base.merge(PartialMergeFunc {
        string: Some("bar".into()),
        vector: Some(vec![4, 5, 6]),
        map: Some(HashMap::from_iter([("b".into(), 2)])),
        nested: None,
    });

    assert_eq!(base.string.unwrap(), "foo-bar");
    assert_eq!(base.vector.unwrap(), vec![4, 5, 6, 1, 2, 3]);
    assert_eq!(base.map, None);
}

mod helpers {
    use super::*;

    #[test]
    fn discard() {
        assert_eq!(merge::discard(1, 2), None);
    }

    #[test]
    fn preserve() {
        assert_eq!(merge::preserve(1, 2), Some(1));
    }

    #[test]
    fn replace() {
        assert_eq!(merge::replace(1, 2), Some(2));
    }

    #[test]
    fn append_vec() {
        assert_eq!(merge::append_vec(vec![1], vec![2]), Some(vec![1, 2]));
    }

    #[test]
    fn prepend_vec() {
        assert_eq!(merge::prepend_vec(vec![1], vec![2]), Some(vec![2, 1]));
    }

    #[test]
    fn merge_btreemap() {
        assert_eq!(
            merge::merge_btreemap(
                BTreeMap::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
                BTreeMap::from_iter([("b".to_string(), 3), ("c".to_string(), 4)])
            ),
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
                BTreeSet::from_iter(["a", "b", "c", "d"])
            ),
            Some(BTreeSet::from_iter(["a", "b", "c", "d"]))
        );
    }

    #[test]
    fn merge_hashmap() {
        assert_eq!(
            merge::merge_hashmap(
                HashMap::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
                HashMap::from_iter([("b".to_string(), 3), ("c".to_string(), 4)])
            ),
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
                HashSet::from_iter(["a", "b", "c", "d"])
            ),
            Some(HashSet::from_iter(["a", "b", "c", "d"]))
        );
    }
}
