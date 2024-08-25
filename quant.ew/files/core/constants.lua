local module = {}

--TODO fix chained entities spawning repeatably on clients
--TODO fix minecart entity disapearing on spawn
module.phys_sync_allowed = {
    -- Starting prop
    ["data/entities/props/physics_skateboard.xml"] = true,
--  ["data/entities/props/physics_minecart.xml"] = true,
--  ["data/entities/props/physics/minecart.xml"] = true,
    ["data/entities/props/physics_cart.xml"] = true,


    ["data/entities/buildings/statue_hand_1.xml"] = true,
    ["data/entities/buildings/statue_hand_2.xml"] = true,
    ["data/entities/buildings/statue_hand_3.xml"] = true,


    ["data/entities/props/physics_brewing_stand.xml"] = true,
    ["data/entities/props/physics_propane_tank.xml"] = true,
    ["data/entities/props/physics_box_explosive.xml"] = true,
    ["data/entities/props/physics_crate.xml"] = true,

    ["data/entities/props/physics_barrel_oil.xml"] = true,
    ["data/entities/props/physics_barrel_radioactive.xml"] = true,
    ["data/entities/props/physics_seamine.xml"] = true,
--  ["data/entities/props/suspended_tank_radioactive.xml"] = true,
--  ["data/entities/props/suspended_tank_acid.xml"] = true,

    ["data/entities/props/physics_box_harmless.xml"] = true,
    ["data/entities/props/physics_tubelamp.xml"] = true,

    ["data/entities/props/physics_torch_stand.xml"] = true,
    ["data/entities/props/vault_apparatus_01.xml"] = true,
    ["data/entities/props/vault_apparatus_02.xml"] = true,
    ["data/entities/props/physics_pressure_tank.xml"] = true,

    ["data/entities/props/crystal_red.xml"] = true,
    ["data/entities/props/crystal_pink.xml"] = true,
    ["data/entities/props/crystal_green.xml"] = true,

    ["data/entities/props/physics_vase.xml"] = true,
    ["data/entities/props/physics_vase_longleg.xml"] = true,


    ["data/entities/props/physics_sun_rock.xml"] = true,
    ["data/entities/props/physics_darksun_rock.xml"] = true,

    ["data/entities/props/music_machines/music_machine_00.xml"] = true,
    ["data/entities/props/music_machines/music_machine_01.xml"] = true,
    ["data/entities/props/music_machines/music_machine_02.xml"] = true,
    ["data/entities/props/music_machines/music_machine_03.xml"] = true,
    -- HM statues
    ["data/entities/props/temple_statue_01.xml"] = true,
    ["data/entities/props/temple_statue_01_green.xml"] = true,
    ["data/entities/props/temple_statue_02.xml"] = true,
--  ["data/entities/props/physics/temple_lantern.xml"] = true,
--  ["data/entities/buildings/physics_worm_deflector_base.xml"] = true,
--  ["data/entities/buildings/physics_worm_deflector_crystal.xml"] = true,
--  ["data/entities/misc/greed_curse/greed_crystal.xml"] = true,
--  ["data/entities/props/physics/lantern_small.xml"] = true,

    -- Traps
    ["data/entities/props/physics_trap_circle_acid.xml"] = true,
    ["data/entities/props/physics_trap_electricity_enabled.xml"] = true,
    ["data/entities/props/physics_trap_electricity.xml"] = true,
    ["data/entities/props/physics_trap_ignite_enabled.xml"] = true,
    ["data/entities/props/physics_trap_ignite.xml"] = true,
    ["data/entities/props/physics/trap_circle_acid.xml"] = true,
    ["data/entities/props/physics/trap_electricity_enabled.xml"] = true,
    ["data/entities/props/physics/trap_electricity_suspended.xml"] = true,
    ["data/entities/props/physics/trap_electricity.xml"] = true,
    ["data/entities/props/physics/trap_ignite_enabled.xml"] = true,
    ["data/entities/props/physics/trap_ignite.xml"] = true,
    ["data/entities/props/physics/trap_laser_enabled_left.xml"] = true,
    ["data/entities/props/physics/trap_laser_enabled.xml"] = true,
    ["data/entities/props/physics/trap_laser_toggling_left.xml"] = true,
    ["data/entities/props/physics/trap_laser_toggling.xml"] = true,
    ["data/entities/props/physics/trap_laser.xml"] = true,
}

module.interned_index_to_filename = {}
module.interned_filename_to_index = {}

for line in string.gmatch(ModTextFileGetContent("mods/quant.ew/files/resource/interned_filenames.txt"), "(.-)\n") do
    -- print("Interned", line)
    table.insert(module.interned_index_to_filename, line)
    module.interned_filename_to_index[line] = #module.interned_index_to_filename
end

return module