use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::MutexGuard,
};

use image::{Pixel, RgbaImage};
use rustc_hash::FxHashMap;
use shared::WorldPos;

use crate::{
    asset::{Asset, AssetConfig, AssetManager},
    color::*,
    net::omni::OmniPeerId,
    player_settings::PlayerAppearance,
    runtime_dir::RuntimeDir,
};

#[rustfmt::skip]
#[allow(clippy::complexity)]
const SPRITES_WITH_MASKS: &[(&[&str], &[(&str, &str)])] = &[
    (
        &["main", "alt", "arm"],
        &[
            ("player_spritesheet"                , "files/system/player/unmodified.png"),
            ("player_lukki_spritesheet"          , "files/system/player/unmodified_lukki.png"),
            ("player_map_icon"                   , "files/system/map/icon.png"),

            ("player_head_sprite"                , "files/system/player/head.png"),
            ("player_knee_sprite"                , "files/system/player/knee.png"),
            ("player_left_arm_sprite"            , "files/system/player/left_arm.png"),
            ("player_left_hand_sprite"           , "files/system/player/left_hand.png"),
            ("player_left_thigh_sprite"          , "files/system/player/left_thigh.png"),
            ("player_right_arm_sprite"           , "files/system/player/right_arm.png"),
            ("player_right_hand_sprite"          , "files/system/player/right_hand.png"),
            ("player_right_thigh_sprite"         , "files/system/player/right_thigh.png"),
            ("player_limb_a_sprite"              , "files/system/player/limb_a.png"),
            ("player_limb_b_sprite"              , "files/system/player/limb_b.png"),
            ("player_torso_sprite"               , "files/system/player/torso.png"),
        ]
    ),
    (
        &["forearm"],
        &[
            ("player_arm_spritesheet", "files/system/player/unmodified_arm.png"),
        ],
    ),
    (
        &["main", "alt", "arm", "border"],
        &[
            ("player_arrow_sprite"               , "files/system/player_arrows/arrow.png"),
            ("player_arrow_notplayer_sprite"     , "files/system/player_arrows/arrow_notplayer.png"),
            ("player_arrow_host_sprite"          , "files/system/player_arrows/arrow_host.png"),
            ("player_arrow_host_notplayer_sprite", "files/system/player_arrows/arrow_host_notplayer.png"),
            ("player_ping_sprite"                , "files/system/player_ping/arrow.png"),
            ("player_cursor_sprite"              , "files/resource/sprites/cursor.png"),
        ],
    ),
    (
        &["main", "alt", "arm", "forearm", "cape", "cape_edge"],
        &[
            ("player_preview_sprite" , "files/resource/sprites/player_preview.png"),
        ]
    ),
    (
        &[],
        &[
            ("player_preview_amulet_sprite"    , "files/resource/sprites/player_preview_amulet.png"),
            ("player_preview_amulet_gem_sprite", "files/resource/sprites/player_preview_amulet_gem.png"),
            ("player_preview_hat_sprite"       , "files/resource/sprites/player_preview_hat.png"),
        ],
    ),
];

#[rustfmt::skip]
const XMLS: &[(&str, &str)] = &[
    ("player_main_xml" , "files/system/player/unmodified.xml"),
    ("player_cape_xml" , "files/system/player/unmodified_cape.xml"),
    ("player_lukki_xml", "files/system/player/unmodified_lukki.xml"),
    ("player_base_xml" , "files/system/player/unmodified_base.xml"),
    ("player_arm_xml"  , "files/system/player/unmodified_arm.xml"),
];

type Rgba = image::Rgba<u8>;

pub fn extend_assets(quantew_install: &Path, asset_manager: &mut AssetManager) {
    for (mask_names, assets) in SPRITES_WITH_MASKS {
        asset_manager.extend(assets.iter().map(|(name, path)| {
            (
                name.to_string(),
                Asset::new(quantew_install.join(path)).with_format_guessed(),
            )
        }));
        for mask_name in *mask_names {
            asset_manager.extend(assets.iter().map(|(name, _)| {
                let path = format!("files/resource/sprite_masks/{name}_{mask_name}_mask.png");
                let asset_name = format!("{name}_{mask_name}_mask");
                (
                    asset_name,
                    Asset::new(quantew_install.join(path)).with_format_guessed(),
                )
            }));
        }
    }

    asset_manager.extend(XMLS.iter().map(|(name, path)| {
        (name.to_string(), {
            Asset::new(quantew_install.join(path)).with_config(AssetConfig::MANUAL)
        })
    }));

    asset_manager.extend([(
        "player_spritesheet_uv".to_string(),
        Asset::new(quantew_install.join("files/system/player/player_uv.png"))
            .with_config(AssetConfig::MANUAL),
    )]);
}

fn write_color_rgb_with_mask_to_image(color: Rgba, mask: &RgbaImage, target: &mut RgbaImage) {
    assert!(
        mask.dimensions() == target.dimensions(),
        "mask and target must be of the same dimensions"
    );
    for (mask_pixel, target_pixel) in mask.pixels().zip(target.pixels_mut()) {
        if mask_pixel.channels()[0..3] == [255, 255, 255] {
            target_pixel.channels_mut()[0] = color.channels()[0];
            target_pixel.channels_mut()[1] = color.channels()[1];
            target_pixel.channels_mut()[2] = color.channels()[2];
        }
    }
}

fn write_overlay_to_image(overlay: &RgbaImage, target: &mut RgbaImage) {
    assert!(
        overlay.dimensions() == target.dimensions(),
        "overlay and target must be of the same dimensions"
    );
    for (overlay_pixel, target_pixel) in overlay.pixels().zip(target.pixels_mut()) {
        // Pixel is not completely transparent
        if overlay_pixel.channels()[3] != 0 {
            *target_pixel = *overlay_pixel;
        }
    }
}

pub fn make_player_preview(assets: &AssetManager, appearance: &PlayerAppearance) -> RgbaImage {
    let mut base = assets
        .get_parsed("player_preview_sprite")
        .as_image()
        .to_rgba8();

    let colors = [
        ("main", appearance.color.main),
        ("alt", appearance.color.alt),
        ("arm", appearance.color.arm),
        ("forearm", appearance.color.forearm),
        ("cape", appearance.color.cape),
        ("cape_edge", appearance.color.cape_edge),
    ];
    for (name, color) in colors {
        let color = Rgba::from(to_u8(color));
        let asset = format!("player_preview_sprite_{name}_mask");
        let mask = assets.get_parsed(&asset).as_image().to_rgba8();
        write_color_rgb_with_mask_to_image(color, &mask, &mut base);
    }

    let cosmetics = [
        ("hat", appearance.cosmetics.hat),
        ("amulet", appearance.cosmetics.amulet),
        ("amulet_gem", appearance.cosmetics.amulet_gem),
    ];
    for (name, enabled) in cosmetics {
        if !enabled {
            continue;
        }
        let asset = format!("player_preview_{name}_sprite");
        let sprite = assets.get_parsed(&asset).as_image().to_rgba8();
        write_overlay_to_image(&sprite, &mut base);
    }
    base
}

fn make<'o>(
    assets: &AssetManager,
    colors: &[(&str, Rgba)],
    sprites: &[(&str, &'o str)],
) -> impl Iterator<Item = (RgbaImage, &'o str)> {
    sprites.iter().map(move |(asset_name, output_suffix)| {
        let mut base = assets.get_parsed(asset_name).as_image().to_rgba8();
        for (color_name, color) in colors {
            let mask = assets
                .get_parsed(&format!("{asset_name}_{color_name}_mask"))
                .as_image()
                .to_rgba8();
            write_color_rgb_with_mask_to_image(*color, &mask, &mut base);
        }
        (base, *output_suffix)
    })
}

#[allow(clippy::type_complexity)]
pub fn create_player_png(
    peer: OmniPeerId,
    assets: &AssetManager,
    runtime_dir: &RuntimeDir,
    quantew_install: &Path,
    appearance: &PlayerAppearance,
    is_host: bool,
    player_map: &mut MutexGuard<FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, RgbaImage)>>,
) {
    let id = peer.as_hex();
    let runtime_dir = runtime_dir.for_peer(peer);

    let icon = make_player_preview(assets, appearance);
    icon.save(runtime_dir.full_path("_icon.png")).unwrap();
    player_map.insert(peer, (None, false, false, icon.clone()));

    let colors = [
        ("main", Rgba::from(to_u8(appearance.color.main))),
        ("alt", Rgba::from(to_u8(appearance.color.alt))),
        ("arm", Rgba::from(to_u8(appearance.color.arm))),
    ];
    let sprites = [
        ("player_spritesheet", ".png"),
        ("player_lukki_spritesheet", "_lukki.png"),
        ("player_knee_sprite", "_knee.png"),
        ("player_limb_a_sprite", "_limb_a.png"),
        ("player_limb_b_sprite", "_limb_b.png"),
        ("player_map_icon", "_map.png"),
    ];
    make(assets, &colors, &sprites)
        .for_each(|(image, suffix)| image.save(runtime_dir.full_path(suffix)).unwrap());

    let ragdoll_sprites = [
        ("player_head_sprite", "_head.png"),
        ("player_left_arm_sprite", "_left_arm.png"),
        ("player_left_hand_sprite", "_left_hand.png"),
        ("player_left_thigh_sprite", "_left_thigh.png"),
        ("player_right_arm_sprite", "_right_arm.png"),
        ("player_right_hand_sprite", "_right_hand.png"),
        ("player_right_thigh_sprite", "_right_thigh.png"),
        ("player_torso_sprite", "_torso.png"),
    ];
    make(assets, &colors, &ragdoll_sprites)
        .for_each(|(image, suffix)| image.save(runtime_dir.full_path(suffix)).unwrap());

    let ragdoll_list_path = runtime_dir.full_path("_ragdoll.txt");
    let mut ragdoll_list_file = fs::File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(ragdoll_list_path)
        .unwrap();
    for (_, suffix) in ragdoll_sprites {
        let path = runtime_dir.noita_path(suffix);
        ragdoll_list_file
            .write_all(path.as_os_str().as_encoded_bytes())
            .unwrap();
        ragdoll_list_file.write_all(b"\n").unwrap();
    }

    make(assets, &colors, &[("player_spritesheet", "_dc.png")]).for_each(|(mut image, suffix)| {
        for px in image.pixels_mut() {
            px.channels_mut()[3] = px.channels()[3].min(64)
        }
        image.save(runtime_dir.full_path(suffix)).unwrap();
    });

    make(
        assets,
        &[("forearm", Rgba::from(to_u8(appearance.color.forearm)))],
        &[("player_arm_spritesheet", "_arm.png")],
    )
    .for_each(|(image, suffix)| image.save(runtime_dir.full_path(suffix)).unwrap());

    let colors = [
        ("main", Rgba::from(to_u8(appearance.color.main))),
        ("alt", Rgba::from(to_u8(appearance.color.alt))),
        ("arm", Rgba::from(to_u8(appearance.color.arm))),
        (
            "border",
            // I suppose this is what the original code does
            // even though the config make no sense.
            if appearance.invert_border {
                invert(Rgba::from(to_u8(appearance.color.main)))
            } else {
                Rgba::from([0, 0, 0, 255])
            },
        ),
    ];
    let sprites = [
        (
            if is_host {
                "player_arrow_host_sprite"
            } else {
                "player_arrow_sprite"
            },
            "_arrow.png",
        ),
        ("player_ping_sprite", "_ping.png"),
        ("player_cursor_sprite", "_cursor.png"),
    ];
    make(assets, &colors, &sprites)
        .for_each(|(image, suffix)| image.save(runtime_dir.full_path(suffix)).unwrap());

    edit_by_insert_line(
        assets.get("player_cape_xml").unwrap().path(),
        &runtime_dir.full_path("_cape.xml"),
        &[
            (
                16,
                format!(
                    "cloth_color=\"0xFF{}\"",
                    rgb_to_hex(to_u8(appearance.color.cape))
                ),
            ),
            (
                16,
                format!(
                    "cloth_color_edge=\"0xFF{}\"",
                    rgb_to_hex(to_u8(appearance.color.cape_edge))
                ),
            ),
        ],
    );
    edit_by_insert_line(
        assets.get("player_main_xml").unwrap().path(),
        &runtime_dir.full_path("_dc.xml"),
        &[(
            1,
            format!(
                "filename=\"{}\"",
                runtime_dir.noita_path("_dc.png").to_str().unwrap()
            ),
        )],
    );
    edit_by_insert_line(
        assets.get("player_main_xml").unwrap().path(),
        &runtime_dir.full_path(".xml"),
        &[(
            1,
            format!(
                "filename=\"{}\"",
                &runtime_dir.noita_path(".png").to_str().unwrap(),
            ),
        )],
    );
    edit_by_replacing(
        assets.get("player_lukki_xml").unwrap().path(),
        &runtime_dir.full_path("_lukki.xml"),
        &[(
            "MARKER_LUKKI_PNG",
            runtime_dir
                .noita_path("_lukki.png")
                .to_str()
                .unwrap()
                .to_string(),
        )],
    );
    edit_by_replacing(
        assets.get("player_base_xml").unwrap().path(),
        &runtime_dir.full_path("_base.xml"),
        &[
            (
                "MARKER_HAT2_ENABLED",
                if appearance.cosmetics.hat {
                    "image_file=\"data/enemies_gfx/player_hat2.xml\""
                } else {
                    ""
                }
                .into(),
            ),
            (
                "MARKER_AMULET_ENABLED",
                (if appearance.cosmetics.amulet {
                    "image_file=\"data/enemies_gfx/player_amulet.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_AMULET_GEM_ENABLED",
                (if appearance.cosmetics.amulet_gem {
                    "image_file=\"data/enemies_gfx/player_amulet_gem.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_MAIN_SPRITE",
                runtime_dir.noita_path(".xml").to_str().unwrap().to_string(),
            ),
            (
                "MARKER_LUKKI_SPRITE",
                runtime_dir
                    .noita_path("_lukki.xml")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "MARKER_ARM_SPRITE",
                runtime_dir
                    .noita_path("_arm.xml")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "MARKER_CAPE",
                runtime_dir
                    .noita_path("_cape.xml")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "RAGDOLL_MARKER",
                runtime_dir
                    .noita_path("_ragdoll.txt")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
        ],
    );
    edit_by_insert_line(
        assets.get("player_arm_xml").unwrap().path(),
        &runtime_dir.full_path("_arm.xml"),
        &[(
            1,
            format!(
                "filename=\"{}\"",
                &runtime_dir
                    .noita_path("_arm.png")
                    .to_str()
                    .unwrap()
                    .to_string()
            ),
        )],
    );
    // mods.quant.ew.files.system.player.tmp.0000000000000001.png
    fs::copy(
        assets.get("player_spritesheet_uv").unwrap().path(),
        quantew_install
            .join("data/generated/sprite_uv_maps")
            .join(format!("mods.quant.ew.files.system.player.tmp.{id}.png")),
    )
    .unwrap();
}

fn edit_by_insert_line(input: &Path, output: &Path, to_insert: &[(usize, String)]) {
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
    for (i, line) in to_insert {
        lines.insert(*i, line.clone());
    }
    let mut file = File::create(output).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
}

fn edit_by_replacing(input: &Path, output: &Path, replace_pair: &[(&'static str, String)]) {
    // Probably not a very good idea to unwrap here. Mod files should exist by this point, but...
    let mut contents = fs::read_to_string(input).unwrap();
    for pair in replace_pair {
        contents = contents.replace(pair.0, &pair.1);
    }
    fs::write(output, contents).unwrap();
}
