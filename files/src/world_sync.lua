local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = require("noitapatcher.nsew.world")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}

function world_sync.host_upload()
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    local thread_impl = grid_world.mThreadImpl

    local begin = thread_impl.updated_grid_worlds.begin
    local end_ = begin + thread_impl.chunk_update_count

    local count = thread_impl.chunk_update_count
    -- GamePrint("w update "..count)
    for i = 0, count - 1 do
        local it = begin[i]

        local start_x = it.update_region.top_left.x
        local start_y = it.update_region.top_left.y
        local end_x = it.update_region.bottom_right.x
        local end_y = it.update_region.bottom_right.y

        start_x = start_x - 1
        start_y = start_y - 1
        end_x = end_x + 1
        end_y = end_y + 2

        -- if i < 9 then            
        --     GamePrint(start_x.." "..start_y.." "..end_x.." "..end_y)
        -- end
        local rectangle = rect.Rectangle(start_x, start_y, end_x, end_y)
        rect_optimiser:submit(rectangle)
    end
    for i = 0, tonumber(thread_impl.world_update_params_count) - 1 do
        local wup = thread_impl.world_update_params.begin[i]
        local start_x = wup.update_region.top_left.x
        local start_y = wup.update_region.top_left.y
        local end_x = wup.update_region.bottom_right.x
        local end_y = wup.update_region.bottom_right.y

        local rectangle = rect.Rectangle(start_x, start_y, end_x, end_y)
        rect_optimiser:submit(rectangle)
    end
    if GameGetFrameNum() % 20 == 0 then
        rect_optimiser:scan()

        local result = {}
        for crect in rect.parts(rect_optimiser:iterate(), 256) do
            local area = world.encode_area(chunk_map, crect.left, crect.top, crect.right, crect.bottom, encoded_area)
            if area ~= nil then
                local str = ffi.string(area, world.encoded_size(area))
                result[#result+1] = str
            end
        end
        rect_optimiser:reset()
        return result
    end
end

local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

function world_sync.handle_world_data(world_data)
    local grid_world = world_ffi.get_grid_world()
    for i, datum in ipairs(world_data) do
        -- GamePrint("Decoding world data "..i)
        local header = ffi.cast("struct EncodedAreaHeader const*", ffi.cast('char const*', datum))
        local runs = ffi.cast(PixelRun_const_ptr, ffi.cast("const char*", datum) + ffi.sizeof(world.EncodedAreaHeader))
        world.decode(grid_world, header, runs)
    end
end


return world_sync