use std::{
    ffi::OsString,
    fs::{self, File, remove_file},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::MutexGuard,
};

use image::{Pixel, RgbaImage};
use rustc_hash::FxHashMap;
use shared::WorldPos;

use crate::{
    asset::{Asset, AssetManager},
    color::*,
    net::omni::OmniPeerId,
    player_settings::{PlayerAppearance, PlayerColor},
};

#[rustfmt::skip]
const SPRITES: &[(&str, &str)] = &[
    ("player_spritesheet"                , "files/system/player/unmodified.png"),
    ("player_lukki_spritesheet"          , "files/system/player/unmodified_lukki.png"),

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

    ("player_arrow_sprite"               , "files/system/player_arrows/arrow.png"),
    ("player_arrow_notplayer_sprite"     , "files/system/player_arrows/arrow_notplayer.png"),
    ("player_arrow_host_sprite"          , "files/system/player_arrows/arrow_host.png"),
    ("player_arrow_host_notplayer_sprite", "files/system/player_arrows/arrow_host_notplayer.png"),
    ("player_ping_sprite"                , "files/system/player_ping/arrow.png"),
    ("player_cursor_sprite"              , "files/resource/sprites/cursor.png"),
];

#[rustfmt::skip]
const SPRITES_MANUAL_MASKS: &[(&str, &str)] = &[
    ("player_arm_spritesheet"             , "files/system/player/unmodified_arm.png"),
    ("player_arm_spritesheet_forearm_mask", "files/resource/sprite_masks/player_arm_spritesheet_forearm_mask.png"),
];

#[rustfmt::skip]
const PLAYER_PREIVEW_SPRITES: &[(&str, &str)] = &[
    ("player_preview_sprite"           , "files/resource/sprites/player_preview.png"),

    ("player_preview_amulet_sprite"    , "files/resource/sprites/player_preview_amulet.png"),
    ("player_preview_amulet_gem_sprite", "files/resource/sprites/player_preview_amulet_gem.png"),
    ("player_preview_hat_sprite"       , "files/resource/sprites/player_preview_hat.png"),

    ("player_preview_main_mask"        , "files/resource/sprite_masks/player_preview_main_mask.png"),
    ("player_preview_alt_mask"         , "files/resource/sprite_masks/player_preview_alt_mask.png"),
    ("player_preview_arm_mask"         , "files/resource/sprite_masks/player_preview_arm_mask.png"),
    ("player_preview_forearm_mask"     , "files/resource/sprite_masks/player_preview_forearm_mask.png"),
    ("player_preview_cape_mask"        , "files/resource/sprite_masks/player_preview_cape_mask.png"),
    ("player_preview_cape_edge_mask"   , "files/resource/sprite_masks/player_preview_cape_edge_mask.png"),
];

type Rgba = image::Rgba<u8>;

pub fn extend_assets(quantew_install: &Path, assets: &mut AssetManager) {
    let map_to_asset = |(name, path): &(&str, &str)| {
        (
            name.to_string(),
            Asset::new(quantew_install.join(path)).with_format_guessed(),
        )
    };
    assets.extend(SPRITES.iter().map(map_to_asset));
    let sprite_masks = SPRITES.iter().flat_map(|(name, _)| {
        ["alt", "arm", "main"].map(|mask_name| {
            let path = format!("files/resource/sprite_masks/{name}_{mask_name}_mask.png");
            (
                format!("{name}_{mask_name}_mask"),
                Asset::new(quantew_install.join(path)).with_format_guessed(),
            )
        })
    });
    assets.extend(sprite_masks);
    assets.extend(SPRITES_MANUAL_MASKS.iter().map(map_to_asset));
    assets.extend(PLAYER_PREIVEW_SPRITES.iter().map(map_to_asset));
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

pub fn arrows_path(path: PathBuf, is_host: bool) -> (PathBuf, PathBuf, PathBuf) {
    let parent = path.parent().unwrap();
    let p = parent.join("player_arrows");
    let o = parent.join("player_ping");
    (
        if is_host {
            p.join("arrow_host.png")
        } else {
            p.join("arrow.png")
        },
        o.join("arrow.png"),
        parent.join("map/icon.png"),
    )
}

pub fn cursor_path(path: PathBuf) -> PathBuf {
    path.parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("resource/sprites/cursor.png")
}

pub fn replace_color(image: &mut RgbaImage, main: Rgba, alt: Rgba, arm: Rgba) {
    let target_main = Rgba::from([155, 111, 154, 255]);
    let target_alt = Rgba::from([127, 84, 118, 255]);
    let target_arm = Rgba::from([89, 67, 84, 255]);
    for pixel in image.pixels_mut() {
        if compare_rgb(*pixel, target_main) {
            set_rgb(pixel, main);
        } else if compare_rgb(*pixel, target_alt) {
            set_rgb(pixel, alt);
        } else if compare_rgb(*pixel, target_arm) {
            set_rgb(pixel, arm);
        }
    }
}

pub fn replace_color_opt(image: &mut RgbaImage, main: Rgba, alt: Rgba, arm: Rgba, inv: bool) {
    let target_main = Rgba::from([155, 111, 154, 255]);
    let target_alt = Rgba::from([127, 84, 118, 255]);
    let target_arm = Rgba::from([89, 67, 84, 255]);
    let target_border = Rgba::from([0, 0, 0, 255]);
    for pixel in image.pixels_mut() {
        if compare_rgb(*pixel, target_main) {
            set_rgb(pixel, main);
        } else if compare_rgb(*pixel, target_alt) {
            set_rgb(pixel, alt);
        } else if compare_rgb(*pixel, target_arm) {
            set_rgb(pixel, arm);
        } else if inv && compare_rgb(*pixel, target_border) {
            set_rgb(pixel, invert(main));
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
        let asset = format!("player_preview_{name}_mask");
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

pub fn create_arm(arm: Rgba) -> RgbaImage {
    let hand = Rgba::from([219, 192, 103, 255]);
    let mut img = RgbaImage::new(5, 15);
    for (i, pixel) in img.pixels_mut().enumerate() {
        match i {
            10 | 11 | 17 | 18 | 35 | 40 | 41 | 55 | 56 | 57 | 58 => *pixel = arm,
            19 | 42 | 59 => *pixel = hand,
            _ => {}
        }
    }
    img
}

fn replace_colors(path: PathBuf, save: PathBuf, rgb: &PlayerColor) {
    let mut img = image::open(path).unwrap().into_rgba8();
    replace_color(
        &mut img,
        Rgba::from(to_u8(rgb.main)),
        Rgba::from(to_u8(rgb.alt)),
        Rgba::from(to_u8(rgb.arm)),
    );
    img.save(save).unwrap();
}

fn replace_colors_opt(path: PathBuf, save: PathBuf, rgb: &PlayerColor, inv: bool) {
    let mut img = image::open(path).unwrap().into_rgba8();
    replace_color_opt(
        &mut img,
        Rgba::from(to_u8(rgb.main)),
        Rgba::from(to_u8(rgb.alt)),
        Rgba::from(to_u8(rgb.arm)),
        inv,
    );
    img.save(save).unwrap();
}

#[allow(clippy::type_complexity)]
pub fn create_player_png(
    peer: OmniPeerId,
    mod_path: &Path,
    player_path: &Path,
    assets: &AssetManager,
    rgb: &PlayerAppearance,
    is_host: bool,
    player_map: &mut MutexGuard<FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, RgbaImage)>>,
) {
    // let icon = get_player_skin(
    //     image::open(player_path)
    //         .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
    //         .crop(1, 1, 7, 16)
    //         .into_rgba8(),
    //     *rgb,
    // );
    let icon = make_player_preview(assets, rgb);
    player_map.insert(peer, (None, false, false, icon.clone()));
    let inv = rgb.invert_border;
    let id = peer.as_hex();
    let cosmetics = rgb.cosmetics;
    let rgb = rgb.color;
    let tmp_path = player_path.parent().unwrap();
    let (arrows_path, ping_path, map_icon) = arrows_path(tmp_path.into(), is_host);
    let cursor_path = cursor_path(tmp_path.into());
    let player_lukki = tmp_path.join("unmodified_lukki.png");
    icon.save(tmp_path.join(format!("tmp/{id}_icon.png")))
        .unwrap();
    replace_colors(
        player_path.into(),
        tmp_path.join(format!("tmp/{id}.png")),
        &rgb,
    );
    {
        let mut img = image::open(player_path).unwrap().into_rgba8();
        replace_color(
            &mut img,
            Rgba::from(to_u8(rgb.main)),
            Rgba::from(to_u8(rgb.alt)),
            Rgba::from(to_u8(rgb.arm)),
        );
        for px in img.pixels_mut() {
            px.0[3] = px.0[3].min(64)
        }
        img.save(tmp_path.join(format!("tmp/{id}_dc.png"))).unwrap();
    }
    replace_colors(
        player_lukki,
        tmp_path.join(format!("tmp/{id}_lukki.png")),
        &rgb,
    );
    replace_colors_opt(
        arrows_path,
        tmp_path.join(format!("tmp/{id}_arrow.png")),
        &rgb,
        inv,
    );
    replace_colors_opt(
        ping_path,
        tmp_path.join(format!("tmp/{id}_ping.png")),
        &rgb,
        inv,
    );
    replace_colors_opt(
        cursor_path,
        tmp_path.join(format!("tmp/{id}_cursor.png")),
        &rgb,
        inv,
    );
    replace_colors(
        tmp_path.join("knee.png"),
        tmp_path.join(format!("tmp/{id}_knee.png")),
        &rgb,
    );
    replace_colors(
        tmp_path.join("limb_a.png"),
        tmp_path.join(format!("tmp/{id}_limb_a.png")),
        &rgb,
    );
    replace_colors(
        tmp_path.join("limb_b.png"),
        tmp_path.join(format!("tmp/{id}_limb_b.png")),
        &rgb,
    );
    replace_colors(map_icon, tmp_path.join(format!("tmp/{id}_map.png")), &rgb);
    let ragdoll_path = tmp_path.join(format!("tmp/{id}_ragdoll.txt"));
    if ragdoll_path.exists() {
        remove_file(ragdoll_path.clone()).unwrap()
    }
    let mut ragdoll = File::create(ragdoll_path).unwrap();
    let mut files = String::new();
    for s in [
        "head.png",
        "left_hand.png",
        "left_arm.png",
        "left_thigh.png",
        "right_hand.png",
        "right_arm.png",
        "right_thigh.png",
        "torso.png",
    ]
    .iter()
    .rev()
    {
        let f = tmp_path.join(s);
        replace_colors(f, tmp_path.join(format!("tmp/{id}_ragdoll_{s}")), &rgb);
        files = format!("{files}mods/quant.ew/files/system/player/tmp/{id}_ragdoll_{s}\n");
    }
    ragdoll.write_all(files.as_bytes()).unwrap();
    let img = create_arm(Rgba::from(to_u8(rgb.forearm)));
    let path = tmp_path.join(format!("tmp/{id}_arm.png"));
    img.save(path).unwrap();
    edit_nth_line(
        tmp_path.join("unmodified_cape.xml").into(),
        tmp_path.join(format!("tmp/{id}_cape.xml")).into_os_string(),
        vec![16, 16],
        vec![
            format!("cloth_color=\"0xFF{}\"", rgb_to_hex(to_u8(rgb.cape))),
            format!(
                "cloth_color_edge=\"0xFF{}\"",
                rgb_to_hex(to_u8(rgb.cape_edge))
            ),
        ],
    );
    edit_nth_line(
        tmp_path.join("unmodified.xml").into(),
        tmp_path.join(format!("tmp/{id}_dc.xml")).into_os_string(),
        vec![1],
        vec![format!(
            "filename=\"mods/quant.ew/files/system/player/tmp/{}_dc.png\"",
            id
        )],
    );
    edit_nth_line(
        tmp_path.join("unmodified.xml").into(),
        tmp_path.join(format!("tmp/{id}.xml")).into_os_string(),
        vec![1],
        vec![format!(
            "filename=\"mods/quant.ew/files/system/player/tmp/{}.png\"",
            id
        )],
    );
    edit_by_replacing(
        tmp_path.join("unmodified_lukki.xml"),
        tmp_path.join("tmp/".to_owned() + &id.clone() + "_lukki.xml"),
        &[(
            "MARKER_LUKKI_PNG",
            format!("mods/quant.ew/files/system/player/tmp/{id}_lukki.png"),
        )],
    );
    edit_by_replacing(
        tmp_path.join("unmodified_base.xml"),
        tmp_path.join("tmp/".to_owned() + &id.clone() + "_base.xml"),
        &[
            (
                "MARKER_HAT2_ENABLED",
                (if cosmetics.hat {
                    "image_file=\"data/enemies_gfx/player_hat2.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_AMULET_ENABLED",
                (if cosmetics.amulet {
                    "image_file=\"data/enemies_gfx/player_amulet.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_AMULET_GEM_ENABLED",
                (if cosmetics.amulet_gem {
                    "image_file=\"data/enemies_gfx/player_amulet_gem.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_MAIN_SPRITE",
                format!("mods/quant.ew/files/system/player/tmp/{id}.xml"),
            ),
            (
                "MARKER_LUKKI_SPRITE",
                format!("mods/quant.ew/files/system/player/tmp/{id}_lukki.xml"),
            ),
            (
                "MARKER_ARM_SPRITE",
                format!("mods/quant.ew/files/system/player/tmp/{id}_arm.xml"),
            ),
            (
                "MARKER_CAPE",
                format!("mods/quant.ew/files/system/player/tmp/{id}_cape.xml"),
            ),
            (
                "RAGDOLL_MARKER",
                format!("mods/quant.ew/files/system/player/tmp/{id}_ragdoll.txt"),
            ),
        ],
    );
    edit_nth_line(
        tmp_path.join("unmodified_arm.xml").into(),
        tmp_path.join(format!("tmp/{id}_arm.xml")).into_os_string(),
        vec![1],
        vec![format!(
            "filename=\"mods/quant.ew/files/system/player/tmp/{}_arm.png\"",
            id
        )],
    );
    // mods.quant.ew.files.system.player.tmp.0000000000000001.png
    fs::copy(
        tmp_path.join("player_uv.png"),
        mod_path
            .join("data/generated/sprite_uv_maps/")
            .join(format!("mods.quant.ew.files.system.player.tmp.{id}.png")),
    )
    .unwrap();
}

fn edit_nth_line(path: OsString, exit: OsString, v: Vec<usize>, newline: Vec<String>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<String>>();
    for (i, line) in v.iter().zip(newline) {
        lines.insert(*i, line);
    }
    let mut file = File::create(exit).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
}

fn edit_by_replacing(
    path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
    replace_pair: &[(&'static str, String)],
) {
    // Probably not a very good idea to unwrap here. Mod files should exist by this point, but...
    let mut contents = fs::read_to_string(path).unwrap();
    for pair in replace_pair {
        contents = contents.replace(pair.0, &pair.1);
    }
    fs::write(out_path, contents).unwrap();
}
