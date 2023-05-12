#![allow(dead_code)]

use schematic::*;
use std::collections::HashMap;

#[derive(Config)]
pub struct MergeNormal {
    string: String,
    vector: Vec<usize>,
    map: HashMap<String, usize>,
}

fn merge_string(prev: String, next: String) -> Option<String> {
    Some(format!("{}-{}", prev, next))
}

fn merge_vec(prev: Vec<usize>, next: Vec<usize>) -> Option<Vec<usize>> {
    let mut new = vec![];
    new.extend(next);
    new.extend(prev);

    Some(new)
}

fn merge_map<T>(_: T, _: T) -> Option<T> {
    None
}

#[derive(Config)]
pub struct MergeFunc {
    #[setting(merge = merge_string)]
    string: String,
    #[setting(merge = merge_vec)]
    vector: Vec<usize>,
    #[setting(merge = merge_map)]
    map: HashMap<String, usize>,
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
    };

    base.merge(PartialMergeFunc {
        string: Some("bar".into()),
        vector: Some(vec![4, 5, 6]),
        map: Some(HashMap::from_iter([("b".into(), 2)])),
    });

    assert_eq!(base.string.unwrap(), "foo-bar");
    assert_eq!(base.vector.unwrap(), vec![4, 5, 6, 1, 2, 3]);
    assert_eq!(base.map, None);
}
