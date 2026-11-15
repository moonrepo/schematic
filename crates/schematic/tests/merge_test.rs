#![allow(dead_code)]

use indexmap::{IndexMap, IndexSet};
use schematic::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::hash::BuildHasherDefault;

// A map with a hasher of its own, like `FxHashMap`
type CustomHasher = BuildHasherDefault<DefaultHasher>;
type CustomMap<K, V> = HashMap<K, V, CustomHasher>;
type CustomSet<T> = HashSet<T, CustomHasher>;

#[derive(Config)]
pub struct MergeNormal {
    string: String,
    vector: Vec<usize>,
    map: HashMap<String, usize>,
}

fn merge_string<C>(prev: String, next: String, _: &C) -> MergeResult<String> {
    Ok(Some(format!("{prev}-{next}")))
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
    let result = ConfigLoader::<MergeBase>::new().load().unwrap();

    assert_eq!(result.config.string, "abc");
    assert_eq!(result.config.vector, vec![1, 2, 3]);
    assert_eq!(result.config.nested.string, "xyz");
    assert_eq!(result.config.nested.other, 10);
    assert!(result.config.opt_nested.is_none());
}

#[test]
fn can_merge_with_defaults() {
    let result = ConfigLoader::<MergeBase>::new()
        .code("string: def", "code.yaml")
        .unwrap()
        .code("vector: [4]", "code.yaml")
        .unwrap()
        .code(
            r"vector: [5]
nested:
  string: zyx
",
            "code.yaml",
        )
        .unwrap()
        .code(
            r"nested:
  other: 15
",
            "code.yaml",
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
    let result = ConfigLoader::<MergeBase>::new()
        .code(
            r"
opt_nested:
    string: hij",
            "code.yaml",
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
                HashMap::<_, _>::from_iter([("a".to_string(), 1), ("b".to_string(), 2)]),
                HashMap::<_, _>::from_iter([("b".to_string(), 3), ("c".to_string(), 4)]),
                &()
            )
            .unwrap(),
            Some(HashMap::<_, _>::from_iter([
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
                HashSet::<_>::from_iter(["a", "b"]),
                HashSet::<_>::from_iter(["a", "b", "c", "d"]),
                &()
            )
            .unwrap(),
            Some(HashSet::<_>::from_iter(["a", "b", "c", "d"]))
        );
    }

    #[test]
    fn append_vec_unique() {
        assert_eq!(
            merge::append_vec_unique(vec![1, 2, 1], vec![2, 3, 3, 4], &()).unwrap(),
            Some(vec![1, 2, 1, 3, 4])
        );
    }

    #[test]
    fn merge_iter() {
        assert_eq!(
            merge::merge_iter(VecDeque::from([1, 2]), VecDeque::from([2, 3]), &()).unwrap(),
            Some(VecDeque::from([1, 2, 2, 3]))
        );
        assert_eq!(
            merge::merge_iter(
                BTreeMap::from_iter([("a", 1), ("b", 2)]),
                BTreeMap::from_iter([("b", 3), ("c", 4)]),
                &()
            )
            .unwrap(),
            Some(BTreeMap::from_iter([("a", 1), ("b", 3), ("c", 4)]))
        );
    }

    #[test]
    fn merge_hashmap_with_a_custom_hasher() {
        assert_eq!(
            merge::merge_hashmap(
                CustomMap::from_iter([("a", 1), ("b", 2)]),
                CustomMap::from_iter([("b", 3), ("c", 4)]),
                &()
            )
            .unwrap(),
            Some(CustomMap::from_iter([("a", 1), ("b", 3), ("c", 4)]))
        );
    }

    #[test]
    fn merge_hashset_with_a_custom_hasher() {
        assert_eq!(
            merge::merge_hashset(
                CustomSet::from_iter(["a", "b"]),
                CustomSet::from_iter(["b", "c"]),
                &()
            )
            .unwrap(),
            Some(CustomSet::from_iter(["a", "b", "c"]))
        );
    }

    // Neither key type implements `Hash`
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Unhashable(usize);

    #[test]
    fn merge_btree_without_hash() {
        assert_eq!(
            merge::merge_btreemap(
                BTreeMap::from_iter([(Unhashable(1), 1)]),
                BTreeMap::from_iter([(Unhashable(1), 2), (Unhashable(2), 3)]),
                &()
            )
            .unwrap(),
            Some(BTreeMap::from_iter([
                (Unhashable(1), 2),
                (Unhashable(2), 3)
            ]))
        );
        assert_eq!(
            merge::merge_btreeset(
                BTreeSet::from_iter([Unhashable(1)]),
                BTreeSet::from_iter([Unhashable(2)]),
                &()
            )
            .unwrap(),
            Some(BTreeSet::from_iter([Unhashable(1), Unhashable(2)]))
        );
    }

    #[test]
    fn merge_indexmap() {
        let merged = merge::merge_indexmap(
            IndexMap::<_, _>::from_iter([("b", 1), ("a", 2)]),
            IndexMap::from_iter([("c", 3), ("b", 4)]),
            &(),
        )
        .unwrap()
        .unwrap();

        // A key keeps its first position
        assert_eq!(
            merged.into_iter().collect::<Vec<_>>(),
            vec![("b", 4), ("a", 2), ("c", 3)]
        );
    }

    #[test]
    fn merge_indexset() {
        let merged = merge::merge_indexset(
            IndexSet::<_>::from_iter(["b", "a"]),
            IndexSet::from_iter(["c", "b"]),
            &(),
        )
        .unwrap()
        .unwrap();

        assert_eq!(merged.into_iter().collect::<Vec<_>>(), vec!["b", "a", "c"]);
    }

    fn nested(string: Option<&str>, other: Option<usize>) -> PartialMergeNested {
        PartialMergeNested {
            string: string.map(|value| value.into()),
            other,
        }
    }

    #[test]
    fn merge_partials() {
        let prev = || {
            [
                ("a", nested(Some("a"), Some(1))),
                ("b", nested(Some("b"), None)),
            ]
        };
        let next = || [("b", nested(None, Some(2))), ("c", nested(Some("c"), None))];
        let expected = || {
            [
                ("a", nested(Some("a"), Some(1))),
                ("b", nested(Some("b"), Some(2))),
                ("c", nested(Some("c"), None)),
            ]
        };

        assert_eq!(
            merge::merge_partials(
                HashMap::<_, _>::from_iter(prev()),
                HashMap::from_iter(next()),
                &()
            )
            .unwrap(),
            Some(HashMap::from_iter(expected()))
        );
        assert_eq!(
            merge::merge_partials(
                BTreeMap::from_iter(prev()),
                BTreeMap::from_iter(next()),
                &()
            )
            .unwrap(),
            Some(BTreeMap::from_iter(expected()))
        );
        assert_eq!(
            merge::merge_partials(
                IndexMap::<_, _>::from_iter(prev()),
                IndexMap::from_iter(next()),
                &()
            )
            .unwrap()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
            Vec::from(expected())
        );
    }
}

mod loading {
    use super::*;

    #[derive(Debug, Config)]
    struct Plugin {
        #[setting(default = "latest")]
        version: String,

        enabled: bool,
    }

    #[derive(Debug, Config)]
    struct Plugins {
        #[setting(default = vec!["a".to_owned()], merge = merge::append_vec_unique)]
        list: Vec<String>,

        #[setting(merge = merge::merge_iter)]
        queue: VecDeque<usize>,

        #[setting(merge = merge::merge_hashmap)]
        custom: CustomMap<String, usize>,

        #[setting(merge = merge::merge_indexset)]
        ordered: IndexSet<String>,

        #[setting(nested, merge = merge::merge_partials)]
        plugins: HashMap<String, Plugin>,

        #[setting(nested, merge = merge::merge_partials)]
        sorted: BTreeMap<String, Plugin>,

        #[setting(nested, merge = merge::merge_partials)]
        indexed: IndexMap<String, Plugin>,
    }

    fn load() -> Plugins {
        ConfigLoader::<Plugins>::new()
            .code(
                r"
list: [a, b]
queue: [1]
custom: { x: 1, y: 2 }
ordered: [y, x]
plugins:
  foo: { version: '1.0' }
  bar: { enabled: true }
sorted:
  foo: { enabled: true }
indexed:
  foo: { version: '1.0' }
",
                "a.yml",
            )
            .unwrap()
            .code(
                r"
list: [b, c]
queue: [2]
custom: { y: 3 }
ordered: [z, x]
plugins:
  foo: { enabled: true }
  baz: { version: '2.0' }
sorted:
  foo: { version: '3.0' }
indexed:
  bar: { enabled: true }
  foo: { enabled: true }
",
                "b.yml",
            )
            .unwrap()
            .load()
            .unwrap()
            .config
    }

    #[test]
    fn appends_unique_items_over_defaults() {
        assert_eq!(load().list, vec!["a", "b", "c"]);
    }

    #[test]
    fn extends_any_collection() {
        let config = load();

        assert_eq!(config.queue, VecDeque::from([1, 2]));
        assert_eq!(
            config.custom,
            CustomMap::from_iter([("x".into(), 1), ("y".into(), 3)])
        );
        assert_eq!(
            config.ordered.into_iter().collect::<Vec<_>>(),
            vec!["y", "x", "z"]
        );
    }

    #[test]
    fn deep_merges_partials_by_key() {
        let config = load();

        let foo = &config.plugins["foo"];
        assert_eq!(foo.version, "1.0");
        assert!(foo.enabled);

        // Items are finalized, so a key from one layer still gets defaults
        let bar = &config.plugins["bar"];
        assert_eq!(bar.version, "latest");
        assert!(bar.enabled);

        let baz = &config.plugins["baz"];
        assert_eq!(baz.version, "2.0");
        assert!(!baz.enabled);

        let foo = &config.sorted["foo"];
        assert_eq!(foo.version, "3.0");
        assert!(foo.enabled);

        assert_eq!(
            config
                .indexed
                .iter()
                .map(|(key, plugin)| (key.as_str(), plugin.version.as_str(), plugin.enabled))
                .collect::<Vec<_>>(),
            vec![("foo", "1.0", true), ("bar", "latest", true)]
        );
    }

    fn fail<C>(_: String, _: String, _: &C) -> MergeResult<String> {
        Err(MergeError::new("can't merge"))
    }

    #[derive(Debug, Config)]
    struct Failing {
        #[setting(merge = fail)]
        value: String,
    }

    #[derive(Debug, Config)]
    struct FailingMap {
        #[setting(nested, merge = merge::merge_partials)]
        items: HashMap<String, Failing>,
    }

    #[test]
    fn reports_an_item_that_fails_to_merge() {
        let error = ConfigLoader::<FailingMap>::new()
            .code("items: { a: { value: x } }", "a.yml")
            .unwrap()
            .code("items: { a: { value: y } }", "b.yml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(error.to_full_string(), "can't merge");
    }
}
