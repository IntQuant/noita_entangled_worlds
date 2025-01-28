local old_LoadPixelScene = LoadPixelScene

function LoadPixelScene(materials_filename, colors_filename, x, y, background_file, skip_biome_checks, skip_edge_textures, color_to_material_table, background_z_index, load_even_if_duplicate)
    old_LoadPixelScene(materials_filename, colors_filename, x, y, background_file, skip_biome_checks, skip_edge_textures, color_to_material_table, background_z_index, load_even_if_duplicate)
    -- TODO there are a couple more parameters, tho they don't seem to be used in vanilla
    CrossCall("ew_sync_pixel_scene", materials_filename, colors_filename, x, y, background_file, skip_biome_checks, skip_edge_textures)
end
