mod component;
mod component_data;
mod entity;
mod misc;
mod objects;
mod platform;
mod vftables;
mod world;
pub use component::*;
pub use component_data::*;
pub use entity::*;
pub use misc::*;
pub use objects::*;
pub use platform::*;
use std::cmp::Ordering;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};
use std::{ptr, slice};
pub use vftables::*;
pub use world::*;

use crate::heap;
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
            buffer: Default::default(),
            capacity: value.len(),
            size: value.len(),
        };
        if res.capacity > 16 {
            let buffer = heap::place_new(value);
            res.buffer.buffer = buffer.cast();
        } else {
            let mut iter = value.as_bytes().iter();
            res.buffer.sso_buffer = std::array::from_fn(|_| iter.next().copied().unwrap_or(0))
        }
        res
    }
}
impl StdString {
    pub const fn from_str(value: &'static str) -> Self {
        let mut res = StdString {
            buffer: Buffer {
                sso_buffer: [0; 16],
            },
            capacity: value.len(),
            size: value.len(),
        };
        if res.capacity > 16 {
            res.buffer.buffer = value.as_ptr();
        } else {
            let iter = value.as_bytes();
            res.buffer.sso_buffer = [
                if 0 < res.size { iter[0] } else { 0 },
                if 1 < res.size { iter[1] } else { 0 },
                if 2 < res.size { iter[2] } else { 0 },
                if 3 < res.size { iter[3] } else { 0 },
                if 4 < res.size { iter[4] } else { 0 },
                if 5 < res.size { iter[5] } else { 0 },
                if 6 < res.size { iter[6] } else { 0 },
                if 7 < res.size { iter[7] } else { 0 },
                if 8 < res.size { iter[8] } else { 0 },
                if 9 < res.size { iter[9] } else { 0 },
                if 10 < res.size { iter[10] } else { 0 },
                if 11 < res.size { iter[11] } else { 0 },
                if 12 < res.size { iter[12] } else { 0 },
                if 13 < res.size { iter[13] } else { 0 },
                if 14 < res.size { iter[14] } else { 0 },
                if 15 < res.size { iter[15] } else { 0 },
            ]
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
pub struct CString(pub *const u8);
impl From<&str> for CString {
    fn from(value: &str) -> Self {
        let value = value.to_owned() + "\0";
        let str = heap::place_new(value).cast();
        CString(str)
    }
}
impl CString {
    pub const fn from_str(value: &'static str) -> Self {
        CString(value.as_ptr())
    }
}
#[test]
fn test_str() {
    let str;
    let str2;
    {
        str = CString::from("test");
        str2 = const { CString::from_str("test\0") };
        assert_eq!(str.to_string(), "test");
        assert_eq!(str2.to_string(), "test")
    }
    assert_eq!(str.to_string(), "test");
    assert_eq!(str2.to_string(), "test");
    assert_eq!(WalletComponent::NAME, WalletComponent::C_NAME.to_string());
    assert_eq!(WalletComponent::NAME, WalletComponent::STD_NAME.to_string());
}
#[test]
fn test_cstring() {
    let a = CString::from_str("test");
    let b = CString::from_str("test");
    let c = CString::from_str("testb");
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(c, a);
}
impl PartialEq for CString {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let mut ptra = self.0;
            let mut ptrb = other.0;
            let mut a = ptra.read();
            let mut b = ptrb.read();
            while a != 0 {
                if a != b {
                    return false;
                }
                ptra = ptra.offset(1);
                a = ptra.read();
                ptrb = ptrb.offset(1);
                b = ptrb.read();
            }
            b == 0
        }
    }
}
impl Display for CString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_null() {
            return write!(f, "");
        }
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
#[repr(transparent)]
pub struct CStr<const N: usize>(pub [u8; N]);
impl<const N: usize> Display for CStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for c in self.0 {
            if c == 0 {
                break;
            }
            string.push(char::from(c));
        }
        write!(f, "{string}")
    }
}
impl<const N: usize> Debug for CStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CStr").field(&self.to_string()).finish()
    }
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct StdSet<T: Default + Debug> {
    pub a: T,
    pub b: T,
}
#[repr(C)]
pub struct StdVec<T> {
    pub start: *mut T,
    pub end: *mut T,
    pub cap: *mut T,
}
impl<T> Index<usize> for StdVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}
impl<T> IndexMut<usize> for StdVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).unwrap()
    }
}
impl<T> AsRef<[T]> for StdVec<T> {
    fn as_ref(&self) -> &[T] {
        if self.start.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.start, self.len()) }
        }
    }
}
impl<T> AsMut<[T]> for StdVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.start, self.len()) }
    }
}
impl<T: Debug> Debug for StdVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdVec").field(&self.as_ref()).finish()
    }
}
impl<T> Default for StdVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StdVec<T> {
    pub const fn null() -> Self {
        Self {
            start: ptr::null_mut(),
            end: ptr::null_mut(),
            cap: ptr::null_mut(),
        }
    }
    pub fn copy(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            cap: self.cap,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_ref().iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_mut().iter_mut()
    }
    pub fn capacity(&self) -> usize {
        unsafe { self.cap.offset_from_unsigned(self.start) }
    }
    pub fn len(&self) -> usize {
        unsafe { self.end.offset_from_unsigned(self.start) }
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn get_static(&self, index: usize) -> Option<&'static T> {
        let ptr = unsafe { self.start.add(index) };
        if self.end > ptr {
            unsafe { ptr.as_ref() }
        } else {
            None
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        let ptr = unsafe { self.start.add(index) };
        if self.end > ptr {
            unsafe { ptr.as_ref() }
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let ptr = unsafe { self.start.add(index) };
        if self.end > ptr {
            unsafe { ptr.as_mut() }
        } else {
            None
        }
    }
    fn alloc(&mut self, n: usize) {
        if self.cap < unsafe { self.end.add(n) } {
            let old_len = self.len();
            let new_cap = (old_len + n).next_power_of_two();
            let new_ptr = heap::raw_new(new_cap * size_of::<T>());
            if old_len > 0 {
                unsafe {
                    ptr::copy_nonoverlapping(self.start, new_ptr, old_len);
                }
            }
            if !self.start.is_null() {
                unsafe {
                    heap::delete(self.start);
                }
            }
            self.start = new_ptr;
            self.end = unsafe { new_ptr.add(old_len) };
            self.cap = unsafe { new_ptr.add(new_cap) };
        }
    }
    pub fn with_capacity(n: usize) -> StdVec<T> {
        let mut v = Self::null();
        v.alloc(n);
        v
    }
    pub fn new() -> StdVec<T> {
        Self::with_capacity(4)
    }
    pub fn push(&mut self, value: T) {
        self.alloc(1);
        unsafe {
            self.end.write(value);
            self.end = self.end.add(1);
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.start == self.end {
            return None;
        }
        unsafe {
            self.end = self.end.sub(1);
            let ret = self.end.read();
            Some(ret)
        }
    }
    pub fn last(&self) -> Option<&T> {
        unsafe { self.end.sub(1).as_ref() }
    }
    pub fn last_mut(&mut self) -> Option<&mut T> {
        unsafe { self.end.sub(1).as_mut() }
    }
    pub fn insert(&mut self, index: usize, value: T) {
        self.alloc(1);
        for i in (index..self.len()).rev() {
            unsafe { self.start.add(i + 1).write(self.start.add(i).read()) }
        }
        unsafe {
            self.end = self.end.add(1);
            self.start.add(index).write(value);
        }
    }
    pub fn remove(&mut self, index: usize) -> T {
        unsafe {
            let ret = self.start.add(index).read();
            for i in index..self.len() - 1 {
                self.start.add(i).write(self.start.add(i + 1).read())
            }
            self.end = self.end.sub(1);
            ret
        }
    }
}
impl<T> Drop for StdVec<T> {
    fn drop(&mut self) {
        if !self.start.is_null() {
            unsafe {
                heap::delete(self.start);
            }
        }
    }
}
#[test]
fn test_stdvec() {
    let mut v = StdVec::<u8>::null();
    assert_eq!(v.capacity(), 0);
    assert_eq!(v.len(), 0);
    v.alloc(1);
    assert_eq!(v.capacity(), 1);
    assert_eq!(v.len(), 0);
    let mut v = StdVec::<usize>::new();
    v.push(1usize);
    v.push(2usize);
    v.push(3usize);
    v.push(4usize);
    v.push(5usize);
    for (i, l) in std::hint::black_box(&v).iter().enumerate() {
        assert_eq!(i, *l);
    }
    assert_eq!(v.capacity(), 8);
    std::hint::black_box(v);
}

#[repr(C)]
#[derive(Debug)]
pub struct StdMapNode<K, V> {
    pub left: *mut StdMapNode<K, V>,
    pub parent: *mut StdMapNode<K, V>,
    pub right: *mut StdMapNode<K, V>,
    pub color: bool,
    pub end: bool,
    unk: [u8; 2],
    pub key: K,
    pub value: V,
}
impl<K: Default, V: Default> Default for StdMapNode<K, V> {
    fn default() -> Self {
        Self {
            left: ptr::null_mut(),
            parent: ptr::null_mut(),
            right: ptr::null_mut(),
            color: false,
            end: true,
            unk: [0, 0],
            key: Default::default(),
            value: Default::default(),
        }
    }
}

impl<K, V> StdMapNode<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            left: ptr::null_mut(),
            parent: ptr::null_mut(),
            right: ptr::null_mut(),
            color: false,
            end: true,
            unk: [0, 0],
            key,
            value,
        }
    }
}

#[repr(C)]
pub struct StdMap<K: 'static, V: 'static> {
    pub root: &'static mut StdMapNode<K, V>,
    pub len: usize,
}
impl<K: Default + 'static, V: Default + 'static> Default for StdMap<K, V> {
    fn default() -> Self {
        Self {
            root: unsafe { &mut *heap::place_new(StdMapNode::default()) },
            len: 0,
        }
    }
}
impl<K: Debug + 'static, V: Debug + 'static> Debug for StdMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StdMap")
            .field(&self.iter().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Debug)]
pub struct StdMapIter<K: 'static, V: 'static> {
    pub root: *mut StdMapNode<K, V>,
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.is_empty() {
            self.len += 1;
            let node = StdMapNode::new(key, value);
            self.root.parent = heap::place_new(node);
            None
        } else {
            todo!()
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (&'static K, &'static V)> + '_ {
        StdMapIter {
            root: (self.root as *const StdMapNode<K, V>).cast_mut(),
            current: self.root.parent,
            parents: Vec::with_capacity(8),
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
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
        let mut node = unsafe { self.root.parent.as_ref()? };
        loop {
            let next = match key.cmp(&node.key) {
                Ordering::Less => node.left,
                Ordering::Greater => node.right,
                Ordering::Equal => return Some(&node.value),
            };
            if next == (self.root as *const StdMapNode<K, V>).cast_mut() {
                return None;
            }
            node = unsafe { next.as_ref()? };
        }
    }
}

#[repr(transparent)]
pub struct ThiscallFn(c_void);

#[derive(Debug, Default)]
#[repr(C)]
pub struct LensValueBool {
    pub value: bool,
    pub valueb: bool,
    padding: [u8; 2],
    pub frame: isize,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct LensValue<T> {
    pub value: T,
    pub valueb: T,
    pub frame: isize,
}
#[derive(Debug, Default)]
#[repr(C)]
pub struct ValueRange<T> {
    pub min: T,
    pub max: T,
}
#[derive(Debug, Default)]
#[repr(C)]
pub struct DebugSettings {
    vftable: *const usize,
    unk: [*const usize; 12],
}
