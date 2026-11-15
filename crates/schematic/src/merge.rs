use crate::config::{MergeError, MergeResult, PartialConfig};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{BuildHasher, Hash},
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

/// Append the items from the next vector to the end of the previous vector,
/// skipping any item the result already contains, so each item stays where it
/// was first seen. Duplicates within the previous vector are kept.
///
/// Each item is compared with every item before it, which suits the short
/// lists a config holds.
pub fn append_vec_unique<T: PartialEq, C>(
    mut prev: Vec<T>,
    next: Vec<T>,
    _: &C,
) -> MergeResult<Vec<T>> {
    for item in next {
        if !prev.contains(&item) {
            prev.push(item);
        }
    }

    Ok(Some(prev))
}

/// Prepend the items from the next vector to the start of the previous vector.
pub fn prepend_vec<T, C>(prev: Vec<T>, next: Vec<T>, _: &C) -> MergeResult<Vec<T>> {
    let mut new = vec![];
    new.extend(next);
    new.extend(prev);

    Ok(Some(new))
}

/// Extend the previous collection with the items of the next, for any collection
/// that implements [`Extend`], like a [`VecDeque`](std::collections::VecDeque),
/// or a map or set with a custom hasher.
///
/// The collection decides how a duplicate is handled: a map keeps the next value
/// of a key, a set keeps the previous item, and a vector keeps both.
pub fn merge_iter<I, C>(mut prev: I, next: I, _: &C) -> MergeResult<I>
where
    I: IntoIterator + Extend<I::Item>,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// Shallow merge the next [`BTreeMap`] into the previous [`BTreeMap`]. Any items in the
/// next [`BTreeMap`] will overwrite items in the previous [`BTreeMap`] of the same key.
pub fn merge_btreemap<K, V, C>(
    mut prev: BTreeMap<K, V>,
    next: BTreeMap<K, V>,
    _: &C,
) -> MergeResult<BTreeMap<K, V>>
where
    K: Ord,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// Shallow merge the next [`BTreeSet`] into the previous [`BTreeSet`], overwriting duplicates.
pub fn merge_btreeset<T, C>(
    mut prev: BTreeSet<T>,
    next: BTreeSet<T>,
    _: &C,
) -> MergeResult<BTreeSet<T>>
where
    T: Ord,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// Shallow merge the next [`HashMap`] into the previous [`HashMap`]. Any items in the
/// next [`HashMap`] will overwrite items in the previous [`HashMap`] of the same key.
/// Supports any hasher, like `FxHashMap`'s.
pub fn merge_hashmap<K, V, S, C>(
    mut prev: HashMap<K, V, S>,
    next: HashMap<K, V, S>,
    _: &C,
) -> MergeResult<HashMap<K, V, S>>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// Shallow merge the next [`HashSet`] into the previous [`HashSet`], overwriting duplicates.
/// Supports any hasher, like `FxHashSet`'s.
pub fn merge_hashset<T, S, C>(
    mut prev: HashSet<T, S>,
    next: HashSet<T, S>,
    _: &C,
) -> MergeResult<HashSet<T, S>>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// Shallow merge the next [`IndexMap`](indexmap::IndexMap) into the previous one. Any items
/// in the next map will overwrite items in the previous map of the same key, keeping the
/// position of the previous key, while new keys are added to the end.
#[cfg(feature = "type_indexmap")]
pub fn merge_indexmap<K, V, S, C>(
    mut prev: indexmap::IndexMap<K, V, S>,
    next: indexmap::IndexMap<K, V, S>,
    _: &C,
) -> MergeResult<indexmap::IndexMap<K, V, S>>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// Shallow merge the next [`IndexSet`](indexmap::IndexSet) into the previous one, keeping the
/// position of duplicates, while new items are added to the end.
#[cfg(feature = "type_indexmap")]
pub fn merge_indexset<T, S, C>(
    mut prev: indexmap::IndexSet<T, S>,
    next: indexmap::IndexSet<T, S>,
    _: &C,
) -> MergeResult<indexmap::IndexSet<T, S>>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    prev.extend(next);

    Ok(Some(prev))
}

/// A map whose values [`merge_partials`] can merge by key.
///
/// Implemented for [`HashMap`] and [`BTreeMap`], and for `IndexMap` with the
/// `type_indexmap` feature.
pub trait MergeableMap<K, V>: IntoIterator<Item = (K, V)> {
    /// Return the value of the key, if the map holds it.
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    /// Insert the value at the key.
    fn insert(&mut self, key: K, value: V);
}

impl<K: Eq + Hash, V, S: BuildHasher> MergeableMap<K, V> for HashMap<K, V, S> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        HashMap::get_mut(self, key)
    }

    fn insert(&mut self, key: K, value: V) {
        HashMap::insert(self, key, value);
    }
}

impl<K: Ord, V> MergeableMap<K, V> for BTreeMap<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        BTreeMap::get_mut(self, key)
    }

    fn insert(&mut self, key: K, value: V) {
        BTreeMap::insert(self, key, value);
    }
}

#[cfg(feature = "type_indexmap")]
impl<K: Eq + Hash, V, S: BuildHasher> MergeableMap<K, V> for indexmap::IndexMap<K, V, S> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        indexmap::IndexMap::get_mut(self, key)
    }

    fn insert(&mut self, key: K, value: V) {
        indexmap::IndexMap::insert(self, key, value);
    }
}

/// Deep merge the next map of partial configs into the previous map. A key in
/// both has its next partial merged into its previous partial, with the
/// partial's own merge strategies, while a key only in the next map is added.
///
/// For a nested map, like `#[setting(nested, merge = merge_partials)]`, as a nested
/// collection is otherwise replaced. The partials receive the context as is, so
/// it can't be combined with a `context` function.
pub fn merge_partials<M, K, V, C>(mut prev: M, next: M, context: &C) -> MergeResult<M>
where
    M: MergeableMap<K, V>,
    V: PartialConfig<Context = C>,
{
    for (key, value) in next {
        match prev.get_mut(&key) {
            Some(existing) => {
                existing.merge(context, value).map_err(MergeError::new)?;
            }
            None => {
                prev.insert(key, value);
            }
        }
    }

    Ok(Some(prev))
}
