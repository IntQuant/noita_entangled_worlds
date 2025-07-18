mod component;
mod component_data;
mod entity;
mod objects;
mod tags;
mod world;
pub use component::*;
pub use entity::*;
pub use objects::*;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter};
use std::slice;
pub use tags::*;
pub use world::*;
#[repr(C)]
union Buffer {
    buffer: *const u8,
    sso_buffer: [u8; 16],
}
impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            sso_buffer: [0; 16],
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StdString {
    buffer: Buffer,
    size: usize,
    capacity: usize,
}
impl<'a> StdString {
    pub fn as_str(&'a self) -> &'a str {
        let slice: &[u8] = unsafe {
            if self.capacity <= 16 {
                &self.buffer.sso_buffer
            } else {
                slice::from_raw_parts(self.buffer.buffer, self.size)
            }
        };
        let actual_len = slice.iter().position(|&b| b == 0).unwrap_or(self.size);
        str::from_utf8(&slice[..actual_len]).unwrap()
    }
}
impl From<&str> for StdString {
    fn from(value: &str) -> Self {
        let mut res = StdString {
            capacity: value.len(),
            size: value.len(),
            ..Default::default()
        };
        if res.capacity > 16 {
            let buffer = Box::leak(Box::new(value));
            res.buffer.buffer = buffer.as_ptr();
        } else {
            let mut iter = value.as_bytes().iter();
            res.buffer.sso_buffer = std::array::from_fn(|_| iter.next().copied().unwrap_or(0))
        }
        res
    }
}
impl Display for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Debug for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdString").field(&self.as_str()).finish()
    }
}
impl PartialEq for StdString {
    fn eq(&self, other: &Self) -> bool {
        if self.size == other.size {
            self.as_str() == other.as_str()
        } else {
            false
        }
    }
}
impl PartialOrd for StdString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for StdString {}
impl Ord for StdString {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.size.cmp(&other.size) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => {
                if self == other {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            }
            Ordering::Greater => Ordering::Greater,
        }
    }
}
#[repr(transparent)]
pub struct CString(*mut u8);
impl Display for CString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        unsafe {
            let mut ptr = self.0;
            let mut c = ptr.read();
            while c != 0 {
                string.push(char::from(c));
                ptr = ptr.offset(1);
                c = ptr.read();
            }
        }
        write!(f, "{string}")
    }
}
impl Debug for CString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CString").field(&self.to_string()).finish()
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct StdVec<T> {
    pub start: *mut T,
    pub end: *mut T,
    pub cap: *mut T,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdMapNode<K, V> {
    pub left: *const StdMapNode<K, V>,
    pub parent: *const StdMapNode<K, V>,
    pub right: *const StdMapNode<K, V>,
    pub color: bool,
    pad: [u8; 3],
    pub key: K,
    pub value: V,
}

#[repr(C)]
#[derive(Debug)]
pub struct StdMap<K, V> {
    pub root: *mut StdMapNode<K, V>,
    pub len: u32,
}

#[derive(Debug)]
pub struct StdMapIter<K, V> {
    pub root: *const StdMapNode<K, V>,
    pub current: *const StdMapNode<K, V>,
    pub parents: Vec<*const StdMapNode<K, V>>,
}

impl<K: 'static, V: 'static> Iterator for StdMapIter<K, V> {
    type Item = (&'static K, &'static V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.root {
            return None;
        }
        let tag = unsafe { self.current.as_ref()? };
        self.current = if tag.right != self.root {
            if tag.left == self.root {
                tag.right
            } else {
                self.parents.push(tag.right);
                tag.left
            }
        } else if tag.left == self.root {
            self.parents.pop().unwrap_or(self.root)
        } else {
            tag.left
        };
        Some((&tag.key, &tag.value))
    }
}

impl<K: 'static, V: 'static> StdMap<K, V> {
    pub fn iter(&self) -> impl Iterator<Item = (&'static K, &'static V)> {
        StdMapIter {
            root: self.root,
            current: unsafe { self.root.as_ref().unwrap().parent },
            parents: Vec::with_capacity(12),
        }
    }
    pub fn iter_keys(&self) -> impl Iterator<Item = &'static K> {
        self.iter().map(|(k, _)| k)
    }
    pub fn iter_values(&self) -> impl Iterator<Item = &'static V> {
        self.iter().map(|(_, v)| v)
    }
}
impl<K: 'static + Ord, V: 'static> StdMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&'static V> {
        let mut node = unsafe { self.root.as_ref()?.parent.as_ref()? };
        loop {
            let next = match key.cmp(&node.key) {
                Ordering::Less => node.left,
                Ordering::Greater => node.right,
                Ordering::Equal => return Some(&node.value),
            };
            if next == self.root {
                return None;
            }
            node = unsafe { next.as_ref()? };
        }
    }
}

#[repr(transparent)]
pub struct ThiscallFn(c_void);
