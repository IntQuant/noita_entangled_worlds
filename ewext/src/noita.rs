use std::ffi::c_void;

pub(crate) struct ChunkMap {
    this: *mut c_void,
}

impl ChunkMap {
    unsafe fn get_cell(&self, x: u32, y: u32) {
        let x = x as isize;
        let y = y as isize;
        let index = ((((y) >> 9) - 256 & 511) * 512 + (((x) >> 9) - 256 & 511)) * 4;
        let chunk_arr = self.this.offset(8).cast::<*const c_void>().read();
        let chunk = chunk_arr.offset(index).cast::<*const c_void>().read();
        let pixel = chunk.offset(((y & 511) << 9 | x & 511) * 4);
    }
}
