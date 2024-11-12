// Type defs borrowed from NoitaPatcher.

use std::ffi::{c_char, c_void};

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
pub(crate) enum CellType {
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
pub(crate) struct Cell {
    pub(crate) vtable: *const CellVTable,

    hp: i32,
    unknown1: [u8; 8],
    is_burning: bool,
    unknown2: [u8; 3],
    material_ptr: *const CellData,
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

#[repr(C)]
pub(crate) struct Entity {
    _unknown0: [u8; 8],
    _filename_index: u32,
    // More stuff, not that relevant currently.
}

#[repr(C)]
pub(crate) struct EntityManager {
    _fld: c_void,
    // Unknown
}
