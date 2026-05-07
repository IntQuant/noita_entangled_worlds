use std::path::Path;

use bitcode::{Decode, Encode};
use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};

use eframe::egui::{
    self, Color32, Image, Slider, TextureHandle, TextureOptions, Ui,
    color_picker::{Alpha, color_picker_color32},
};

use crate::{
    asset::AssetManager,
    color::{f_to_u, shift_hue},
    player_cosmetics::{PlayerPngDesc, make_player_preview},
    tr,
};

#[derive(Default, Debug, Serialize, Deserialize, Decode, Encode, Copy, Clone)]
pub struct Cosmetics {
    pub hat: bool,
    pub amulet: bool,
    pub amulet_gem: bool,
}

impl Cosmetics {
    pub fn get(noita_save: &Path) -> Cosmetics {
        let flags = noita_save.join("save00/persistent/flags");
        let hat = flags.join("secret_hat").exists();
        let amulet = flags.join("secret_amulet").exists();
        let amulet_gem = flags.join("secret_amulet_gem").exists();
        Cosmetics {
            hat,
            amulet,
            amulet_gem,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PlayerAppearanceSettings {
    pub player_color: PlayerColor,
    pub player_picker: PlayerPicker,
    pub hue: f64,
    pub cosmetics: Cosmetics,
    pub invert_border: bool,
}

impl PlayerAppearanceSettings {
    pub fn create_png_desc(&self) -> PlayerPngDesc {
        PlayerPngDesc {
            cosmetics: self.cosmetics,
            colors: self.player_color,
            invert_border: self.invert_border,
        }
    }

    pub fn mina_color_picker(&mut self, ui: &mut Ui, asset_mananger: &AssetManager) {
        let old_hue = self.hue;
        let old = ui.style_mut().spacing.slider_width;
        ui.style_mut().spacing.slider_width = 256.0;
        ui.add(
            Slider::new(&mut self.hue, 0.0..=360.0)
                .text(tr("Shift-hue"))
                .min_decimals(0)
                .max_decimals(0)
                .step_by(2.0),
        );
        ui.style_mut().spacing.slider_width = old;
        if old_hue != self.hue {
            let diff = self.hue - old_hue;
            match self.player_picker {
                PlayerPicker::PlayerAlt => {
                    shift_hue(diff, &mut self.player_color.player_alt);
                }
                PlayerPicker::PlayerArm => {
                    shift_hue(diff, &mut self.player_color.player_arm);
                }
                PlayerPicker::PlayerCape => {
                    shift_hue(diff, &mut self.player_color.player_cape);
                }
                PlayerPicker::PlayerForearm => {
                    shift_hue(diff, &mut self.player_color.player_forearm);
                }
                PlayerPicker::PlayerCapeEdge => {
                    shift_hue(diff, &mut self.player_color.player_cape_edge);
                }
                PlayerPicker::PlayerMain => {
                    shift_hue(diff, &mut self.player_color.player_main);
                }
                PlayerPicker::None => {
                    shift_hue(diff, &mut self.player_color.player_main);
                    shift_hue(diff, &mut self.player_color.player_alt);
                    shift_hue(diff, &mut self.player_color.player_arm);
                    shift_hue(diff, &mut self.player_color.player_forearm);
                    shift_hue(diff, &mut self.player_color.player_cape);
                    shift_hue(diff, &mut self.player_color.player_cape_edge);
                }
            }
        }
        ui.horizontal(|ui| {
            display_player_skin(
                ui,
                make_player_preview(asset_mananger, &self.create_png_desc()),
                12.0,
            );
            player_select_current_color_slot(ui, self);
            player_skin_display_color_picker(ui, &mut self.player_color, &self.player_picker);
        });
        if ui.button(tr("Reset-colors-to-default")).clicked() {
            self.hue = 0.0;
            self.player_color = Default::default();
        }
    }
}

impl Default for PlayerAppearanceSettings {
    fn default() -> Self {
        Self {
            player_color: PlayerColor::default(),
            player_picker: PlayerPicker::None,
            hue: 0.0,
            cosmetics: Default::default(),
            invert_border: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Decode, Encode, Copy, Clone)]
pub struct PlayerColor {
    pub player_main: [f64; 4],
    pub player_alt: [f64; 4],
    pub player_arm: [f64; 4],
    pub player_cape: [f64; 4],
    pub player_cape_edge: [f64; 4],
    pub player_forearm: [f64; 4],
}

impl Default for PlayerColor {
    fn default() -> Self {
        Self {
            player_main: [155.0, 111.0, 154.0, 255.0],
            player_alt: [127.0, 84.0, 118.0, 255.0],
            player_arm: [89.0, 67.0, 84.0, 255.0],
            player_cape: [118.0, 84.0, 127.0, 255.0],
            player_cape_edge: [154.0, 111.0, 155.0, 255.0],
            player_forearm: [158.0, 115.0, 154.0, 255.0],
        }
    }
}
/*impl PlayerColor {
    pub fn notplayer() -> Self {
        Self {
            player_main: [155.0, 111.0, 154.0, 255.0],
            player_alt: [127.0, 84.0, 118.0, 255.0],
            player_arm: [89.0, 67.0, 84.0, 255.0],
            player_cape: [118.0, 84.0, 127.0, 255.0],
            player_cape_edge: [154.0, 111.0, 155.0, 255.0],
            player_forearm: [158.0, 115.0, 154.0, 255.0],
        }
    }
}*/

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum PlayerPicker {
    None,
    PlayerMain,
    PlayerAlt,
    PlayerArm,
    PlayerCape,
    PlayerCapeEdge,
    PlayerForearm,
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

pub fn color_picker(ui: &mut Ui, color: &mut [f64; 4]) {
    let mut rgb = Color32::from_rgb(f_to_u(color[0]), f_to_u(color[1]), f_to_u(color[2]));
    if color_picker_color32(ui, &mut rgb, Alpha::Opaque) {
        *color = [rgb.r() as f64, rgb.g() as f64, rgb.b() as f64, 255.0]
    }
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

pub fn player_select_current_color_slot(ui: &mut Ui, appearance: &mut PlayerAppearanceSettings) {
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

            ui.checkbox(&mut appearance.invert_border, "Invert border");
            if appearance.cosmetics.hat {
                ui.checkbox(&mut appearance.cosmetics.hat, tr("Crown"));
            }
            if appearance.cosmetics.amulet {
                ui.checkbox(&mut appearance.cosmetics.amulet, tr("Amulet"));
            }
            if appearance.cosmetics.amulet_gem {
                ui.checkbox(&mut appearance.cosmetics.amulet_gem, tr("Gem"));
            }
        });
    });
    if clicked && last == appearance.player_picker {
        appearance.player_picker = PlayerPicker::None
    }
}
