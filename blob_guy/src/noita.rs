use crate::chunk::{CellType, Chunk};
use crate::{CHUNK_SIZE, construct_cell, remove_cell};
use std::arch::asm;
use std::{ffi::c_void, mem};
pub(crate) mod ntypes;
//pub(crate) mod pixel;

pub(crate) struct ParticleWorldState {
    pub(crate) world_ptr: *mut c_void,
    pub(crate) chunk_map_ptr: *mut c_void,
    pub(crate) material_list_ptr: *const c_void,
    pub(crate) blob_guy: u16,
    pub(crate) pixel_array: *const c_void,
    pub(crate) construct_ptr: *mut c_void,
    pub(crate) remove_ptr: *mut c_void,
}
impl ParticleWorldState {
    fn create_cell(
        &mut self,
        x: i32,
        y: i32,
        material: *const c_void,
        _memory: *const c_void,
    ) -> *mut ntypes::Cell {
        construct_cell(self.world_ptr, x, y, material).unwrap_or_default()
        /*unsafe {
            let cell_ptr: *mut ntypes::Cell;
            asm!(
                "mov ecx, {world}",
                "push 0",
                "push {material}",
                "push {y:e}",
                "push {x:e}",
                "call {construct}",
                world = in(reg) self.world_ptr,
                x = in(reg) x,
                y = in(reg) y,
                material = in(reg) material,
                construct = in(reg) self.construct_ptr,
                clobber_abi("C"),
                out("eax") cell_ptr,
            );
            cell_ptr
        }*/
    }
    fn remove_cell(&mut self, cell: *mut ntypes::Cell, x: i32, y: i32) {
        let _ = remove_cell(self.world_ptr, cell, x, y);
        /*unsafe {
            asm!(
                "mov ecx, {world}",
                "push 0",
                "push {y:e}",
                "push {x:e}",
                "push {cell}",
                "call {remove}",
                world = in(reg) self.world_ptr,
                cell = in(reg) cell,
                x = in(reg) x,
                y = in(reg) y,
                remove = in(reg) self.remove_ptr,
                clobber_abi("C"),
            );
        };*/
    }
    fn set_chunk(&mut self, x: i32, y: i32) -> bool {
        let x = x as isize;
        let y = y as isize;
        let chunk_index = (((((y >> 3) - 256) & 511) << 9) | (((x >> 3) - 256) & 511)) * 4;
        // Deref 1/3
        let chunk_arr = unsafe { self.chunk_map_ptr.offset(8).cast::<*const c_void>().read() };
        // Deref 2/3
        let chunk = unsafe { chunk_arr.offset(chunk_index).cast::<*const c_void>().read() };
        if chunk.is_null() {
            return true;
        }
        // Deref 3/3
        let pixel_array = unsafe { chunk.cast::<*const c_void>().read() };
        self.pixel_array = pixel_array;
        false
    }
    fn get_cell_raw(&self, x: i32, y: i32) -> Option<&ntypes::Cell> {
        let x = x as isize;
        let y = y as isize;
        let pixel = unsafe { self.pixel_array.offset(((y << 9) | x) * 4) };
        if pixel.is_null() {
            return None;
        }

        unsafe { pixel.cast::<*const ntypes::Cell>().read().as_ref() }
    }
    fn get_cell_raw_mut(&mut self, x: i32, y: i32) -> Option<*mut ntypes::Cell> {
        let x = x as isize;
        let y = y as isize;
        let pixel = unsafe { self.pixel_array.offset(((y << 9) | x) * 4) as *const ntypes::Cell };
        if pixel.is_null() {
            return None;
        }
        let cell = pixel as *mut ntypes::Cell;
        if cell.is_null() {
            return None;
        }
        Some(cell)
    }
    fn get_cell_raw_mut_nil(&mut self, x: i32, y: i32) -> *mut *mut ntypes::Cell {
        let x = x as isize;
        let y = y as isize;
        let pixel = unsafe { self.pixel_array.offset(((y << 9) | x) * 4) };
        pixel as *mut *mut ntypes::Cell
    }
    fn get_cell_material_id(&self, cell: &ntypes::Cell) -> u16 {
        let mat_ptr = cell.material_ptr();
        let offset = unsafe { mat_ptr.cast::<c_void>().offset_from(self.material_list_ptr) };
        (offset / ntypes::CELLDATA_SIZE) as u16
    }

    fn get_cell_type(&self, cell: &ntypes::Cell) -> Option<ntypes::CellType> {
        unsafe { Some(cell.material_ptr().as_ref()?.cell_type) }
    }

    pub(crate) unsafe fn encode_area(&mut self, x: i32, y: i32, chunk: &mut Chunk) -> bool {
        if self.set_chunk(x, y) {
            return false;
        }
        for i in 0..CHUNK_SIZE as i32 {
            for j in 0..CHUNK_SIZE as i32 {
                let k = (i * CHUNK_SIZE as i32 + j) as usize;
                let cell = self.get_cell_raw(i, j);
                if let Some(cell) = cell
                    && let Some(cell_type) = self.get_cell_type(cell)
                    && ntypes::CellType::Liquid == cell_type
                {
                    if self.get_cell_material_id(cell) == self.blob_guy {
                        chunk[k] = CellType::Remove
                    } else {
                        let cell: &ntypes::LiquidCell = unsafe { mem::transmute(cell) };
                        if cell.is_static {
                            chunk[k] = CellType::Solid;
                        } else {
                            chunk[k] = CellType::Liquid;
                        }
                    }
                }
            }
        }
        true
    }
    pub(crate) unsafe fn decode_area(&mut self, x: i32, y: i32, chunk: &Chunk) -> bool {
        if self.set_chunk(x, y) {
            return false;
        }
        for i in 0..CHUNK_SIZE as i32 {
            for j in 0..CHUNK_SIZE as i32 {
                match chunk[(i * CHUNK_SIZE as i32 + j) as usize] {
                    CellType::Blob => {
                        let x = x * CHUNK_SIZE as i32 + i;
                        let y = y * CHUNK_SIZE as i32 + j;
                        unsafe {
                            let cell = self.get_cell_raw_mut_nil(i, j);
                            self.remove_cell(*cell, x, y);
                            let blob_ptr = self
                                .material_list_ptr
                                .offset(ntypes::CELLDATA_SIZE * self.blob_guy as isize);
                            let src = self.create_cell(x, y, blob_ptr, std::ptr::null::<c_void>());
                            if !src.is_null() {
                                let liquid: &mut ntypes::LiquidCell =
                                    &mut *(src as *mut ntypes::LiquidCell);
                                liquid.is_static = false;
                                *cell = src;
                            }
                        }
                    }
                    CellType::Remove => {
                        let x = x * CHUNK_SIZE as i32 + i;
                        let y = y * CHUNK_SIZE as i32 + j;
                        if let Some(cell) = self.get_cell_raw_mut(i, j) {
                            self.remove_cell(cell, x, y);
                        }
                    }
                    _ => {}
                }
            }
        }
        true
    }
}
