local ew_api = dofile_once("mods/quant.ew/files/api/ew_api.lua")

-- Needs a unique, but preferably short identifier.
local rpc = ew_api.new_rpc_namespace("api_example")

-- Make the next rpc be delivered reliably.
-- This means that it will be called exactly once (unless a disconnection happens), and will be ordered the same way.
-- E. g. if you call rpc.rpc1(), rpc.rpc1(), rpc.rpc2() that are reliable, they will get called in the same order on other clients.
rpc.opts_reliable()
-- This rpc will also get called locally.
rpc.opts_everywhere()
function rpc.send_hi()
    GamePrint("Hi from " .. ew_api.rpc_player_data().name)
end

local module = {}

function module.on_world_update()
    if GameGetFrameNum() % 60 == 0 then
        GamePrint("Hi from api example!")
        rpc.send_hi()
    end
end

return module
-- Real implementation examples:
-- https://github.com/Conga0/Apotheosis/pull/45/commits/ebc4544a3614736d6e394a953ba58fa229e6aa81x

-- tags:
-- player_unit, only on the local player, besides when a client fires a wand, just for the fire function
-- ew_peer, on all players when not polied
-- ew_client, on all players besides your own, always
-- ew_synced, syncs an entity that isn't synced otherwise, needed before next preupdate loop after entity is spawned
-- ew_synced_var, syncs a variable storage component by name, needs to be on a synced entity
-- ew_no_enemy_sync, dont sync an entity that would be synced otherwise, needed before next preupdate loop after entity is spawned
-- ew_des, denotes if entity is synced by des, if unloaded and reloaded the entity will be culled on next preupdate loop
-- ew_des_lua, on some lua components related to des
-- ew_immortal, on some entitys you dont have auth over to fix damage numbers
-- ew_notplayer, on notplayer
-- ew_ghost_rnd, for something to sync perk ghosts rng
-- ew_projectile_position_sync, syncs a projectile by des
-- ew_unique, kills an entity if another one was spawned within the same chunk on any peer, by filename

-- var components(by name not tag):
-- ew_peer_id, on all peers, contains peerid in value_string
-- ew_gid_lid, on all* synced entitys by des, value_string has gid(constant across reload),
--   value_int has lid(constant across clients but not reload),
--   value_bool is true iff you have authority over entity
-- ew_frame_num, on mom or perk ghosts to sync position nicely
-- ew_rng, on some entitys(dice) to sync rng
-- ew_transmutation, on certain spells to sync rng
