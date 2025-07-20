mod component;
mod component_data;
mod entity;
mod objects;
mod world;
pub use component::*;
pub use component_data::*;
pub use entity::*;
pub use objects::*;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter};
use std::slice;
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
impl StdString {
    pub fn get(&self, index: usize) -> u8 {
        unsafe {
            if self.capacity <= 16 {
                self.buffer.sso_buffer[index]
            } else {
                self.buffer.buffer.add(index).read()
            }
        }
    }
}

impl AsRef<str> for StdString {
    fn as_ref(&self) -> &str {
        let slice: &[u8] = unsafe {
            if self.capacity <= 16 {
                &self.buffer.sso_buffer
            } else {
                slice::from_raw_parts(self.buffer.buffer, self.size)
            }
        };
        let actual_len = slice.iter().position(|&b| b == 0).unwrap_or(self.size);
        str::from_utf8(&slice[..actual_len]).unwrap_or("UTF8_ERR")
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
        write!(f, "{}", self.as_ref())
    }
}
impl Debug for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdString").field(&self.as_ref()).finish()
    }
}
impl PartialEq for StdString {
    fn eq(&self, other: &Self) -> bool {
        if self.size == other.size {
            self.as_ref() == other.as_ref()
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
        let smallest = self.size.min(other.size);
        for i in 0..smallest {
            match self.get(i).cmp(&other.get(i)) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        self.size.cmp(&other.size)
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
pub struct StdVec<T> {
    pub start: *mut T,
    pub end: *mut T,
    pub cap: *mut T,
}
impl<T: 'static> AsRef<[T]> for StdVec<T> {
    fn as_ref(&self) -> &'static [T] {
        unsafe { slice::from_raw_parts(self.start, self.len()) }
    }
}
impl<T: 'static> AsMut<[T]> for StdVec<T> {
    fn as_mut(&mut self) -> &'static mut [T] {
        unsafe { slice::from_raw_parts_mut(self.start, self.len()) }
    }
}
impl<T: Debug + 'static> Debug for StdVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdVec")
            .field(&format!("{:?}", self.as_ref()))
            .finish()
    }
}
impl<T> StdVec<T> {
    pub fn len(&self) -> usize {
        unsafe { self.end.byte_offset_from_unsigned(self.start) }
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        unsafe { self.start.add(index).as_ref() }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        unsafe { self.start.add(index).as_mut() }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct StdMapNode<K, V> {
    pub left: *const StdMapNode<K, V>,
    pub parent: *const StdMapNode<K, V>,
    pub right: *const StdMapNode<K, V>,
    pub color: bool,
    pub end: bool,
    unk: [u8; 2],
    pub key: K,
    pub value: V,
}

#[repr(C)]
pub struct StdMap<K, V> {
    pub root: *mut StdMapNode<K, V>,
    pub len: usize,
}
impl<K: Debug + 'static, V: Debug + 'static> Debug for StdMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdMap")
            .field(&format!("{:?}", self.iter().collect::<Vec<_>>()))
            .finish()
    }
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
            parents: Vec::with_capacity(8),
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
