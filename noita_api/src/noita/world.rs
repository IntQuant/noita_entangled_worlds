use crate::noita::types;
use eyre::ContextCompat;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
#[derive(Debug)]
pub struct ParticleWorldState {
    pub world_ptr: &'static mut types::GridWorld,
    pub material_list: &'static [types::CellData],
    pub construct_ptr: types::ConstructPtr,
    pub remove_ptr: types::RemovePtr,
}
unsafe impl Sync for ParticleWorldState {}
unsafe impl Send for ParticleWorldState {}
impl ParticleWorldState {
    pub fn get_shift<const CHUNK_SIZE: usize>(&self, x: isize, y: isize) -> (isize, isize) {
        let shift_x = (x * CHUNK_SIZE as isize).rem_euclid(512);
        let shift_y = (y * CHUNK_SIZE as isize).rem_euclid(512);
        (shift_x, shift_y)
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
            .chunk_array
            .slice()
            .par_iter()
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
    pub unsafe fn debug_mouse_pos(&mut self) -> eyre::Result<()> {
        let (x, y) = crate::raw::debug_get_mouse_world()?;
        let (x, y) = (x.floor(), y.floor());
        let (x, y) = (x as isize, y as isize);
        if let Some(pixel_array) = self
            .world_ptr
            .chunk_map
            .chunk_array
            .get_mut(x.div_euclid(512), y.div_euclid(512))
        {
            if let Some(cell) = pixel_array.get(x.rem_euclid(512), y.rem_euclid(512)) {
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
        let cell_factory =
            unsafe { global.m_cell_factory.as_mut() }.wrap_err("no cell factory?")?;
        let material_list_ptr = cell_factory.cell_data_ptr;
        let material_list =
            unsafe { std::slice::from_raw_parts(material_list_ptr, cell_factory.cell_data_len) };
        Ok(ParticleWorldState {
            world_ptr: unsafe { global.m_grid_world.as_mut() }.wrap_err("no grid world")?,
            material_list,
            construct_ptr,
            remove_ptr,
        })
    }
}
