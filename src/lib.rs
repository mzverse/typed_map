use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;

// // util
// fn is_dyn_any<T: ?Sized + Any>(any: &T) -> bool {
//     any.type_id() != TypeId::of::<T>()
// }

// map
pub trait Map<K, V, Q: ?Sized>
{
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &Q) -> Option<&V>;
}
impl<K, V, Q: ?Sized, S> Map<K, V, Q> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
    K: Borrow<Q>,
    Q: Eq + Hash,
{
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        HashMap::insert(self, key, value)
    }

    fn get(&self, key: &Q) -> Option<&V> {
         HashMap::get(self, key)
    }
}
impl<K, V, Q: ?Sized> Map<K, V, Q> for BTreeMap<K, V>
where
    K: Ord,
    K: Borrow<Q>,
    Q: Ord,
{
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }

    fn get(&self, key: &Q) -> Option<&V> {
        BTreeMap::get(self, key)
    }
}


// key
pub trait MapType {
    type Key<T>;
    type Value<T>;
}

pub trait TypedMapSpecialized<Type: MapType, T, K: Any + ?Sized>
where
    Self: Impl<Type, K, T>,
    Type::Key<T>: BorrowKey<K>,
{
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) -> Option<Type::Value<T>>;
    fn get<Q>(&self, key: &Q) -> Option<&Type::Value<T>>
    where
        Type::Key<T>: Borrow<Q>,
        Q: BorrowKey<K>;
}

pub trait BorrowKey<K: Any + ?Sized> {
    fn borrow(&self) -> &K;
}

// impl
pub struct TypedMap<Type: MapType, K: Any + ?Sized = dyn KeyDataHash<DefaultHasher>, M = HashMap<Box<K>, Box<dyn Any>>> {
    inner: M,
    _marker: PhantomData<(Type, K)>,
}

pub trait Impl<Type: MapType, K: Any + ?Sized, T> {
    fn new_key(obj: Type::Key<T>) -> Box<K>;
}

impl<Type: MapType, T, K: Any + ?Sized, M> TypedMapSpecialized<Type, T, K> for TypedMap<Type, K, M>
where
    Self: Impl<Type, K, T>,
    Type::Value<T>: 'static,
    M: Map<Box<K>, Box<dyn Any>, K>,
    Type::Key<T>: BorrowKey<K>,
{
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) -> Option<Type::Value<T>> {
        self.inner.insert(Self::new_key(key), Box::new(value) as Box<dyn Any>)
            .and_then(|boxed| *boxed.downcast().unwrap())
    }

    fn get<Q>(&self, key: &Q) -> Option<&Type::Value<T>>
    where
        Type::Key<T>: Borrow<Q>,
        Q: BorrowKey<K>,
    {
        self.inner
            .get(BorrowKey::borrow(key))
            .and_then(|boxed| boxed.downcast_ref())
    }
}

// hash
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

impl<H: Hasher + 'static, T: KeyDataHash<H>> BorrowKey<dyn KeyDataHash<H>> for T {
    fn borrow(&self) -> &dyn KeyDataHash<H> {
        self
    }
}

impl<Type: MapType, S: BuildHasher, T> Impl<Type, dyn KeyDataHash<S::Hasher>, T> for TypedMap<Type, dyn KeyDataHash<S::Hasher>, HashMap<Box<dyn KeyDataHash<S::Hasher>>, Box<dyn Any>, S>>
where
    S::Hasher: 'static,
    Type::Key<T>: KeyDataHash<S::Hasher> + 'static,
{
    fn new_key(obj: Type::Key<T>) -> Box<dyn KeyDataHash<S::Hasher>> {
        Box::new(obj)
    }
}
impl<Type: MapType> TypedMap<Type> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            _marker: PhantomData,
        }
    }
}
impl<Type: MapType, S: BuildHasher> TypedMap<Type, dyn KeyDataHash<S::Hasher>, HashMap<Box<dyn KeyDataHash<S::Hasher>>, Box<dyn Any>, S>>
where
    S::Hasher: 'static
{
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            inner: HashMap::with_hasher(hash_builder),
            _marker: PhantomData,
        }
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
impl<T: KeyDataOrd> BorrowKey<dyn KeyDataOrd> for T {
    fn borrow(&self) -> &dyn KeyDataOrd {
        self
    }
}

impl<Type: MapType, T> Impl<Type, dyn KeyDataOrd, T> for TypedMap<Type, dyn KeyDataOrd, BTreeMap<Box<dyn KeyDataOrd>, Box<dyn Any>>>
where
    Type::Key<T>: KeyDataOrd + 'static,
{
    fn new_key(obj: Type::Key<T>) -> Box<dyn KeyDataOrd> {
        Box::new(obj)
    }
}
impl<Type: MapType> TypedMap<Type, dyn KeyDataOrd, BTreeMap<Box<dyn KeyDataOrd>, Box<dyn Any>>>
{
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::cmp::Ordering;
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;

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
