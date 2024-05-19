--- Rectangle utilities.
---@module 'noitapatcher.nsew.rect'

local rect = {}

local ffi = require("ffi")
local native_dll = require("noitapatcher.nsew.native_dll")

ffi.cdef([[

struct nsew_rectangle {
    int32_t left;
    int32_t top;
    int32_t right;
    int32_t bottom;
};


struct nsew_rectangle_optimiser;

struct nsew_rectangle_optimiser* rectangle_optimiser_new();
void rectangle_optimiser_delete(struct nsew_rectangle_optimiser* rectangle_optimiser);
void rectangle_optimiser_reset(struct nsew_rectangle_optimiser* rectangle_optimiser);
void rectangle_optimiser_submit(struct nsew_rectangle_optimiser* rectangle_optimiser, struct nsew_rectangle* rectangle);
void rectangle_optimiser_scan(struct nsew_rectangle_optimiser* rectangle_optimiser);
int32_t rectangle_optimiser_size(const struct nsew_rectangle_optimiser* rectangle_optimiser);
const struct nsew_rectangle* rectangle_optimiser_get(const struct nsew_rectangle_optimiser* rectangle_optimiser, int32_t index);


struct lua_nsew_rectangle_optimiser {
    struct nsew_rectangle_optimiser* impl;
};

]])

local Rectangle_mt = {
    __index = {
        area = function(r)
            return (r.right - r.left) * (r.bottom - r.top)
        end,
        height = function(r)
            return r.bottom - r.top
        end,
        width = function(r)
            return r.right - r.left
        end,
    },
}
rect.Rectangle = ffi.metatype("struct nsew_rectangle", Rectangle_mt)

--- Given an iterator that returns rectangles, return an iterator where the
--- rectangle extents never exceed `size`.
-- @param it iterator returning squares
-- @tparam int size maximum width and height
-- @return rectangle iterator where the extents never exceed `size`
function rect.parts(it, size)
    local region
    local posx
    local posy
    return function()
        if region == nil then
            region = it()
            if region == nil then
                return nil
            end
            posx = region.left
            posy = region.top
        end

        local endx = math.min(posx + size, region.right)
        local endy = math.min(posy + size, region.bottom)

        local ret = rect.Rectangle(posx, posy, endx, endy)

        -- Setup for next iteration: place to the right, wraparound, or
        -- we're done with this region.
        if endx ~= region.right then
            posx = endx
        elseif endy ~= region.bottom then
            posx = region.left
            posy = endy
        else
            region = nil
        end

        return ret
    end
end

local Optimiser_mt = {
    __gc = function(opt)
        native_dll.lib.rectangle_optimiser_delete(opt.impl)
    end,

    __index = {
        submit = function(opt, rectangle)
            native_dll.lib.rectangle_optimiser_submit(opt.impl, rectangle)
        end,
        scan = function(opt)
            native_dll.lib.rectangle_optimiser_scan(opt.impl)
        end,
        reset = function(opt)
            native_dll.lib.rectangle_optimiser_reset(opt.impl)
        end,
        size = function(opt)
            return native_dll.lib.rectangle_optimiser_size()
        end,
        get = function(opt, index)
            return native_dll.lib.rectangle_optimiser_get(index)
        end,
        iterate = function(opt)
            local size = native_dll.lib.rectangle_optimiser_size(opt.impl)
            local index = 0
            return function()
                if index >= size then
                    return nil
                end

                ret = native_dll.lib.rectangle_optimiser_get(opt.impl, index)
                index = index + 1
                return ret
            end
        end,
    }
}
rect.Optimiser = ffi.metatype("struct lua_nsew_rectangle_optimiser", Optimiser_mt)

--- Create a new rectangle Optimiser
-- @treturn Optimiser empty optimiser
function rect.Optimiser_new()
    return rect.Optimiser(native_dll.lib.rectangle_optimiser_new())
end

return rect
