use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::ffi::c_void;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::{PhantomData};


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


pub trait MapType {
    type Key<T>;
    type Value<T>;
}

pub trait TypedMapSpecialized<Type: MapType, T> {
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>);
    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>>;
}


enum Key<'a, T> {
    Borrow(&'a T),
    Hash(Box<dyn Any>, fn(&dyn Any, *mut c_void), fn(&dyn Any, &dyn Any) -> bool),
    Ord(Box<dyn Any>, fn(&dyn Any, &dyn Any) -> Ordering),
}

type Never = Infallible;

pub struct KeyStore(Key<'static, Never>);
impl<'a, T> Borrow<Key<'a, T>> for KeyStore {
    fn borrow(&self) -> &Key<'a, T> {
        unsafe {
            std::mem::transmute::<&Key<'static, Never>, &Key<'a, T>>(&self.0)
        }
    }
}

impl<T: Hash> Hash for Key<'_, T> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        use Key::*;
        match self {
            Hash(b, f, _) => f(&**b, state as *mut H1 as *mut c_void),
            Borrow(b) => b.hash(state),
            Ord(..) => panic!()
        }
    }
}

impl<T: Eq + 'static> PartialEq<Self> for Key<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        use Key::*;
        match self {
            Hash(b, _, f) => match other {
                Hash(b1, ..) => f(&**b, &**b1),
                Ord(..) => panic!(),
                Borrow(_) => other.eq(self)
            },
            Ord(..) => panic!(),
            Borrow(b) => match other {
                Hash(b1, ..) => match b1.downcast_ref::<T>() {
                    Some(b1) => (*b).eq(b1),
                    None => false,
                },
                Ord(..) => panic!(),
                Borrow(b1) => (*b).eq(b1)
            }
        }
    }
}
impl<T: Eq + 'static> Eq for Key<'_, T> {
}

impl<T: Ord + 'static> PartialOrd<Self> for Key<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord + 'static> Ord for Key<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Key::*;
        match self {
            Hash(..) => panic!(),
            Ord(b, f) => match other {
                Hash(..) => panic!(),
                Ord(b1, ..) => f(&**b, &**b1),
                Borrow(_) => other.cmp(self).reverse()
            },
            Borrow(b) => match other {
                Hash(..) => panic!(),
                Ord(b1, ..) => match b1.downcast_ref::<T>() {
                    Some(b1) => (*b).cmp(b1),
                    None => TypeId::of::<T>().cmp(&b1.type_id()),
                },
                Borrow(b1) => (*b).cmp(b1)
            }
        }
    }
}

impl Hash for KeyStore {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        self.0.hash(state);
    }
}

impl PartialEq<Self> for KeyStore {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl Eq for KeyStore {
}
impl PartialOrd<Self> for KeyStore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for KeyStore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

pub struct TypedMap<Type: MapType, M = HashMap<KeyStore, Box<dyn Any>>> {
    inner: M,
    _marker: PhantomData<Type>,
}

impl<Type: MapType> TypedMap<Type> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            _marker: PhantomData,
        }
    }
}
impl<Type: MapType> TypedMap<Type, BTreeMap<KeyStore, Box<dyn Any>>> {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            _marker: PhantomData,
        }
    }
}

pub trait Impl<Type: MapType, T> {
    fn new_key(obj: Type::Key<T>) -> KeyStore;
}

impl<Type: MapType, T, S: BuildHasher> Impl<Type, T> for TypedMap<Type, HashMap<KeyStore, Box<dyn Any>, S>>
where
    Type::Key<T>: Hash + Eq + 'static,
    Type::Value<T>: 'static,
{
    fn new_key(obj: Type::Key<T>) -> KeyStore {
        KeyStore(Key::Hash(Box::new(obj), |s, h| {
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
impl<Type: MapType, T> Impl<Type, T> for TypedMap<Type, BTreeMap<KeyStore, Box<dyn Any>>>
where
    Type::Key<T>: Ord + 'static,
    Type::Value<T>: 'static,
{
    fn new_key(obj: Type::Key<T>) -> KeyStore {
        KeyStore(Key::Ord(Box::new(obj), |s, o| {
            assert!(s.is::<Type::Key<T>>());
            match o.downcast_ref::<Type::Key<T>>() {
                None => TypeId::of::<Type::Key<T>>().cmp(&o.type_id()),
                Some(b) => { s.downcast_ref::<Type::Key<T>>().unwrap().cmp(b) }
            }
        }))
    }
}

impl<Type: MapType, T, M> TypedMapSpecialized<Type, T> for TypedMap<Type, M>
where
    Self: Impl<Type, T>,
    Type::Value<T>: 'static,
    M: for<'a> Map<KeyStore, Box<dyn Any>, Key<'a, Type::Key<T>>>,
{
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) {
        self.inner.insert(Self::new_key(key), Box::new(value) as Box<dyn Any>);
    }

    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>> {
        self.inner
            .get(&Key::Borrow(key))
            .and_then(|boxed| boxed.downcast_ref())
    }
}

fn main() {
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
    // 2. 定义具体的构造器 (对应你原来的 KeyConstructor 和 ValueConstructor)
    pub struct MyMap;
    impl MapType for MyMap {
        type Key<T> = MyTypeId<T>;
        type Value<T> = Vec<T>;
    }
    let mut map = TypedMap::<MyMap, BTreeMap<_, _>>::new();
    // 场景 1: T = i32
    map.insert(MyTypeId::<i32>(PhantomData), vec![10, 20, 30]);

    // 场景 2: T = String
    map.insert(MyTypeId::<String>(PhantomData), vec!["Hello".to_string(), "World".to_string()]);
    let option;
    {
        let id = MyTypeId::<i32>(PhantomData);
        // 取回 i32
        option = map.get(&id);
    }
    if let Some(val) = option {
        println!("Got i32 vec: {:?}", val); // 输出: Got i32 vec: [10, 20, 30]
    }

    // 取回 String
    if let Some(val) = map.get(&MyTypeId::<String>(PhantomData)) {
        println!("Got String vec: {:?}", val); // 输出: Got String vec: ["Hello", "World"]
    }
}
