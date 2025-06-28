// Type defs borrowed from NoitaPatcher.

use std::ffi::c_char;

pub(crate) const CELLDATA_SIZE: isize = 0x290;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct StdString {
    buffer: *const i8,
    sso_buffer: [i8; 12],
    size: usize,
    capacity: usize,
}
#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[expect(dead_code)]
pub enum CellType {
    None = 0,
    Liquid = 1,
    Gas = 2,
    Solid = 3,
    Fire = 4,
    Invalid = 4294967295,
}

#[repr(C)]
pub(crate) struct CellData {
    name: StdString,
    ui_name: StdString,
    material_type: i32,
    id_2: i32,
    pub(crate) cell_type: CellType,
    // Has a bunch of other fields that aren't that relevant.
}

#[repr(C)]
pub(crate) struct CellVTable {}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct Cell {
    pub vtable: *const CellVTable,

    hp: i32,
    unknown1: [u8; 8],
    is_burning: bool,
    unknown2: [u8; 3],
    pub material_ptr: *const CellData,
}

#[repr(C)]
pub(crate) struct LiquidCell {
    cell: Cell,
    x: i32,
    y: i32,
    unknown1: c_char,
    unknown2: c_char,
    pub(crate) is_static: bool,
    // Has a bunch of other fields that aren't that relevant.
}

impl Cell {
    pub(crate) fn material_ptr(&self) -> *const CellData {
        self.material_ptr
    }
}
