--- World read / write functionality.
---@module 'noitapatcher.nsew.world'
local world = {}

local ffi = require("ffi")
local world_ffi = require("noitapatcher.nsew.world_ffi")

print("get_cell: " .. tostring(world_ffi.get_cell))

local C = ffi.C

ffi.cdef([[

enum ENCODE_CONST {
    // Maximum amount of runs 128*128 pixels can result in, plus one just in case.
    PIXEL_RUN_MAX = 16385,

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

world.last_material_id = 0

world.EncodedAreaHeader = ffi.typeof("struct EncodedAreaHeader")
world.PixelRun = ffi.typeof("struct PixelRun")
world.EncodedArea = ffi.typeof("struct EncodedArea")

local pliquid_cell = ffi.typeof("struct CLiquidCell*")

--- Total bytes taken up by the encoded area
-- @tparam EncodedArea encoded_area
-- @treturn int total number of bytes that encodes the area
-- @usage
-- local data = ffi.string(area, world.encoded_size(area))
-- peer:send(data)
function world.encoded_size(encoded_area)
    return (ffi.sizeof(world.EncodedAreaHeader) + encoded_area.header.pixel_run_count * ffi.sizeof(world.PixelRun))
end

--[[
--- Encode the given rectangle of the world
-- The rectangle defined by {`start_x`, `start_y`, `end_x`, `end_y`} must not
-- exceed 256 in width or height.
-- @param chunk_map
-- @tparam int start_x coordinate
-- @tparam int start_y coordinate
-- @tparam int end_x coordinate
-- @tparam int end_y coordinate
-- @tparam EncodedArea encoded_area memory to use, if nil this function allocates its own memory
-- @return returns an EncodedArea or nil if the area could not be encoded
-- @see decode
function world.encode_area(chunk_map, start_x, start_y, end_x, end_y, encoded_area)
    start_x = ffi.cast('int32_t', start_x)
    start_y = ffi.cast('int32_t', start_y)
    end_x = ffi.cast('int32_t', end_x)
    end_y = ffi.cast('int32_t', end_y)

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
]]

--- Encode the given rectangle of the world
-- The rectangle defined by {`start_x`, `start_y`, `end_x`, `end_y`} must not
-- exceed 256 in width or height.
-- @param chunk_map
-- @tparam int start_x coordinate
-- @tparam int start_y coordinate
-- @tparam int end_x coordinate
-- @tparam int end_y coordinate
-- @tparam EncodedArea encoded_area memory to use, if nil this function allocates its own memory
-- @return returns an EncodedArea or nil if the area could not be encoded
-- @see decode
function world.encode_area(chunk_map, start_x_ini, start_y_ini, end_x_ini, end_y_ini, encoded_area)
    start_x = ffi.cast('int32_t', start_x_ini)
    start_y = ffi.cast('int32_t', start_y_ini)
    end_x = ffi.cast('int32_t', end_x_ini)
    end_y = ffi.cast('int32_t', end_y_ini)

    encoded_area = encoded_area or world.EncodedArea()

    local width = end_x - start_x
    local height = end_y - start_y

    if width <= 0 or height <= 0 then
        print("Invalid world part, negative dimension")
        return nil
    end

    if width > 128 or height > 128 then
        print("Invalid world part, dimension greater than 128")
        return nil
    end

    encoded_area.header.x = start_x
    encoded_area.header.y = start_y
    encoded_area.header.width = width - 1
    encoded_area.header.height = height - 1

    encoded_area.header.pixel_run_count = ewext.encode_area(start_x_ini, start_y_ini, end_x_ini, end_y_ini, tonumber(ffi.cast("intptr_t", encoded_area.pixel_runs)))
    return encoded_area
end

--local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

--- Load an encoded area back into the world.
-- @param grid_world
-- @tparam EncodedAreaHeader header header of the encoded area
-- @param received pointer or ffi array of PixelRun from the encoded area
-- @see encode_area
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

                if new_material == -1 then
                    goto next_pixel
                end

                if ppixel[0] ~= nil then
                    local pixel = ppixel[0]
                    local cell_type = pixel.vtable.get_cell_type(pixel)
                    if cell_type == C.CELL_TYPE_SOLID then
                        goto next_pixel
                    end
                    current_material = world_ffi.get_material_id(pixel.vtable.get_material(pixel))

                    if new_material ~= current_material then
                        world_ffi.remove_cell(grid_world, pixel, x, y, false)
                    end
                end

                if current_material ~= new_material and new_material ~= 0 then
                    if new_material > world.last_material_id then
                        goto next_pixel
                    end
                    local mat_ptr = world_ffi.get_material_ptr(new_material)
                    if mat_ptr == nil then
                        goto next_pixel
                    end
                    local pixel = world_ffi.construct_cell(grid_world, x, y, mat_ptr, nil)
                    if pixel == nil then
                        -- TODO: This can happen when the material texture has a
                        -- transparent pixel at the given coordinate. There's
                        -- probably a better way to deal with this, but for now
                        -- we skip positions like this.
                        goto next_pixel
                    end

                    local cell_type = pixel.vtable.get_cell_type(pixel)

                    if cell_type == C.CELL_TYPE_LIQUID then
                        local liquid_cell = ffi.cast(pliquid_cell, pixel)
                        liquid_cell.is_static = bit.band(flags, C.CELL_TYPE_LIQUID) == C.LIQUID_FLAG_STATIC
                    end

                    ppixel[0] = pixel
                end
            end

            ::next_pixel::

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