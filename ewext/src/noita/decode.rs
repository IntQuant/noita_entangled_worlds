//! Native world decode.
//!
//! The inbound half of world sync used to run in Lua (`quant.ew/files/system/
//! world_sync/world.lua:world.decode`): a per-pixel loop making two or more FFI
//! calls per pixel, over a 128x128 chunk, with no per-frame budget. That is
//! ~16k iterations and ~50-100k FFI transitions for a single chunk, executed
//! synchronously inside `OnWorldPreUpdate`.
//!
//! This is a direct transliteration of that loop. It deliberately calls the
//! same game functions the Lua path used - resolved by NoitaPatcher and handed
//! to us as raw pointers at init - rather than reimplementing cell construction
//! against hand-reversed struct layouts. That keeps the semantics identical to
//! the proven path and avoids taking on a second copy of Noita's memory layout.
//!
//! The Lua implementation remains in the tree and is still the fallback; see
//! `quant.ew.rust_world_decode`.

// The DLL only ever loads into 32-bit Noita. Host builds exist so `cargo check`
// and clippy can run on Linux, and there `decode_area` is a stub, which leaves
// its helpers unused.
#![cfg_attr(not(target_arch = "x86"), allow(dead_code))]

use std::ffi::c_void;

use super::ParticleWorldState;
use super::ntypes::{self, CELLDATA_SIZE};
use super::pixel::NoitaPixelRun;

/// Mirrors `struct EncodedAreaHeader` from `world.lua`'s cdef. Packed, 12 bytes.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(crate) struct EncodedAreaHeader {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) pixel_run_count: u16,
}

const _: () = assert!(size_of::<EncodedAreaHeader>() == 12);
const _: () = assert!(size_of::<NoitaPixelRun>() == 5);

/// `LIQUID_FLAG_STATIC` from `world.lua`'s cdef.
const LIQUID_FLAG_STATIC: u8 = 1;

/// Game functions needed to write cells, resolved by NoitaPatcher on the Lua
/// side (`noitapatcher.nsew.world_ffi`) and passed to us as raw addresses.
///
/// These are `__thiscall`, so the first argument goes in `ecx`. Rust only has
/// that ABI on 32-bit x86, which is the only architecture this DLL is ever
/// loaded into; the whole call surface is cfg'd out elsewhere.
#[cfg(target_arch = "x86")]
mod abi {
    use super::*;

    pub(super) type GetCellFn =
        unsafe extern "thiscall" fn(*mut c_void, i32, i32) -> *mut *mut ntypes::Cell;
    pub(super) type ChunkLoadedFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32) -> bool;
    pub(super) type RemoveCellFn =
        unsafe extern "thiscall" fn(*mut c_void, *mut c_void, i32, i32, bool);
    pub(super) type ConstructCellFn = unsafe extern "thiscall" fn(
        *mut c_void,
        i32,
        i32,
        *const ntypes::CellData,
        *mut c_void,
    ) -> *mut ntypes::Cell;
}

#[cfg(not(target_arch = "x86"))]
mod abi {
    use super::*;

    // Placeholders so the crate still type-checks on the host for `cargo check`
    // and clippy. `decode_area` refuses to run on any non-x86 target, so these
    // are never called.
    pub(super) type GetCellFn =
        unsafe extern "C" fn(*mut c_void, i32, i32) -> *mut *mut ntypes::Cell;
    pub(super) type ChunkLoadedFn = unsafe extern "C" fn(*mut c_void, i32, i32) -> bool;
    pub(super) type RemoveCellFn = unsafe extern "C" fn(*mut c_void, *mut c_void, i32, i32, bool);
    pub(super) type ConstructCellFn = unsafe extern "C" fn(
        *mut c_void,
        i32,
        i32,
        *const ntypes::CellData,
        *mut c_void,
    ) -> *mut ntypes::Cell;
}

#[derive(Clone, Copy)]
pub(crate) struct WorldFns {
    get_cell: abi::GetCellFn,
    chunk_loaded: abi::ChunkLoadedFn,
    remove_cell: abi::RemoveCellFn,
    construct_cell: abi::ConstructCellFn,
    /// Highest material id the game knows about. Discovered at runtime on the
    /// Lua side, so it accounts for modded material sets.
    last_material_id: i16,
}

impl WorldFns {
    /// # Safety
    ///
    /// Every address must be the corresponding function from
    /// `noitapatcher.nsew.world_ffi`, taken from the currently running Noita
    /// process. Passing anything else will execute arbitrary memory.
    pub(crate) unsafe fn from_raw(
        get_cell: *const c_void,
        chunk_loaded: *const c_void,
        remove_cell: *const c_void,
        construct_cell: *const c_void,
        last_material_id: i16,
    ) -> eyre::Result<Self> {
        if get_cell.is_null()
            || chunk_loaded.is_null()
            || remove_cell.is_null()
            || construct_cell.is_null()
        {
            eyre::bail!("Got a null world function pointer from NoitaPatcher");
        }
        // SAFETY: caller guarantees these are the real world_ffi functions, and
        // the signatures match world_ffi.lua's cdef block.
        unsafe {
            Ok(Self {
                get_cell: std::mem::transmute::<*const c_void, abi::GetCellFn>(get_cell),
                chunk_loaded: std::mem::transmute::<*const c_void, abi::ChunkLoadedFn>(
                    chunk_loaded,
                ),
                remove_cell: std::mem::transmute::<*const c_void, abi::RemoveCellFn>(remove_cell),
                construct_cell: std::mem::transmute::<*const c_void, abi::ConstructCellFn>(
                    construct_cell,
                ),
                last_material_id,
            })
        }
    }
}

impl ParticleWorldState {
    /// Material id for a cell, by offset into the material list. Mirrors
    /// `world_ffi.get_material_id`.
    fn material_id_of(&self, cell: &ntypes::Cell) -> i16 {
        let mat_ptr = cell.material_ptr();
        // Not `offset_from`: these are two independently obtained raw pointers,
        // so they are not provably part of one allocation.
        let delta = (mat_ptr as usize).wrapping_sub(self.material_list_ptr as usize);
        (delta / CELLDATA_SIZE as usize) as i16
    }

    /// Mirrors `world_ffi.get_material_ptr`.
    fn material_ptr(&self, id: i16) -> *const ntypes::CellData {
        self.material_list_ptr
            .wrapping_byte_offset(CELLDATA_SIZE * id as isize)
            .cast()
    }

    fn cell_type_of(cell: &ntypes::Cell) -> Option<ntypes::CellType> {
        // SAFETY: material_ptr on a live cell points into the game's material
        // list; `as_ref` handles the null case.
        unsafe { Some(cell.material_ptr().as_ref()?.cell_type) }
    }

    /// Apply an encoded area to the live world.
    ///
    /// `data` must be the full `EncodedArea` blob as produced by `encode_area`:
    /// a packed [`EncodedAreaHeader`] followed by `pixel_run_count` runs.
    ///
    /// # Safety
    ///
    /// Must be called on the game's main thread, from inside a world update,
    /// with `fns` resolved from the running process. This mutates the live
    /// pixel grid.
    #[cfg(target_arch = "x86")]
    pub(crate) unsafe fn decode_area(
        &mut self,
        data: &[u8],
        fns: &WorldFns,
    ) -> eyre::Result<usize> {
        let (header, runs, run_count) = parse_encoded_area(data)?;

        let width = i32::from(header.width) + 1;
        let height = i32::from(header.height) + 1;
        let top_left_x = header.x;
        let top_left_y = header.y;
        let bottom_right_x = top_left_x + width;
        let bottom_right_y = top_left_y + height;

        let grid_world = self._world_ptr;
        let chunk_map = self.chunk_map_ptr;

        let mut run_ix = 0usize;
        // SAFETY: parse_encoded_area verified there are at least run_count runs.
        let mut current = unsafe { runs.read_unaligned() };
        let mut new_material = current.material as i16;
        let mut flags = current.flags;
        let mut left = i32::from(current.length) + 1;
        let mut written = 0usize;

        for y in top_left_y..bottom_right_y {
            for x in top_left_x..bottom_right_x {
                // A labelled block replaces the Lua's `goto next_pixel`: the
                // run bookkeeping below must run for every pixel regardless of
                // whether the pixel itself was skipped.
                'pixel: {
                    // SAFETY: chunk_map is the game's ChunkMap for this world.
                    if !unsafe { (fns.chunk_loaded)(chunk_map, x, y) } {
                        break 'pixel;
                    }
                    // -1 means "sender had no data here", not a material.
                    if new_material == -1 {
                        break 'pixel;
                    }
                    // SAFETY: the chunk is loaded, so this slot exists.
                    let ppixel = unsafe { (fns.get_cell)(chunk_map, x, y) };
                    if ppixel.is_null() {
                        break 'pixel;
                    }

                    let mut current_material = 0i16;
                    // SAFETY: ppixel points at a Cell* slot in the grid.
                    let existing = unsafe { *ppixel };
                    if let Some(cell) = unsafe { existing.as_ref() } {
                        // Nobody knows how box2d pixels work; leave them alone.
                        if Self::cell_type_of(cell) == Some(ntypes::CellType::Solid) {
                            break 'pixel;
                        }
                        current_material = self.material_id_of(cell);
                        if new_material != current_material {
                            // SAFETY: `existing` is a live cell at (x, y).
                            unsafe {
                                (fns.remove_cell)(grid_world, existing.cast(), x, y, false);
                            }
                        }
                    }

                    if current_material == new_material || new_material == 0 {
                        break 'pixel;
                    }
                    if new_material > fns.last_material_id || new_material < 0 {
                        break 'pixel;
                    }
                    let mat_ptr = self.material_ptr(new_material);
                    if mat_ptr.is_null() {
                        break 'pixel;
                    }
                    // SAFETY: mat_ptr indexes the game's material list.
                    let pixel = unsafe {
                        (fns.construct_cell)(grid_world, x, y, mat_ptr, std::ptr::null_mut())
                    };
                    if pixel.is_null() {
                        // Happens when the material texture is transparent at
                        // this coordinate. Matches the Lua path: skip.
                        break 'pixel;
                    }

                    // SAFETY: just constructed by the game.
                    if let Some(cell) = unsafe { pixel.as_ref() }
                        && Self::cell_type_of(cell) == Some(ntypes::CellType::Liquid)
                    {
                        // LiquidCell begins with a Cell, so this reinterpret is
                        // the documented prefix relationship, not a punning.
                        let liquid = pixel.cast::<ntypes::LiquidCell>();
                        // SAFETY: cell_type says this really is a LiquidCell.
                        unsafe {
                            (*liquid).is_static =
                                (flags & LIQUID_FLAG_STATIC) == LIQUID_FLAG_STATIC;
                        }
                    }

                    // SAFETY: hand ownership of the new cell to the grid.
                    unsafe { *ppixel = pixel };
                    written += 1;
                }

                left -= 1;
                if left <= 0 {
                    run_ix += 1;
                    if run_ix >= run_count {
                        // Ran out of runs. The Lua asserted that this coincides
                        // with the last pixel; a truncated or over-long buffer
                        // from the network should not be fatal, so just stop.
                        return Ok(written);
                    }
                    // SAFETY: run_ix < run_count, checked directly above.
                    current = unsafe { runs.add(run_ix).read_unaligned() };
                    new_material = current.material as i16;
                    flags = current.flags;
                    left = i32::from(current.length) + 1;
                }
            }
        }

        Ok(written)
    }

    #[cfg(not(target_arch = "x86"))]
    pub(crate) unsafe fn decode_area(
        &mut self,
        _data: &[u8],
        _fns: &WorldFns,
    ) -> eyre::Result<usize> {
        eyre::bail!("Native world decode is only implemented for 32-bit x86 (Noita's target)")
    }
}

/// Split an `EncodedArea` blob into its header and run array, validating that
/// the buffer actually contains the runs the header claims.
///
/// The blob arrives over the network, so the run count cannot be trusted.
fn parse_encoded_area(
    data: &[u8],
) -> eyre::Result<(EncodedAreaHeader, *const NoitaPixelRun, usize)> {
    let header_size = size_of::<EncodedAreaHeader>();
    if data.len() < header_size {
        eyre::bail!(
            "Encoded area is {} bytes, shorter than its {header_size} byte header",
            data.len()
        );
    }
    // SAFETY: length checked above; EncodedAreaHeader is packed and made
    // entirely of integers, so any bit pattern is valid.
    let header = unsafe { data.as_ptr().cast::<EncodedAreaHeader>().read_unaligned() };
    let run_count = usize::from(header.pixel_run_count);
    if run_count == 0 {
        eyre::bail!("Encoded area declares zero pixel runs");
    }
    let needed = header_size + run_count * size_of::<NoitaPixelRun>();
    if data.len() < needed {
        eyre::bail!(
            "Encoded area declares {run_count} runs ({needed} bytes) but is only {} bytes",
            data.len()
        );
    }
    let runs = unsafe { data.as_ptr().add(header_size) }.cast::<NoitaPixelRun>();
    Ok((header, runs, run_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blob(header: EncodedAreaHeader, runs: &[NoitaPixelRun]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&header.x.to_ne_bytes());
        out.extend_from_slice(&header.y.to_ne_bytes());
        out.push(header.width);
        out.push(header.height);
        out.extend_from_slice(&header.pixel_run_count.to_ne_bytes());
        for run in runs {
            out.extend_from_slice(&run.length.to_ne_bytes());
            out.extend_from_slice(&run.material.to_ne_bytes());
            out.push(run.flags);
        }
        out
    }

    fn header(run_count: u16) -> EncodedAreaHeader {
        EncodedAreaHeader {
            x: 0,
            y: 0,
            width: 127,
            height: 127,
            pixel_run_count: run_count,
        }
    }

    #[test]
    fn parses_a_well_formed_area() {
        let runs = [
            NoitaPixelRun {
                length: 3,
                material: 5,
                flags: 1,
            },
            NoitaPixelRun {
                length: 0,
                material: 7,
                flags: 0,
            },
        ];
        let data = blob(header(2), &runs);
        let (h, ptr, count) = parse_encoded_area(&data).expect("should parse");
        assert_eq!(count, 2);
        assert_eq!({ h.width }, 127);
        let first = unsafe { ptr.read_unaligned() };
        assert_eq!({ first.material }, 5);
        let second = unsafe { ptr.add(1).read_unaligned() };
        assert_eq!({ second.material }, 7);
    }

    #[test]
    fn rejects_a_truncated_header() {
        assert!(parse_encoded_area(&[0u8; 4]).is_err());
    }

    #[test]
    fn rejects_a_run_count_the_buffer_cannot_back() {
        // Declares 500 runs but supplies one: reading them would run off the
        // end of the allocation.
        let data = blob(
            header(500),
            &[NoitaPixelRun {
                length: 1,
                material: 1,
                flags: 0,
            }],
        );
        assert!(parse_encoded_area(&data).is_err());
    }

    #[test]
    fn rejects_zero_runs() {
        let data = blob(header(0), &[]);
        assert!(parse_encoded_area(&data).is_err());
    }
}
