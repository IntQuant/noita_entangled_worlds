// Type defs borrowed from NoitaPatcher.

use std::ffi::c_void;

//pub(crate) const CELLDATA_SIZE: isize = 0x290;
#[cfg(target_arch = "x86")]
const _: () = assert!(0x290 == size_of::<CellData>());
#[cfg(target_arch = "x86")]
use std::arch::asm;
use std::fmt::{Debug, Display, Formatter};
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Default, Clone, Copy)]
pub struct ChunkPtr(pub *mut &'static mut [CellPtr; 512 * 512]);
impl Debug for ChunkPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}
unsafe impl Sync for ChunkPtr {}
unsafe impl Send for ChunkPtr {}

#[repr(C)]
#[derive(Debug)]
pub struct ChunkMap {
    unknown: [isize; 2],
    pub cell_array: &'static [ChunkPtr; 512 * 512],
    unknown2: [isize; 8],
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorldVtable {
    unknown: [*const c_void; 3],
    pub get_chunk_map: *const c_void,
    unknown2: [*const c_void; 30],
}
#[allow(dead_code)]
impl GridWorldVtable {
    pub fn get_chunk_map(&self) -> *const ChunkMap {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: *const ChunkMap;
            asm!(
                "mov ecx, 0",
                "call {fn}",
                out("eax") ret,
                fn = in(reg) self.get_chunk_map,
                clobber_abi("C"),
            );
            ret
        }
        #[cfg(target_arch = "x86_64")]
        {
            unreachable!()
        }
    }
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct AABB {
    top_left: Position,
    bottom_right: Position,
}

#[repr(C)]
struct GridWorldThreaded {
    grid_world_threaded_vtable: *const c_void,
    unknown: [isize; 287],
    update_region: AABB,
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorld {
    pub vtable: *const GridWorldVtable,
    unknown: [isize; 318],
    pub world_update_count: isize,
    pub chunk_map: ChunkMap,
    unknown2: [isize; 41],
    m_thread_impl: *const GridWorldThreaded,
}

#[repr(C)]
union Buffer {
    buffer: *const u8,
    sso_buffer: [u8; 16],
}

#[repr(C)]
pub struct StdString {
    buffer: Buffer,
    size: usize,
    capacity: usize,
}
impl Display for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slice: &[u8] = unsafe {
            if self.capacity <= 16 {
                &self.buffer.sso_buffer[0..self.size]
            } else {
                std::slice::from_raw_parts(self.buffer.buffer, self.size)
            }
        };
        let actual_len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        let string = str::from_utf8(&slice[..actual_len]).unwrap_or("UTF8_ERR");
        write!(f, "{string}")
    }
}
impl Debug for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StdString(\"{self}\")")
    }
}
#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CellType {
    None = 0,
    Liquid = 1,
    Gas = 2,
    Solid = 3,
    Fire = 4,
}

#[repr(C)]
#[derive(Debug)]
pub struct CellData {
    pub name: StdString,
    pub ui_name: StdString,
    pub material_type: isize,
    pub id_2: isize,
    pub cell_type: CellType,
    pub platform_type: isize,
    pub wang_color: Colour,
    pub gfx_glow: isize,
    pub gfx_glow_color: Colour,
    unknown1: [isize; 6],
    pub default_primary_colour: Colour,
    unknown2: [isize; 9],
    pub cell_holes_in_texture: bool,
    pub stainable: bool,
    pub burnable: bool,
    pub on_fire: bool,
    pub fire_hp: isize,
    pub autoignition_temperature: isize,
    pub _100_minus_autoignition_temp: isize,
    pub temperature_of_fire: isize,
    pub generates_smoke: isize,
    pub generates_flames: isize,
    pub requires_oxygen: bool,
    padding1: [u8; 3],
    pub on_fire_convert_to_material: StdString,
    pub on_fire_convert_to_material_id: isize,
    pub on_fire_flame_material: StdString,
    pub on_fire_flame_material_id: isize,
    pub on_fire_smoke_material: StdString,
    pub on_fire_smoke_material_id: isize,
    pub explosion_config: *const c_void,
    pub durability: isize,
    pub crackability: isize,
    pub electrical_conductivity: bool,
    pub slippery: bool,
    padding2: [u8; 2],
    pub stickyness: f32,
    pub cold_freezes_to_material: StdString,
    pub warmth_melts_to_material: StdString,
    pub warmth_melts_to_material_id: isize,
    pub cold_freezes_to_material_id: isize,
    pub cold_freezes_chance_rev: i16,
    pub warmth_melts_chance_rev: i16,
    pub cold_freezes_to_dont_do_reverse_reaction: bool,
    padding3: [u8; 3],
    pub lifetime: isize,
    pub hp: isize,
    pub density: f32,
    pub liquid_sand: bool,
    pub liquid_slime: bool,
    pub liquid_static: bool,
    pub liquid_stains_self: bool,
    pub liquid_sticks_to_ceiling: isize,
    pub liquid_gravity: f32,
    pub liquid_viscosity: isize,
    pub liquid_stains: isize,
    pub liquid_stains_custom_color: Colour,
    pub liquid_sprite_stain_shaken_drop_chance: f32,
    pub liquid_sprite_stain_ignited_drop_chance: f32,
    pub liquid_sprite_stains_check_offset: i8,
    padding4: [u8; 3],
    pub liquid_sprite_stains_status_threshold: f32,
    pub liquid_damping: f32,
    pub liquid_flow_speed: f32,
    pub liquid_sand_never_box2d: bool,
    unknown7: [u8; 3],
    pub gas_speed: i8,
    pub gas_upwards_speed: i8,
    pub gas_horizontal_speed: i8,
    pub gas_downwards_speed: i8,
    pub solid_friction: f32,
    pub solid_restitution: f32,
    pub solid_gravity_scale: f32,
    pub solid_static_type: isize,
    pub solid_on_collision_splash_power: f32,
    pub solid_on_collision_explode: bool,
    pub solid_on_sleep_convert: bool,
    pub solid_on_collision_convert: bool,
    pub solid_on_break_explode: bool,
    pub solid_go_through_sand: bool,
    pub solid_collide_with_self: bool,
    padding5: [u8; 2],
    pub solid_on_collision_material: StdString,
    pub solid_on_collision_material_id: isize,
    pub solid_break_to_type: StdString,
    pub solid_break_to_type_id: isize,
    pub convert_to_box2d_material: StdString,
    pub convert_to_box2d_material_id: isize,
    pub vegetation_full_lifetime_growth: isize,
    pub vegetation_sprite: StdString,
    pub vegetation_random_flip_x_scale: bool,
    padding6: [u8; 3],
    unknown11: [isize; 3],
    pub wang_noise_percent: f32,
    pub wang_curvature: f32,
    pub wang_noise_type: isize,
    unknown12: [isize; 3],
    pub danger_fire: bool,
    pub danger_radioactive: bool,
    pub danger_poison: bool,
    pub danger_water: bool,
    unknown13: [u8; 23],
    pub always_ignites_damagemodel: bool,
    pub ignore_self_reaction_warning: bool,
    padding7: [u8; 2],
    unknown14: [isize; 3],
    pub audio_size_multiplier: f32,
    pub audio_is_soft: bool,
    padding8: [u8; 3],
    unknown15: [isize; 2],
    pub show_in_creative_mode: bool,
    pub is_just_particle_fx: bool,
    padding9: [u8; 2],
    pub grid_cosmetic_particle_config: *const c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct CellVTable {
    pub destroy: *const c_void,
    pub get_cell_type: *const c_void,
    _field01: *const c_void,
    _field02: *const c_void,
    _field03: *const c_void,
    pub get_color: *const c_void,
    _field04: *const c_void,
    pub set_color: *const c_void,
    _field05: *const c_void,
    _field06: *const c_void,
    _field07: *const c_void,
    _field08: *const c_void,
    pub get_material: *const c_void,
    _field09: *const c_void,
    _field10: *const c_void,
    _field11: *const c_void,
    _field12: *const c_void,
    _field13: *const c_void,
    _field14: *const c_void,
    _field15: *const c_void,
    pub get_position: *const c_void,
    _field16: *const c_void,
    _field17: *const c_void,
    _field18: *const c_void,
    _field19: *const c_void,
    _field20: *const c_void,
    _field21: *const c_void,
    _field22: *const c_void,
    _field23: *const c_void,
    pub is_burning: *const c_void,
    _field24: *const c_void,
    _field25: *const c_void,
    _field26: *const c_void,
    pub stop_burning: *const c_void,
    _field27: *const c_void,
    _field28: *const c_void,
    _field29: *const c_void,
    _field30: *const c_void,
    _field31: *const c_void,
    pub remove: *const c_void,
    _field32: *const c_void,
}
#[repr(C)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}
#[allow(dead_code)]
impl CellVTable {
    pub fn destroy(&self, cell: *const Cell) {
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.destroy,
                clobber_abi("C"),
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn get_cell_type(&self, cell: *const Cell) -> CellType {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: u32;
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.get_cell_type,
                out("eax") ret,
                clobber_abi("C"),
            );
            std::mem::transmute(ret)
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn get_color(&self, cell: *const Cell) -> Colour {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: u32;
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.get_color,
                out("eax") ret,
                clobber_abi("C"),
            );
            std::mem::transmute(ret)
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn set_color(&self, cell: *const Cell, color: Colour) {
        #[cfg(target_arch = "x86")]
        unsafe {
            let color: u32 = std::mem::transmute(color);
            asm!(
                "mov ecx, {cell}",
                "push {color}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.set_color,
                color = in(reg) color,
                clobber_abi("C"),
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((cell, color));
            unreachable!()
        }
    }
    pub fn get_material(&self, cell: *const Cell) -> *const CellData {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: *const CellData;
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.get_material,
                out("eax") ret,
                clobber_abi("C"),
            );
            ret
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn get_position(&self, cell: *const Cell) -> *const Position {
        #[cfg(target_arch = "x86")]
        unsafe {
            let mut ret: *const Position;
            asm!(
                "mov ecx, {cell}",
                "push 0",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.get_position,
                out("eax") ret,
                clobber_abi("C"),
            );
            ret
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn is_burning(&self, cell: *const Cell) -> bool {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: u16;
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.is_burning,
                out("eax") ret,
                clobber_abi("C"),
            );
            let ret: [u8; 2] = ret.to_ne_bytes();
            ret[0] == 1
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn stop_burning(&self, cell: *const Cell) {
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.stop_burning,
                clobber_abi("C"),
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
    pub fn remove(&self, cell: *const Cell) {
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!(
                "mov ecx, {cell}",
                "call {fn}",
                cell = in(reg) cell,
                fn = in(reg) self.remove,
                clobber_abi("C"),
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box(cell);
            unreachable!()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub vtable: *const CellVTable,

    pub hp: isize,
    unknown1: [isize; 2],
    pub is_burning: bool,
    unknown2: [u8; 3],
    pub material_ptr: *const CellData,
}
unsafe impl Sync for Cell {}
unsafe impl Send for Cell {}

#[derive(Default, Clone, Debug, Copy)]
pub enum FullCell {
    Cell(Cell),
    LiquidCell(LiquidCell),
    #[default]
    None,
}
impl From<Cell> for FullCell {
    fn from(value: Cell) -> Self {
        if let Some(mat) = unsafe { value.material_ptr.as_ref() }
            && mat.cell_type == CellType::Liquid
        {
            FullCell::LiquidCell(*unsafe { value.get_liquid() })
        } else {
            FullCell::Cell(value)
        }
    }
}

impl Cell {
    ///# Safety
    pub unsafe fn get_liquid(&self) -> &LiquidCell {
        unsafe { std::mem::transmute::<&Cell, &LiquidCell>(self) }
    }
}

pub struct CellPtr(pub *const Cell);
impl Debug for CellPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { self.0.as_ref() })
    }
}
unsafe impl Sync for CellPtr {}
unsafe impl Send for CellPtr {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LiquidCell {
    pub cell: Cell,
    pub x: isize,
    pub y: isize,
    unknown1: u8,
    unknown2: u8,
    pub is_static: bool,
    unknown3: u8,
    unknown4: [u8; 3],
    pub colour: Colour,
    pub not_colour: Colour,
}
