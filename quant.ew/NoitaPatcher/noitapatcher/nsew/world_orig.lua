---@diagnostic disable: cast-local-type
---World read / write functionality.
---@module 'noitapatcher.nsew.world'
---@class World
local world = {}

local ffi = require("ffi")
local world_ffi = require("noitapatcher.nsew.world_ffi")

local C = ffi.C

ffi.cdef([[

enum ENCODE_CONST {
    PIXEL_RUN_MAX = 4096,

    LIQUID_FLAG_STATIC = 1,
};

struct __attribute__ ((__packed__)) EncodedAreaHeader {
    int32_t x;
    int32_t y;
    uint8_t width;
    uint8_t height;

    uint16_t pixel_run_count;
};

struct __attribute__ ((__packed__)) PixelRun {
    uint16_t length;
    int16_t material;
    uint8_t flags;
};

struct __attribute__ ((__packed__)) EncodedArea {
    struct EncodedAreaHeader header;
    struct PixelRun pixel_runs[PIXEL_RUN_MAX];
};

]])

---@class PixelRun
---@field flags integer
---@field material integer
---@field length integer

---@class EncodedAreaHeader
---@field x integer
---@field y integer
---@field width integer
---@field height integer
---@field pixel_run_count integer

---@class EncodedArea
---@field header EncodedAreaHeader
---@field pixel_runs PixelRun[] a pointer

world.EncodedAreaHeader = ffi.typeof("struct EncodedAreaHeader")
world.PixelRun = ffi.typeof("struct PixelRun")
---@type fun(): EncodedArea
---@diagnostic disable-next-line: assign-type-mismatch
world.EncodedArea = ffi.typeof("struct EncodedArea")

local pliquid_cell = ffi.typeof("struct CLiquidCell*")

---Total bytes taken up by the encoded area
---@param encoded_area EncodedArea
---@return integer total number of bytes that encodes the area
---```lua
---local data = ffi.string(area, world.encoded_size(area))
---peer:send(data)
---```
function world.encoded_size(encoded_area)
    return (ffi.sizeof(world.EncodedAreaHeader) + encoded_area.header.pixel_run_count * ffi.sizeof(world.PixelRun))
end

---Encode the given rectangle of the world
---The rectangle defined by {`start_x`, `start_y`, `end_x`, `end_y`} must not exceed 256 in width or height.
---@param chunk_map unknown
---@param start_x integer coordinate
---@param start_y integer coordinate
---@param end_x integer coordinate
---@param end_y integer coordinate
---@param encoded_area EncodedArea? memory to use, if nil this function allocates its own memory
---@return EncodedArea? encoded_area returns an EncodedArea or nil if the area could not be encoded
---@see decode
function world.encode_area(chunk_map, start_x, start_y, end_x, end_y, encoded_area)
    start_x = ffi.cast('int32_t', start_x)
    start_y = ffi.cast('int32_t', start_y)
    end_x = ffi.cast('int32_t', end_x)
    end_y = ffi.cast('int32_t', end_y)
    ---@cast start_x integer
    ---@cast start_y integer
    ---@cast end_x integer
    ---@cast end_x integer

    encoded_area = encoded_area or world.EncodedArea()

    local width = end_x - start_x
    local height = end_y - start_y

    if width <= 0 or height <= 0 then
        print("Invalid world part, negative dimension")
        return nil
    end

    if width > 256 or height > 256 then
        print("Invalid world part, dimension greater than 256")
        return nil
    end

    encoded_area.header.x = start_x
    encoded_area.header.y = start_y
    encoded_area.header.width = width - 1
    encoded_area.header.height = height - 1

    local run_count = 1

    local current_run = encoded_area.pixel_runs[0]
    local run_length = 0
    local current_material = 0
    local current_flags = 0

    local y = start_y
    while y < end_y do
        local x = start_x
        while x < end_x do
            local material_number = 0
            local flags = 0

            local ppixel = world_ffi.get_cell(chunk_map, x, y)
            local pixel = ppixel[0]
            if pixel ~= nil then
                local cell_type = pixel.vtable.get_cell_type(pixel)

                if cell_type ~= C.CELL_TYPE_SOLID then
                    local material_ptr = pixel.vtable.get_material(pixel)
                    material_number = world_ffi.get_material_id(material_ptr)
                end

                if cell_type == C.CELL_TYPE_LIQUID then
                    local liquid_cell = ffi.cast(pliquid_cell, pixel)
                    if liquid_cell.is_static then
                        flags = bit.bor(flags, C.LIQUID_FLAG_STATIC)
                    end
                end
            end

            if x == start_x and y == start_y then
                -- Initial run
                current_material = material_number
                current_flags = flags
            elseif current_material ~= material_number or current_flags ~= flags then
                -- Next run
                current_run.length = run_length - 1
                current_run.material = current_material
                current_run.flags = current_flags

                if run_count == C.PIXEL_RUN_MAX then
                    print("Area too complicated to encode")
                    return nil
                end

                current_run = encoded_area.pixel_runs[run_count]
                run_count = run_count + 1

                run_length = 0
                current_material = material_number
                current_flags = flags
            end

            run_length = run_length + 1

            x = x + 1
        end
        y = y + 1
    end

    current_run.length = run_length - 1
    current_run.material = current_material
    current_run.flags = current_flags

    encoded_area.header.pixel_run_count = run_count

    return encoded_area
end

local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

---Load an encoded area back into the world.
---@param grid_world unknown
---@param header EncodedAreaHeader header of the encoded area
---@param pixel_runs PixelRun[] or ffi array of PixelRun from the encoded area
---@see encode_area
function world.decode(grid_world, header, pixel_runs)
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)

    local top_left_x = header.x
    local top_left_y = header.y
    local width = header.width + 1
    local height = header.height + 1
    local bottom_right_x = top_left_x + width
    local bottom_right_y = top_left_y + height

    local current_run_ix = 0
    local current_run = pixel_runs[current_run_ix]
    local new_material = current_run.material
    local flags = current_run.flags
    local left = current_run.length + 1

    local y = top_left_y
    while y < bottom_right_y do
        local x = top_left_x
        while x < bottom_right_x do
            if world_ffi.chunk_loaded(chunk_map, x, y) then
                local ppixel = world_ffi.get_cell(chunk_map, x, y)
                local current_material = 0

                if ppixel[0] ~= nil then
                    local pixel = ppixel[0]
                    current_material = world_ffi.get_material_id(pixel.vtable.get_material(pixel))

                    if new_material ~= current_material then
                        world_ffi.remove_cell(grid_world, pixel, x, y, false)
                    end
                end

                if current_material ~= new_material and new_material ~= 0 then
                    local pixel = world_ffi.construct_cell(grid_world, x, y, world_ffi.get_material_ptr(new_material), nil)
                    local cell_type = pixel.vtable.get_cell_type(pixel)

                    if cell_type == C.CELL_TYPE_LIQUID then
                        local liquid_cell = ffi.cast(pliquid_cell, pixel)
                        liquid_cell.is_static = bit.band(flags, C.CELL_TYPE_LIQUID) == C.LIQUID_FLAG_STATIC
                    end

                    ppixel[0] = pixel
                end
            end

            left = left - 1
            if left <= 0 then
                current_run_ix = current_run_ix + 1
                if current_run_ix >= header.pixel_run_count then
                    -- No more runs, done
                    assert(x == bottom_right_x - 1)
                    assert(y == bottom_right_y - 1)
                    return
                end

                current_run = pixel_runs[current_run_ix]
                new_material = current_run.material
                flags = current_run.flags
                left = current_run.length + 1
            end

            x = x + 1
        end
        y = y + 1
    end
end

return world
