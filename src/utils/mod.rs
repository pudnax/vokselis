use std::{
    collections::HashMap,
    hash::Hash,
    io::{self, Write},
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    path::Path,
};

pub mod frame_counter;
pub mod input;
pub mod recorder;
pub mod shader_compiler;

pub fn create_folder<P: AsRef<Path>>(name: P) -> io::Result<()> {
    match std::fs::create_dir(name) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }

    Ok(())
}

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

pub trait NonZeroSized: Sized {
    const SIZE: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(std::mem::size_of::<Self>() as _) };
}
/// Holds invariants? Nah!
impl<T> NonZeroSized for T where T: Sized {}

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

#[derive(Debug, Clone, Copy)]
pub struct ImageDimentions {
    pub width: u32,
    pub height: u32,
    pub unpadded_bytes_per_row: u32,
    pub padded_bytes_per_row: u32,
}

impl ImageDimentions {
    pub fn new(width: u32, height: u32, align: u32) -> Self {
        let height = height.saturating_sub(height % 2);
        let width = width.saturating_sub(width % 2);
        let bytes_per_pixel = std::mem::size_of::<[u8; 4]>() as u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }

    pub fn linear_size(&self) -> u64 {
        self.padded_bytes_per_row as u64 * self.height as u64
    }
}
