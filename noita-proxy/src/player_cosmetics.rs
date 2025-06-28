use crate::lang::tr;
use crate::net::omni::OmniPeerId;
use crate::{PlayerAppearance, PlayerColor, PlayerPicker};
use bitcode::{Decode, Encode};
use eframe::egui;
use eframe::egui::color_picker::{Alpha, color_picker_color32};
use eframe::egui::{Color32, Image, TextureHandle, TextureOptions, Ui};
use image::DynamicImage::ImageRgba8;
use image::{ImageBuffer, Pixel, Rgba, RgbaImage};
use rustc_hash::FxHashMap;
use shared::WorldPos;
use std::ffi::OsString;
use std::fs::{self, File, remove_file};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
pub fn player_path(path: PathBuf) -> PathBuf {
    path.join("files/system/player/unmodified.png")
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

pub fn compare_rgb(a: Rgba<u8>, b: Rgba<u8>) -> bool {
    a.channels()[0..3] == b.channels()[0..3]
}

pub fn set_rgb(a: &mut Rgba<u8>, b: Rgba<u8>) {
    for i in 0..3 {
        a.channels_mut()[i] = b.channels()[i];
    }
}

pub fn replace_color(image: &mut RgbaImage, main: Rgba<u8>, alt: Rgba<u8>, arm: Rgba<u8>) {
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

fn invert(mut a: Rgba<u8>) -> Rgba<u8> {
    for i in 0..3 {
        a.channels_mut()[i] = 255 - a.channels()[i];
    }
    a
}

pub fn replace_color_opt(
    image: &mut RgbaImage,
    main: Rgba<u8>,
    alt: Rgba<u8>,
    arm: Rgba<u8>,
    inv: bool,
) {
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

fn f_to_u(n: f64) -> u8 {
    255.0_f64.min(0.0_f64.max(n.round())) as u8
}

fn to_u8(c: [f64; 4]) -> [u8; 4] {
    [f_to_u(c[0]), f_to_u(c[1]), f_to_u(c[2]), f_to_u(c[3])]
}

pub fn make_player_image(image: &mut RgbaImage, colors: PlayerColor) {
    let target_main = Rgba::from([155, 111, 154, 255]);
    let target_alt = Rgba::from([127, 84, 118, 255]);
    let target_arm = Rgba::from([89, 67, 84, 255]);
    let main = Rgba::from(to_u8(colors.player_main));
    let alt = Rgba::from(to_u8(colors.player_alt));
    let arm = Rgba::from(to_u8(colors.player_arm));
    let cape = Rgba::from(to_u8(colors.player_cape));
    let cape_edge = Rgba::from(to_u8(colors.player_cape_edge));
    let forearm = Rgba::from(to_u8(colors.player_forearm));
    for (i, pixel) in image.pixels_mut().enumerate() {
        if *pixel == target_main {
            *pixel = main
        } else if *pixel == target_alt {
            *pixel = alt
        } else if *pixel == target_arm {
            *pixel = arm
        } else {
            match i {
                29 | 36 | 43 => *pixel = forearm,
                22 => *pixel = Rgba::from([219, 192, 103, 255]),
                105 | 98 | 91 | 84 | 112 | 99 | 92 | 85 | 78 | 106 | 86 | 79 | 72 | 93 => {
                    *pixel = cape
                }
                70 | 77 | 64 | 71 | 65 => *pixel = cape_edge,
                _ => {}
            }
        }
    }
}

pub fn add_cosmetics(image: &mut RgbaImage, cosmetics: &[bool]) {
    for (i, pixel) in image.pixels_mut().enumerate() {
        match i {
            2 | 4 | 6 if cosmetics[0] => *pixel = Rgba::from([255, 244, 140, 255]),
            9 | 13 if cosmetics[0] => *pixel = Rgba::from([191, 141, 65, 255]),
            10..=12 if cosmetics[0] => *pixel = Rgba::from([255, 206, 98, 255]),
            54 if cosmetics[2] => *pixel = Rgba::from([255, 242, 162, 255]),
            60 if cosmetics[2] => *pixel = Rgba::from([255, 227, 133, 255]),
            61 if cosmetics[2] => *pixel = Rgba::from([255, 94, 38, 255]),
            62 | 68 if cosmetics[2] => *pixel = Rgba::from([247, 188, 86, 255]),
            45 | 53 if cosmetics[1] => *pixel = Rgba::from([247, 188, 86, 255]),
            54 if cosmetics[1] => *pixel = Rgba::from([255, 227, 133, 255]),
            61 if cosmetics[1] => *pixel = Rgba::from([255, 242, 162, 255]),
            55 if cosmetics[1] => *pixel = Rgba::from([198, 111, 57, 255]),
            48 if cosmetics[1] => *pixel = Rgba::from([177, 97, 48, 149]),
            _ => {}
        }
    }
}

pub fn get_lch(color: [f64; 4]) -> (f64, f64, f64) {
    let c = (color[1].powi(2) + color[2].powi(2)).sqrt();
    let h = color[2].atan2(color[1]);
    (color[0], c, h)
}

pub fn rgb_to_oklch(color: &mut [f64; 4]) {
    let mut l = 0.4122214694707629 * color[0]
        + 0.5363325372617349 * color[1]
        + 0.0514459932675022 * color[2];
    let mut m = 0.2119034958178251 * color[0]
        + 0.6806995506452344 * color[1]
        + 0.1073969535369405 * color[2];
    let mut s = 0.0883024591900564 * color[0]
        + 0.2817188391361215 * color[1]
        + 0.6299787016738222 * color[2];

    l = l.cbrt();
    m = m.cbrt();
    s = s.cbrt();

    color[0] = 0.210454268309314 * l + 0.7936177747023054 * m - 0.0040720430116193 * s;
    color[1] = 1.9779985324311684 * l - 2.42859224204858 * m + 0.450593709617411 * s;
    color[2] = 0.0259040424655478 * l + 0.7827717124575296 * m - 0.8086757549230774 * s;
}

fn oklch_to_rgb(color: &mut [f64; 4]) {
    let mut l = color[0] + 0.3963377773761749 * color[1] + 0.2158037573099136 * color[2];
    let mut m = color[0] - 0.1055613458156586 * color[1] - 0.0638541728258133 * color[2];
    let mut s = color[0] - 0.0894841775298119 * color[1] - 1.2914855480194092 * color[2];

    l = l.powi(3);
    m = m.powi(3);
    s = s.powi(3);

    color[0] = 4.07674163607596 * l - 3.3077115392580635 * m + 0.2309699031821046 * s;
    color[1] = -1.2684379732850317 * l + 2.6097573492876887 * m - 0.3413193760026572 * s;
    color[2] = -0.0041960761386754 * l - 0.7034186179359363 * m + 1.7076146940746116 * s;
}

fn shift_hue_by(color: &mut [f64; 4], diff: f64) {
    let tau = std::f64::consts::TAU;
    let diff = tau * diff / 360.0;
    let (_, c, hue) = get_lch(*color);
    let mut new_hue = (hue + diff) % tau;
    if new_hue.is_sign_negative() {
        new_hue += tau;
    }
    color[1] = c * new_hue.cos();
    color[2] = c * new_hue.sin();
}

pub fn shift_hue(diff: f64, color: &mut [f64; 4]) {
    rgb_to_oklch(color);
    shift_hue_by(color, diff);
    oklch_to_rgb(color);
}

pub fn player_skin_display_color_picker(
    ui: &mut Ui,
    player_color: &mut PlayerColor,
    player_picker: &PlayerPicker,
) {
    match player_picker {
        PlayerPicker::PlayerMain => {
            color_picker(ui, &mut player_color.player_main);
        }
        PlayerPicker::PlayerAlt => {
            color_picker(ui, &mut player_color.player_alt);
        }
        PlayerPicker::PlayerArm => {
            color_picker(ui, &mut player_color.player_arm);
        }
        PlayerPicker::PlayerForearm => {
            color_picker(ui, &mut player_color.player_forearm);
        }
        PlayerPicker::PlayerCape => {
            color_picker(ui, &mut player_color.player_cape);
        }
        PlayerPicker::PlayerCapeEdge => {
            color_picker(ui, &mut player_color.player_cape_edge);
        }
        PlayerPicker::None => {}
    }
}

pub fn color_picker(ui: &mut Ui, color: &mut [f64; 4]) {
    let mut rgb = Color32::from_rgb(f_to_u(color[0]), f_to_u(color[1]), f_to_u(color[2]));
    if color_picker_color32(ui, &mut rgb, Alpha::Opaque) {
        *color = [rgb.r() as f64, rgb.g() as f64, rgb.b() as f64, 255.0]
    }
}

pub fn player_select_current_color_slot(
    ui: &mut Ui,
    appearance: &mut PlayerAppearance,
    game_save_path: Option<PathBuf>,
) {
    let mut clicked = false;
    let last = appearance.player_picker.clone();
    ui.scope(|ui| {
        ui.set_max_width(100.0);
        ui.vertical_centered_justified(|ui| {
            if ui.button(tr("Main-color")).clicked() {
                clicked = true;
                appearance.player_picker = PlayerPicker::PlayerMain
            }
            if ui.button(tr("Alt-color")).clicked() {
                clicked = true;
                appearance.player_picker = PlayerPicker::PlayerAlt
            }
            if ui.button(tr("Arm-color")).clicked() {
                clicked = true;
                appearance.player_picker = PlayerPicker::PlayerArm
            }
            if ui.button(tr("Forearm-color")).clicked() {
                clicked = true;
                appearance.player_picker = PlayerPicker::PlayerForearm
            }
            if ui.button(tr("Cape-color")).clicked() {
                clicked = true;
                appearance.player_picker = PlayerPicker::PlayerCape
            }
            if ui.button(tr("Cape-edge-color")).clicked() {
                clicked = true;
                appearance.player_picker = PlayerPicker::PlayerCapeEdge
            }
            if let Some(path) = game_save_path {
                let flags = path.join("save00/persistent/flags");
                let hat = flags.join("secret_hat").exists();
                let amulet = flags.join("secret_amulet").exists();
                let gem = flags.join("secret_amulet_gem").exists();
                ui.checkbox(&mut appearance.invert_border, "Invert border");
                if hat {
                    ui.checkbox(&mut appearance.cosmetics.0, tr("Crown"));
                }
                if amulet {
                    ui.checkbox(&mut appearance.cosmetics.1, tr("Amulet"));
                }
                if gem {
                    ui.checkbox(&mut appearance.cosmetics.2, tr("Gem"));
                }
            }
        });
    });
    if clicked && last == appearance.player_picker {
        appearance.player_picker = PlayerPicker::None
    }
}

pub fn get_player_skin(
    mut img: RgbaImage,
    colors: PlayerPngDesc,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    add_cosmetics(&mut img, &colors.cosmetics);
    make_player_image(&mut img, colors.colors);
    img
}

pub fn display_player_skin(ui: &mut Ui, img: ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f32) {
    let texture: TextureHandle = ui.ctx().load_texture(
        "player",
        egui::ColorImage::from_rgba_unmultiplied(
            [img.width() as usize, img.height() as usize],
            &img.into_raw(),
        ),
        TextureOptions::NEAREST,
    );
    ui.add(Image::new(&texture).fit_to_original_size(scale));
}

pub fn create_arm(arm: Rgba<u8>) -> RgbaImage {
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

#[derive(Clone, Copy, Debug, Decode, Encode, Default)]
pub struct PlayerPngDesc {
    pub(crate) cosmetics: [bool; 3],
    pub(crate) colors: PlayerColor,
    pub(crate) invert_border: bool,
}

fn replace_colors(path: PathBuf, save: PathBuf, rgb: &PlayerColor) {
    let mut img = image::open(path).unwrap().into_rgba8();
    replace_color(
        &mut img,
        Rgba::from(to_u8(rgb.player_main)),
        Rgba::from(to_u8(rgb.player_alt)),
        Rgba::from(to_u8(rgb.player_arm)),
    );
    img.save(save).unwrap();
}

fn replace_colors_opt(path: PathBuf, save: PathBuf, rgb: &PlayerColor, inv: bool) {
    let mut img = image::open(path).unwrap().into_rgba8();
    replace_color_opt(
        &mut img,
        Rgba::from(to_u8(rgb.player_main)),
        Rgba::from(to_u8(rgb.player_alt)),
        Rgba::from(to_u8(rgb.player_arm)),
        inv,
    );
    img.save(save).unwrap();
}

#[allow(clippy::type_complexity)]
pub fn create_player_png(
    peer: OmniPeerId,
    mod_path: &Path,
    player_path: &Path,
    rgb: &PlayerPngDesc,
    is_host: bool,
    player_map: &mut MutexGuard<FxHashMap<OmniPeerId, (Option<WorldPos>, bool, bool, RgbaImage)>>,
) {
    let icon = get_player_skin(
        image::open(player_path)
            .unwrap_or(ImageRgba8(RgbaImage::new(20, 20)))
            .crop(1, 1, 7, 16)
            .into_rgba8(),
        *rgb,
    );
    player_map.insert(peer, (None, false, false, icon.clone()));
    let inv = rgb.invert_border;
    let id = peer.as_hex();
    let cosmetics = rgb.cosmetics;
    let rgb = rgb.colors;
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
            Rgba::from(to_u8(rgb.player_main)),
            Rgba::from(to_u8(rgb.player_alt)),
            Rgba::from(to_u8(rgb.player_arm)),
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
    let img = create_arm(Rgba::from(to_u8(rgb.player_forearm)));
    let path = tmp_path.join(format!("tmp/{id}_arm.png"));
    img.save(path).unwrap();
    edit_nth_line(
        tmp_path.join("unmodified_cape.xml").into(),
        tmp_path.join(format!("tmp/{id}_cape.xml")).into_os_string(),
        vec![16, 16],
        vec![
            format!("cloth_color=\"0xFF{}\"", rgb_to_hex(to_u8(rgb.player_cape))),
            format!(
                "cloth_color_edge=\"0xFF{}\"",
                rgb_to_hex(to_u8(rgb.player_cape_edge))
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
                (if cosmetics[0] {
                    "image_file=\"data/enemies_gfx/player_hat2.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_AMULET_ENABLED",
                (if cosmetics[1] {
                    "image_file=\"data/enemies_gfx/player_amulet.xml\""
                } else {
                    ""
                })
                .into(),
            ),
            (
                "MARKER_AMULET_GEM_ENABLED",
                (if cosmetics[2] {
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

fn rgb_to_hex(rgb: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}
