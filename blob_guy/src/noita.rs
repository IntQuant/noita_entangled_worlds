use crate::CHUNK_SIZE;
use crate::chunk::{CellType, Chunk};
use eyre::eyre;
#[cfg(target_arch = "x86")]
use std::arch::asm;
use std::ffi::c_void;
use std::{mem, ptr};
pub(crate) mod ntypes;
//pub(crate) mod pixel;
#[derive(Debug)]
pub(crate) struct ParticleWorldState<'a> {
    pub(crate) world_ptr: *const ntypes::GridWorld,
    pub(crate) chunk_arr: ntypes::ChunkArray,
    pub(crate) material_list: &'a [ntypes::CellData],
    pub(crate) blob_guy: u16,
    pub(crate) pixel_array: &'a mut [ntypes::CellPtr],
    pub(crate) construct_ptr: *const c_void,
    pub(crate) remove_ptr: *const c_void,
    pub(crate) shift_x: isize,
    pub(crate) shift_y: isize,
}
impl<'a> ParticleWorldState<'a> {
    pub fn set_chunk(&mut self, x: isize, y: isize) -> eyre::Result<()> {
        const SCALE: isize = (512 / CHUNK_SIZE as isize).ilog2() as isize;
        self.shift_x = (x * CHUNK_SIZE as isize).rem_euclid(512);
        self.shift_y = (y * CHUNK_SIZE as isize).rem_euclid(512);
        let chunk_index = ((((y >> SCALE) - 256) & 511) << 9) | (((x >> SCALE) - 256) & 511);
        let array = self.chunk_arr.index(chunk_index);
        if array.0.is_null() {
            return Err(eyre!(format!("cant find chunk index {}", chunk_index)));
        }
        self.pixel_array = unsafe { std::slice::from_raw_parts_mut(array.0, 512 * 512) };
        Ok(())
    }
    pub fn get_cell_raw(&self, x: isize, y: isize) -> Option<&ntypes::Cell> {
        let x = x + self.shift_x;
        let y = y + self.shift_y;
        let index = ((y & 511) << 9) | (x & 511);
        let pixel = &self.pixel_array[index as usize];
        if pixel.0.is_null() {
            return None;
        }
        unsafe { pixel.0.as_ref() }
    }
    fn get_cell_material_id(&self, cell: &ntypes::Cell) -> u16 {
        let mat_ptr = cell.material_ptr();
        let offset = unsafe { mat_ptr.0.offset_from(self.material_list.as_ptr()) };
        offset as u16
    }

    fn get_cell_type(&self, cell: &ntypes::Cell) -> Option<ntypes::CellType> {
        Some(unsafe { cell.material_ptr().0.as_ref()? }.cell_type)
    }

    pub(crate) unsafe fn encode_area(
        &mut self,
        x: isize,
        y: isize,
        chunk: &mut Chunk,
    ) -> eyre::Result<()> {
        self.set_chunk(x, y)?;
        let mut modified = false;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter_mut())
        {
            *pixel = if let Some(cell) = self.get_cell_raw(i, j)
                && let Some(cell_type) = self.get_cell_type(cell)
            {
                match cell_type {
                    ntypes::CellType::Liquid => {
                        if self.get_cell_material_id(cell) == self.blob_guy {
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
    pub(crate) unsafe fn decode_area(
        &mut self,
        x: isize,
        y: isize,
        chunk: &Chunk,
    ) -> eyre::Result<()> {
        self.set_chunk(x, y)?;
        let x = x * CHUNK_SIZE as isize;
        let y = y * CHUNK_SIZE as isize;
        for ((i, j), pixel) in (0..CHUNK_SIZE as isize)
            .flat_map(|i| (0..CHUNK_SIZE as isize).map(move |j| (i, j)))
            .zip(chunk.iter())
        {
            macro_rules! get_cell_raw_mut {
                ($x:tt,$y:tt) => {{
                    let x = x + self.shift_x;
                    let y = y + self.shift_y;
                    let index = ((y & 511) << 9) | (x & 511);
                    &mut self.pixel_array[index as usize]
                }};
            }
            match pixel {
                CellType::Blob => {
                    let x = x + i;
                    let y = y + j;
                    unsafe {
                        let cell = get_cell_raw_mut!(i, j);
                        if !cell.0.is_null() {
                            remove_cell(self.world_ptr, self.remove_ptr, cell.0, x, y);
                            cell.0 = ptr::null_mut();
                        }
                        let src = create_cell(
                            self.world_ptr,
                            self.construct_ptr,
                            x,
                            y,
                            &self.material_list[self.blob_guy as usize],
                        );
                        if !src.is_null() {
                            let liquid: &mut ntypes::LiquidCell =
                                &mut *src.cast::<ntypes::LiquidCell>();
                            liquid.is_static = true;
                            cell.0 = src;
                        }
                    }
                }
                CellType::Remove => {
                    let x = x + i;
                    let y = y + j;
                    let cell = get_cell_raw_mut!(i, j);
                    if !cell.0.is_null() {
                        remove_cell(self.world_ptr, self.remove_ptr, cell.0, x, y);
                        cell.0 = ptr::null_mut();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
fn create_cell(
    world_ptr: *const ntypes::GridWorld,
    construct_ptr: *const c_void,
    x: isize,
    y: isize,
    material: &ntypes::CellData,
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
            world = in(reg) world_ptr,
            x = in(reg) x,
            y = in(reg) y,
            material = in(reg) material as *const ntypes::CellData,
            construct = in(reg) construct_ptr,
            clobber_abi("C"),
            out("eax") cell_ptr,
        );
        cell_ptr
    }
    #[cfg(target_arch = "x86_64")]
    {
        std::hint::black_box((x, y, material, world_ptr, construct_ptr));
        unreachable!()
    }
}
fn remove_cell(
    world_ptr: *const ntypes::GridWorld,
    remove_ptr: *const c_void,
    cell: *const ntypes::Cell,
    x: isize,
    y: isize,
) {
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!(
            "mov ecx, {world}",
            "push 0",
            "push {y:e}",
            "push {x:e}",
            "push {cell}",
            "call {remove}",
            world = in(reg) world_ptr,
            cell = in(reg) cell,
            x = in(reg) x,
            y = in(reg) y,
            remove = in(reg) remove_ptr,
            clobber_abi("C"),
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        std::hint::black_box((x, y, cell, world_ptr, remove_ptr));
        unreachable!()
    }
}
