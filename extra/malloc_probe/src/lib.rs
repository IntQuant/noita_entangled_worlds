use std::{
    alloc::{self, Layout},
    ffi::c_void,
};

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
        alloc::dealloc(
            pointer.cast(),
            Layout::from_size_align_unchecked(size + 4, 4),
        );
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn dummy() {}

#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(_: *const u8, _: u32, _: *const u8) -> u32 {
    // unsafe { ((0x00f07500) as *mut *const usize).write(operator_new as *const usize) };
    // unsafe { ((0x00f07504) as *mut *const usize).write(operator_delete as *const usize) };
    1
}
