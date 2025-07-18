mod component;
mod component_data;
mod entity;
mod objects;
mod tags;
mod world;
pub use component::*;
pub use entity::*;
pub use objects::*;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter};
use std::slice;
pub use tags::*;
pub use world::*;
#[repr(C)]
union Buffer {
    buffer: *mut u8,
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
impl From<&str> for StdString {
    fn from(value: &str) -> Self {
        let mut res = StdString {
            capacity: value.len(),
            size: value.len(),
            ..Default::default()
        };
        if res.capacity > 16 {
            let buffer = value.as_bytes().to_vec();
            res.buffer.buffer = buffer.as_ptr().cast_mut();
            std::mem::forget(buffer);
        } else {
            let mut iter = value.as_bytes().iter();
            res.buffer.sso_buffer = std::array::from_fn(|_| *iter.next().unwrap())
        }
        res
    }
}
impl Display for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slice: &[u8] = unsafe {
            if self.capacity <= 16 {
                &self.buffer.sso_buffer[0..self.size]
            } else {
                slice::from_raw_parts(self.buffer.buffer, self.size)
            }
        };
        let actual_len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        let string = str::from_utf8(&slice[..actual_len]).unwrap_or("UTF8_ERR");
        write!(f, "{string}")
    }
}
impl Debug for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdString").field(&self.to_string()).finish()
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
    pub left: *mut StdMapNode<K, V>,
    pub parent: *mut StdMapNode<K, V>,
    pub right: *mut StdMapNode<K, V>,
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
pub struct StdMapIter<K: 'static, V: 'static> {
    pub root: *mut StdMapNode<K, V>,
    pub current: *mut StdMapNode<K, V>,
    pub parents: Vec<*mut StdMapNode<K, V>>,
}

impl<K: 'static, V: 'static> Iterator for StdMapIter<K, V> {
    type Item = (&'static K, &'static V);
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag = if self.current == self.root {
                self.parents.pop()?.as_ref()?
            } else {
                self.current.as_ref()?
            };
            self.current = tag.left;
            if tag.right != self.root {
                self.parents.push(tag.right);
            }
            Some((&tag.key, &tag.value))
        }
    }
}

impl<K: 'static, V: 'static> StdMap<K, V> {
    pub fn iter(&self) -> impl Iterator<Item = (&'static K, &'static V)> {
        StdMapIter {
            root: self.root,
            current: unsafe { self.root.as_ref().unwrap().parent },
            parents: Vec::new(),
        }
    }
}

#[repr(transparent)]
pub struct ThiscallFn(c_void);
