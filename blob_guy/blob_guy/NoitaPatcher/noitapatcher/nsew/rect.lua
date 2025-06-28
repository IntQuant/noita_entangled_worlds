---Rectangle utilities.
---@module 'noitapatcher.nsew.rect'

---@class Rect
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

---@class Rectangle_fields
---@field top integer
---@field bottom integer
---@field right integer
---@field left integer

---@alias Rectangle Rectangle_mt | Rectangle_fields

---@class Optimiser_fields
---@field top integer
---@field bottom integer
---@field right integer
---@field left integer

---@alias Optimiser Optimiser_fields | Optimiser_mt

---@class Rectangle_mt
local Rectangle_mt_index = {
    ---@param r Rectangle
    ---@return integer
    area = function(r)
        return (r.right - r.left) * (r.bottom - r.top)
    end,
    ---@param r Rectangle
    ---@return integer
    height = function(r)
        return r.bottom - r.top
    end,
    ---@param r Rectangle
    ---@return integer
    width = function(r)
        return r.right - r.left
    end,
}
local Rectangle_mt = {
    __index = Rectangle_mt_index,
}

---@type fun(left, top, right, bottom): Rectangle
---@diagnostic disable-next-line: assign-type-mismatch
rect.Rectangle = ffi.metatype("struct nsew_rectangle", Rectangle_mt)

---Given an iterator that returns rectangles, return an iterator where the
---rectangle extents never exceed `size`.
---@param iterator fun(): Rectangle? returning rectangles
---@param size integer maximum width and height
---@return fun(): Rectangle? rectangles where the extents never exceed `size`
function rect.parts(iterator, size)
    local region
    local posx
    local posy
    return function()
        if region == nil then
            region = iterator()
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

---@class Optimiser_mt
local Optimiser_mt_index = {
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
        return native_dll.lib.rectangle_optimiser_size(opt.impl)
    end,
    get = function(opt, index)
        return native_dll.lib.rectangle_optimiser_get(opt.impl, index)
    end,
    iterate = function(opt)
        local size = native_dll.lib.rectangle_optimiser_size(opt.impl)
        local index = 0
        return function()
            if index >= size then
                return nil
            end

            local ret = native_dll.lib.rectangle_optimiser_get(opt.impl, index)
            index = index + 1
            return ret
        end
    end,
}

local Optimiser_mt = {
    __gc = function(opt)
        native_dll.lib.rectangle_optimiser_delete(opt.impl)
    end,
    __index = Optimiser_mt_index,
}

---@type fun(unknown): Optimiser
---@diagnostic disable-next-line: assign-type-mismatch
rect.Optimiser = ffi.metatype("struct lua_nsew_rectangle_optimiser", Optimiser_mt)

---Create a new rectangle Optimiser
---@return Optimiser optimiser
function rect.Optimiser_new()
    return rect.Optimiser(native_dll.lib.rectangle_optimiser_new())
end

return rect
