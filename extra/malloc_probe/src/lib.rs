use std::{
    alloc::{self, Layout},
    ffi::c_void,
};
use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, PAGE_READWRITE, VirtualProtect};
unsafe extern "cdecl" fn operator_new(size: usize) -> *mut c_void {
    let buffer =
        unsafe { alloc::alloc(Layout::from_size_align_unchecked(size + 4, 4)).cast::<usize>() };
    unsafe { buffer.write(size) };
    unsafe { buffer.offset(1).cast() }
}

unsafe extern "cdecl" fn operator_delete(pointer: *mut c_void) {
    let ptr = unsafe { pointer.cast::<usize>().offset(-1) };
    let size = unsafe { ptr.read() };
    unsafe {
        alloc::dealloc(ptr.cast(), Layout::from_size_align_unchecked(size + 4, 4));
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn dummy() {}

unsafe fn make_writable(addr: *mut *const usize) {
    let mut old = PAGE_PROTECTION_FLAGS(0);
    unsafe {
        VirtualProtect(addr.cast(), 8, PAGE_READWRITE, &mut old).unwrap();
    }
}

fn write(addr: *mut *const usize, new: *const usize) {
    unsafe {
        make_writable(addr);
        addr.write(new)
    }
}

#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(_: *const u8, _: u32, _: *const u8) -> u32 {
    /*write(
        0x00f074f0 as *mut *const usize, //malloc
        operator_new as *const usize,
    );*/
    write(
        0x00f07500 as *mut *const usize, //operator_new
        operator_new as *const usize,
    );

    /*write(
        0x00f074f4 as *mut *const usize, //free
        operator_delete as *const usize,
    );*/
    write(
        0x00f07504 as *mut *const usize, //operator_delete
        operator_delete as *const usize,
    );
    1
}
