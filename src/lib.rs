use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::hash::{BuildHasher, Hash, Hasher, RandomState};
use std::marker::PhantomData;

// map
pub trait Map<K, V, Q>
{
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &Q) -> Option<&V>;
}
impl<K, V, Q, S> Map<K, V, Q> for HashMap<K, V, S>
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
impl<K, V, Q> Map<K, V, Q> for BTreeMap<K, V>
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

pub trait TypedMapSpecialized<Type: MapType, T> {
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) -> Option<Type::Value<T>>;
    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>>;
}

type Never = Infallible;

pub trait KeyData {
    type Map;
    fn as_ref(&self) -> &dyn Any;
}

pub trait KeyFlag {
}
pub struct KeyFlagOwn;
struct KeyFlagBorrow;
impl KeyFlag for KeyFlagOwn {
}
impl KeyFlag for KeyFlagBorrow {
}

#[repr(C)]
pub enum Key<'a, T, D: KeyData, F: KeyFlag> {
    Own(D),
    Borrow(&'a T, F),
}

impl<'a, T, D: KeyData> Borrow<Key<'a, T, D, KeyFlagBorrow>> for Key<'static, Never, D, KeyFlagOwn> {
    fn borrow(&self) -> &Key<'a, T, D, KeyFlagBorrow> {
        assert!(matches!(self, Key::Own(..)));
        unsafe {
            std::mem::transmute::<&Key<'static, Never, D, KeyFlagOwn>, &Key<'a, T, D, KeyFlagBorrow>>(&self)
        }
    }
}

// impl
pub struct TypedMap<Type: MapType, K: KeyData = KeyDataHash<RandomState>> {
    inner: K::Map,
    _marker: PhantomData<(Type, K)>,
}

pub trait Impl<Type: MapType, K: KeyData, T> {
    fn new_key(obj: Type::Key<T>) -> Key<'static, Never, K, KeyFlagOwn>;
}

impl<Type: MapType, T, K: KeyData> TypedMapSpecialized<Type, T> for TypedMap<Type, K>
where
    Self: Impl<Type, K, T>,
    Type::Value<T>: 'static,
    K::Map: for<'a> Map<Key<'static, Never, K, KeyFlagOwn>, Box<dyn Any>, Key<'a, Type::Key<T>, K, KeyFlagBorrow>>,
{
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) -> Option<Type::Value<T>> {
        self.inner.insert(Self::new_key(key), Box::new(value) as Box<dyn Any>)
            .and_then(|boxed| *boxed.downcast().unwrap())
    }

    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>> {
        self.inner
            .get(&Key::Borrow(key, KeyFlagBorrow))
            .and_then(|boxed| boxed.downcast_ref())
    }
}

// hash
pub struct KeyDataHash<S: BuildHasher>(Box<dyn Any>, fn(&dyn Any, &mut S::Hasher), fn(&dyn Any, &dyn Any) -> bool);
impl<S: BuildHasher> KeyData for KeyDataHash<S> {
    type Map = HashMap<Key<'static, Never, Self, KeyFlagOwn>, Box<dyn Any>, S>;
    fn as_ref(&self) -> &dyn Any {
        self.0.as_ref()
    }
}
impl<T: Hash, S: BuildHasher, F: KeyFlag> Hash for Key<'_, T, KeyDataHash<S>, F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Key::*;
        match self {
            Own(own) => {
                own.1(own.as_ref(), unsafe {
                    std::mem::transmute::<&mut H, &mut S::Hasher>(state) // assert
                });
            }
            Borrow(borrow, ..) => borrow.hash(state),
        }
    }
}
impl<T: Eq + 'static, S: BuildHasher, F: KeyFlag> PartialEq<Self> for Key<'_, T, KeyDataHash<S>, F> {
    fn eq(&self, other: &Self) -> bool {
        use Key::*;
        match self {
            Own(s) => match other {
                Own(o) => s.2(s.as_ref(), o.as_ref()),
                Borrow(..) => other.eq(self),
            },
            Borrow(s, ..) => match other {
                Own(o) => match o.as_ref().downcast_ref::<T>() {
                    None => false,
                    Some(o) => s.eq(&o),
                },
                Borrow(o, ..) => s.eq(o)
            }
        }
    }
}
impl<T: Eq + 'static, S: BuildHasher, F: KeyFlag> Eq for Key<'_, T, KeyDataHash<S>, F> {
}
impl<Type: MapType, T, S: BuildHasher> Impl<Type, KeyDataHash<S>, T> for TypedMap<Type, KeyDataHash<S>>
where
    Type::Key<T>: Hash + Eq + 'static,
    Type::Value<T>: 'static,
{
    fn new_key(obj: Type::Key<T>) -> Key<'static, Never, KeyDataHash<S>, KeyFlagOwn> {
        Key::Own(KeyDataHash(Box::new(obj), |s, h| {
            s.downcast_ref::<Type::Key<T>>().unwrap().hash(unsafe {
                &mut *(h as *mut S::Hasher)
            })
        }, |a, b| {
            match b.downcast_ref::<Type::Key<T>>() {
                None => { false }
                Some(b) => { a.downcast_ref::<Type::Key<T>>().unwrap().eq(b) }
            }
        }))
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
impl<Type: MapType, S: BuildHasher> TypedMap<Type, KeyDataHash<S>> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            inner: HashMap::with_hasher(hash_builder),
            _marker: PhantomData,
        }
    }
}

// b tree
pub struct KeyDataBTree(Box<dyn Any>, fn(&dyn Any, &dyn Any) -> Ordering);
impl KeyData for KeyDataBTree {
    type Map = BTreeMap<Key<'static, Never, Self, KeyFlagOwn>, Box<dyn Any>>;
    fn as_ref(&self) -> &dyn Any {
        self.0.as_ref()
    }
}

impl<T: Ord + 'static, F: KeyFlag> PartialEq<Self> for Key<'_, T, KeyDataBTree, F> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<T: Ord + 'static, F: KeyFlag> Eq for Key<'_, T, KeyDataBTree, F> {
}
impl<T: Ord + 'static, F: KeyFlag> PartialOrd<Self> for Key<'_, T, KeyDataBTree, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord + 'static, F: KeyFlag> Ord for Key<'_, T, KeyDataBTree, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Key::*;
        match self {
            Own(s) => match other {
                Own(o) => s.1(s.as_ref(), o.as_ref()),
                Borrow(..) => other.cmp(self).reverse(),
            },
            Borrow(s, ..) => match other {
                Own(o) => match o.as_ref().downcast_ref::<T>() {
                    None => TypeId::of::<T>().cmp(&o.as_ref().type_id()),
                    Some(o) => (*s).cmp(o),
                },
                Borrow(o, ..) => s.cmp(o),
            },
        }
    }
}

impl<Type: MapType, T> Impl<Type, KeyDataBTree, T> for TypedMap<Type, KeyDataBTree>
where
    Type::Key<T>: Ord + 'static,
    Type::Value<T>: 'static,
{
    fn new_key(obj: Type::Key<T>) -> Key<'static, Never, KeyDataBTree, KeyFlagOwn> {
        Key::Own(KeyDataBTree(Box::new(obj), |s, o| {
            assert!(s.is::<Type::Key<T>>());
            match o.downcast_ref::<Type::Key<T>>() {
                None => TypeId::of::<Type::Key<T>>().cmp(&o.type_id()),
                Some(b) => { s.downcast_ref::<Type::Key<T>>().unwrap().cmp(b) }
            }
        }))
    }
}
impl<Type: MapType> TypedMap<Type, KeyDataBTree> {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;
    use crate::*;

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
        let mut map = TypedMap::<MyMap, KeyDataBTree>::new();
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
