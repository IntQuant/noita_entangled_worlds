use crate::CHUNK_SIZE;
use crate::chunk::{CellType, Chunk};
#[cfg(target_arch = "x86")]
use std::arch::asm;
use std::{ffi::c_void, mem, ptr};
pub(crate) mod ntypes;
//pub(crate) mod pixel;
#[derive(Default)]
pub(crate) struct ParticleWorldState {
    #[cfg(target_arch = "x86")]
    pub(crate) world_ptr: *mut c_void,
    pub(crate) chunk_map_ptr: *mut c_void,
    pub(crate) material_list_ptr: *const c_void,
    pub(crate) blob_guy: u16,
    pub(crate) blob_ptr: *const c_void,
    pub(crate) pixel_array: *const c_void,
    #[cfg(target_arch = "x86")]
    pub(crate) construct_ptr: *mut c_void,
    #[cfg(target_arch = "x86")]
    pub(crate) remove_ptr: *mut c_void,
    pub(crate) shift_x: isize,
    pub(crate) shift_y: isize,
}
impl ParticleWorldState {
    fn create_cell(
        &mut self,
        x: isize,
        y: isize,
        material: *const c_void,
        //_memory: *const c_void,
    ) -> *mut ntypes::Cell {
        #[cfg(target_arch = "x86")]
        unsafe {
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
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((x, y, material));
            Default::default()
        }
    }
    fn remove_cell(&mut self, cell: *mut ntypes::Cell, x: isize, y: isize) {
        #[cfg(target_arch = "x86")]
        unsafe {
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
        };
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((x, y, cell));
        }
    }
    fn set_chunk(&mut self, x: isize, y: isize) -> bool {
        const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
        self.shift_x = (x * CHUNK_SIZE as isize).rem_euclid(512);
        self.shift_y = (y * CHUNK_SIZE as isize).rem_euclid(512);
        let chunk_index = (((((y >> SCALE) - 256) & 511) << 9) | (((x >> SCALE) - 256) & 511)) * 4;
        // Deref 1/3
        let chunk_arr = unsafe { self.chunk_map_ptr.cast::<*const c_void>().read() };
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
    fn get_cell_raw(&self, x: isize, y: isize) -> Option<&ntypes::Cell> {
        let x = x + self.shift_x;
        let y = y + self.shift_y;
        let pixel = unsafe { self.pixel_array.offset((((y & 511) << 9) | (x & 511)) * 4) };
        if pixel.is_null() {
            return None;
        }

        unsafe { pixel.cast::<*const ntypes::Cell>().read().as_ref() }
    }
    fn get_cell_raw_mut(&mut self, x: isize, y: isize) -> *mut *mut ntypes::Cell {
        let x = x + self.shift_x;
        let y = y + self.shift_y;
        let pixel = unsafe { self.pixel_array.offset((((y & 511) << 9) | (x & 511)) * 4) };
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

    pub(crate) unsafe fn encode_area(&mut self, x: isize, y: isize, chunk: &mut Chunk) -> bool {
        if self.set_chunk(x, y) {
            return false;
        }
        for (k, (i, j)) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .enumerate()
        {
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
        true
    }
    pub(crate) unsafe fn decode_area(&mut self, x: isize, y: isize, chunk: &Chunk) {
        if self.set_chunk(x, y) {
            return;
        }
        let x = x * CHUNK_SIZE as isize;
        let y = y * CHUNK_SIZE as isize;
        for (k, (i, j)) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .enumerate()
        {
            match chunk[k] {
                CellType::Blob => {
                    let x = x + i;
                    let y = y + j;
                    unsafe {
                        let cell = self.get_cell_raw_mut(i, j);
                        if !(*cell).is_null() {
                            self.remove_cell(*cell, x, y);
                            *cell = ptr::null_mut();
                        }
                        let src = self.create_cell(x, y, self.blob_ptr);
                        if !src.is_null() {
                            let liquid: &mut ntypes::LiquidCell =
                                &mut *(src as *mut ntypes::LiquidCell);
                            liquid.is_static = true;
                            *cell = src;
                        }
                    }
                }
                CellType::Remove => {
                    let x = x + i;
                    let y = y + j;
                    unsafe {
                        let cell = self.get_cell_raw_mut(i, j);
                        if !(*cell).is_null() {
                            self.remove_cell(*cell, x, y);
                            *cell = ptr::null_mut();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
