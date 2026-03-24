use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::{BuildHasher, Hash, Hasher, RandomState};
use std::marker::PhantomData;

pub trait MapType {
    type Key<T>;
    type Value<T>;
}

pub trait TypedMapSpecialized<Type: MapType, T> {
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>);
    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>>;
}


enum Key<'a, T, H: Hasher> {
    KeyBox(Box<dyn Any>, fn(&dyn Any, &mut H), fn(&dyn Any, &dyn Any) -> bool),
    KeyBorrow(&'a T),
}

type Never = Infallible;

struct KeyStore<H: Hasher>(Key<'static, Never, H>);
impl<'a, T, H: Hasher> Borrow<Key<'a, T, H>> for KeyStore<H> {
    fn borrow(&self) -> &Key<'a, T, H> {
        unsafe {
            std::mem::transmute::<&Key<'static, Never, H>, &Key<'a, T, H>>(&self.0)
        }
    }
}

impl<T: Hash, H: Hasher> Hash for Key<'_, T, H> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        // assert_eq!(TypeId::of::<H1>(), TypeId::of::<H>());
        use Key::*;
        match self {
            KeyBox(b, f, _) => f(&**b, unsafe {
                std::mem::transmute::<&mut H1, &mut H>(state)
            }),
            KeyBorrow(b) => b.hash(state)
        }
    }
}

impl<T: Eq + 'static, H: Hasher> PartialEq<Self> for Key<'_, T, H> {
    fn eq(&self, other: &Self) -> bool {
        use Key::*;
        match self {
            KeyBox(b, _, f) => match other {
                KeyBox(b1, _, _) => f(b, b1),
                KeyBorrow(b1) => match b.downcast_ref::<T>() {
                    Some(b) => (*b1).eq(b),
                    None => false,
                }
            },
            KeyBorrow(b) => match other {
                KeyBox(b1, _, _) => match b1.downcast_ref::<T>() {
                    Some(b1) => (*b).eq(b1),
                    None => false,
                },
                KeyBorrow(b1) => (*b).eq(b1)
            }
        }
    }
}
impl<T: Eq + 'static, H: Hasher> Eq for Key<'_, T, H> {
}

impl<H: Hasher> Hash for KeyStore<H> {
    fn hash<H1: Hasher>(&self, state: &mut H1) {
        self.0.hash(state);
    }
}

impl<H: Hasher> PartialEq<Self> for KeyStore<H> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<H: Hasher> Eq for KeyStore<H> {
}

fn new_key<T: Hash + Eq + 'static, H: Hasher>(obj: T) -> Key<'static, Never, H> {
    Key::KeyBox(Box::new(obj), |s, h| {
        s.downcast_ref::<T>().unwrap().hash(h)
    }, |a, b| {
        match b.downcast_ref::<T>() {
            None => { false }
            Some(b) => { a.downcast_ref::<T>().unwrap().eq(b) }
        }
    })
}

pub struct TypedMap<Type: MapType, S: BuildHasher = RandomState> {
    inner: HashMap<KeyStore<S::Hasher>, Box<dyn Any>, S>,
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

impl<Type: MapType, T> TypedMapSpecialized<Type, T> for TypedMap<Type>
where
    Type::Key<T>: Hash + Eq + 'static,
    Type::Value<T>: 'static,
{
    fn insert(&mut self, key: Type::Key<T>, value: Type::Value<T>) {
        self.inner.insert(KeyStore(new_key(key)), Box::new(value));
    }

    fn get(&self, key: &Type::Key<T>) -> Option<&Type::Value<T>> {
        self.inner
            .get(&Key::KeyBorrow(key))
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
    // 2. 定义具体的构造器 (对应你原来的 KeyConstructor 和 ValueConstructor)
    pub struct MyMap;
    impl MapType for MyMap {
        type Key<T> = MyTypeId<T>;
        type Value<T> = Vec<T>;
    }
    let mut map = TypedMap::<MyMap>::new();
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
