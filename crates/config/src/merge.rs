use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::Hash,
};

/// Discard both previous and next values and return [None].
pub fn discard<T>(_: T, _: T) -> Option<T> {
    None
}

/// Always preserve the previous value over the next value.
pub fn preserve<T>(prev: T, _: T) -> Option<T> {
    Some(prev)
}

/// Always replace the previous value with the next value.
pub fn replace<T>(_: T, next: T) -> Option<T> {
    Some(next)
}

/// Append the items from the next vector to the end of the previous vector.
pub fn append_vec<T>(mut prev: Vec<T>, next: Vec<T>) -> Option<Vec<T>> {
    prev.extend(next);

    Some(prev)
}

/// Prepend the items from the next vector to the start of the previous vector.
pub fn prepend_vec<T>(prev: Vec<T>, next: Vec<T>) -> Option<Vec<T>> {
    let mut new = vec![];
    new.extend(next);
    new.extend(prev);

    Some(new)
}

/// Shallow merge the next [BTreeMap] into the previous [BTreeMap]. Any items in the
/// next [BTreeMap] will overwrite items in the previous [BTreeMap] of the same key.
pub fn merge_btreemap<K, V>(
    mut prev: BTreeMap<K, V>,
    next: BTreeMap<K, V>,
) -> Option<BTreeMap<K, V>>
where
    K: Eq + Hash + Ord,
{
    for (key, value) in next {
        prev.insert(key, value);
    }

    Some(prev)
}

/// Shallow merge the next [BTreeSet] into the previous [BTreeSet], overwriting duplicates.
pub fn merge_btreeset<T>(mut prev: BTreeSet<T>, next: BTreeSet<T>) -> Option<BTreeSet<T>>
where
    T: Eq + Hash + Ord,
{
    for item in next {
        prev.insert(item);
    }

    Some(prev)
}

/// Shallow merge the next [HashMap] into the previous [HashMap]. Any items in the
/// next [HashMap] will overwrite items in the previous [HashMap] of the same key.
pub fn merge_hashmap<K, V>(mut prev: HashMap<K, V>, next: HashMap<K, V>) -> Option<HashMap<K, V>>
where
    K: Eq + Hash,
{
    for (key, value) in next {
        prev.insert(key, value);
    }

    Some(prev)
}

/// Shallow merge the next [HashSet] into the previous [HashSet], overwriting duplicates.
pub fn merge_hashset<T>(mut prev: HashSet<T>, next: HashSet<T>) -> Option<HashSet<T>>
where
    T: Eq + Hash,
{
    for item in next {
        prev.insert(item);
    }

    Some(prev)
}
