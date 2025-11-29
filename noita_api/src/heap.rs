use std::ops::{Deref, DerefMut};
use std::os::raw::{c_uint, c_void};
use std::ptr::null_mut;
use std::sync::LazyLock;

struct Msvcr {
    op_new: unsafe extern "C" fn(n: c_uint) -> *mut c_void,
    op_delete: unsafe extern "C" fn(*mut c_void),
    // op_delete_array: unsafe extern "C" fn(*mut c_void),
}

static MSVCR: LazyLock<Msvcr> = LazyLock::new(|| unsafe {
    let lib = libloading::Library::new("./msvcr120.dll").expect("library to exist");
    let op_new = *lib.get(b"??2@YAPAXI@Z\0").expect("symbol to exist");
    let op_delete = *lib.get(b"??3@YAXPAX@Z\0").expect("symbol to exist");
    // let op_delete_array = *lib.get(b"operator_delete[]\0").expect("symbol to exist");
    Msvcr {
        op_new,
        op_delete,
        // op_delete_array,
    }
});

/// Allocate some memory, using the same allocator noita uses.
pub fn raw_new<T>(size: usize) -> *mut T {
    let size = size as c_uint;
    assert!(size > 0, "Doesn't make sense to allocate memory of size 0");
    unsafe { (MSVCR.op_new)(size).cast() }
}

/// Allocates memory using noita's allocator and moves *value* to it.
pub fn place_new<T>(value: T) -> *mut T {
    let size = size_of::<T>();
    let place = raw_new::<T>(size);
    unsafe {
        place.copy_from_nonoverlapping(&value, 1);
    }
    place
}

/// Same as place_new, but returns &'static mut
pub fn place_new_ref<T>(value: T) -> &'static mut T {
    unsafe { place_new(value).as_mut().unwrap() }
}

// Too easy to misuse, leaving it crate-local cause it's still useful for e. g. StdVec
/// # Safety
///
/// Pointer has to be non null, allocated by noita's allocator, and not yet freed.
pub(crate) unsafe fn delete<T>(pointer: *mut T) {
    unsafe { (MSVCR.op_delete)(pointer.cast()) }
}

/// Pointer for memory allocated by noita's allocator
#[repr(transparent)]
pub struct Ptr<T>(*mut T);

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Ptr<T> {}

impl<T> Deref for Ptr<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Ptr<T> {
    /// # Safety
    ///
    /// Has to be a noita pointer
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Self(raw)
    }

    pub fn place_new(value: T) -> Self {
        unsafe { Self::from_raw(place_new(value)) }
    }

    pub const fn null() -> Self {
        Self(null_mut())
    }

    /// Deallocates pointer if it isn't null, and sets internal pointer to null.
    ///
    /// # Safety
    ///
    /// Pointer has to be not yet freed.
    pub unsafe fn delete(&mut self) {
        if !self.0.is_null() {
            unsafe {
                delete(self.0);
                self.0 = null_mut();
            }
        }
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn cast<U>(self) -> Ptr<U> {
        Ptr(self.0.cast())
    }

    /// # Safety
    ///
    /// Pointer has to be not yet freed and of correct type.
    pub unsafe fn as_ref(&self) -> Option<&T> {
        if self.is_null() {
            return None;
        }
        Some(unsafe { &*self.0 })
    }

    /// # Safety
    ///
    /// Pointer has to be not yet freed and of correct type.
    pub unsafe fn as_mut(&mut self) -> Option<&mut T> {
        if self.is_null() {
            return None;
        }
        Some(unsafe { &mut *self.0 })
    }

    pub fn as_raw(&self) -> *mut T {
        self.0
    }
}
