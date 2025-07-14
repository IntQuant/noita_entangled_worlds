use crate::noita::types;
use eyre::ContextCompat;
use object::{Object, ObjectSection};
use std::arch::asm;
use std::ffi::c_void;
use std::fs::File;
use std::io::Read;
pub fn get_functions() -> eyre::Result<(&'static types::CellVTable, *mut types::GameGlobal)> {
    let exe = std::env::current_exe()?;
    let mut file = File::open(exe)?;
    let mut vec = Vec::with_capacity(15460864);
    file.read_to_end(&mut vec)?;
    let obj = object::File::parse(vec.as_slice())?;
    let text = obj.section_by_name(".text").wrap_err("obj err")?;
    let data = text.data()?;
    let game_global: &[u8] = &[0xe8];
    let game_global2 = &[0x8b, 0x40, 0x48, 0x8b, 0x00, 0xc1, 0xe8, 0x02, 0xa8, 0x01];
    let start = text.address() as *const c_void;
    let (game_global, offset) = find_pattern_global(data, game_global, game_global2)?;
    let ptr = unsafe { start.add(game_global) };
    let game_global_ptr = get_rela_call(ptr, offset);
    let game_global_ptr = get_global(game_global_ptr);
    let rdata = obj.section_by_name(".rdata").wrap_err("obj err")?;
    let data = rdata.data()?;
    let cellvtable: &[u8] = &[
        0x20, 0xaf, 0x70, 0x00, 0xa0, 0x01, 0x5b, 0x00, 0x50, 0xb0, 0x70, 0x00, 0x60, 0xb0, 0x70,
        0x00, 0xc0, 0x01, 0x5b, 0x00, 0xd0, 0x01, 0x5b, 0x00, 0x90, 0xd0, 0x70, 0x00, 0xe0, 0x01,
        0x5b, 0x00, 0x00, 0x02, 0x5b, 0x00, 0xf0, 0x01, 0x5b, 0x00, 0x70, 0xb0, 0x70, 0x00, 0xb0,
        0xb0, 0x70, 0x00, 0xd0, 0xc0, 0x4a, 0x00, 0xb0, 0xd0, 0x70, 0x00, 0x60, 0xbf, 0x4a, 0x00,
        0xa0, 0xd1, 0x70, 0x00, 0xe0, 0xd1, 0x70, 0x00, 0x80, 0xd1, 0x70, 0x00, 0x40, 0xcb, 0x70,
        0x00, 0x80, 0xcd, 0x70, 0x00, 0xd0, 0xcd, 0x70, 0x00, 0xe0, 0xc6, 0x70, 0x00, 0xb0, 0x01,
        0x5b, 0x00, 0x90, 0xbf, 0x4a, 0x00, 0xa0, 0xbf, 0x4a, 0x00, 0x10, 0xb1, 0x70, 0x00, 0x20,
        0xb1, 0x70, 0x00, 0x60, 0xb1, 0x70, 0x00, 0xb0, 0xf5, 0x70, 0x00, 0xd0, 0xf5, 0x70, 0x00,
        0xf0, 0xcd, 0x70, 0x00, 0x50, 0xf7, 0x70, 0x00, 0xe0, 0xc0, 0x4a, 0x00, 0xf0, 0xf7, 0x70,
        0x00, 0x20, 0xc0, 0x4a, 0x00, 0x60, 0xf1, 0x70, 0x00, 0xf0, 0xea, 0x70, 0x00, 0x90, 0xef,
        0x70, 0x00, 0x60, 0xf3, 0x70, 0x00, 0x50, 0xaf, 0x70, 0x00, 0xd0, 0xb1, 0x70,
        0x00,
        //TODO i should search for a function in the vtable then find the vtable prob
    ];
    let start = rdata.address() as *const c_void;
    let cellvtable_ptr = unsafe {
        (start.add(find_pattern(data, cellvtable)?) as *const types::CellVTable).as_ref()
    }
    .wrap_err("cell data err")?;
    Ok((cellvtable_ptr, game_global_ptr))
}
fn get_global(global: *const c_void) -> *mut types::GameGlobal {
    unsafe {
        let ptr: *mut types::GameGlobal;
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
