use std::{ffi::c_void, ptr::null};

pub(crate) struct ParticleWorldState {
    world_pointer: *mut c_void,
    chunk_map_this: *mut c_void,
}

impl ParticleWorldState {
    pub(crate) unsafe fn get_cell(&self, x: i32, y: i32) -> *const c_void {
        let x = dbg!(x as isize);
        let y = dbg!(y as isize);
        let chunk_index = (((((y) >> 9) - 256) & 511) * 512 + ((((x) >> 9) - 256) & 511)) * 4;
        let chunk_arr = self.chunk_map_this.offset(8).cast::<*const c_void>().read();
        // dbg!(chunk_arr);
        let chunk = chunk_arr.offset(chunk_index).cast::<*const c_void>().read();
        dbg!(chunk);
        if chunk.is_null() {
            return null();
        }
        let pixel = chunk.offset(((y & 511) << 9 | x & 511) * 4);
        pixel
    }

    pub(crate) fn new(world_pointer: *mut c_void, chunk_map_pointer: *mut c_void) -> Self {
        Self {
            world_pointer,
            chunk_map_this: chunk_map_pointer,
        }
    }
}
