use crate::noita::types;
use eyre::ContextCompat;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
#[cfg(target_arch = "x86")]
use std::arch::asm;
use std::ffi::c_void;
#[derive(Debug)]
pub struct ParticleWorldState {
    pub world_ptr: &'static mut types::GridWorld,
    pub material_list: &'static [types::CellData],
    pub construct_ptr: *const c_void,
    pub remove_ptr: *const c_void,
}
unsafe impl Sync for ParticleWorldState {}
unsafe impl Send for ParticleWorldState {}
#[allow(clippy::result_unit_err)]
impl ParticleWorldState {
    pub fn create_cell(
        &mut self,
        x: isize,
        y: isize,
        material: u16,
        //_memory: *mut c_void,
    ) -> *mut types::Cell {
        #[cfg(target_arch = "x86")]
        unsafe {
            let cell_ptr: *mut types::Cell;
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
                material = in(reg) &self.material_list[material as usize],
                construct = in(reg) self.construct_ptr,
                clobber_abi("C"),
                out("eax") cell_ptr,
            );
            cell_ptr
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((x, y, material, self.construct_ptr));
            unreachable!()
        }
    }
    pub fn remove_cell(&mut self, cell: *mut types::Cell, x: isize, y: isize) {
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
            std::hint::black_box((x, y, cell, self.remove_ptr));
            unreachable!()
        }
    }
    #[allow(clippy::mut_from_ref)]
    pub fn set_chunk<const CHUNK_SIZE: usize, const SCALE: isize>(
        &self,
        x: isize,
        y: isize,
    ) -> Result<(isize, isize, *mut &'static mut [types::CellPtr; 512 * 512]), ()> {
        let shift_x = (x * CHUNK_SIZE as isize).rem_euclid(512);
        let shift_y = (y * CHUNK_SIZE as isize).rem_euclid(512);
        let chunk_index = ((((y >> SCALE) - 256) & 511) << 9) | (((x >> SCALE) - 256) & 511);
        let chunk = self.world_ptr.chunk_map.cell_array[chunk_index as usize].0;
        if chunk.is_null() {
            return Err(());
        }
        Ok((shift_x, shift_y, chunk))
    }
    pub fn get_cell_raw(
        &self,
        x: isize,
        y: isize,
        pixel_array: &&mut [types::CellPtr; 512 * 512],
    ) -> Option<&types::Cell> {
        let index = (y << 9) | x;
        let pixel = pixel_array[index as usize].0;
        if pixel.is_null() {
            return None;
        }

        unsafe { pixel.as_ref() }
    }
    pub fn get_cell_raw_mut<'a>(
        &self,
        x: isize,
        y: isize,
        pixel_array: &'a mut &'static mut [types::CellPtr; 512 * 512],
    ) -> &'a mut *mut types::Cell {
        let index = (y << 9) | x;
        &mut pixel_array[index as usize].0
    }
    pub fn get_cell_material_id(&self, cell: &mut types::Cell) -> u16 {
        let offset = unsafe {
            (cell.material as *const types::CellData).offset_from(self.material_list.as_ptr())
        };
        offset as u16
    }
    ///# Safety
    #[allow(clippy::type_complexity)]
    pub unsafe fn clone_chunks(&mut self) -> Vec<((isize, isize), Vec<types::FullCell>)> {
        self.world_ptr
            .chunk_map
            .cell_array
            .into_par_iter()
            .enumerate()
            .filter_map(|(i, c)| {
                unsafe { c.0.as_ref() }.map(|c| {
                    let x = i as isize % 512 - 256;
                    let y = i as isize / 512 - 256;
                    (
                        (x, y),
                        c.iter()
                            .map(|p| {
                                unsafe { p.0.as_ref() }
                                    .map(types::FullCell::from)
                                    .unwrap_or_default()
                            })
                            .collect(),
                    )
                })
            })
            .collect::<Vec<((isize, isize), Vec<types::FullCell>)>>()
    }
    ///# Safety
    pub unsafe fn debug_mouse_pos(&self) -> eyre::Result<()> {
        let (x, y) = crate::raw::debug_get_mouse_world()?;
        let (x, y) = (x.floor(), y.floor());
        let (x, y) = (x as isize, y as isize);
        if let Ok((_, _, pixel_array)) =
            self.set_chunk::<512, 0>(x.div_euclid(512), y.div_euclid(512))
        {
            if let Some(cell) = self.get_cell_raw(
                x.rem_euclid(512),
                y.rem_euclid(512),
                unsafe { pixel_array.as_ref() }.unwrap(),
            ) {
                let full = types::FullCell::from(cell);
                crate::print!("{full:?}");
            } else {
                crate::print!("mat nil");
            }
        }
        Ok(())
    }
    pub fn new() -> eyre::Result<Self> {
        let (construct_ptr, remove_ptr, global_ptr) = crate::noita::init_data::get_functions()?;
        let global = unsafe { global_ptr.as_mut() }.wrap_err("no global?")?;
        let material_list_ptr = global.m_cell_factory.cell_data_ptr;
        let material_list = unsafe {
            std::slice::from_raw_parts(material_list_ptr, global.m_cell_factory.cell_data_len)
        };
        Ok(ParticleWorldState {
            world_ptr: global.m_grid_world,
            material_list,
            construct_ptr,
            remove_ptr,
        })
    }
}
