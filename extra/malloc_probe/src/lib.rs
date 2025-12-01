use std::{
    alloc::{self, Layout},
    ffi::c_void,
};
use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, PAGE_READWRITE, VirtualProtect};
unsafe extern "cdecl" fn operator_new(size: usize) -> *mut c_void {
    unsafe {
        let buffer = alloc::alloc(Layout::from_size_align_unchecked(size + 16, 4)).cast::<usize>();
        buffer.write(buffer.offset(4) as usize);
        buffer.offset(1).write(1);
        buffer.offset(2).write(usize::MAX - 1);
        buffer.offset(3).write(size);
        buffer.offset(4).cast()
    }
}

unsafe extern "cdecl" fn operator_delete(pointer: *mut c_void) {
    unsafe {
        let size_ptr = pointer.cast::<usize>().offset(-1);
        let size = size_ptr.read();
        alloc::dealloc(pointer.offset(-4).cast(), Layout::from_size_align_unchecked(size + 16, 4));
    }
}

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
    1
}
