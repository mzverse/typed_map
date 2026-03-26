use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::hash::{BuildHasher, DefaultHasher, Hash, Hasher};
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
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>);
    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>>;
}

type Never = Infallible;

pub trait KeyData {
    fn as_ref(&self) -> &dyn Any;
}

enum Key<'a, T, D: KeyData> {
    Own(D),
    Borrow(&'a T),
}

pub struct KeyStore<D: KeyData>(Key<'static, Never, D>);
impl<'a, T, D: KeyData> Borrow<Key<'a, T, D>> for KeyStore<D> {
    fn borrow(&self) -> &Key<'a, T, D> {
        assert!(matches!(self.0, Key::Own(..)));
        unsafe {
            std::mem::transmute::<&Key<'static, Never, D>, &Key<'a, T, D>>(&self.0)
        }
    }
}

// impl
pub struct TypedMap<Type: MapType, K = KeyDataHash<DefaultHasher>, M = HashMap<KeyStore<KeyDataHash<DefaultHasher>>, Box<dyn Any>>> {
    inner: M,
    _marker: PhantomData<Type>,
    _marker1: PhantomData<K>,
}

pub trait Impl<Type: MapType, K: KeyData, T> {
    fn new_key(obj: Type::Key<T>) -> KeyStore<K>;
}

impl<Type: MapType, T, K: KeyData, M> TypedMapSpecialized<Type, T> for TypedMap<Type, K, M>
where
    Self: Impl<Type, K, T>,
    Type::Value<T>: 'static,
    M: for<'a> Map<KeyStore<K>, Box<dyn Any>, Key<'a, Type::Key<T>, K>>,
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

// hash
pub struct KeyDataHash<H: Hasher>(Box<dyn Any>, fn(&dyn Any, &mut H), fn(&dyn Any, &dyn Any) -> bool);
impl<H: Hasher> KeyData for KeyDataHash<H> {
    fn as_ref(&self) -> &dyn Any {
        self.0.as_ref()
    }
}
impl<H: Hasher> Hash for KeyDataHash<H> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        self.1(self.as_ref(), unsafe {
            std::mem::transmute::<&mut H1, &mut H>(state) // assert
        });
    }
}
impl<T: Hash, H: Hasher> Hash for Key<'_, T, KeyDataHash<H>> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        use Key::*;
        match self {
            Own(own) => own.hash(state),
            Borrow(borrow) => borrow.hash(state),
        }
    }
}
impl<H: Hasher> Hash for KeyStore<KeyDataHash<H>> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        self.0.hash(state);
    }
}

impl<H: Hasher> PartialEq<Self> for KeyDataHash<H> {
    fn eq(&self, other: &Self) -> bool {
        self.2(self.as_ref(), other.as_ref())
    }
}
impl<H: Hasher> Eq for KeyDataHash<H> {
}
impl<T: Eq + 'static, H: Hasher> PartialEq<Self> for Key<'_, T, KeyDataHash<H>> {
    fn eq(&self, other: &Self) -> bool {
        use Key::*;
        match self {
            Own(s) => match other {
                Own(o) => s.eq(o),
                Borrow(..) => other.eq(self),
            },
            Borrow(s) => match other {
                Own(o) => match o.as_ref().downcast_ref::<T>() {
                    None => false,
                    Some(o) => s.eq(&o),
                },
                Borrow(o) => s.eq(o)
            }
        }
    }
}
impl<T: Eq + 'static, H: Hasher> Eq for Key<'_, T, KeyDataHash<H>> {
}
impl<H: Hasher> PartialEq<Self> for KeyStore<KeyDataHash<H>> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<H: Hasher> Eq for KeyStore<KeyDataHash<H>> {
}
impl<Type: MapType, T, S: BuildHasher> Impl<Type, KeyDataHash<S::Hasher>, T> for TypedMap<Type, KeyDataHash<S::Hasher>, HashMap<KeyStore<KeyDataHash<S::Hasher>>, Box<dyn Any>, S>>
where
    Type::Key<T>: Hash + Eq + 'static,
    Type::Value<T>: 'static,
{
    fn new_key(obj: Type::Key<T>) -> KeyStore<KeyDataHash<S::Hasher>> {
        KeyStore(Key::Own(KeyDataHash(Box::new(obj), |s, h| {
            s.downcast_ref::<Type::Key<T>>().unwrap().hash(unsafe {
                &mut *(h as *mut S::Hasher)
            })
        }, |a, b| {
            match b.downcast_ref::<Type::Key<T>>() {
                None => { false }
                Some(b) => { a.downcast_ref::<Type::Key<T>>().unwrap().eq(b) }
            }
        })))
    }
}

// ord
pub struct KeyDataOrd(Box<dyn Any>, fn(&dyn Any, &dyn Any) -> Ordering);
impl KeyData for KeyDataOrd {
    fn as_ref(&self) -> &dyn Any {
        self.0.as_ref()
    }
}
impl PartialEq<Self> for KeyDataOrd {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for KeyDataOrd {}
impl PartialOrd<Self> for KeyDataOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for KeyDataOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1(self.as_ref(), other.as_ref())
    }
}

impl<T: Ord + 'static> PartialEq<Self> for Key<'_, T, KeyDataOrd> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<T: Ord + 'static> Eq for Key<'_, T, KeyDataOrd> {
}
impl<T: Ord + 'static> PartialOrd<Self> for Key<'_, T, KeyDataOrd> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord + 'static> Ord for Key<'_, T, KeyDataOrd> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Key::*;
        match self {
            Own(s) => match other {
                Own(o) => s.cmp(o),
                Borrow(..) => other.cmp(self).reverse(),
            },
            Borrow(s) => match other {
                Own(o) => match o.as_ref().downcast_ref::<T>() {
                    None => TypeId::of::<T>().cmp(&o.as_ref().type_id()),
                    Some(o) => (*s).cmp(o),
                },
                Borrow(o) => s.cmp(o),
            },
        }
    }
}

impl PartialEq<Self> for KeyStore<KeyDataOrd> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl Eq for KeyStore<KeyDataOrd> {
}
impl PartialOrd<Self> for KeyStore<KeyDataOrd> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for KeyStore<KeyDataOrd> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<Type: MapType> TypedMap<Type> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            _marker: PhantomData,
            _marker1: PhantomData,
        }
    }
}
impl<Type: MapType, S: BuildHasher> TypedMap<Type, KeyDataHash<S::Hasher>, HashMap<KeyStore<KeyDataHash<S::Hasher>>, Box<dyn Any>, S>> {
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            inner: HashMap::with_hasher(hash_builder),
            _marker: PhantomData,
            _marker1: PhantomData,
        }
    }
}
impl<Type: MapType> TypedMap<Type, KeyDataOrd, BTreeMap<KeyStore<KeyDataOrd>, Box<dyn Any>>> {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            _marker: PhantomData,
            _marker1: PhantomData,
        }
    }
}
impl<Type: MapType, T> Impl<Type, KeyDataOrd, T> for TypedMap<Type, KeyDataOrd, BTreeMap<KeyStore<KeyDataOrd>, Box<dyn Any>>>
where
    Type::Key<T>: Ord + 'static,
    Type::Value<T>: 'static,
{
    fn new_key(obj: Type::Key<T>) -> KeyStore<KeyDataOrd> {
        KeyStore(Key::Own(KeyDataOrd(Box::new(obj), |s, o| {
            assert!(s.is::<Type::Key<T>>());
            match o.downcast_ref::<Type::Key<T>>() {
                None => TypeId::of::<Type::Key<T>>().cmp(&o.type_id()),
                Some(b) => { s.downcast_ref::<Type::Key<T>>().unwrap().cmp(b) }
            }
        })))
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
    let mut map = TypedMap::<MyMap, KeyDataOrd, BTreeMap<KeyStore<KeyDataOrd>, Box<dyn Any>>>::new();
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
