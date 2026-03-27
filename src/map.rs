use std::borrow::Borrow;
use std::collections::{btree_map, hash_map, BTreeMap, HashMap, HashSet};
use std::hash::{BuildHasher, Hash};

pub trait MapEntry<K, V> {
    // TODO
}
pub trait Map<K, V, Q: ?Sized>: IntoIterator<Item=(K, V)>
{
    type Entry<'a>: MapEntry<K, V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type Iter<'a>: Iterator<Item = (&'a K, &'a V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type IterMut<'a>: Iterator<Item = (&'a K, &'a mut V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type Keys<'a>: Iterator<Item = &'a K>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type Values<'a>: Iterator<Item = &'a V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type ValuesMut<'a>: Iterator<Item = &'a mut V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type IntoKeys: Iterator<Item = K>;

    type IntoValues: Iterator<Item = V>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn clear(&mut self);

    fn entry(&mut self, key: K) -> Self::Entry<'_>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;

    fn contains_key(&self, key: &Q) -> bool;

    fn get(&self, key: &Q) -> Option<&V>;

    fn get_mut(&mut self, key: &Q) -> Option<&mut V>;

    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)>;

    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N];

    fn remove(&mut self, key: &Q) -> Option<V>;

    fn remove_entry(&mut self, key: &Q) -> Option<(K, V)>;

    fn iter(&self) -> Self::Iter<'_>;

    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    fn keys(&self) -> Self::Keys<'_>;

    fn values(&self) -> Self::Values<'_>;

    fn values_mut(&mut self) -> Self::ValuesMut<'_>;

    fn into_keys(self) -> Self::IntoKeys;

    fn into_values(self) -> Self::IntoValues;
}
impl<K, V> MapEntry<K, V> for hash_map::Entry<'_, K, V> {

}
impl<K, V, Q: ?Sized, S> Map<K, V, Q> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
    K: Borrow<Q>,
    Q: Eq + Hash,
{
    type Entry<'a> = hash_map::Entry<'a, K, V>
    where
        K: 'a,
        V: 'a,
        S: 'a;

    type Iter<'a> = hash_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a,
        S: 'a;

    type IterMut<'a> = hash_map::IterMut<'a, K, V>
    where
        K: 'a,
        V: 'a,
        S: 'a;

    type Keys<'a> = hash_map::Keys<'a, K, V>
    where
        K: 'a,
        V: 'a,
        S: 'a;

    type Values<'a> = hash_map::Values<'a, K, V>
    where
        K: 'a,
        V: 'a,
        S: 'a;

    type ValuesMut<'a> = hash_map::ValuesMut<'a, K, V>
    where
        K: 'a,
        V: 'a,
        S: 'a;

    type IntoKeys = hash_map::IntoKeys<K, V>;

    type IntoValues = hash_map::IntoValues<K, V>;

    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        HashMap::is_empty(self)
    }

    fn clear(&mut self) {
        HashMap::clear(self)
    }

    fn entry(&mut self, key: K) -> Self::Entry<'_> {
        HashMap::entry(self, key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        HashMap::insert(self, key, value)
    }

    fn contains_key(&self, key: &Q) -> bool {
        HashMap::contains_key(self, key)
    }

    fn get(&self, key: &Q) -> Option<&V> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &Q) -> Option<&mut V> {
        HashMap::get_mut(self, key)
    }

    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)> {
        HashMap::get_key_value(self, key)
    }

    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N] {
        HashMap::get_disjoint_mut(self, ks)
    }

    fn remove(&mut self, key: &Q) -> Option<V> {
        HashMap::remove(self, key)
    }

    fn remove_entry(&mut self, key: &Q) -> Option<(K, V)> {
        HashMap::remove_entry(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        HashMap::iter(self)
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        HashMap::iter_mut(self)
    }

    fn keys(&self) -> Self::Keys<'_> {
        HashMap::keys(self)
    }

    fn values(&self) -> Self::Values<'_> {
        HashMap::values(self)
    }

    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        HashMap::values_mut(self)
    }

    fn into_keys(self) -> Self::IntoKeys {
        HashMap::into_keys(self)
    }

    fn into_values(self) -> Self::IntoValues {
        HashMap::into_values(self)
    }
}
impl<K, V> MapEntry<K, V> for btree_map::Entry<'_, K, V> {

}
impl<K, V, Q: ?Sized> Map<K, V, Q> for BTreeMap<K, V>
where
    K: Ord,
    K: Borrow<Q>,
    Q: Ord,
{
    type Entry<'a> = btree_map::Entry<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type Iter<'a> = btree_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type IterMut<'a> = btree_map::IterMut<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type Keys<'a> = btree_map::Keys<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type Values<'a> = btree_map::Values<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type ValuesMut<'a> = btree_map::ValuesMut<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type IntoKeys = btree_map::IntoKeys<K, V>;

    type IntoValues = btree_map::IntoValues<K, V>;

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeMap::is_empty(self)
    }

    fn clear(&mut self) {
        BTreeMap::clear(self)
    }

    fn entry(&mut self, key: K) -> Self::Entry<'_> {
        BTreeMap::entry(self, key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }

    fn contains_key(&self, key: &Q) -> bool {
        BTreeMap::contains_key(self, key)
    }

    fn get(&self, key: &Q) -> Option<&V> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &Q) -> Option<&mut V> {
        BTreeMap::get_mut(self, key)
    }

    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)> {
        BTreeMap::get_key_value(self, key)
    }

    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N] {
        unsafe { unsafe_get_disjoint_mut(self, ks) }
    }

    fn remove(&mut self, key: &Q) -> Option<V> {
        BTreeMap::remove(self, key)
    }

    fn remove_entry(&mut self, key: &Q) -> Option<(K, V)> {
        BTreeMap::remove_entry(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        BTreeMap::iter(self)
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        BTreeMap::iter_mut(self)
    }

    fn keys(&self) -> Self::Keys<'_> {
        BTreeMap::keys(self)
    }

    fn values(&self) -> Self::Values<'_> {
        BTreeMap::values(self)
    }

    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        BTreeMap::values_mut(self)
    }

    fn into_keys(self) -> Self::IntoKeys {
        BTreeMap::into_keys(self)
    }

    fn into_values(self) -> Self::IntoValues {
        BTreeMap::into_values(self)
    }
}
unsafe fn unsafe_get_disjoint_mut<'s, K, V, Q: ?Sized, S: Map<K, V, Q>, const N: usize>(s: &'s mut S, ks: [&Q; N]) -> [Option<&'s mut V>; N] {
    let mut set = HashSet::<*mut V>::with_capacity(N);
    ks.map(|k| {
        let r = unsafe { std::mem::transmute::<&mut S, &mut S>(s) }.get_mut(k);
        if let Some(r) = r {
            if set.insert(r) {
                return Some(r);
            }
        }
        return None;
    })
}
