local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = require("noitapatcher.nsew.world")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

local module = {}

local KEY_WORLD_FRAME = 0
local KEY_WORLD_END = 1

local CHUNK_SIZE = 128

function module.on_world_update()
    if GameGetFrameNum() % 30 ~= 6 then
        return
    end
    for peer_id, player_data in pairs(ctx.players) do
        local x, y = EntityGetTransform(player_data.entity)
        if x ~= nil and y ~= nil then
            net.proxy_send("peer_pos", peer_id.." "..x.." "..y)
        end
    end
end

return module
