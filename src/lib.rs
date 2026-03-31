#![cfg_attr(all(feature = "no_std", not(test)), no_std)]

extern crate alloc;

pub mod map;

use core::any::{Any, TypeId};
use core::cmp::Ordering;
#[allow(unused_imports)]
use core::hash::{Hash, Hasher, BuildHasher};

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::marker::PhantomData;

use crate::map::{Entry, MapQuery};


// // util
// fn is_dyn_any<T: ?Sized + Any>(any: &T) -> bool {
//     any.type_id() != TypeId::of::<T>()
// }

// key
pub trait MapType {
    type Key<T>;
    type Value<T>;
}

pub trait Key<T: Sized>: Any {
    fn new(data: T) -> Box<Self>;

    fn borrow(data: &T) -> &Self;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    fn as_any(&self) -> &dyn Any;
}

// impl
macro_rules! define {
    () => {
        #[derive(Clone, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct TypedMap<Type: MapType, K: ?Sized + Any, M> (M, PhantomData<(Type, K)>);
    };
    ($($DefaultHasher: ident)::+, $($HashMap: ident)::+) => {
        #[derive(Clone, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct TypedMap<Type: MapType, K: ?Sized + Any = dyn KeyDataHash<$($DefaultHasher)::+>, M = $($HashMap)::+<Box<K>, Box<dyn Any>>> (M, PhantomData<(Type, K)>);
    };
}
#[cfg(feature = "hashbrown")]
define!(hashbrown::DefaultHasher, hashbrown::HashMap);
#[cfg(all(not(feature = "hashbrown"), not(feature = "no_std")))]
define!(std::hash::DefaultHasher, std::collections::HashMap);
#[cfg(all(not(feature = "hashbrown"), feature = "no_std"))]
define!();
impl<Type: MapType, K: ?Sized + Any, M> TypedMap<Type, K, M> {
    pub fn with_inner(inner: M) -> Self {
        Self(inner, PhantomData)
    }
}

pub struct OccupiedEntry<'a, Type: MapType, T, K: ?Sized + Any, M>(M::OccupiedEntry<'a>, PhantomData<(Type, T, K)>)
where
    M: MapQuery<Box<K>, Box<dyn Any>, K> + 'a;
pub struct VacantEntry<'a, Type: MapType, T, K: ?Sized + Any, M>(M::VacantEntry<'a>, PhantomData<(Type, T, K)>)
where
    M: MapQuery<Box<K>, Box<dyn Any>, K> + 'a;

impl<'a, Type: MapType, T, K: ?Sized + Any, M> map::OccupiedEntry<'a, Type::Key<T>, Type::Value<T>> for OccupiedEntry<'a, Type, T, K, M>
where
    M: MapQuery<Box<K>, Box<dyn Any>, K> + 'a,
    K: Key<Type::Key<T>>,
    Type::Key<T>: 'static,
    Type::Value<T>: 'static,
{
    fn key(&self) -> &Type::Key<T> {
        self.0.key().as_any().downcast_ref().unwrap()
    }

    fn remove_entry(self) -> (Type::Key<T>, Type::Value<T>) {
        let (k, v) = self.0.remove_entry();
        (*k.into_any().downcast().unwrap(), *v.downcast().unwrap())
    }

    fn get(&self) -> &Type::Value<T> {
        self.0.get().downcast_ref().unwrap()
    }

    fn get_mut(&mut self) -> &mut Type::Value<T> {
        self.0.get_mut().downcast_mut().unwrap()
    }

    fn into_mut(self) -> &'a mut Type::Value<T> {
        self.0.into_mut().downcast_mut().unwrap()
    }

    fn insert(&mut self, value: Type::Value<T>) -> Type::Value<T> {
        *self.0.insert(Box::new(value)).downcast().unwrap()
    }

    fn remove(self) -> Type::Value<T> {
        *self.0.remove().downcast().unwrap()
    }
}
impl<'a, Type: MapType, T, K: ?Sized + Any, M> map::VacantEntry<'a, Type::Key<T>, Type::Value<T>> for VacantEntry<'a, Type, T, K, M>
where
    M: MapQuery<Box<K>, Box<dyn Any>, K> + 'a,
    K: Key<Type::Key<T>>,
    Type::Key<T>: 'static,
    Type::Value<T>: 'static,
{
    type Occupied = OccupiedEntry<'a, Type, T, K, M>;

    fn key(&self) -> &Type::Key<T> {
        self.0.key().as_any().downcast_ref().unwrap()
    }

    fn into_key(self) -> Type::Key<T> {
        *self.0.into_key().into_any().downcast().unwrap()
    }

    fn insert(self, value: Type::Value<T>) -> &'a mut Type::Value<T> {
        self.0.insert(Box::new(value)).downcast_mut().unwrap()
    }

    fn insert_entry(self, value: Type::Value<T>) -> Self::Occupied {
        OccupiedEntry(self.0.insert_entry(Box::new(value)), PhantomData)
    }
}

pub trait Impl<Type: MapType, T, K: ?Sized + Any, M>
where
    M: MapQuery<Box<K>, Box<dyn Any>, K>,
{
    fn entry(&mut self, key: Type::Key<T>) -> Entry<OccupiedEntry<'_, Type, T, K, M>, VacantEntry<'_, Type, T, K, M>>;
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) -> Option<Type::Value<T>>;
    fn contains_key(&self, key: &Type::Key<T>) -> bool;
    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>>;
    fn get_mut(&mut self, key: &Type::Key<T>) -> Option<&mut Type::Value<T>>;
    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Type::Key<T>; N]) -> [Option<&mut Type::Value<T>>; N];
    fn remove(&mut self, key: &Type::Key<T>) -> Option<Type::Value<T>>;
    fn remove_entry(&mut self, key: &Type::Key<T>) -> Option<(Type::Key<T>, Type::Value<T>)>;
}
impl<Type: MapType, T, K: ?Sized + Key<Type::Key<T>>, M> Impl<Type, T, K, M> for TypedMap<Type, K, M>
where
    Type::Key<T>: 'static,
    Type::Value<T>: 'static,
    M: MapQuery<Box<K>, Box<dyn Any>, K>,
{
    fn entry(&mut self, key: Type::Key<T>) -> Entry<OccupiedEntry<'_, Type, T, K, M>, VacantEntry<'_, Type, T, K, M>> {
        match self.0.entry(K::new(key)) {
            Entry::Occupied(r) => Entry::Occupied(OccupiedEntry(r, PhantomData)),
            Entry::Vacant(r) => Entry::Vacant(VacantEntry(r, PhantomData)),
        }
    }

    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) -> Option<Type::Value<T>> {
        self.0.insert(K::new(key), Box::new(value) as Box<dyn Any>)
            .map(|boxed| *boxed.downcast().unwrap())
    }

    fn contains_key(&self, key: &Type::Key<T>) -> bool {
        self.0.contains_key(Key::borrow(key))
    }

    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>> {
        self.0.get(Key::borrow(key))
            .map(|boxed| boxed.downcast_ref().unwrap())
    }

    fn get_mut(&mut self, key: &Type::Key<T>) -> Option<&mut Type::Value<T>> {
        self.0.get_mut(Key::borrow(key))
            .map(|boxed| boxed.downcast_mut().unwrap())
    }

    fn get_disjoint_mut<const N: usize>(&mut self, ks: [&Type::Key<T>; N]) -> [Option<&mut Type::Value<T>>; N] {
        self.0.get_disjoint_mut(ks.map(Key::borrow))
            .map(|it|
                it.map(|boxed| boxed.downcast_mut().unwrap()))
    }

    fn remove(&mut self, key: &Type::Key<T>) -> Option<Type::Value<T>> {
        self.0
            .remove(Key::borrow(key))
            .map(|boxed| *boxed.downcast().unwrap())
    }

    fn remove_entry(&mut self, key: &Type::Key<T>) -> Option<(Type::Key<T>, Type::Value<T>)> {
        self.0.remove_entry(Key::borrow(key))
            .map(|(k, v)| (*k.into_any().downcast().unwrap(), *v.downcast().unwrap()))
    }
}
impl<Type: MapType, K: ?Sized + Any, M> TypedMap<Type, K, M>
where
    M: MapQuery<Box<K>, Box<dyn Any>, K>,
{
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &dyn Any)> {
        self.0.iter().map(|(k, v)| (k.as_ref(), v.as_ref()))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut dyn Any)> {
        self.0.iter_mut().map(|(k, v)| (k.as_ref(), v.as_mut()))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys().map(Box::as_ref)
    }

    pub fn values(&self) -> impl Iterator<Item = &dyn Any> {
        self.0.values().map(Box::as_ref)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut dyn Any> {
        self.0.values_mut().map(Box::as_mut)
    }

    pub fn into_keys(self) -> impl Iterator<Item = Box<K>> {
        self.0.into_keys()
    }

    pub fn into_values(self) -> impl Iterator<Item = Box<dyn Any>> {
        self.0.into_values()
    }
}
impl<Type: MapType, K: ?Sized + Any, M> IntoIterator for TypedMap<Type, K, M>
where
    M: MapQuery<Box<K>, Box<dyn Any>, K>,
{
    type Item = (Box<K>, Box<dyn Any>);
    type IntoIter = M::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// hash
#[allow(unused_macros)]
macro_rules! for_hash {
    ($($DefaultHasher: ident)::+, $($HashMap: ident)::+) => {
        impl<Type: MapType> TypedMap<Type, dyn KeyDataHash<$($DefaultHasher)::+>, $($HashMap)::+<Box<dyn KeyDataHash<$($DefaultHasher)::+>>, Box<dyn Any>>> {
            pub fn new() -> Self {
                Self::with_inner($($HashMap)::+::new())
            }
        }
        impl<Type: MapType, S: BuildHasher> TypedMap<Type, dyn KeyDataHash<S::Hasher>, $($HashMap)::+<Box<dyn KeyDataHash<S::Hasher>>, Box<dyn Any>, S>>
        where
            S::Hasher: 'static
        {
            pub fn with_hasher(hash_builder: S) -> Self {
                Self::with_inner($($HashMap)::+::with_hasher(hash_builder))
            }
        }
    };
}
#[cfg(feature = "hashbrown")]
for_hash!(hashbrown::DefaultHasher, hashbrown::HashMap);
#[cfg(not(feature = "no_std"))]
for_hash!(std::hash::DefaultHasher, std::collections::HashMap);
trait DynHash<H: Hasher> {
    fn hash(&self, state: &mut H);
}
impl<T: Hash, H: Hasher> DynHash<H> for T {
    fn hash(&self, state: &mut H) {
        Hash::hash(self, state)
    }
}
trait DynEq {
    fn eq(&self, other: &dyn Any) -> bool;
}
impl<T: Eq + 'static> DynEq for T {
    fn eq(&self, other: &dyn Any) -> bool {
        match other.downcast_ref::<Self>() {
            None => false,
            Some(other) => PartialEq::eq(self, other),
        }
    }
}
#[allow(private_bounds)]
pub trait KeyDataHash<H: Hasher>: Any + DynHash<H> + DynEq {
}
impl<T: ?Sized + Any + DynHash<H> + DynEq, H: Hasher> KeyDataHash<H> for T {
}

impl<H: Hasher + 'static> Hash for dyn KeyDataHash<H> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        self.hash(unsafe { &mut *(state as *mut H1 as *mut H) });
    }
}
impl<H: Hasher + 'static> PartialEq<Self> for dyn KeyDataHash<H> {
    fn eq(&self, other: &Self) -> bool {
        DynEq::eq(self, other)
    }
}
impl<H: Hasher + 'static> Eq for dyn KeyDataHash<H> {
}

impl<H: Hasher + 'static, T: KeyDataHash<H>> Key<T> for dyn KeyDataHash<H> {
    fn new(data: T) -> Box<Self> {
        Box::new(data)
    }

    fn borrow(data: &T) -> &Self {
        data
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ord
trait DynOrd {
    fn cmp(&self, other: &dyn Any) -> Ordering;
}
impl<T: Ord + 'static> DynOrd for T {
    fn cmp(&self, other: &dyn Any) -> Ordering {
        match other.downcast_ref::<Self>() {
            None => Ord::cmp(&TypeId::of::<Self>(), &other.type_id()),
            Some(other) => Ord::cmp(self, other),
        }
    }
}
#[allow(private_bounds)]
pub trait KeyDataOrd: Any + DynOrd {
}
impl PartialEq<Self> for dyn KeyDataOrd {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for dyn KeyDataOrd {}
impl PartialOrd<Self> for dyn KeyDataOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn KeyDataOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        DynOrd::cmp(self, other)
    }
}
impl<T: ?Sized + Any + DynOrd> KeyDataOrd for T {
}

impl<T: KeyDataOrd> Key<T> for dyn KeyDataOrd {
    fn new(data: T) -> Box<Self> {
        Box::new(data)
    }

    fn borrow(data: &T) -> &Self {
        data
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Type: MapType> TypedMap<Type, dyn KeyDataOrd, BTreeMap<Box<dyn KeyDataOrd>, Box<dyn Any>>>
{
    pub fn new() -> Self {
        Self::with_inner(BTreeMap::new())
    }
}

// fuck RustRover
#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use std::println;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;
    use crate::*;
    use core::cmp::Ordering;
    use core::hash::{Hash, Hasher};
    use core::marker::PhantomData;

    #[cfg(any(feature = "hashbrown", not(feature = "no_std")))]
    #[test]
    fn test_hash() {
        pub struct MyTypeId<T>(PhantomData<T>);
        impl<T> Hash for MyTypeId<T> {
            fn hash<H: Hasher>(&self, _state: &mut H) {
            }
        }
        impl<T> PartialEq<Self> for MyTypeId<T> {
            fn eq(&self, _other: &Self) -> bool {
                true
            }
        }
        impl<T> Eq for MyTypeId<T> {
        }
        impl<T> PartialOrd<Self> for MyTypeId<T> {
            fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
                Some(Ordering::Equal)
            }
        }
        impl<T> Ord for MyTypeId<T> {
            fn cmp(&self, _other: &Self) -> Ordering {
                Ordering::Equal
            }
        }
        pub struct MyMap;
        impl MapType for MyMap {
            type Key<T> = MyTypeId<T>;
            type Value<T> = Vec<T>;
        }
        let mut map = TypedMap::<MyMap>::new();
        map.insert(MyTypeId::<i32>(PhantomData), vec![10, 20, 30]);

        map.insert(MyTypeId::<String>(PhantomData), vec!["Hello".to_string(), "World".to_string()]);
        let option;
        {
            let id = MyTypeId::<i32>(PhantomData);
            option = map.get(&id);
        }
        if let Some(val) = option {
            println!("Got i32 vec: {:?}", val);
        } else {
            panic!();
        }
        if let Some(val) = map.get(&MyTypeId::<String>(PhantomData)) {
            println!("Got String vec: {:?}", val);
        } else {
            panic!();
        }

        for x in map.into_keys() {
            println!("{:?}", x.as_ref() as *const _);
        }
    }

    #[test]
    fn test_btree() {
        pub struct MyTypeId<T>(PhantomData<T>);
        impl<T> Hash for MyTypeId<T> {
            fn hash<H: Hasher>(&self, _state: &mut H) {
            }
        }
        impl<T> PartialEq<Self> for MyTypeId<T> {
            fn eq(&self, _other: &Self) -> bool {
                true
            }
        }
        impl<T> Eq for MyTypeId<T> {
        }
        impl<T> PartialOrd<Self> for MyTypeId<T> {
            fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
                Some(Ordering::Equal)
            }
        }
        impl<T> Ord for MyTypeId<T> {
            fn cmp(&self, _other: &Self) -> Ordering {
                Ordering::Equal
            }
        }
        pub struct MyMap;
        impl MapType for MyMap {
            type Key<T> = MyTypeId<T>;
            type Value<T> = Vec<T>;
        }
        let mut map = TypedMap::<MyMap, dyn KeyDataOrd, BTreeMap<Box<dyn KeyDataOrd>, Box<dyn Any>>>::new();
        map.insert(MyTypeId::<i32>(PhantomData), vec![10, 20, 30]);

        map.insert(MyTypeId::<String>(PhantomData), vec!["Hello".to_string(), "World".to_string()]);
        let option;
        {
            let id = MyTypeId::<i32>(PhantomData);
            option = map.get(&id);
        }
        if let Some(val) = option {
            println!("Got i32 vec: {:?}", val);
        } else {
            panic!();
        }
        if let Some(val) = map.get(&MyTypeId::<String>(PhantomData)) {
            println!("Got String vec: {:?}", val);
        } else {
            panic!();
        }
    }
}
