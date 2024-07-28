use crate::config::MergeResult;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::Hash,
};

/// Discard both previous and next values and return [`None`].
pub fn discard<T, C>(_: T, _: T, _: &C) -> MergeResult<T> {
    Ok(None)
}

/// Always preserve the previous value over the next value.
pub fn preserve<T, C>(prev: T, _: T, _: &C) -> MergeResult<T> {
    Ok(Some(prev))
}

/// Always replace the previous value with the next value.
pub fn replace<T, C>(_: T, next: T, _: &C) -> MergeResult<T> {
    Ok(Some(next))
}

/// Append the items from the next vector to the end of the previous vector.
pub fn append_vec<T, C>(mut prev: Vec<T>, next: Vec<T>, _: &C) -> MergeResult<Vec<T>> {
    prev.extend(next);

    Ok(Some(prev))
}

/// Prepend the items from the next vector to the start of the previous vector.
pub fn prepend_vec<T, C>(prev: Vec<T>, next: Vec<T>, _: &C) -> MergeResult<Vec<T>> {
    let mut new = vec![];
    new.extend(next);
    new.extend(prev);

    Ok(Some(new))
}

/// Shallow merge the next [`BTreeMap`] into the previous [`BTreeMap`]. Any items in the
/// next [`BTreeMap`] will overwrite items in the previous [`BTreeMap`] of the same key.
pub fn merge_btreemap<K, V, C>(
    mut prev: BTreeMap<K, V>,
    next: BTreeMap<K, V>,
    _: &C,
) -> MergeResult<BTreeMap<K, V>>
where
    K: Eq + Hash + Ord,
{
    for (key, value) in next {
        prev.insert(key, value);
    }

    Ok(Some(prev))
}

/// Shallow merge the next [`BTreeSet`] into the previous [`BTreeSet`], overwriting duplicates.
pub fn merge_btreeset<T, C>(
    mut prev: BTreeSet<T>,
    next: BTreeSet<T>,
    _: &C,
) -> MergeResult<BTreeSet<T>>
where
    T: Eq + Hash + Ord,
{
    for item in next {
        prev.insert(item);
    }

    Ok(Some(prev))
}

/// Shallow merge the next [`HashMap`] into the previous [`HashMap`]. Any items in the
/// next [`HashMap`] will overwrite items in the previous [`HashMap`] of the same key.
pub fn merge_hashmap<K, V, C>(
    mut prev: HashMap<K, V>,
    next: HashMap<K, V>,
    _: &C,
) -> MergeResult<HashMap<K, V>>
where
    K: Eq + Hash,
{
    for (key, value) in next {
        prev.insert(key, value);
    }

    Ok(Some(prev))
}

/// Shallow merge the next [`HashSet`] into the previous [`HashSet`], overwriting duplicates.
pub fn merge_hashset<T, C>(mut prev: HashSet<T>, next: HashSet<T>, _: &C) -> MergeResult<HashSet<T>>
where
    T: Eq + Hash,
{
    for item in next {
        prev.insert(item);
    }

    Ok(Some(prev))
}
