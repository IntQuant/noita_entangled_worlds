use crate::CHUNK_SIZE;
use crate::chunk::{CellType, Chunk};
#[cfg(target_arch = "x86")]
use std::arch::asm;
use std::{ffi::c_void, mem, ptr};
pub(crate) mod ntypes;
//pub(crate) mod pixel;
#[derive(Default)]
pub(crate) struct ParticleWorldState {
    pub(crate) world_ptr: *const ntypes::GridWorld,
    pub(crate) chunk_map: &'static [*const &'static mut [*const ntypes::Cell; 512 * 512]],
    //pub(crate) material_list_ptr: *const ntypes::CellData,
    pub(crate) blob_ptr: *const ntypes::CellData,
    pub(crate) construct_ptr: *const c_void,
    pub(crate) remove_ptr: *const c_void,
}
unsafe impl Sync for ParticleWorldState {}
unsafe impl Send for ParticleWorldState {}
impl ParticleWorldState {
    fn create_cell(
        &self,
        x: isize,
        y: isize,
        material: *const ntypes::CellData,
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
            std::hint::black_box((x, y, material, self.world_ptr, self.construct_ptr));
            unreachable!()
        }
    }
    fn remove_cell(&self, cell: *const ntypes::Cell, x: isize, y: isize) {
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
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((x, y, cell, self.world_ptr, self.remove_ptr));
            unreachable!()
        }
    }
    #[allow(clippy::mut_from_ref)]
    pub fn set_chunk(
        &self,
        x: isize,
        y: isize,
    ) -> Result<(isize, isize, &mut [*const ntypes::Cell; 512 * 512]), ()> {
        const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
        let shift_x = (x * CHUNK_SIZE as isize).rem_euclid(512);
        let shift_y = (y * CHUNK_SIZE as isize).rem_euclid(512);
        let chunk_index = ((((y >> SCALE) - 256) & 511) << 9) | (((x >> SCALE) - 256) & 511);
        let chunk = self.chunk_map[chunk_index as usize];
        if chunk.is_null() {
            return Err(());
        }
        let pixel_array = unsafe { chunk.read() };
        Ok((shift_x, shift_y, pixel_array))
    }
    pub fn get_cell_raw(
        &self,
        x: isize,
        y: isize,
        pixel_array: &mut [*const ntypes::Cell; 512 * 512],
    ) -> Option<&ntypes::Cell> {
        let index = (y << 9) | x;
        let pixel = pixel_array[index as usize];
        if pixel.is_null() {
            return None;
        }

        unsafe { pixel.as_ref() }
    }
    fn get_cell_raw_mut<'a>(
        &self,
        x: isize,
        y: isize,
        pixel_array: &'a mut [*const ntypes::Cell; 512 * 512],
    ) -> &'a mut *const ntypes::Cell {
        let index = (y << 9) | x;
        &mut pixel_array[index as usize]
    }
    /*fn get_cell_material_id(&self, cell: &ntypes::Cell) -> u16 {
        let mat_ptr = cell.material_ptr();
        let offset = unsafe { mat_ptr.offset_from(self.material_list_ptr) };
        offset as u16
    }*/

    fn get_cell_type(&self, cell: &ntypes::Cell) -> Option<ntypes::CellType> {
        unsafe { Some(cell.material_ptr().as_ref()?.cell_type) }
    }

    pub(crate) unsafe fn encode_area(
        &self,
        x: isize,
        y: isize,
        chunk: &mut Chunk,
    ) -> Result<(), ()> {
        let (shift_x, shift_y, pixel_array) = self.set_chunk(x, y)?;
        let mut modified = false;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter_mut())
        {
            *pixel = if let Some(cell) = self.get_cell_raw(shift_x + i, shift_y + j, pixel_array)
                && let Some(cell_type) = self.get_cell_type(cell)
            {
                match cell_type {
                    ntypes::CellType::Liquid => {
                        if cell.material_ptr == self.blob_ptr {
                            modified = true;
                            CellType::Remove
                        } else {
                            let cell: &ntypes::LiquidCell = unsafe { mem::transmute(cell) };
                            if cell.is_static {
                                CellType::Solid
                            } else {
                                CellType::Liquid
                            }
                        }
                    }
                    ntypes::CellType::Solid => CellType::Physics,
                    ntypes::CellType::Fire | ntypes::CellType::Gas => CellType::Other,
                    _ => CellType::Unknown,
                }
            } else {
                CellType::Unknown
            }
        }
        chunk.modified = modified;
        Ok(())
    }
    pub(crate) unsafe fn decode_area(&self, x: isize, y: isize, chunk: &Chunk) -> Result<(), ()> {
        if !chunk.modified {
            return Ok(());
        }
        let (shift_x, shift_y, pixel_array) = self.set_chunk(x, y)?;
        let x = x * CHUNK_SIZE as isize;
        let y = y * CHUNK_SIZE as isize;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter())
        {
            match pixel {
                CellType::Blob => {
                    let world_x = x + i;
                    let world_y = y + j;
                    let cell = self.get_cell_raw_mut(shift_x + i, shift_y + j, pixel_array);
                    if !(*cell).is_null() {
                        self.remove_cell(*cell, world_x, world_y);
                        *cell = ptr::null_mut();
                    }
                    let src = self.create_cell(world_x, world_y, self.blob_ptr);
                    if !src.is_null() {
                        if let Some(liquid) = unsafe { src.cast::<ntypes::LiquidCell>().as_mut() } {
                            liquid.is_static = true;
                        }
                        *cell = src;
                    }
                }
                CellType::Remove => {
                    let world_x = x + i;
                    let world_y = y + j;
                    std::thread::sleep(std::time::Duration::from_nanos(0));
                    let cell = self.get_cell_raw_mut(shift_x + i, shift_y + j, pixel_array);
                    if !(*cell).is_null() {
                        self.remove_cell(*cell, world_x, world_y);
                        *cell = ptr::null_mut();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
