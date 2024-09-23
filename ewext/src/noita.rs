use std::{ffi::c_void, ptr::null};

pub(crate) struct ParticleWorldState {
    world_pointer: *mut c_void,
    chunk_map_this: *mut c_void,
}

impl ParticleWorldState {
    pub(crate) unsafe fn get_cell(&self, x: i32, y: i32) -> *const c_void {
        let x = x as isize;
        let y = y as isize;
        let chunk_index = (((((y) >> 9) - 256) & 511) * 512 + ((((x) >> 9) - 256) & 511)) * 4;
        // Deref 1/3
        let chunk_arr = self.chunk_map_this.offset(8).cast::<*const c_void>().read();
        // Deref 2/3
        let chunk = chunk_arr.offset(chunk_index).cast::<*const c_void>().read();
        if chunk.is_null() {
            // TODO this normally returns air material
            return null();
        }
        // Deref 3/3
        let pixel_array = chunk.cast::<*const c_void>().read();
        let pixel = pixel_array.offset(((y & 511) << 9 | x & 511) * 4);
        pixel
    }

    pub(crate) fn new(world_pointer: *mut c_void, chunk_map_pointer: *mut c_void) -> Self {
        Self {
            world_pointer,
            chunk_map_this: chunk_map_pointer,
        }
    }
}
