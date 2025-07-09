use crate::noita::ntypes;
use eyre::ContextCompat;
use object::{Object, ObjectSection};
use std::arch::asm;
use std::ffi::c_void;
use std::fs::File;
use std::io::Read;
pub fn get_functions() -> eyre::Result<(*const c_void, *const c_void, *const ntypes::GameGlobal)> {
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
    let game_global: &[u8] = &[0xe8];
    let game_global2 = &[0x8b, 0x40, 0x48, 0x8b, 0x00, 0xc1, 0xe8, 0x02, 0xa8, 0x01];
    let start = text.address() as *const c_void;
    let construct = find_pattern(data, construct)?;
    let remove = find_pattern(data, remove)?;
    let construct_ptr = get_function_start(unsafe { start.add(construct) })?;
    let remove_ptr = get_function_start(unsafe { start.add(remove) })?;
    let (game_global, offset) = find_pattern_global(data, game_global, game_global2)?;
    let ptr = unsafe { start.add(game_global) };
    let game_global_ptr = get_rela_call(ptr, offset);
    let game_global_ptr = get_global(game_global_ptr);
    Ok((construct_ptr, remove_ptr, game_global_ptr))
}
fn get_global(global: *const c_void) -> *const ntypes::GameGlobal {
    unsafe {
        let ptr: *const ntypes::GameGlobal;
        asm!(
            "call {global}",
            global = in(reg) global,
            clobber_abi("C"),
            out("eax") ptr,
        );
        ptr
    }
}
fn find_pattern(data: &[u8], pattern: &[u8]) -> eyre::Result<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
        .wrap_err("match err")
}
fn find_pattern_global(data: &[u8], pattern: &[u8], other: &[u8]) -> eyre::Result<(usize, isize)> {
    let r = data
        .windows(pattern.len() + 4 + other.len())
        .enumerate()
        .filter(|(_, window)| window.starts_with(pattern) && window.ends_with(other))
        .nth(1)
        .map(|(i, _)| i)
        .wrap_err("match err")?;
    let mut iter = data[r + 1..=r + 4].iter();
    let bytes = std::array::from_fn(|_| *iter.next().unwrap());
    Ok((r, isize::from_ne_bytes(bytes)))
}
fn get_rela_call(ptr: *const c_void, offset: isize) -> *const c_void {
    unsafe {
        let next_instruction = ptr.add(5);
        next_instruction.offset(offset)
    }
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
