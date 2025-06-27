---@diagnostic disable: assign-type-mismatch
---Noita world functionality exposed.
---@module 'noitapatcher.nsew.world_ffi'

---@class WorldFFI
local world_ffi = {}

local ffi = require("ffi")

local np = require("noitapatcher")
local world_info = np.GetWorldInfo()

if not world_info then
    error("Couldn't get world info from NoitaPatcher.")
end

local gg_ptr = world_info.game_global

ffi.cdef([[

typedef void* __thiscall placeholder_memfn(void*);

struct Position {
    int x;
    int y;
};

struct Colour {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
};

struct AABB {
    struct Position top_left;
    struct Position bottom_right;
};

struct std_string { /* VC++ std::string */
    char *buffer;
    char sso_buffer[12];
    size_t size;
    size_t capacity;
};

typedef enum cell_type {
    none=0,
    liquid=1,
    gas=2,
    solid=3,
    fire=4,
    invalid=4294967295
} cell_type;

struct CellData {
    struct std_string name;
    struct std_string ui_name;
    int material_type;
    int id_2;
    enum cell_type cell_type;
    int platform_type;
    unsigned int wang_color;
    int gfx_glow;
    unsigned int gfx_glow_color;
    char unknown1[24];
    unsigned int default_primary_colour;
    char unknown2[36];
    bool cell_holes_in_texture;
    bool stainable;
    bool burnable;
    bool on_fire;
    int fire_hp;
    int autoignition_temperature;
    int _100_minus_autoignition_temp;
    int temperature_of_fire;
    int generates_smoke;
    int generates_flames;
    bool requires_oxygen;
    char padding1[3];
    struct std_string on_fire_convert_to_material;
    int on_fire_convert_to_material_id;
    struct std_string on_fire_flame_material;
    int on_fire_flame_material_id;
    struct std_string on_fire_smoke_material;
    int on_fire_smoke_material_id;
    struct ConfigExplosion *explosion_config;
    int durability;
    int crackability;
    bool electrical_conductivity;
    bool slippery;
    char padding2[2];
    float stickyness;
    struct std_string cold_freezes_to_material;
    struct std_string warmth_melts_to_material;
    int warmth_melts_to_material_id;
    int cold_freezes_to_material_id;
    int16_t cold_freezes_chance_rev;
    int16_t warmth_melts_chance_rev;
    bool cold_freezes_to_dont_do_reverse_reaction;
    char padding3[3];
    int lifetime;
    int hp;
    float density;
    bool liquid_sand;
    bool liquid_slime;
    bool liquid_static;
    bool liquid_stains_self;
    int liquid_sticks_to_ceiling;
    float liquid_gravity;
    int liquid_viscosity;
    int liquid_stains;
    unsigned int liquid_stains_custom_color;
    float liquid_sprite_stain_shaken_drop_chance;
    float liquid_sprite_stain_ignited_drop_chance;
    int8_t liquid_sprite_stains_check_offset;
    char padding4[3];
    float liquid_sprite_stains_status_threshold;
    float liquid_damping;
    float liquid_flow_speed;
    bool liquid_sand_never_box2d;
    char unknown7[3];
    int8_t gas_speed;
    int8_t gas_upwards_speed;
    int8_t gas_horizontal_speed;
    int8_t gas_downwards_speed;
    float solid_friction;
    float solid_restitution;
    float solid_gravity_scale;
    int solid_static_type;
    float solid_on_collision_splash_power;
    bool solid_on_collision_explode;
    bool solid_on_sleep_convert;
    bool solid_on_collision_convert;
    bool solid_on_break_explode;
    bool solid_go_through_sand;
    bool solid_collide_with_self;
    char padding5[2];
    struct std_string solid_on_collision_material;
    int solid_on_collision_material_id;
    struct std_string solid_break_to_type;
    int solid_break_to_type_id;
    struct std_string convert_to_box2d_material;
    int convert_to_box2d_material_id;
    int vegetation_full_lifetime_growth;
    struct std_string vegetation_sprite;
    bool vegetation_random_flip_x_scale;
    char padding6[3];
    char unknown11[12];
    float wang_noise_percent;
    float wang_curvature;
    int wang_noise_type;
    char unknown12[12];
    bool danger_fire;
    bool danger_radioactive;
    bool danger_poison;
    bool danger_water;
    char unknown13[24];
    bool always_ignites_damagemodel;
    bool ignore_self_reaction_warning;
    char padding7[2];
    char unknown14[12];
    float audio_size_multiplier;
    bool audio_is_soft;
    char padding8[3];
    char unknown15[8];
    bool show_in_creative_mode;
    bool is_just_particle_fx;
    char padding9[2];
    // struct grid_CosmeticParticleConfig *ParticleEffect;
};

enum CellType {
    CELL_TYPE_NONE = 0,
    CELL_TYPE_LIQUID = 1,
    CELL_TYPE_GAS = 2,
    CELL_TYPE_SOLID = 3,
    CELL_TYPE_FIRE = 4,
};

struct Cell_vtable {
    void (__thiscall *destroy)(struct Cell*, char dealloc);
    enum CellType (__thiscall *get_cell_type)(struct Cell*);
    void* field2_0x8;
    void* field3_0xc;
    void* field4_0x10;
    struct Colour (__thiscall *get_colour)(struct Cell*);
    void* field6_0x18;
    void (__thiscall *set_colour)(struct Cell*, struct Colour);
    void* field8_0x20;
    void* field9_0x24;
    void* field10_0x28;
    void* field11_0x2c;
    struct CellData* (__thiscall *get_material)(void *);
    void* field13_0x34;
    void* field14_0x38;
    void* field15_0x3c;
    void* field16_0x40;
    void* field17_0x44;
    void* field18_0x48;
    void* field19_0x4c;
    struct Position * (__thiscall *get_position)(void *, struct Position *);
    void* field21_0x54;
    void* field22_0x58;
    void* field23_0x5c;
    void* field24_0x60;
    void* field25_0x64;
    void* field26_0x68;
    void* field27_0x6c;
    void* field28_0x70;
    bool (__thiscall *is_burning)(struct Cell*);
    void* field30_0x78;
    void* field31_0x7c;
    void* field32_0x80;
    void (__thiscall *stop_burning)(struct Cell*);
    void* field34_0x88;
    void* field35_0x8c;
    void* field36_0x90;
    void* field37_0x94;
    void* field38_0x98;
    void (__thiscall *remove)(struct Cell*);
    void* field40_0xa0;
};

// In the Noita code this would be the ICellBurnable class
struct Cell {
    struct Cell_vtable* vtable;

    int hp;
    char unknown1[8];
    bool is_burning;
    char unknown2[3];
    uintptr_t material_ptr;
};

struct CLiquidCell {
    struct Cell cell;
    int x;
    int y;
    char unknown1;
    char unknown2;
    bool is_static;
    char unknown3;
    int unknown4[3];
    struct Colour colour;
    unsigned not_colour;
};

typedef struct Cell (*cell_array)[0x40000];

struct ChunkMap {
    int unknown[2];
    cell_array* (*cells)[0x40000];
    int unknown2[8];
};

struct GridWorld_vtable {
    placeholder_memfn* unknown[3];
    struct ChunkMap* (__thiscall *get_chunk_map)(struct GridWorld* this);
    placeholder_memfn* unknown2[30];
};

struct GridWorld {
    struct GridWorld_vtable* vtable;
    int unknown[318];
    int world_update_count;
    struct ChunkMap chunk_map;
    int unknown2[41];
    struct GridWorldThreadImpl* mThreadImpl;
};

struct GridWorldThreaded_vtable;

struct GridWorldThreaded {
    struct GridWorldThreaded_vtable* vtable;
    int unknown[287];
    struct AABB update_region;
};

struct vec_pGridWorldThreaded {
    struct GridWorldThreaded** begin;
    struct GridWorldThreaded** end_;
    struct GridWorldThreaded** capacity_end;
};

struct WorldUpdateParams {
    struct AABB update_region;
    int unknown;
    struct GridWorldThreaded* grid_world_threaded;
};

struct vec_WorldUpdateParams {
    struct WorldUpdateParams* begin;
    struct WorldUpdateParams* end_;
    struct WorldUpdateParams* capacity_end;
};

struct GridWorldThreadImpl {
    int chunk_update_count;
    struct vec_pGridWorldThreaded updated_grid_worlds;

    int world_update_params_count;
    struct vec_WorldUpdateParams world_update_params;

    int grid_with_area_count;
    struct vec_pGridWorldThreaded with_area_grid_worlds;

    int another_count;
    int another_vec[3];

    int some_kind_of_ptr;
    int some_kind_of_counter;

    int last_vec[3];
};

typedef struct Cell** __thiscall get_cell_f(struct ChunkMap*, int x, int y);
typedef bool __thiscall chunk_loaded_f(struct ChunkMap*, int x, int y);

typedef void __thiscall remove_cell_f(struct GridWorld*, void* cell, int x, int y, bool);
typedef struct Cell* __thiscall construct_cell_f(struct GridWorld*, int x, int y, void* material_ptr, void* memory);

]])

--local function check_celldata_field(f, o)
--    local offset = ffi.offsetof("struct CellData", f)
--    assert(offset == o, "Expected field " .. f .. " to be at offset " .. o)
--end
--
--check_celldata_field("wang_color", 0x40)
--check_celldata_field("generates_flames", 0xa4)
--check_celldata_field("durability", 0x104)
--check_celldata_field("cold_freezes_to_material", 0x114)
--check_celldata_field("liquid_sand", 0x160)
--check_celldata_field("liquid_sprite_stain_ignited_drop_chance", 0x17c)
--check_celldata_field("gas_horizontal_speed", 0x196)
--check_celldata_field("solid_on_sleep_convert", 0x1ad)
--check_celldata_field("solid_break_to_type", 0x1d0)
--check_celldata_field("vegetation_sprite", 0x20c)
--check_celldata_field("wang_noise_type", 0x23c)
--check_celldata_field("ignore_self_reaction_warning", 0x269)
--check_celldata_field("is_just_particle_fx", 0x289)

---@class ChunkMap pointer type
---@class GridWorld pointer type
---@class CellData pointer type
---@class Cell pointer type

---Access a pixel in the world.
---You can write a cell created from world_ffi.construct_cell to this pointer to add a cell into the world.
---If there's already a cell at this position, make sure to call world_ffi.remove_cell first.
---@type fun(chunk_map: ChunkMap, x: integer, y: integer): Cell
world_ffi.get_cell = ffi.cast("get_cell_f*", world_info.get_cell)

---Remove a cell from the world. bool return has unknown meaning.
---@type fun(grid_world: GridWorld, cell: Cell, x: integer, y: integer): boolean
world_ffi.remove_cell = ffi.cast("remove_cell_f*", world_info.remove_cell)

---Create a new cell. If memory is null pointer it will allocate its own memory.
---@type fun(grid_world: GridWorld, x: integer, y: integer, material: CellData, memory: ffi.cdata*)
world_ffi.construct_cell = ffi.cast("construct_cell_f*", world_info.construct_cell)

---Check if a chunk is loaded. x and y are world coordinates.
---```lua
---if world_ffi.chunk_loaded(chunk_map, x, y) then
---  local cell = world_ffi.get_cell(chunk_map, x, y)
---  ..
---```
---@type fun(chunk_map: ChunkMap, x: integer, y: integer): boolean
world_ffi.chunk_loaded = ffi.cast("chunk_loaded_f*", world_info.chunk_loaded)

world_ffi.Position = ffi.typeof("struct Position")
world_ffi.Colour = ffi.typeof("struct Colour")
world_ffi.AABB = ffi.typeof("struct AABB")
world_ffi.CellType = ffi.typeof("enum CellType")
world_ffi.Cell = ffi.typeof("struct Cell")
world_ffi.CLiquidCell = ffi.typeof("struct CLiquidCell")
world_ffi.ChunkMap = ffi.typeof("struct ChunkMap")
world_ffi.GridWorld = ffi.typeof("struct GridWorld")
world_ffi.GridWorldThreaded = ffi.typeof("struct GridWorldThreaded")
world_ffi.WorldUpdateParams = ffi.typeof("struct WorldUpdateParams")
world_ffi.GridWorldThreadImpl = ffi.typeof("struct GridWorldThreadImpl")

---Get the grid world.
---@return GridWorld
function world_ffi.get_grid_world()
    local game_global = ffi.cast("void*", gg_ptr)
    local world_data = ffi.cast("void**", ffi.cast("char*", game_global) + 0xc)[0]
    local grid_world = ffi.cast("struct GridWorld**", ffi.cast("char*", world_data) + 0x44)[0]
    return grid_world
end

local celldata_size = 0x290
local CellData_ptr = ffi.typeof("struct CellData*")

---Turn a standard material id into a material pointer.
---@param id integer material id that is used in the standard Noita functions
---@return CellData material to internal material data (aka cell data).
---```lua
---local gold_ptr = world_ffi.get_material_ptr(CellFactory_GetType("gold"))
---```
function world_ffi.get_material_ptr(id)
    local game_global = ffi.cast("char*", gg_ptr)
    local cell_factory = ffi.cast("char**", (game_global + 0x18))[0]
    local begin = ffi.cast("char**", cell_factory + 0x18)[0]
    local ptr = begin + celldata_size * id
    return ffi.cast(CellData_ptr, ptr) --[[@as CellData]]
end

---Turn a material pointer into a standard material id.
---@param material CellData to a material (aka cell data)
---@return integer material id that is accepted by standard Noita functions such as `CellFactory_GetUIName` and `ConvertMaterialOnAreaInstantly`.
---```lua
---local mat_id = world_ffi.get_material_id(cell.vtable.get_material(cell))
---```
---See: `world_ffi.get_material_ptr`
function world_ffi.get_material_id(material)
    local game_global = ffi.cast("char*", gg_ptr)
    local cell_factory = ffi.cast("char**", (game_global + 0x18))[0]
    local begin = ffi.cast("char**", cell_factory + 0x18)[0]
    local offset = ffi.cast("char*", material) - begin
    return offset / celldata_size
end

return world_ffi
