use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    io::Write,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub fn green_blink() {
    const ESC: &str = "\x1B[";
    const RESET: &str = "\x1B[0m";
    eprint!("\r{}42m{}K{}\r", ESC, ESC, RESET);
    std::io::stdout().flush().unwrap();
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(50));
        eprint!("\r{}40m{}K{}\r", ESC, ESC, RESET);
        std::io::stdout().flush().unwrap();
    });
}

pub(crate) trait RcWrap: Sized {
    fn wrap(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}
impl<T> RcWrap for T where T: Sized {}

/// A hash map with a [HashSet](std::collections::HashSet) to hold unique values
#[derive(Debug)]
pub struct ContiniousHashMap<K, V>(HashMap<K, Vec<V>>);

impl<K, V> Deref for ContiniousHashMap<K, V> {
    type Target = HashMap<K, Vec<V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for ContiniousHashMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V> ContiniousHashMap<K, V> {
    /// Creates an empty [ContiniousHashMap]
    ///
    /// The hash map is initially created with a capacity of 0,
    /// so it will not allocate until it is first inserted into.
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl<K: Eq + Hash, V> ContiniousHashMap<K, V> {
    /// Inserts a key-value pair into the map.
    ///
    /// If the mep already contain this key this method will add
    /// a value instead of rewriting an old value.
    pub fn push_value(&mut self, key: K, value: V) {
        self.0.entry(key).or_insert_with(Vec::new).push(value);
    }
}
