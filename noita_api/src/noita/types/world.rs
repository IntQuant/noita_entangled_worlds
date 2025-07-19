use crate::noita::types::objects::{ExplosionConfig, GridCosmeticParticleConfig};
use crate::noita::types::{StdMap, StdString, StdVec, ThiscallFn};
use shared::world_sync::{CompactPixel, PixelFlags, RawPixel};
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::slice;
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
pub struct CellGraphics {
    pub texture_file: StdString,
    pub color: Color,
    pub fire_colors_index: u32,
    pub randomize_colors: bool,
    pub normal_mapped: bool,
    pub is_grass: bool,
    pub is_grass_hashed: bool,
    pub pixel_info: *const c_void,
    unknown: [isize; 6],
}
#[repr(C)]
pub struct StatusEffect {
    pub id: isize,
    pub duration: f32,
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
    pub graphics: CellGraphics,
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
    padding5: [u8; 3],
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
    padding6: [u8; 2],
    pub solid_on_collision_material: StdString,
    pub solid_on_collision_material_id: isize,
    pub solid_break_to_type: StdString,
    pub solid_break_to_type_id: isize,
    pub convert_to_box2d_material: StdString,
    pub convert_to_box2d_material_id: isize,
    pub vegetation_full_lifetime_growth: isize,
    pub vegetation_sprite: StdString,
    pub vegetation_random_flip_x_scale: bool,
    padding7: [u8; 3],
    pub max_reaction_probability: u32,
    pub max_fast_reaction_probability: u32,
    unknown11: isize,
    pub wang_noise_percent: f32,
    pub wang_curvature: f32,
    pub wang_noise_type: isize,
    pub tags_start: *mut StdString,
    pub tags_end: *mut StdString,
    pub tags_cap: *mut StdString,
    pub danger_fire: bool,
    pub danger_radioactive: bool,
    pub danger_poison: bool,
    pub danger_water: bool,
    pub stain_effects_start: *mut StdString,
    pub stain_effects_end: *mut StdString,
    pub stain_effects_cap: *mut StdString,
    pub ingestion_effects_start: *mut StdString,
    pub ingestion_effects_end: *mut StdString,
    pub ingestion_effects_cap: *mut StdString,
    pub always_ignites_damagemodel: bool,
    pub ignore_self_reaction_warning: bool,
    padding8: [u8; 2],
    pub audio_physics_material_event_idx: isize,
    pub audio_physics_material_wall_idx: isize,
    pub audio_physics_material_solid_idx: isize,
    pub audio_size_multiplier: f32,
    pub audio_is_soft: bool,
    padding9: [u8; 3],
    pub audio_material_audio_type: isize,
    pub audio_material_breakaudio_type: isize,
    pub show_in_creative_mode: bool,
    pub is_just_particle_fx: bool,
    padding10: [u8; 2],
    pub grid_cosmetic_particle_config: *const GridCosmeticParticleConfig,
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

#[repr(transparent)]
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
            color: mat.graphics.color,
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
            color: mat.graphics.color,
            not_color: mat.graphics.color,
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
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    unknown1: [isize; 13],
    pub grid_world: *mut GridWorld,
    //likely more data
}

#[repr(C)]
#[derive(Debug)]
pub struct CellFactory {
    unknown1: isize,
    pub material_ids: StdVec<StdString>,
    pub material_id_indices: StdMap<StdString, usize>,
    pub cell_data: StdVec<CellData>,
    pub material_count: usize,
    unknown2: isize,
    pub reaction_lookup: ReactionLookupTable,
    pub fast_reaction_lookup: ReactionLookupTable,
    pub req_reactions: StdVec<CellReactionBuf>,
    pub materials_by_tag: StdMap<StdString, StdVec<*const CellData>>,
    unknown3: StdVec<*const StdVec<*const c_void>>,
    pub fire_cell_data: *const CellData,
    unknown4: [usize; 4],
    pub fire_material_id: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct ReactionLookupTable {
    pub width: usize,
    pub height: usize,
    pub len: usize,
    _unknown: [usize; 5],
    storage: *const CellReactionBuf,
    _unknown2: usize,
    _unknown3: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct CellReactionBuf {
    base: *const CellReaction,
    _unknown: usize, // only ever saw this equal to len
    len: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct CellReaction {
    pub fast_reaction: bool,
    padding: [u8; 3],
    pub probability_times_100: usize,
    pub input_cell1: isize,
    pub input_cell2: isize,
    pub output_cell1: isize,
    pub output_cell2: isize,
    pub has_input_cell3: bool,
    padding2: [u8; 3],
    pub input_cell3: isize,
    pub output_cell3: isize,
    pub cosmetic_particle: isize,
    pub req_lifetime: isize,
    pub blob_radius1: u8,
    pub blob_radius2: u8,
    pub blob_restrict_to_input_material1: bool,
    pub blob_restrict_to_input_material2: bool,
    pub destroy_horizontally_lonely_pixels: bool,
    pub convert_all: bool,
    padding3: [u8; 2],
    pub entity_file_idx: usize,
    pub direction: ReactionDir,
    pub explosion_config: *const ExplosionConfig,
    pub audio_fx_volume_1: f32,
}

#[derive(Debug)]
pub enum ReactionDir {
    None = -1,
    Top,
    Bottom,
    Left,
    Right,
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
    unk: [isize; 5],
    pub inventory_open: usize,
    unk4: [isize; 79],
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(transparent)]
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
        unsafe { slice::from_raw_parts(self.0, 512 * 512) }.iter()
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
#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkPtrPtr(pub *mut ChunkPtr);
unsafe impl Sync for ChunkPtrPtr {}
unsafe impl Send for ChunkPtrPtr {}
#[repr(transparent)]
#[derive(Debug)]
pub struct ChunkArrayPtr(pub *mut ChunkPtrPtr);
unsafe impl Sync for ChunkArrayPtr {}
unsafe impl Send for ChunkArrayPtr {}
impl ChunkArrayPtr {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ChunkPtrPtr> {
        unsafe { slice::from_raw_parts(self.0, 512 * 512) }.iter()
    }
    #[inline]
    pub fn slice(&self) -> &'static [ChunkPtrPtr] {
        unsafe { slice::from_raw_parts(self.0, 512 * 512) }
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
pub struct AABB {
    pub top_left: Position,
    pub bottom_right: Position,
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
    pub update_region: AABB,
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorld {
    pub vtable: &'static GridWorldVTable,
    pub rng: isize,
    unk: [isize; 292],
    pub cam_x: isize,
    pub cam_y: isize,
    pub cam_w: isize,
    pub cam_h: isize,
    unknown: [isize; 21],
    pub world_update_count: isize,
    pub chunk_map: ChunkMap,
    unknown2: [isize; 41],
    pub m_thread_impl: *mut GridWorldThreaded,
}
