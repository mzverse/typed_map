use alloc::collections::{btree_map, BTreeMap};
use core::borrow::Borrow;
#[allow(unused_imports)]
use core::hash::{BuildHasher, Hash};

pub trait OccupiedEntry<'a, K, V> {
    fn key(&self) -> &K;

    fn remove_entry(self) -> (K, V);

    fn get(&self) -> &V;

    fn get_mut(&mut self) -> &mut V;

    fn into_mut(self) -> &'a mut V;

    fn insert(&mut self, value: V) -> V;

    fn remove(self) -> V;
}
pub trait VacantEntry<'a, K, V> {
    type Occupied: OccupiedEntry<'a, K, V>;

    fn key(&self) -> &K;

    fn into_key(self) -> K;

    fn insert(self, value: V) -> &'a mut V;

    fn insert_entry(self, value: V) -> Self::Occupied;
}
pub enum Entry<Occupied, Vacant> {
    Occupied(Occupied),
    Vacant(Vacant),
}
use Entry::*;
pub trait Map<K, V>: IntoIterator<Item=(K, V)>
{
    type OccupiedEntry<'a>: OccupiedEntry<'a, K, V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;
    type VacantEntry<'a>: VacantEntry<'a, K, V, Occupied = Self::OccupiedEntry<'a>>
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

    fn entry(&mut self, key: K) -> Entry<Self::OccupiedEntry<'_>, Self::VacantEntry<'_>>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;

    fn iter(&self) -> Self::Iter<'_>;

    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    fn keys(&self) -> Self::Keys<'_>;

    fn values(&self) -> Self::Values<'_>;

    fn values_mut(&mut self) -> Self::ValuesMut<'_>;

    fn into_keys(self) -> Self::IntoKeys;

    fn into_values(self) -> Self::IntoValues;
}
pub trait MapQuery<K, V, Q: ?Sized>: Map<K, V> {
    fn contains_key(&self, key: &Q) -> bool;

    fn get(&self, key: &Q) -> Option<&V>;

    fn get_mut(&mut self, key: &Q) -> Option<&mut V>;

    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)>;

    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N];

    fn remove(&mut self, key: &Q) -> Option<V>;

    fn remove_entry(&mut self, key: &Q) -> Option<(K, V)>;
}

#[allow(unused_macros)]
macro_rules! for_hash {
    ($($hash_map:ident)::+, <$($EntryArgs:tt $(: $con:path)?),+>) => {
        impl<'a, $($EntryArgs $(: $con)?),+> OccupiedEntry<'a, K, V> for $($hash_map)::+::OccupiedEntry<'a, $($EntryArgs),*> {
            fn key(&self) -> &K {
                self.key()
            }

            fn remove_entry(self) -> (K, V) {
                self.remove_entry()
            }

            fn get(&self) -> &V {
                self.get()
            }

            fn get_mut(&mut self) -> &mut V {
                self.get_mut()
            }

            fn into_mut(self) -> &'a mut V {
                self.into_mut()
            }

            fn insert(&mut self, value: V) -> V {
                self.insert(value)
            }

            fn remove(self) -> V {
                self.remove()
            }
        }
        impl<'a, $($EntryArgs $(: $con)?),+> VacantEntry<'a, K, V> for $($hash_map)::+::VacantEntry<'a, $($EntryArgs),*> {
            type Occupied = $($hash_map)::+::OccupiedEntry<'a, $($EntryArgs),*>;

            fn key(&self) -> &K {
                self.key()
            }

            fn into_key(self) -> K {
                self.into_key()
            }

            fn insert(self, value: V) -> &'a mut V {
                self.insert(value)
            }

            fn insert_entry(self, value: V) -> Self::Occupied {
                self.insert_entry(value)
            }
        }
        impl<K, V, S> Map<K, V> for $($hash_map)::+::HashMap<K, V, S>
        where
            K: Eq + Hash,
            S: core::hash::BuildHasher,
        {
            type OccupiedEntry<'a> = $($hash_map)::+::OccupiedEntry<'a, $($EntryArgs),*>
            where
                K: 'a,
                V: 'a,
                S: 'a;
            type VacantEntry<'a> = $($hash_map)::+::VacantEntry<'a, $($EntryArgs),*>
            where
                K: 'a,
                V: 'a,
                S: 'a;

            type Iter<'a> = $($hash_map)::+::Iter<'a, K, V>
            where
                K: 'a,
                V: 'a,
                S: 'a;

            type IterMut<'a> = $($hash_map)::+::IterMut<'a, K, V>
            where
                K: 'a,
                V: 'a,
                S: 'a;

            type Keys<'a> = $($hash_map)::+::Keys<'a, K, V>
            where
                K: 'a,
                V: 'a,
                S: 'a;

            type Values<'a> = $($hash_map)::+::Values<'a, K, V>
            where
                K: 'a,
                V: 'a,
                S: 'a;

            type ValuesMut<'a> = $($hash_map)::+::ValuesMut<'a, K, V>
            where
                K: 'a,
                V: 'a,
                S: 'a;

            type IntoKeys = $($hash_map)::+::IntoKeys<K, V>;

            type IntoValues = $($hash_map)::+::IntoValues<K, V>;

            fn len(&self) -> usize {
                self.len()
            }

            fn is_empty(&self) -> bool {
                self.is_empty()
            }

            fn clear(&mut self) {
                self.clear()
            }

            fn entry(&mut self, key: K) -> Entry<Self::OccupiedEntry<'_>, Self::VacantEntry<'_>> {
                match $($hash_map)::+::HashMap::entry(self, key) {
                    $($hash_map)::+::Entry::Occupied(r) => Occupied(r),
                    $($hash_map)::+::Entry::Vacant(r) => Vacant(r),
                }
            }

            fn insert(&mut self, key: K, value: V) -> Option<V> {
                self.insert(key, value)
            }

            fn iter(&self) -> Self::Iter<'_> {
                self.iter()
            }

            fn iter_mut(&mut self) -> Self::IterMut<'_> {
                self.iter_mut()
            }

            fn keys(&self) -> Self::Keys<'_> {
                self.keys()
            }

            fn values(&self) -> Self::Values<'_> {
                self.values()
            }

            fn values_mut(&mut self) -> Self::ValuesMut<'_> {
                self.values_mut()
            }

            fn into_keys(self) -> Self::IntoKeys {
                self.into_keys()
            }

            fn into_values(self) -> Self::IntoValues {
                self.into_values()
            }
        }
        impl<K, V, Q: ?Sized, S> MapQuery<K, V, Q> for $($hash_map)::+::HashMap<K, V, S>
        where
            K: Eq + Hash,
            S: core::hash::BuildHasher,
            K: Borrow<Q>,
            Q: Eq + Hash,
        {
            fn contains_key(&self, key: &Q) -> bool {
                self.contains_key(key)
            }

            fn get(&self, key: &Q) -> Option<&V> {
                self.get(key)
            }

            fn get_mut(&mut self, key: &Q) -> Option<&mut V> {
                self.get_mut(key)
            }

            fn get_key_value(&self, key: &Q) -> Option<(&K, &V)> {
                self.get_key_value(key)
            }

            fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N] {
                self.get_disjoint_mut(ks)
            }

            fn remove(&mut self, key: &Q) -> Option<V> {
                self.remove(key)
            }

            fn remove_entry(&mut self, key: &Q) -> Option<(K, V)> {
                self.remove_entry(key)
            }
        }
    };
}

#[cfg(feature = "hashbrown")]
for_hash!(hashbrown::hash_map, <K: Hash, V, S: BuildHasher>);
#[cfg(not(feature = "no_std"))]
for_hash!(std::collections::hash_map, <K, V>);


impl<'a, K, V> OccupiedEntry<'a, K, V> for btree_map::OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    fn key(&self) -> &K {
        self.key()
    }

    fn remove_entry(self) -> (K, V) {
        self.remove_entry()
    }

    fn get(&self) -> &V {
        self.get()
    }

    fn get_mut(&mut self) -> &mut V {
        self.get_mut()
    }

    fn into_mut(self) -> &'a mut V {
        self.into_mut()
    }

    fn insert(&mut self, value: V) -> V {
        self.insert(value)
    }

    fn remove(self) -> V {
        self.remove()
    }
}
impl<'a, K, V> VacantEntry<'a, K, V> for btree_map::VacantEntry<'a, K, V>
where
    K: Ord,
{
    type Occupied = btree_map::OccupiedEntry<'a, K, V>;

    fn key(&self) -> &K {
        self.key()
    }

    fn into_key(self) -> K {
        self.into_key()
    }

    fn insert(self, value: V) -> &'a mut V {
        self.insert(value)
    }

    fn insert_entry(self, value: V) -> Self::Occupied {
        self.insert_entry(value)
    }
}
impl<K, V> Map<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    type OccupiedEntry<'a> = btree_map::OccupiedEntry<'a, K, V>
    where
        K: 'a,
        V: 'a;
    type VacantEntry<'a> = btree_map::VacantEntry<'a, K, V>
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
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn entry(&mut self, key: K) -> Entry<Self::OccupiedEntry<'_>, Self::VacantEntry<'_>> {
        match BTreeMap::entry(self, key) {
            btree_map::Entry::Occupied(r) => Occupied(r),
            btree_map::Entry::Vacant(r) => Vacant(r),
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.iter_mut()
    }

    fn keys(&self) -> Self::Keys<'_> {
        self.keys()
    }

    fn values(&self) -> Self::Values<'_> {
        self.values()
    }

    fn values_mut(&mut self) -> Self::ValuesMut<'_> {
        self.values_mut()
    }

    fn into_keys(self) -> Self::IntoKeys {
        self.into_keys()
    }

    fn into_values(self) -> Self::IntoValues {
        self.into_values()
    }
}
impl<K, V, Q: ?Sized> MapQuery<K, V, Q> for BTreeMap<K, V>
where
    K: Ord,
    K: Borrow<Q>,
    Q: Ord,
{
    fn contains_key(&self, key: &Q) -> bool {
        self.contains_key(key)
    }

    fn get(&self, key: &Q) -> Option<&V> {
        self.get(key)
    }

    fn get_mut(&mut self, key: &Q) -> Option<&mut V> {
        self.get_mut(key)
    }

    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)> {
        self.get_key_value(key)
    }

    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&mut V>; N] {
        unsafe { unsafe_get_disjoint_mut(self, ks) }
    }

    fn remove(&mut self, key: &Q) -> Option<V> {
        self.remove(key)
    }

    fn remove_entry(&mut self, key: &Q) -> Option<(K, V)> {
        self.remove_entry(key)
    }
}

pub trait EntryImpl<'a, K, V, Occupied: OccupiedEntry<'a, K, V>, Vacant: VacantEntry<'a, K, V>> {
    fn or_insert(self, default: V) -> &'a mut V;

    fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V;

    fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V;

    fn key(&self) -> &K;

    fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V);

    fn insert_entry(self, value: V) -> Occupied;
}
impl<'a, K, V, Occupied: OccupiedEntry<'a, K, V>, Vacant: VacantEntry<'a, K, V, Occupied = Occupied>> EntryImpl<'a, K, V, Occupied, Vacant> for Entry<Occupied, Vacant> {
    fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default),
        }
    }

    fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default()),
        }
    }

    fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    fn key(&self) -> &K {
        match *self {
            Occupied(ref entry) => entry.key(),
            Vacant(ref entry) => entry.key(),
        }
    }

    fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Occupied(mut entry) => {
                f(entry.get_mut());
                Occupied(entry)
            }
            Vacant(entry) => Vacant(entry),
        }
    }

    fn insert_entry(self, value: V) -> Occupied {
        match self {
            Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Vacant(entry) => entry.insert_entry(value),
        }
    }
}
unsafe fn unsafe_get_disjoint_mut<'s, K, V, Q: ?Sized, S: MapQuery<K, V, Q>, const N: usize>(s: &'s mut S, ks: [&Q; N]) -> [Option<&'s mut V>; N] {
    #[cfg(feature = "hashbrown")]
    macro_rules! hash_set {
        ($N: expr) => { hashbrown::HashSet::with_capacity($N) };
    }
    #[cfg(all(not(feature = "hashbrown"), not(feature = "no_std")))]
    macro_rules! hash_set {
        ($N: expr) => { std::collections::HashSet::with_capacity($N) };
    }
    #[cfg(all(not(feature = "hashbrown"), feature = "no_std"))]
    mod set {
        use alloc::vec::Vec;

        pub struct HashSet<const N: usize>([Vec<usize>; N]);
        impl<const N: usize> HashSet<N> {
            pub fn new() -> Self {
                Self(core::array::from_fn(|_| Vec::new()))
            }
            pub fn insert(&mut self, value: usize) -> bool {
                let slots = &mut self.0[value % N];
                if slots.contains(&value) {
                    false
                } else {
                    slots.push(value);
                    true
                }
            }
        }
    }
    #[cfg(all(not(feature = "hashbrown"), feature = "no_std"))]
    macro_rules! hash_set {
        ($N: expr) => { set::HashSet::<$N>::new() };
    }
    let mut set = hash_set!(N);
    ks.map(|k| {
        let r = unsafe { core::mem::transmute::<&mut S, &mut S>(s) }.get_mut(k);
        if let Some(r) = r {
            if set.insert(r as *const V as usize) {
                return Some(r);
            }
        }
        return None;
    })
}
