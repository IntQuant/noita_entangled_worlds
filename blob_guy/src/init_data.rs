use eyre::ContextCompat;
use object::{Object, ObjectSection};
use std::ffi::c_void;
use std::fs::File;
use std::io::Read;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
pub fn get_functions() -> eyre::Result<(*const c_void, *const c_void)> {
    let exe = std::env::current_exe()?;
    let mut file = File::open(exe)?;
    let mut vec = Vec::with_capacity(15460864);
    file.read_to_end(&mut vec)?;
    let obj = object::File::parse(vec.as_slice())?;
    let text = obj.section_by_name(".text").wrap_err("obj err")?;
    let data = text.data()?;
    let construct: &[u8] = &[0x8b, 0x46, 0x38, 0x33, 0xc9, 0x83, 0xf8, 0x01];
    let remove: &[u8] = &[
        0x8b, 0x06, 0x8b, 0xce, 0xff, 0x90, 0x9c, 0x00, 0x00, 0x00, 0x8b, 0x06, 0x8b, 0xce, 0x6a,
        0x01, 0xff, 0x10,
    ];
    let start = get_module();
    let text_start = find_pattern(vec.as_slice(), data)?;
    let construct = find_pattern(data, construct)?;
    let remove = find_pattern(data, remove)?;
    let construct_ptr = get_function_start(unsafe { start.add(2968 + text_start + construct) })?;
    let remove_ptr = get_function_start(unsafe { start.add(2968 + text_start + remove) })?;
    Ok((construct_ptr, remove_ptr))
}
fn find_pattern(data: &[u8], pattern: &[u8]) -> eyre::Result<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
        .wrap_err("match err")
}
fn get_module() -> *const c_void {
    unsafe { GetModuleHandleA(None).unwrap().0 }
}
fn get_function_start(func: *const c_void) -> eyre::Result<*const c_void> {
    let mut it = func.cast::<u8>();
    loop {
        unsafe {
            if it as isize % 16 == 0
                && (it.offset(-1).read() == 0xcc
                    || it.offset(-1).read() == 0xc3
                    || it.offset(-3).read() == 0xc2)
                && (it.read() >= 0x50 && it.read() < 0x58)
                && ((it.offset(1).read() >= 0x50 && it.offset(1).read() < 0x58)
                    || (it.offset(1).read() == 0x8b && it.offset(2).read() == 0xec))
            {
                return Ok(it.cast::<c_void>());
            }
        }
        it = unsafe { it.offset(-1) }
    }
}
