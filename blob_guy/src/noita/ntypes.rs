// Type defs borrowed from NoitaPatcher.

use std::ffi::{c_char, c_void};

pub(crate) const CELLDATA_SIZE: isize = 0x290;
#[cfg(target_arch = "x86")]
use std::arch::asm;
#[repr(C)]
#[derive(Debug)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct StdString {
    buffer: *const i8,
    sso_buffer: [i8; 12],
    size: usize,
    capacity: usize,
}
#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[expect(dead_code)]
pub enum CellType {
    None = 0,
    Liquid = 1,
    Gas = 2,
    Solid = 3,
    Fire = 4,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct CellData {
    name: StdString,
    ui_name: StdString,
    material_type: isize,
    id_2: isize,
    pub(crate) cell_type: CellType,
    platform_type: isize,
    wang_color: usize, //TODO Colour?
    gfx_glow: isize,
    gfx_glow_color: usize,
    unknown1: [c_char; 24],
    default_primary_colour: usize,
    unknown2: [c_char; 36],
    cell_holes_in_texture: bool,
    stainable: bool,
    burnable: bool,
    on_fire: bool,
    fire_hp: isize,
    autoignition_temperature: isize,
    _100_minus_autoignition_temp: isize,
    temperature_of_fire: isize,
    generates_smoke: isize,
    generates_flames: isize,
    requires_oxygen: bool,
    padding1: [c_char; 3],
    on_fire_convert_to_material: StdString,
    on_fire_convert_to_material_id: isize,
    on_fire_flame_material: StdString,
    on_fire_flame_material_id: isize,
    on_fire_smoke_material: StdString,
    on_fire_smoke_material_id: isize,
    explosion_config: *const c_void,
    durability: isize,
    crackability: isize,
    electrical_conductivity: bool,
    slippery: bool,
    padding2: [c_char; 2],
    stickyness: f32,
    cold_freezes_to_material: StdString,
    warmth_melts_to_material: StdString,
    warmth_melts_to_material_id: isize,
    cold_freezes_to_material_id: isize,
    cold_freezes_chance_rev: i16,
    warmth_melts_chance_rev: i16,
    cold_freezes_to_dont_do_reverse_reaction: bool,
    padding3: [c_char; 3],
    lifetime: isize,
    hp: isize,
    density: f32,
    liquid_sand: bool,
    liquid_slime: bool,
    liquid_static: bool,
    liquid_stains_self: bool,
    liquid_sticks_to_ceiling: isize,
    liquid_gravity: f32,
    liquid_viscosity: isize,
    liquid_stains: isize,
    liquid_stains_custom_color: usize,
    liquid_sprite_stain_shaken_drop_chance: f32,
    liquid_sprite_stain_ignited_drop_chance: f32,
    liquid_sprite_stains_check_offset: i8,
    padding4: [c_char; 3],
    liquid_sprite_stains_status_threshold: f32,
    liquid_damping: f32,
    liquid_flow_speed: f32,
    liquid_sand_never_box2d: bool,
    unknown7: [c_char; 3],
    gas_speed: i8,
    gas_upwards_speed: i8,
    gas_horizontal_speed: i8,
    gas_downwards_speed: i8,
    solid_friction: f32,
    solid_restitution: f32,
    solid_gravity_scale: f32,
    solid_static_type: isize,
    solid_on_collision_splash_power: f32,
    solid_on_collision_explode: bool,
    solid_on_sleep_convert: bool,
    solid_on_collision_convert: bool,
    solid_on_break_explode: bool,
    solid_go_through_sand: bool,
    solid_collide_with_self: bool,
    padding5: [c_char; 2],
    solid_on_collision_material: StdString,
    solid_on_collision_material_id: isize,
    solid_break_to_type: StdString,
    solid_break_to_type_id: isize,
    convert_to_box2d_material: StdString,
    convert_to_box2d_material_id: isize,
    vegetation_full_lifetime_growth: isize,
    vegetation_sprite: StdString,
    vegetation_random_flip_x_scale: bool,
    padding6: [c_char; 3],
    unknown11: [c_char; 12],
    wang_noise_percent: f32,
    wang_curvature: f32,
    wang_noise_type: isize,
    unknown12: [c_char; 12],
    danger_fire: bool,
    danger_radioactive: bool,
    danger_poison: bool,
    danger_water: bool,
    unknown13: [c_char; 23],
    always_ignites_damagemodel: bool,
    ignore_self_reaction_warning: bool,
    padding7: [c_char; 2],
    unknown14: [c_char; 12],
    audio_size_multiplier: f32,
    audio_is_soft: bool,
    padding8: [c_char; 3],
    unknown15: [c_char; 8],
    show_in_creative_mode: bool,
    is_just_particle_fx: bool,
    padding9: [c_char; 2],
}

#[repr(C)]
pub(crate) struct CellVTable {
    destroy: *const c_void,
    get_cell_type: *const c_void,
    _field01: *const c_void,
    _field02: *const c_void,
    _field03: *const c_void,
    get_color: *const c_void,
    _field04: *const c_void,
    set_color: *const c_void,
    _field05: *const c_void,
    _field06: *const c_void,
    _field07: *const c_void,
    _field08: *const c_void,
    get_material: *const c_void,
    _field09: *const c_void,
    _field10: *const c_void,
    _field11: *const c_void,
    _field12: *const c_void,
    _field13: *const c_void,
    _field14: *const c_void,
    _field15: *const c_void,
    get_position: *const c_void,
    _field16: *const c_void,
    _field17: *const c_void,
    _field18: *const c_void,
    _field19: *const c_void,
    _field20: *const c_void,
    _field21: *const c_void,
    _field22: *const c_void,
    _field23: *const c_void,
    is_burning: *const c_void,
    _field24: *const c_void,
    _field25: *const c_void,
    _field26: *const c_void,
    stop_burning: *const c_void,
    _field27: *const c_void,
    _field28: *const c_void,
    _field29: *const c_void,
    _field30: *const c_void,
    _field31: *const c_void,
    remove: *const c_void,
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
    pub fn get_material(&self, cell: *const c_void) -> *const CellData {
        //TODO ask why c_void
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
    pub fn get_position(&self, cell: *const c_void) -> *const Position {
        //TODO ask about probable mistake
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: *const Position;
            asm!(
                "mov ecx, {cell}",
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
            std::mem::transmute(ret[0])
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
#[derive(Debug)]
pub(crate) struct Cell {
    pub vtable: *const CellVTable,

    hp: isize,
    unknown1: [u8; 8],
    is_burning: bool,
    unknown2: [u8; 3],
    pub material_ptr: *const CellData,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LiquidCell {
    cell: Cell,
    x: isize,
    y: isize,
    unknown1: c_char,
    unknown2: c_char,
    pub(crate) is_static: bool,
    unknown3: c_char,
    unknown4: [u8; 3],
    colour: Colour,
    not_colour: usize,
}

impl Cell {
    pub(crate) fn material_ptr(&self) -> *const CellData {
        self.material_ptr
    }
}
