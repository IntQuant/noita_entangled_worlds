use std::sync::LazyLock;

struct Msvcr {
    op_new: unsafe extern "C" fn(n: std::os::raw::c_uint) -> *mut std::os::raw::c_void,
    op_delete: unsafe extern "C" fn(*const std::os::raw::c_void),
    // op_delete_array: unsafe extern "C" fn(*const std::os::raw::c_void),
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
pub fn raw_new(size: usize) -> *mut std::os::raw::c_void {
    let size = size as std::os::raw::c_uint;
    assert!(size > 0, "Doesn't make sense to allocate memory of size 0");
    unsafe { (MSVCR.op_new)(size) }
}

/// Allocates memory using noita's allocator and moves *value* to it.
pub fn place_new<T>(value: T) -> *mut T {
    let size = size_of::<T>();
    let place = raw_new(size) as *mut T;
    unsafe {
        place.copy_from_nonoverlapping(&value, size);
    }
    place
}

/// Same as place_new, but returns &'static mut
pub fn place_new_ref<T>(value: T) -> &'static mut T {
    unsafe { &mut *place_new(value) }
}

/// # Safety
///
/// Pointer has to be non null, allocated by noita's allocator, and not yet freed.
pub unsafe fn delete<T>(pointer: *const T) {
    unsafe { (MSVCR.op_delete)(pointer.cast()) }
}
