use std::path::PathBuf;
use eframe::egui;
use eframe::egui::{Color32, TextureHandle, TextureOptions, Ui};
use eframe::egui::color_picker::{color_picker_color32, Alpha};
use eframe::epaint::Hsva;
use image::{Rgba, RgbaImage};
use crate::{App, PlayerColor, PlayerPicker};

pub fn player_path(path: PathBuf) -> PathBuf {
    let path = path
        .join("files/system/player/unmodified.png");
    path
}

pub fn replace_color(image: &mut RgbaImage, main: Rgba<u8>, alt: Rgba<u8>, arm: Rgba<u8>) {
    let target_main = Rgba::from([155, 111, 154, 255]);
    let target_alt = Rgba::from([127, 84, 118, 255]);
    let target_arm = Rgba::from([89, 67, 84, 255]);
    for pixel in image.pixels_mut() {
        if *pixel == target_main {
            *pixel = main;
        } else if *pixel == target_alt {
            *pixel = alt
        } else if *pixel == target_arm {
            *pixel = arm
        }
    }
}

pub fn make_player_image(image: &mut RgbaImage, colors: PlayerColor) {
    let target_main = Rgba::from([155, 111, 154, 255]);
    let target_alt = Rgba::from([127, 84, 118, 255]);
    let target_arm = Rgba::from([89, 67, 84, 255]);
    let main = Rgba::from(colors.player_main);
    let alt = Rgba::from(colors.player_alt);
    let arm = Rgba::from(colors.player_arm);
    let cape = Rgba::from(colors.player_cape);
    let cape_edge = Rgba::from(colors.player_cape_edge);
    let forearm = Rgba::from(colors.player_forearm);
    for (i, pixel) in image.pixels_mut().enumerate() {
        if *pixel == target_main {
            *pixel = main;
        } else if *pixel == target_alt {
            *pixel = alt
        } else if *pixel == target_arm {
            *pixel = arm
        } else {
            match i {
                49 | 41 | 33 => *pixel = forearm,
                25 => *pixel = Rgba::from([219, 192, 103, 255]),
                82 | 90 | 98 | 106 | 89 | 97 | 105 | 113 | 121 | 96 | 104 | 112 | 120 | 128 => {
                    *pixel = cape
                }
                74 | 73 | 81 | 80 | 88 => *pixel = cape_edge,
                _ => {}
            }
        }
    }
}

pub fn add_cosmetics(image: &mut RgbaImage, saves_paths: Option<PathBuf>, cosmetics: (bool, bool, bool))
{
    if let Some(path) = saves_paths
    {
        let flags = path.join("save00/persistent/flags");
        let hat = flags.join("secret_hat").exists();
        let amulet = flags.join("secret_amulet").exists();
        let gem = flags.join("secret_amulet_gem").exists();
        for (i, pixel) in image.pixels_mut().enumerate()
        {
            match i
            {
                2 | 4 | 6 if hat && cosmetics.0 => *pixel = Rgba::from([255, 244, 140, 255]),
                10 | 14 if hat && cosmetics.0 => *pixel = Rgba::from([191, 141, 65, 255]),
                11 | 12 | 13 if hat && cosmetics.0 => *pixel = Rgba::from([255, 206, 98, 255]),
                61 if gem && cosmetics.2 => *pixel = Rgba::from([255, 242, 162, 255]),
                68 if gem && cosmetics.2 => *pixel = Rgba::from([255, 227, 133, 255]),
                69 if gem && cosmetics.2 => *pixel = Rgba::from([255, 94, 38, 255]),
                70 | 77 if gem && cosmetics.2 => *pixel = Rgba::from([247, 188, 86, 255]),
                51 | 60 if amulet && cosmetics.1 => *pixel = Rgba::from([247, 188, 86, 255]),
                61 if amulet && cosmetics.1 => *pixel = Rgba::from([255, 227, 133, 255]),
                69 if amulet && cosmetics.1 => *pixel = Rgba::from([255, 242, 162, 255]),
                54 if amulet && cosmetics.1 => *pixel = Rgba::from([198, 111, 57, 255]),
                62 if amulet && cosmetics.1 => *pixel = Rgba::from([177, 97, 48, 149]),
                _ => {}
            }
        }
    }
}

pub fn shift_hue(diff: f32, color: &mut [u8; 4]) {
    let rgb = Color32::from_rgb(color[0], color[1], color[2]);
    let mut hsv = Hsva::from(rgb);
    hsv.h += diff / 360.0;
    hsv.h = hsv.h.fract();
    let rgb = hsv.to_srgb();
    *color = [rgb[0], rgb[1], rgb[2], 255];
}

pub fn player_skin_display_color_picker(ui: &mut Ui, player_color: &mut PlayerColor, player_picker: &PlayerPicker) {
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

pub fn color_picker(ui: &mut Ui, color: &mut [u8; 4]) {
    let mut rgb = Color32::from_rgb(color[0], color[1], color[2]);
    color_picker_color32(ui, &mut rgb, Alpha::Opaque);
    *color = [rgb.r(), rgb.g(), rgb.b(), 255]
}

pub fn player_select_current_color_slot(ui: &mut Ui, app: &mut App) {
    let mut clicked = false;
    let last = app.app_saved_state.player_picker.clone();
    ui.scope(|ui| {
        ui.set_max_width(100.0);
        ui.vertical_centered_justified(|ui| {
            if ui.button("Main color").clicked() {
                clicked = true;
                app.app_saved_state.player_picker = PlayerPicker::PlayerMain
            }
            if ui.button("Alt color").clicked() {
                clicked = true;
                app.app_saved_state.player_picker = PlayerPicker::PlayerAlt
            }
            if ui.button("Arm color").clicked() {
                clicked = true;
                app.app_saved_state.player_picker = PlayerPicker::PlayerArm
            }
            if ui.button("Forearm color").clicked() {
                clicked = true;
                app.app_saved_state.player_picker = PlayerPicker::PlayerForearm
            }
            if ui.button("Cape color").clicked() {
                clicked = true;
                app.app_saved_state.player_picker = PlayerPicker::PlayerCape
            }
            if ui.button("Cape edge color").clicked() {
                clicked = true;
                app.app_saved_state.player_picker = PlayerPicker::PlayerCapeEdge
            }
            if let Some(path) = &app.modmanager_settings.game_save_path
            {
                let flags = path.join("save00/persistent/flags");
                let hat = flags.join("secret_hat").exists();
                let amulet = flags.join("secret_amulet").exists();
                let gem = flags.join("secret_amulet_gem").exists();
                if hat {
                    ui.checkbox(
                        &mut app.app_saved_state.cosmetics.0,
                        "Crown",
                    );
                }
                if amulet {
                    ui.checkbox(
                        &mut app.app_saved_state.cosmetics.1,
                        "Amulet",
                    );
                }
                if gem {
                    ui.checkbox(
                        &mut app.app_saved_state.cosmetics.2,
                        "Gem",
                    );
                }
            }
        });
    });
    if clicked && last == app.app_saved_state.player_picker {
        app.app_saved_state.player_picker = PlayerPicker::None
    }
}

pub fn display_player_skin(ui: &mut Ui, app: &App) {
    let mut img = app.player_image.clone();
    add_cosmetics(&mut img, app.modmanager_settings.game_save_path.clone(), app.app_saved_state.cosmetics);
    make_player_image(&mut img, app.app_saved_state.player_color);
     let texture: TextureHandle = ui.ctx().load_texture(
         "player",
         egui::ColorImage::from_rgba_unmultiplied([8, 18], &img.into_raw()),
         TextureOptions::NEAREST,
     );
     ui.add(egui::Image::new(&texture).fit_to_original_size(11.0));
}