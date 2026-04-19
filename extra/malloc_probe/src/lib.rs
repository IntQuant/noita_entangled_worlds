#[cfg(feature = "log")]
use std::fs::OpenOptions;
#[cfg(feature = "log")]
use std::io::Write;
#[cfg(feature = "log")]
use std::sync::{Arc, LazyLock, Mutex};
#[cfg(windows)]
use std::{
    alloc::{self, Layout},
    ffi::c_void,
    slice,
};
#[cfg(windows)]
use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, PAGE_READWRITE, VirtualProtect};
#[allow(clippy::type_complexity)]
#[cfg(feature = "log")]
static LIST: LazyLock<Arc<Mutex<Vec<(usize, usize)>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::with_capacity(65536))));
#[cfg(windows)]
unsafe extern "cdecl" fn operator_new(size: usize) -> *mut c_void {
    unsafe {
        let buffer = alloc::alloc(Layout::from_size_align_unchecked(size + 16, 4)).cast::<usize>();
        buffer.write(buffer.offset(4) as usize);
        #[cfg(feature = "log")]
        LIST.lock().unwrap().push((buffer.offset(4) as usize, size));
        buffer.offset(1).write(1);
        buffer.offset(2).write(usize::MAX - 1);
        buffer.offset(3).write(size);
        buffer.offset(4).cast()
    }
}

#[cfg(windows)]
unsafe extern "cdecl" fn operator_delete(pointer: *mut c_void) {
    unsafe {
        let size_ptr = pointer.cast::<usize>().offset(-1);
        let size = size_ptr.read();
        let ptr = pointer.offset(-4);
        alloc::dealloc(ptr.cast(), Layout::from_size_align_unchecked(size + 16, 4));
    }
}

#[cfg(feature = "log")]
#[unsafe(no_mangle)]
/// # Safety
/// input must be a &str
pub unsafe extern "C" fn put_data(s: *const u8, l: usize) {
    let s = unsafe { slice::from_raw_parts(s, l) };
    let s = unsafe { str::from_utf8_unchecked(s) };
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(s)
        .unwrap();
    for (ptr, size) in LIST.lock().unwrap().iter().map(|(a, b)| (*a, *b)) {
        file.write_all(&ptr.to_le_bytes()).unwrap();
        file.write_all(&size.to_le_bytes()).unwrap();
        let ptr = ptr as *const u8;
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        file.write_all(slice).unwrap();
    }
}

#[cfg(windows)]
unsafe fn make_writable(addr: *mut *const usize) {
    let mut old = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtect(addr.cast(), 4, PAGE_READWRITE, &mut old).unwrap();
    }
}

#[cfg(windows)]
fn write(addr: *mut *const usize, new: *const usize) {
    unsafe {
        #[cfg(windows)]
        make_writable(addr);
        addr.write(new)
    }
}

#[cfg(windows)]
#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(_: *const u8, _: u32, _: *const u8) -> u32 {
    write(
        0x00f074f0 as *mut *const usize, //malloc
        operator_new as *const usize,
    );
    write(
        0x00f07500 as *mut *const usize, //operator_new
        operator_new as *const usize,
    );

    write(
        0x00f074f4 as *mut *const usize, //free
        operator_delete as *const usize,
    );
    write(
        0x00f07504 as *mut *const usize, //operator_delete
        operator_delete as *const usize,
    );
    write(
        0x00f0750c as *mut *const usize, //operator_delete[]
        operator_delete as *const usize,
    );
    1
}
