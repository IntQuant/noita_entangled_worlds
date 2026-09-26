use std::{ffi::c_void, mem};

pub(crate) mod decode;
pub(crate) mod ntypes;
pub(crate) mod pixel;

pub(crate) struct ParticleWorldState {
    pub(crate) _world_ptr: *mut c_void,
    pub(crate) chunk_map_ptr: *mut c_void,
    pub(crate) material_list_ptr: *const c_void,

    pub(crate) runner: pixel::PixelRunner<pixel::RawPixel>,
}

/// Index of the chunk holding (x, y) in the ChunkMap's chunk table, which has
/// 512x512 entries.
pub(crate) fn chunk_index(x: i32, y: i32) -> isize {
    let (x, y) = (x as isize, y as isize);
    (((y >> 9) - 256) & 511) * 512 + (((x >> 9) - 256) & 511)
}

/// Index of (x, y) within its chunk's 512x512 array of cell pointers.
pub(crate) fn cell_index(x: i32, y: i32) -> isize {
    let (x, y) = (x as isize, y as isize);
    ((y & 511) << 9) | (x & 511)
}

/// The array of cell pointers for the chunk holding (x, y), or null if that
/// chunk is not loaded.
///
/// This is the game's own lookup: its chunk_loaded is `chunk != null` and its
/// get_cell is `&(*chunk)[cell_index]`, with the chunk table read from
/// ChunkMap+8 (see `decode::GET_CELL_CODE`).
///
/// # Safety
///
/// `chunk_map` must be the live game ChunkMap.
pub(crate) unsafe fn chunk_cells(
    chunk_map: *const c_void,
    x: i32,
    y: i32,
) -> *mut *mut ntypes::Cell {
    // Deref 1/3: the chunk table.
    let table = unsafe {
        chunk_map
            .byte_add(8)
            .cast::<*const *mut *mut *mut ntypes::Cell>()
            .read()
    };
    // Deref 2/3: the chunk, null when not loaded.
    let chunk = unsafe { table.offset(chunk_index(x, y)).read() };
    if chunk.is_null() {
        return std::ptr::null_mut();
    }
    // Deref 3/3: a loaded chunk starts with its cell array.
    unsafe { chunk.read() }
}

/// The slot holding the cell at (x, y), given its chunk's cell array.
///
/// # Safety
///
/// `cells` must be the non-null result of `chunk_cells` for the same (x, y).
pub(crate) unsafe fn cell_slot(
    cells: *mut *mut ntypes::Cell,
    x: i32,
    y: i32,
) -> *mut *mut ntypes::Cell {
    unsafe { cells.offset(cell_index(x, y)) }
}

impl ParticleWorldState {
    fn get_cell_raw(&self, x: i32, y: i32) -> Option<&ntypes::Cell> {
        let cells = unsafe { chunk_cells(self.chunk_map_ptr, x, y) };
        if cells.is_null() {
            return None;
        }
        unsafe { cell_slot(cells, x, y).read().as_ref() }
    }

    fn get_cell_material_id(&self, cell: &ntypes::Cell) -> u16 {
        let mat_ptr = cell.material_ptr();
        let offset = unsafe { mat_ptr.cast::<c_void>().offset_from(self.material_list_ptr) };
        (offset / ntypes::CELLDATA_SIZE) as u16
    }

    fn get_cell_type(&self, cell: &ntypes::Cell) -> Option<ntypes::CellType> {
        unsafe { Some(cell.material_ptr().as_ref()?.cell_type) }
    }

    pub(crate) unsafe fn encode_area(
        &mut self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        mut pixel_runs: *mut pixel::NoitaPixelRun,
    ) -> usize {
        // Allow compiler to generate better code.
        assert_eq!(start_x % 128, 0);
        assert_eq!(start_y % 128, 0);
        assert!((end_x - start_x) <= 128);
        assert!((end_y - start_y) <= 128);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let mut raw_pixel = pixel::RawPixel {
                    material: 0,
                    flags: 0,
                };
                let cell = self.get_cell_raw(x, y);
                if let Some(cell) = cell {
                    let cell_type = self.get_cell_type(cell).unwrap_or(ntypes::CellType::None);
                    match cell_type {
                        ntypes::CellType::None => {}
                        // Nobody knows how box2d pixels work.
                        ntypes::CellType::Solid => {}
                        ntypes::CellType::Liquid => {
                            raw_pixel.material = self.get_cell_material_id(cell);
                            let cell: &ntypes::LiquidCell = unsafe { mem::transmute(cell) };
                            raw_pixel.flags = cell.is_static as u8;
                        }
                        ntypes::CellType::Gas | ntypes::CellType::Fire => {
                            raw_pixel.material = self.get_cell_material_id(cell);
                        }
                        // ???
                        _ => {}
                    }
                }
                self.runner.put_pixel(raw_pixel);
            }
        }

        let built_runner = self.runner.build();
        let runs = built_runner.len();
        for run in built_runner {
            let noita_pixel_run = unsafe { pixel_runs.as_mut().unwrap() };
            noita_pixel_run.length = (run.length - 1) as u16;
            noita_pixel_run.material = run.data.material;
            noita_pixel_run.flags = run.data.flags;
            pixel_runs = unsafe { pixel_runs.offset(1) };
        }
        self.runner.clear();
        runs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The expressions `get_cell_raw` used before they were factored out.
    fn old_chunk_index(x: i32, y: i32) -> isize {
        let (x, y) = (x as isize, y as isize);
        ((((y) >> 9) - 256) & 511) * 512 + ((((x) >> 9) - 256) & 511)
    }
    fn old_cell_index(x: i32, y: i32) -> isize {
        let (x, y) = (x as isize, y as isize);
        ((y & 511) << 9) | x & 511
    }

    #[test]
    fn indices_match_the_original_arithmetic() {
        for y in (-70_000..70_000).step_by(997) {
            for x in (-70_000..70_000).step_by(1009) {
                assert_eq!(chunk_index(x, y), old_chunk_index(x, y));
                assert_eq!(cell_index(x, y), old_cell_index(x, y));
                assert!((0..512 * 512).contains(&chunk_index(x, y)));
                assert!((0..512 * 512).contains(&cell_index(x, y)));
            }
        }
    }

    /// A ChunkMap laid out like the game's: two ints, then the chunk table.
    #[repr(C)]
    struct FakeChunkMap {
        _unknown: [u32; 2],
        table: *const *mut *mut *mut ntypes::Cell,
    }

    #[test]
    fn chunk_walk_finds_the_right_slot() {
        let mut cells: Vec<*mut ntypes::Cell> = vec![std::ptr::null_mut(); 512 * 512];
        let cells_ptr = cells.as_mut_ptr();
        // A loaded chunk starts with a pointer to its cell array.
        let mut chunk: Box<*mut *mut ntypes::Cell> = Box::new(cells_ptr);
        let mut table: Vec<*mut *mut *mut ntypes::Cell> = vec![std::ptr::null_mut(); 512 * 512];
        let (x, y) = (-700, 1300);
        table[chunk_index(x, y) as usize] = &mut *chunk;
        let map = FakeChunkMap {
            _unknown: [0; 2],
            table: table.as_ptr(),
        };
        let map_ptr = (&map as *const FakeChunkMap).cast::<c_void>();

        unsafe {
            let found = chunk_cells(map_ptr, x, y);
            assert_eq!(found, cells_ptr);
            // Anywhere else in the same 512x512 chunk shares the array.
            assert_eq!(chunk_cells(map_ptr, x - 211, y + 150), cells_ptr);
            assert_eq!(cell_slot(found, x, y), cells_ptr.offset(cell_index(x, y)));
            // Neighbouring chunks are not loaded.
            assert!(chunk_cells(map_ptr, x + 512, y).is_null());
            assert!(chunk_cells(map_ptr, x, y - 512).is_null());
        }
    }
}
