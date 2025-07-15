use crate::noita::types;
use crate::noita::types::{
    CellVTable, CellVTables, FireCellVTable, GasCellVTable, LiquidCellVTable, NoneCellVTable,
    SolidCellVTable,
};
use eyre::ContextCompat;
use object::{Object, ObjectSection};
use std::arch::asm;
use std::ffi::c_void;
use std::fs::File;
use std::io::Read;
pub fn get_functions() -> eyre::Result<(types::CellVTables, *mut types::GameGlobal)> {
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
    let cellvtables = unsafe {
        let none = CellVTable {
            none: (0xff2040 as *const NoneCellVTable).as_ref().unwrap(),
        };
        let liquid = CellVTable {
            liquid: (0x100bb90 as *const LiquidCellVTable).as_ref().unwrap(),
        };
        let gas = CellVTable {
            gas: (0x1007bcc as *const GasCellVTable).as_ref().unwrap(),
        };
        let solid = CellVTable {
            solid: (0xff8a6c as *const SolidCellVTable).as_ref().unwrap(),
        };
        let fire = CellVTable {
            fire: (0x10096e0 as *const FireCellVTable).as_ref().unwrap(),
        };
        CellVTables([none, liquid, gas, solid, fire])
    };
    Ok((cellvtables, game_global_ptr))
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
/*fn find_pattern(data: &[u8], pattern: &[u8]) -> eyre::Result<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
        .wrap_err("match err")
}*/
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
