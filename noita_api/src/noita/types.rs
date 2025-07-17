// Type defs borrowed from NoitaPatcher.

use std::ffi::c_void;

use shared::world_sync::{CompactPixel, PixelFlags, RawPixel};
use std::fmt::{Debug, Display, Formatter};
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
pub struct ChunkPtr(pub *mut CellPtr);
impl Debug for ChunkPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChunkPtr {{{:?} {:?}}}",
            self.0,
            self.iter().filter(|c| !c.0.is_null()).count(),
        )
    }
}
unsafe impl Sync for ChunkPtr {}
unsafe impl Send for ChunkPtr {}

impl ChunkPtr {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &CellPtr> {
        unsafe { std::slice::from_raw_parts(self.0, 512 * 512) }.iter()
    }
    #[inline]
    pub fn get(&self, x: isize, y: isize) -> Option<&Cell> {
        let index = (y << 9) | x;
        unsafe { self.0.offset(index).as_ref()?.0.as_ref() }
    }
    #[inline]
    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut CellPtr> {
        unsafe { self.get_mut_raw(x, y).as_mut() }
    }
    #[inline]
    pub fn get_mut_raw(&mut self, x: isize, y: isize) -> *mut CellPtr {
        let index = (y << 9) | x;
        unsafe { self.0.offset(index) }
    }
    #[inline]
    pub fn get_raw_pixel(&self, x: isize, y: isize) -> RawPixel {
        if let Some(cell) = self.get(x, y) {
            if cell.material.cell_type == CellType::Liquid {
                RawPixel {
                    material: cell.material.material_type as u16,
                    flags: if cell.get_liquid().is_static == cell.material.liquid_static {
                        PixelFlags::Normal
                    } else {
                        PixelFlags::Abnormal
                    },
                }
            } else {
                RawPixel {
                    material: cell.material.material_type as u16,
                    flags: PixelFlags::Normal,
                }
            }
        } else {
            RawPixel {
                material: 0,
                flags: PixelFlags::Normal,
            }
        }
    }
    #[inline]
    pub fn get_compact_pixel(&self, x: isize, y: isize) -> Option<CompactPixel> {
        self.get(x, y).map(|cell| {
            CompactPixel(if cell.material.cell_type == CellType::Liquid {
                (cell.material.material_type as u16
                    | if cell.get_liquid().is_static == cell.material.liquid_static {
                        PixelFlags::Normal
                    } else {
                        PixelFlags::Abnormal
                    } as u16)
                    .try_into()
                    .unwrap()
            } else {
                (cell.material.material_type as u16 | PixelFlags::Normal as u16)
                    .try_into()
                    .unwrap()
            })
        })
    }
}

#[repr(C)]
pub struct ChunkMap {
    unknown: [isize; 2],
    pub chunk_array: ChunkArrayPtr,
    unknown2: [isize; 8],
}
#[repr(C)]
#[derive(Debug)]
pub struct ChunkPtrPtr(pub *mut ChunkPtr);
unsafe impl Sync for ChunkPtrPtr {}
unsafe impl Send for ChunkPtrPtr {}
#[repr(C)]
#[derive(Debug)]
pub struct ChunkArrayPtr(pub *mut ChunkPtrPtr);
unsafe impl Sync for ChunkArrayPtr {}
unsafe impl Send for ChunkArrayPtr {}
impl ChunkArrayPtr {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ChunkPtrPtr> {
        unsafe { std::slice::from_raw_parts(self.0, 512 * 512) }.iter()
    }
    #[inline]
    pub fn slice(&self) -> &'static [ChunkPtrPtr] {
        unsafe { std::slice::from_raw_parts(self.0, 512 * 512) }
    }
    #[inline]
    pub fn get(&self, x: isize, y: isize) -> Option<&ChunkPtr> {
        let index = (((y - 256) & 511) << 9) | ((x - 256) & 511);
        unsafe { self.0.offset(index).as_ref()?.0.as_ref() }
    }
    #[inline]
    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut ChunkPtr> {
        let index = (((y - 256) & 511) << 9) | ((x - 256) & 511);
        unsafe { self.0.offset(index).as_mut()?.0.as_mut() }
    }
}
impl Debug for ChunkMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChunkMap {{ unknown: {:?}, cell_array: {{{}}} ,unknown2: {:?} }}",
            self.unknown,
            self.chunk_array
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.0.is_null())
                .map(|(i, c)| {
                    let x = i as isize % 512 - 256;
                    let y = i as isize / 512 - 256;
                    format!("{i}: {{ x: {x}, y: {y}, {c:?}}}",)
                })
                .collect::<Vec<String>>()
                .join(", "),
            self.unknown2
        )
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorldVTable {
    //ptr is 0x10013bc
    unknown: [*const ThiscallFn; 3],
    pub get_chunk_map: *const ThiscallFn,
    unknownmagic: *const ThiscallFn,
    unknown2: [*const ThiscallFn; 29],
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
struct AABB {
    top_left: Position,
    bottom_right: Position,
}

#[repr(C)]
#[derive(Debug)]
//ptr is 0x17f83e30, seems not constant
pub struct GridWorldThreadedVTable {
    //TODO find some data maybe
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorldThreaded {
    pub grid_world_threaded_vtable: &'static GridWorldThreadedVTable,
    unknown: [isize; 287],
    update_region: AABB,
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorld {
    pub vtable: &'static GridWorldVTable,
    pub rng: isize,
    unknown: [isize; 317],
    pub world_update_count: isize,
    pub chunk_map: ChunkMap,
    unknown2: [isize; 41],
    pub m_thread_impl: *mut GridWorldThreaded,
}

#[repr(C)]
union Buffer {
    buffer: *mut u8,
    sso_buffer: [u8; 16],
}
impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            sso_buffer: [0; 16],
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StdString {
    buffer: Buffer,
    size: usize,
    capacity: usize,
}
impl From<&str> for StdString {
    fn from(value: &str) -> Self {
        let mut res = StdString {
            capacity: value.len(),
            size: value.len(),
            ..Default::default()
        };
        if res.capacity > 16 {
            let buffer = value.as_bytes().to_vec();
            res.buffer.buffer = buffer.as_ptr().cast_mut();
            std::mem::forget(buffer);
        } else {
            let mut iter = value.as_bytes().iter();
            res.buffer.sso_buffer = std::array::from_fn(|_| *iter.next().unwrap())
        }
        res
    }
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
pub struct ExplosionConfig {
    //TODO find some data maybe
}

#[repr(C)]
#[derive(Debug)]
pub struct GridCosmeticParticleConfig {
    //TODO find some data maybe
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
    pub wang_color: Color,
    pub gfx_glow: isize,
    pub gfx_glow_color: Color,
    unknown1: [isize; 6],
    pub default_primary_color: Color,
    unknown2: [isize; 9],
    pub cell_holes_in_texture: bool,
    pub stainable: bool,
    pub burnable: bool,
    pub on_fire: bool,
    pub fire_hp: isize,
    pub autoignition_temperature: isize,
    pub minus_100_autoignition_temp: isize,
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
    pub explosion_config: *const ExplosionConfig,
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
    pub liquid_stains_custom_color: Color,
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
    unknown13: [u8; 24],
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
    pub grid_cosmetic_particle_config: *const GridCosmeticParticleConfig,
}
impl Default for CellData {
    fn default() -> Self {
        Self {
            name: StdString::default(),
            ui_name: StdString::default(),
            material_type: -1,
            id_2: -1,
            cell_type: CellType::Liquid,
            platform_type: -1,
            wang_color: Color::default(),
            gfx_glow: 0,
            gfx_glow_color: Color::default(),
            unknown1: [0, 0, 0, 0, 0, 15],
            default_primary_color: Color::default(),
            unknown2: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            cell_holes_in_texture: false,
            stainable: true,
            burnable: false,
            on_fire: false,
            fire_hp: 0,
            autoignition_temperature: 100,
            minus_100_autoignition_temp: 0,
            temperature_of_fire: 10,
            generates_smoke: 0,
            generates_flames: 30,
            requires_oxygen: true,
            padding1: [0, 0, 0],
            on_fire_convert_to_material: StdString::default(),
            on_fire_convert_to_material_id: -1,
            on_fire_flame_material: StdString::from("fire"),
            on_fire_flame_material_id: 1,
            on_fire_smoke_material: StdString::from("smoke"),
            on_fire_smoke_material_id: 36,
            explosion_config: std::ptr::null(),
            durability: 0,
            crackability: 0,
            electrical_conductivity: true,
            slippery: false,
            padding2: [0, 0],
            stickyness: 0.0,
            cold_freezes_to_material: StdString::default(),
            warmth_melts_to_material: StdString::default(),
            warmth_melts_to_material_id: 0,
            cold_freezes_to_material_id: 0,
            cold_freezes_chance_rev: 100,
            warmth_melts_chance_rev: 100,
            cold_freezes_to_dont_do_reverse_reaction: false,
            padding3: [0, 0, 0],
            lifetime: 0,
            hp: 100,
            density: 1.0,
            liquid_sand: false,
            liquid_slime: false,
            liquid_static: false,
            liquid_stains_self: false,
            liquid_sticks_to_ceiling: 0,
            liquid_gravity: 0.5,
            liquid_viscosity: 50,
            liquid_stains: 0,
            liquid_stains_custom_color: Color::default(),
            liquid_sprite_stain_shaken_drop_chance: 1.0,
            liquid_sprite_stain_ignited_drop_chance: 10.0,
            liquid_sprite_stains_check_offset: 0,
            padding4: [0, 0, 0],
            liquid_sprite_stains_status_threshold: 0.01,
            liquid_damping: 0.8,
            liquid_flow_speed: 0.9,
            liquid_sand_never_box2d: false,
            unknown7: [0, 0, 0],
            gas_speed: 50,
            gas_upwards_speed: 100,
            gas_horizontal_speed: 100,
            gas_downwards_speed: 90,
            solid_friction: 0.3,
            solid_restitution: 0.2,
            solid_gravity_scale: 1.0,
            solid_static_type: 0,
            solid_on_collision_splash_power: 1.0,
            solid_on_collision_explode: false,
            solid_on_sleep_convert: false,
            solid_on_collision_convert: false,
            solid_on_break_explode: false,
            solid_go_through_sand: false,
            solid_collide_with_self: true,
            padding5: [0, 0],
            solid_on_collision_material: StdString::default(),
            solid_on_collision_material_id: 0,
            solid_break_to_type: StdString::default(),
            solid_break_to_type_id: 0,
            convert_to_box2d_material: StdString::default(),
            convert_to_box2d_material_id: 0,
            vegetation_full_lifetime_growth: 10000,
            vegetation_sprite: StdString::default(),
            vegetation_random_flip_x_scale: false,
            padding6: [0, 0, 0],
            unknown11: [50, 0, 0],
            wang_noise_percent: 1.0,
            wang_curvature: 0.5,
            wang_noise_type: 0,
            unknown12: [0, 0, 0],
            danger_fire: false,
            danger_radioactive: false,
            danger_poison: false,
            danger_water: false,
            unknown13: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            always_ignites_damagemodel: false,
            ignore_self_reaction_warning: false,
            padding7: [0, 0],
            unknown14: [0, 0, 0],
            audio_size_multiplier: 1.0,
            audio_is_soft: false,
            padding8: [0, 0, 0],
            unknown15: [0, 0],
            show_in_creative_mode: false,
            is_just_particle_fx: false,
            padding9: [0, 0],
            grid_cosmetic_particle_config: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CellVTables(pub [CellVTable; 5]);

impl CellVTables {
    pub fn none(&self) -> &'static NoneCellVTable {
        unsafe { self.0[0].none }
    }
    pub fn liquid(&self) -> &'static LiquidCellVTable {
        unsafe { self.0[1].liquid }
    }
    pub fn gas(&self) -> &'static GasCellVTable {
        unsafe { self.0[2].gas }
    }
    pub fn solid(&self) -> &'static SolidCellVTable {
        unsafe { self.0[3].solid }
    }
    pub fn fire(&self) -> &'static FireCellVTable {
        unsafe { self.0[4].fire }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union CellVTable {
    //ptr is 0xff2040
    pub none: &'static NoneCellVTable,
    //ptr is 0x100bb90
    pub liquid: &'static LiquidCellVTable,
    //ptr is 0x1007bcc
    pub gas: &'static GasCellVTable,
    //ptr is 0xff8a6c
    pub solid: &'static SolidCellVTable,
    //ptr is 0x10096e0
    pub fire: &'static FireCellVTable,
}

impl Debug for CellVTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self as *const CellVTable)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SolidCellVTable {
    unknown0: *const ThiscallFn,
    unknown1: *const ThiscallFn,
    unknown2: *const ThiscallFn,
    unknown3: *const ThiscallFn,
    unknown4: *const ThiscallFn,
    unknown5: *const ThiscallFn,
    unknown6: *const ThiscallFn,
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NoneCellVTable {
    unknown: [*const ThiscallFn; 41],
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GasCellVTable {
    unknown: [*const ThiscallFn; 41],
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FireCellVTable {
    unknown: [*const ThiscallFn; 41],
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LiquidCellVTable {
    pub destroy: *const ThiscallFn,
    pub get_cell_type: *const ThiscallFn,
    unknown01: *const ThiscallFn,
    unknown02: *const ThiscallFn,
    unknown03: *const ThiscallFn,
    pub get_color: *const ThiscallFn,
    unknown04: *const ThiscallFn,
    pub set_color: *const ThiscallFn,
    unknown05: *const ThiscallFn,
    unknown06: *const ThiscallFn,
    unknown07: *const ThiscallFn,
    unknown08: *const ThiscallFn,
    pub get_material: *const ThiscallFn,
    unknown09: *const ThiscallFn,
    unknown10: *const ThiscallFn,
    unknown11: *const ThiscallFn,
    unknown12: *const ThiscallFn,
    unknown13: *const ThiscallFn,
    unknown14: *const ThiscallFn,
    unknown15: *const ThiscallFn,
    pub get_position: *const ThiscallFn,
    unknown16: *const ThiscallFn,
    unknown17: *const ThiscallFn,
    unknown18: *const ThiscallFn,
    unknown19: *const ThiscallFn,
    unknown20: *const ThiscallFn,
    unknown21: *const ThiscallFn,
    unknown22: *const ThiscallFn,
    unknown23: *const ThiscallFn,
    pub is_burning: *const ThiscallFn,
    unknown24: *const ThiscallFn,
    unknown25: *const ThiscallFn,
    unknown26: *const ThiscallFn,
    pub stop_burning: *const ThiscallFn,
    unknown27: *const ThiscallFn,
    unknown28: *const ThiscallFn,
    unknown29: *const ThiscallFn,
    unknown30: *const ThiscallFn,
    unknown31: *const ThiscallFn,
    pub remove: *const ThiscallFn,
    unknown32: *const ThiscallFn,
}
#[repr(C)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Cell {
    pub vtable: &'static CellVTable,

    pub hp: isize,
    unknown1: [isize; 2],
    pub is_burning: bool,
    pub temperature_of_fire: u8,
    unknown2: [u8; 2],
    pub material: &'static CellData,
}

unsafe impl Sync for Cell {}
unsafe impl Send for Cell {}

#[derive(Default, Debug)]
pub enum FullCell {
    Cell(Cell),
    LiquidCell(LiquidCell),
    GasCell(GasCell),
    FireCell(FireCell),
    #[default]
    None,
}
impl From<&Cell> for FullCell {
    fn from(value: &Cell) -> Self {
        match value.material.cell_type {
            CellType::Liquid => FullCell::LiquidCell(*value.get_liquid()),
            CellType::Fire => FullCell::FireCell(*value.get_fire()),
            CellType::Gas => FullCell::GasCell(*value.get_gas()),
            CellType::None | CellType::Solid => FullCell::Cell(*value),
        }
    }
}

impl Cell {
    pub fn get_liquid(&self) -> &LiquidCell {
        unsafe { std::mem::transmute::<&Cell, &LiquidCell>(self) }
    }
    pub fn get_fire(&self) -> &FireCell {
        unsafe { std::mem::transmute::<&Cell, &FireCell>(self) }
    }
    pub fn get_gas(&self) -> &GasCell {
        unsafe { std::mem::transmute::<&Cell, &GasCell>(self) }
    }
}

#[repr(C)]
pub struct CellPtr(pub *mut Cell);
impl Debug for CellPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c = unsafe { self.0.as_ref() };
        let Some(c) = c else {
            return write!(f, "{c:?}");
        };
        write!(
            f,
            "CellPtr{{{:?}}}",
            format!("{c:?}")
                .split_once("material_ptr")
                .unwrap_or_default()
                .0
        )
    }
}
unsafe impl Sync for CellPtr {}
unsafe impl Send for CellPtr {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FireCell {
    pub cell: Cell,
    pub x: isize,
    pub y: isize,
    pub lifetime: isize,
    unknown: isize,
}

impl FireCell {
    ///# Safety
    pub unsafe fn create(
        mat: &'static CellData,
        vtable: &'static FireCellVTable,
        world: *mut GridWorld,
    ) -> Self {
        let lifetime = if let Some(world) = unsafe { world.as_mut() } {
            world.rng *= 0x343fd;
            world.rng += 0x269ec3;
            (world.rng >> 0x10 & 0x7fff) % 0x15
        } else {
            -1
        };
        let mut cell = Cell::create(mat, unsafe {
            (vtable as *const FireCellVTable)
                .cast::<CellVTable>()
                .as_ref()
                .unwrap()
        });
        cell.is_burning = true;
        Self {
            cell,
            x: 0,
            y: 0,
            lifetime,
            unknown: 1,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GasCell {
    pub cell: Cell,
    unknown5: isize,
    unknown6: isize,
    pub x: isize,
    pub y: isize,
    unknown1: u8,
    unknown2: u8,
    unknown3: u8,
    unknown4: u8,
    pub color: Color,
    unknown7: isize,
    unknown8: isize,
    lifetime: isize,
}

impl GasCell {
    ///# Safety
    pub unsafe fn create(
        mat: &'static CellData,
        vtable: &'static GasCellVTable,
        world: *mut GridWorld,
    ) -> Self {
        let (bool, lifetime) = if let Some(world) = unsafe { world.as_mut() } {
            let life = ((mat.lifetime as f32 * 0.3) as u64).max(1);
            world.rng *= 0x343fd;
            world.rng += 0x269ec3;
            (
                (world.rng >> 0x10 & 0x7fff) % 0x65 < 0x32,
                (((world.rng >> 0x10 & 0x7fff) as u64 % (life * 2 + 1)) - life) as isize,
            )
        } else {
            (false, -1)
        };
        let mut cell = Cell::create(mat, unsafe {
            (vtable as *const GasCellVTable)
                .cast::<CellVTable>()
                .as_ref()
                .unwrap()
        });
        cell.is_burning = true;
        Self {
            cell,
            unknown5: if bool { 1 } else { 0xff },
            unknown6: 0,
            x: 0,
            y: 0,
            unknown1: 0,
            unknown2: 0,
            unknown3: 0,
            unknown4: 0,
            unknown7: 0,
            unknown8: 0,
            color: mat.default_primary_color,
            lifetime,
        }
    }
}

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
    unknown4: isize,
    unknown5: isize,
    unknown6: isize,
    pub color: Color,
    pub not_color: Color,
    lifetime: isize,
    unknown8: isize,
}

impl LiquidCell {
    /// # Safety
    pub unsafe fn create(
        mat: &'static CellData,
        vtable: &'static LiquidCellVTable,
        world: *mut GridWorld,
    ) -> Self {
        let lifetime = if mat.lifetime > 0
            && let Some(world) = (unsafe { world.as_mut() })
        {
            let life = ((mat.lifetime as f32 * 0.3) as u64).max(1);
            world.rng *= 0x343fd;
            world.rng += 0x269ec3;
            (((world.rng >> 0x10 & 0x7fff) as u64 % (life * 2 + 1)) - life) as isize
        } else {
            -1
        };
        Self {
            cell: Cell::create(mat, unsafe {
                (vtable as *const LiquidCellVTable)
                    .cast::<CellVTable>()
                    .as_ref()
                    .unwrap()
            }),
            x: 0,
            y: 0,
            unknown1: 3,
            unknown2: 0,
            is_static: mat.liquid_static,
            unknown3: 0,
            unknown4: 0,
            unknown5: 0,
            unknown6: 0,
            color: mat.default_primary_color,
            not_color: mat.default_primary_color,
            lifetime,
            unknown8: 0,
        }
    }
}

impl Cell {
    fn create(material: &'static CellData, vtable: &'static CellVTable) -> Self {
        Self {
            vtable,
            hp: material.hp,
            unknown1: [-1000, 0],
            is_burning: material.on_fire,
            temperature_of_fire: material.temperature_of_fire as u8,
            unknown2: [0, 0],
            material,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct GameWorld {
    unknown1: [isize; 17],
    pub grid_world: *mut GridWorld,
    //likely more data
}

#[repr(C)]
#[derive(Debug)]
pub struct CellFactory {
    unknown1: [isize; 5],
    pub cell_data_len: usize,
    pub cell_data_ptr: *const CellData,
    //TODO likely more data
}

#[repr(C)]
#[derive(Debug)]
pub struct Textures {
    //TODO find some data maybe
}

#[repr(C)]
#[derive(Debug)]
pub struct GameGlobal {
    pub frame_num: usize,
    pub frame_num_start: usize,
    unknown1: isize,
    pub m_game_world: *mut GameWorld,
    pub m_grid_world: *mut GridWorld,
    pub m_textures: *mut Textures,
    pub m_cell_factory: *mut CellFactory,
    unknown2: [isize; 11],
    pub pause_state: isize,
}
#[repr(C)]
#[derive(Debug)]
pub struct Entity {
    pub id: isize,
    pub entry: isize,
    pub filename_index: usize,
    pub kill_flag: isize,
    unknown1: isize,
    pub name: StdString,
    unknown2: isize,
    unknown3: isize,
    unknown4: isize,
    unknown5: isize,
    unknown6: isize,
    unknown7: isize,
    unknown8: isize,
    unknown9: isize,
    unknown10: isize,
    unknown11: isize,
    unknown12: isize,
    unknown13: isize,
    unknown14: isize,
    unknown15: isize,
    unknown16: isize,
    unknown17: isize,
    unknown18: isize,
    pub x: f32,
    pub y: f32,
    unknown19: isize,
    pub angle: f32,
    unknown20: isize,
    unknown21: isize,
    pub scale_x: f32,
    pub scale_y: f32,
    pub children: *mut Child,
    unknown22: isize,
}

#[repr(C)]
#[derive(Debug)]
pub struct Child {
    pub start: *mut *mut Entity,
    pub end: *mut *mut Entity,
}

impl Entity {
    pub fn kill(&mut self) {
        self.kill_flag = 1;
        self.iter_children_mut().for_each(|e| e.kill());
    }
    pub fn iter_children(&self) -> impl Iterator<Item = &'static Entity> {
        unsafe {
            if let Some(child) = self.children.as_ref() {
                let len = child.end.offset_from(child.start);
                std::slice::from_raw_parts(child.start, len as usize)
            } else {
                &[]
            }
            .iter()
            .filter_map(|e| e.as_ref())
        }
    }
    pub fn iter_children_mut(&mut self) -> impl Iterator<Item = &'static mut Entity> {
        unsafe {
            if let Some(child) = self.children.as_ref() {
                let len = child.end.offset_from(child.start);
                std::slice::from_raw_parts(child.start, len as usize)
            } else {
                &[]
            }
            .iter()
            .filter_map(|e| e.as_mut())
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct EntityManager {
    unknown: [isize; 5],
    pub entity_list: *mut *mut Entity,
    pub entity_list_end: *mut *mut Entity,
    unk1: isize,
    unk2: isize,
    unk3: isize,
    unk4: isize,
    pub component_list: *mut *mut ComponentManager,
    pub component_list_end: *mut *mut ComponentManager,
    unknown2: [isize; 120],
    //TODO Unknown
}

#[repr(C)]
#[derive(Debug)]
pub struct ComponentManager {
    pub vtable: *const ComponentManagerVTable,
    pub end: isize,
    unk: [isize; 2],
    pub entity_entry: *mut isize,
    unk2: [isize; 8],
    pub next: *mut isize,
    unk3: isize,
    unk4: isize,
    pub component_list: *mut *mut Component,
}
impl ComponentManager {
    pub fn iter_components(&self, ent: &'static Entity) -> ComponentIter {
        unsafe {
            if let Some(off) = self.entity_entry.offset(ent.entry).as_ref() {
                ComponentIter {
                    component_list: self.component_list as *const *const Component,
                    off: *off,
                    next: self.next,
                    end: self.end,
                }
            } else {
                ComponentIter {
                    component_list: std::ptr::null_mut(),
                    off: 0,
                    next: std::ptr::null_mut(),
                    end: 0,
                }
            }
        }
    }
    pub fn iter_components_mut(&mut self, ent: &'static Entity) -> ComponentIterMut {
        unsafe {
            if let Some(off) = self.entity_entry.offset(ent.entry).as_ref() {
                ComponentIterMut {
                    component_list: self.component_list,
                    off: *off,
                    next: self.next,
                    end: self.end,
                }
            } else {
                ComponentIterMut {
                    component_list: std::ptr::null_mut(),
                    off: 0,
                    next: std::ptr::null_mut(),
                    end: 0,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ComponentIter {
    component_list: *const *const Component,
    off: isize,
    end: isize,
    next: *const isize,
}

impl Iterator for ComponentIter {
    type Item = &'static Component;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.offset(self.off).as_ref()?.as_ref();
            if let Some(n) = self.next.offset(self.off).as_ref() {
                self.off = *n
            } else {
                self.off = self.end
            }
            com
        }
    }
}
#[derive(Debug)]
pub struct ComponentIterMut {
    component_list: *const *mut Component,
    off: isize,
    end: isize,
    next: *const isize,
}

impl Iterator for ComponentIterMut {
    type Item = &'static mut Component;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.off == self.end {
                return None;
            }
            let com = self.component_list.offset(self.off).as_ref()?.as_mut();
            if let Some(n) = self.next.offset(self.off).as_ref() {
                self.off = *n
            } else {
                self.off = self.end
            }
            com
        }
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct Component {
    unk: [isize; 10],
}
#[repr(C)]
#[derive(Debug)]
pub struct ComponentManagerVTable {
    //TODO technically a union, or maybe ComponentManager is
}

impl EntityManager {
    pub fn get_entity(&self, id: isize) -> Option<&'static Entity> {
        unsafe {
            let len = self.entity_list_end.offset_from(self.entity_list) as usize;
            let o = std::slice::from_raw_parts(self.entity_list.offset(id), len - id as usize)
                .iter()
                .find_map(|c| c.as_ref().map(|c| c.id - c.entry))
                .unwrap_or(id);
            let start = self.entity_list.offset(id - o);
            let list = std::slice::from_raw_parts(start, len - (id - o) as usize);
            list.iter().find_map(|c| c.as_ref().filter(|c| c.id == id))
        }
    }
    pub fn get_entity_mut(&mut self, id: isize) -> Option<&'static mut Entity> {
        unsafe {
            let len = self.entity_list_end.offset_from(self.entity_list) as usize;
            let o = std::slice::from_raw_parts(self.entity_list.offset(id), len - id as usize)
                .iter()
                .find_map(|c| c.as_ref().map(|c| c.id - c.entry))
                .unwrap_or(id);
            let start = self.entity_list.offset(id - o);
            let list = std::slice::from_raw_parts(start, len - (id - o) as usize);
            list.iter().find_map(|c| c.as_mut().filter(|c| c.id == id))
        }
    }
    pub fn iter_entities(&self) -> impl Iterator<Item = &'static Entity> {
        unsafe {
            let len = self.entity_list_end.offset_from(self.entity_list) as usize;
            std::slice::from_raw_parts(self.entity_list, len)
                .iter()
                .filter_map(|e| e.as_ref())
        }
    }
    pub fn iter_entities_mut(&mut self) -> impl Iterator<Item = &'static mut Entity> {
        unsafe {
            let len = self.entity_list_end.offset_from(self.entity_list) as usize;
            std::slice::from_raw_parts(self.entity_list, len)
                .iter()
                .filter_map(|e| e.as_mut())
        }
    }
    pub fn iter_component_managers(&self) -> impl Iterator<Item = &'static ComponentManager> {
        unsafe {
            let len = self.component_list_end.offset_from(self.component_list) as usize;
            std::slice::from_raw_parts(self.component_list, len)
                .iter()
                .filter_map(|e| e.as_ref())
        }
    }
    pub fn iter_component_managers_mut(
        &mut self,
    ) -> impl Iterator<Item = &'static mut ComponentManager> {
        unsafe {
            let len = self.component_list_end.offset_from(self.component_list) as usize;
            std::slice::from_raw_parts(self.component_list, len)
                .iter()
                .filter_map(|e| e.as_mut())
        }
    }
    pub fn iter_all_components(
        &self,
        ent: &'static Entity,
    ) -> impl Iterator<Item = &'static Component> {
        self.iter_component_managers()
            .flat_map(move |c| c.iter_components(ent))
    }
    pub fn iter_all_components_mut(
        &mut self,
        ent: &'static Entity,
    ) -> impl Iterator<Item = &'static mut Component> {
        self.iter_component_managers_mut()
            .flat_map(move |c| c.iter_components_mut(ent))
    }
}
#[repr(C)]
pub struct ThiscallFn(c_void);
