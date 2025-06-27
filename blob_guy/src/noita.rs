use crate::CHUNK_SIZE;
use crate::chunk::Chunk;
use std::{ffi::c_void, mem};
pub(crate) mod ntypes;
//pub(crate) mod pixel;

pub(crate) struct ParticleWorldState {
    pub(crate) _world_ptr: *mut c_void,
    pub(crate) chunk_map_ptr: *mut c_void,
    pub(crate) material_list_ptr: *const c_void,
    pub(crate) blob_guy: u16,
}

impl ParticleWorldState {
    fn get_cell_raw(&self, x: i32, y: i32) -> Option<&ntypes::Cell> {
        let x = x as isize;
        let y = y as isize;
        let chunk_index = (((((y) >> 9) - 256) & 511) * 512 + ((((x) >> 9) - 256) & 511)) * 4;
        // Deref 1/3
        let chunk_arr = unsafe { self.chunk_map_ptr.offset(8).cast::<*const c_void>().read() };
        // Deref 2/3
        let chunk = unsafe { chunk_arr.offset(chunk_index).cast::<*const c_void>().read() };
        if chunk.is_null() {
            return None;
        }
        // Deref 3/3
        let pixel_array = unsafe { chunk.cast::<*const c_void>().read() };
        let pixel = unsafe { pixel_array.offset((((y & 511) << 9) | x & 511) * 4) };
        if pixel.is_null() {
            return None;
        }

        unsafe { pixel.cast::<*const ntypes::Cell>().read().as_ref() }
    }

    fn get_cell_material_id(&self, cell: &ntypes::Cell) -> u16 {
        let mat_ptr = cell.material_ptr();
        let offset = unsafe { mat_ptr.cast::<c_void>().offset_from(self.material_list_ptr) };
        (offset / ntypes::CELLDATA_SIZE) as u16
    }

    fn get_cell_type(&self, cell: &ntypes::Cell) -> Option<ntypes::CellType> {
        unsafe { Some(cell.material_ptr().as_ref()?.cell_type) }
    }

    pub(crate) unsafe fn encode_area(&mut self, x: i32, y: i32) -> Chunk {
        unsafe {
            std::hint::assert_unchecked(x % 128 == 0);
            std::hint::assert_unchecked(y % 128 == 0);
        }

        let mut is_blob = [false; CHUNK_SIZE * CHUNK_SIZE];
        let mut is_solid = [false; CHUNK_SIZE * CHUNK_SIZE];
        let mut is_liquid = [false; CHUNK_SIZE * CHUNK_SIZE];
        let mut pixels = [0; CHUNK_SIZE * CHUNK_SIZE];
        for i in 0..CHUNK_SIZE {
            let y = y + i as i32;
            for j in 0..CHUNK_SIZE {
                let k = i * CHUNK_SIZE + j;
                let x = x + j as i32;
                let cell = self.get_cell_raw(x, y);
                if let Some(cell) = cell {
                    let cell_type = self.get_cell_type(cell).unwrap_or(ntypes::CellType::None);
                    match cell_type {
                        ntypes::CellType::None => {}
                        // Nobody knows how box2d pixels work.
                        ntypes::CellType::Solid => {}
                        ntypes::CellType::Liquid => {
                            pixels[k] = self.get_cell_material_id(cell);
                            if pixels[k] == self.blob_guy {
                                is_blob[k] = true
                            }
                            let cell: &ntypes::LiquidCell = unsafe { mem::transmute(cell) };
                            if cell.is_static {
                                is_solid[k] = true;
                            } else {
                                is_liquid[k] = true;
                            }
                        }
                        ntypes::CellType::Gas | ntypes::CellType::Fire => {}
                        // ???
                        _ => {}
                    }
                }
            }
        }
        Chunk {
            pixels,
            is_blob,
            is_liquid,
            is_solid,
        }
    }
}
