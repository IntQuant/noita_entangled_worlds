// Type defs borrowed from NoitaPatcher.

use std::ffi::c_void;

#[cfg(target_arch = "x86")]
use std::arch::asm;
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
    pub fn iter(&self) -> impl Iterator<Item = &CellPtr> {
        unsafe { std::slice::from_raw_parts(self.0, 512 * 512) }.iter()
    }
    pub fn get(&self, x: isize, y: isize) -> Option<&Cell> {
        let index = (y << 9) | x;
        unsafe { self.0.offset(index).as_ref().and_then(|c| c.0.as_ref()) }
    }
    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut CellPtr> {
        unsafe { self.get_mut_raw(x, y).as_mut() }
    }
    pub fn get_mut_raw(&mut self, x: isize, y: isize) -> *mut CellPtr {
        let index = (y << 9) | x;
        unsafe { self.0.offset(index) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RemovePtr(pub *const c_void);
impl Default for RemovePtr {
    fn default() -> Self {
        Self(0x71b480 as *const c_void)
    }
}
impl RemovePtr {
    pub fn remove_cell(self, world: *mut GridWorld, cell: *mut Cell, x: isize, y: isize) {
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!(
                "mov ecx, {world}",
                "push 0",
                "push {y:e}",
                "push {x:e}",
                "push {cell}",
                "call {remove}",
                world = in(reg) world,
                cell = in(reg) cell,
                x = in(reg) x,
                y = in(reg) y,
                remove = in(reg) self.0,
                clobber_abi("C"),
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((x, y, cell, world));
            unreachable!()
        }
    }
    pub fn print_bytes(self) {
        let mut start = unsafe { self.0.offset(-1).cast::<u8>() };
        let mut bytes = String::new();
        let end = get_function_end(self.0);
        while start != end.cast() {
            unsafe {
                start = start.offset(1);
                bytes += &format!("\\x{:x}", start.read());
            }
        }
        crate::print!("{}", bytes);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConstructPtr(pub *const c_void);
impl Default for ConstructPtr {
    fn default() -> Self {
        Self(0x7048c0 as *const c_void)
    }
}
impl ConstructPtr {
    pub fn create_cell(
        self,
        world: *mut GridWorld,
        x: isize,
        y: isize,
        material: &CellData,
        //_memory: *mut c_void,
    ) -> *mut Cell {
        #[cfg(target_arch = "x86")]
        unsafe {
            let cell_ptr: *mut Cell;
            asm!(
                "mov ecx, {world}",
                "push 0",
                "push {material}",
                "push {y:e}",
                "push {x:e}",
                "call {construct}",
                world = in(reg) world,
                x = in(reg) x,
                y = in(reg) y,
                material = in(reg) material,
                construct = in(reg) self.0,
                clobber_abi("C"),
                out("eax") cell_ptr,
            );
            cell_ptr
        }
        #[cfg(target_arch = "x86_64")]
        {
            std::hint::black_box((x, y, material, world));
            unreachable!()
        }
    }
    pub fn print_bytes(self) {
        let mut start = unsafe { self.0.offset(-1).cast::<u8>() };
        let mut bytes = String::new();
        let end = get_function_end(self.0);
        while start != end.cast() {
            unsafe {
                start = start.offset(1);
                bytes += &format!("\\x{:x}", start.read());
            }
        }
        crate::print!("{}", bytes);
    }
}
fn get_function_end(func: *const c_void) -> *const c_void {
    let mut it = func.cast::<u8>();
    loop {
        unsafe {
            if (it.offset(-1).as_ref() >= Some(&0x58) && it.offset(-1).as_ref() < Some(&0x60))
                && matches!(it.as_ref(), Some(&0xc3) | Some(&0xc2))
            {
                return if it.as_ref() == Some(&0xc3) {
                    it.offset(1).cast::<c_void>()
                } else {
                    it.offset(3).cast::<c_void>()
                };
            }
            it = it.offset(1)
        }
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
    pub fn iter(&self) -> impl Iterator<Item = &ChunkPtrPtr> {
        unsafe { std::slice::from_raw_parts(self.0, 512 * 512) }.iter()
    }
    pub fn slice(&self) -> &'static [ChunkPtrPtr] {
        unsafe { std::slice::from_raw_parts(self.0, 512 * 512) }
    }
    pub fn get(&self, x: isize, y: isize) -> Option<&ChunkPtr> {
        let index = (((y - 256) & 511) << 9) | ((x - 256) & 511);
        unsafe { self.0.offset(index).as_ref().and_then(|c| c.0.as_ref()) }
    }
    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut ChunkPtr> {
        let index = (((y - 256) & 511) << 9) | ((x - 256) & 511);
        unsafe { self.0.offset(index).as_mut().and_then(|c| c.0.as_mut()) }
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
    unknown: [*const c_void; 3],
    pub get_chunk_map: *const c_void,
    unknown2: [*const c_void; 30],
}
#[allow(dead_code)]
impl GridWorldVTable {
    pub fn get_chunk_map(&self) -> *mut ChunkMap {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: *mut ChunkMap;
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
#[derive(Debug)]
struct AABB {
    top_left: Position,
    bottom_right: Position,
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorldThreadedVTable {
    //TODO find some data maybe
}

#[repr(C)]
#[derive(Debug)]
struct GridWorldThreaded {
    grid_world_threaded_vtable: &'static GridWorldThreadedVTable,
    unknown: [isize; 287],
    update_region: AABB,
}

#[repr(C)]
#[derive(Debug)]
pub struct GridWorld {
    pub vtable: &'static GridWorldVTable,
    unknown: [isize; 318],
    pub world_update_count: isize,
    pub chunk_map: ChunkMap,
    unknown2: [isize; 41],
    m_thread_impl: *mut GridWorldThreaded,
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
#[allow(clippy::derivable_impls)]
impl Default for ExplosionConfig {
    fn default() -> Self {
        Self {}
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct GridCosmeticParticleConfig {
    //TODO find some data maybe
}
#[allow(clippy::derivable_impls)]
impl Default for GridCosmeticParticleConfig {
    fn default() -> Self {
        Self {}
    }
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
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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
impl Default for CellVTable {
    //ptr is 0x100bb90
    fn default() -> Self {
        Self {
            destroy: 0x70af20 as *const c_void,
            get_cell_type: 0x5b01a0 as *const c_void,
            _field01: 0x70b050 as *const c_void,
            _field02: 0x70b060 as *const c_void,
            _field03: 0x5b01c0 as *const c_void,
            get_color: 0x5b01d0 as *const c_void,
            _field04: 0x70d090 as *const c_void,
            set_color: 0x5b01e0 as *const c_void,
            _field05: 0x5b0200 as *const c_void,
            _field06: 0x5b01f0 as *const c_void,
            _field07: 0x70b070 as *const c_void,
            _field08: 0x70b0b0 as *const c_void,
            get_material: 0x4ac0d0 as *const c_void,
            _field09: 0x70d0b0 as *const c_void,
            _field10: 0x4abf60 as *const c_void,
            _field11: 0x70d1a0 as *const c_void,
            _field12: 0x70d1e0 as *const c_void,
            _field13: 0x70d180 as *const c_void,
            _field14: 0x70cb40 as *const c_void,
            _field15: 0x70cd80 as *const c_void,
            get_position: 0x70cdd0 as *const c_void,
            _field16: 0x70c6e0 as *const c_void,
            _field17: 0x5b01b0 as *const c_void,
            _field18: 0x4abf90 as *const c_void,
            _field19: 0x4abfa0 as *const c_void,
            _field20: 0x70b110 as *const c_void,
            _field21: 0x70b120 as *const c_void,
            _field22: 0x70b160 as *const c_void,
            _field23: 0x70f5b0 as *const c_void,
            is_burning: 0x70f5d0 as *const c_void,
            _field24: 0x70cdf0 as *const c_void,
            _field25: 0x70f750 as *const c_void,
            _field26: 0x4ac0e0 as *const c_void,
            stop_burning: 0x70f7f0 as *const c_void,
            _field27: 0x4ac020 as *const c_void,
            _field28: 0x70f160 as *const c_void,
            _field29: 0x70eaf0 as *const c_void,
            _field30: 0x70ef90 as *const c_void,
            _field31: 0x70f360 as *const c_void,
            remove: 0x70af50 as *const c_void,
            _field32: 0x70b1d0 as *const c_void,
        }
    }
}
#[allow(dead_code)]
impl CellVTable {
    pub fn destroy(&self, cell: *mut Cell) {
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
    pub fn get_cell_type(&self, cell: *mut Cell) -> CellType {
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
    pub fn get_color(&self, cell: *mut Cell) -> Color {
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
    pub fn set_color(&self, cell: *mut Cell, color: Color) {
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
    pub fn get_material(&self, cell: *mut Cell) -> *mut CellData {
        #[cfg(target_arch = "x86")]
        unsafe {
            let ret: *mut CellData;
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
    pub fn get_position(&self, cell: *mut Cell) -> *mut Position {
        #[cfg(target_arch = "x86")]
        unsafe {
            let mut ret: *mut Position;
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
    pub fn is_burning(&self, cell: *mut Cell) -> bool {
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
    pub fn stop_burning(&self, cell: *mut Cell) {
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
    pub fn remove(&self, cell: *mut Cell) {
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
#[derive(Debug, Clone)]
pub struct Cell {
    pub vtable: &'static CellVTable,

    pub hp: isize,
    unknown1: [isize; 2],
    pub is_burning: bool,
    unknown2: [u8; 3],
    pub material: &'static CellData,
}
unsafe impl Sync for Cell {}
unsafe impl Send for Cell {}

#[derive(Default, Debug)]
pub enum FullCell {
    Cell(Cell),
    LiquidCell(LiquidCell),
    #[default]
    None,
}
impl From<&Cell> for FullCell {
    fn from(value: &Cell) -> Self {
        if value.material.cell_type == CellType::Liquid {
            FullCell::LiquidCell(value.get_liquid().clone())
        } else {
            FullCell::Cell(value.clone())
        }
    }
}

impl Cell {
    pub fn get_liquid(&self) -> &LiquidCell {
        unsafe { std::mem::transmute::<&Cell, &LiquidCell>(self) }
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
#[derive(Debug, Clone)]
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
    unknown7: isize,
    unknown8: isize,
}

impl LiquidCell {
    pub fn blob(mat: &'static CellData, vtable: &'static CellVTable) -> Self {
        Self {
            cell: Cell::blob(mat, vtable),
            x: 0,
            y: 0,
            unknown1: 3,
            unknown2: 0,
            is_static: true,
            unknown3: 0,
            unknown4: 0,
            unknown5: 0,
            unknown6: 0,
            color: Default::default(),
            not_color: Default::default(),
            unknown7: 0,
            unknown8: 0,
        }
    }
}

impl Cell {
    fn blob(material: &'static CellData, vtable: &'static CellVTable) -> Self {
        Self {
            vtable,
            hp: material.hp,
            unknown1: [-1000, 0],
            is_burning: false,
            unknown2: [10, 0, 0],
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
    //likely more data
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
    unknown1: [isize; 2],
    pub m_game_world: *mut GameWorld,
    pub m_grid_world: *mut GridWorld,
    pub m_textures: *mut Textures,
    pub m_cell_factory: *mut CellFactory,
    unknown2: [isize; 11],
    pub pause_state: isize,
}
