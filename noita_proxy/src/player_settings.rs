use std::path::PathBuf;

use bitcode::{Decode, Encode};
use image::RgbaImage;
use serde::{Deserialize, Serialize};

use eframe::egui::{Slider, Ui};

use crate::player_cosmetics::{
    PlayerPngDesc, display_player_skin, get_player_skin, player_select_current_color_slot,
    player_skin_display_color_picker, shift_hue,
};
use crate::tr;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PlayerAppearance {
    pub player_color: PlayerColor,
    pub player_picker: PlayerPicker,
    pub hue: f64,
    pub cosmetics: (bool, bool, bool),
    pub invert_border: bool,
}

impl PlayerAppearance {
    pub fn create_png_desc(&self, game_save_path: Option<PathBuf>) -> PlayerPngDesc {
        let mut cosmetics = self.cosmetics;
        if let Some(path) = &game_save_path {
            let flags = path.join("save00/persistent/flags");
            let hat = flags.join("secret_hat").exists();
            let amulet = flags.join("secret_amulet").exists();
            let gem = flags.join("secret_amulet_gem").exists();
            if !hat {
                cosmetics.0 = false
            }
            if !amulet {
                cosmetics.1 = false
            }
            if !gem {
                cosmetics.2 = false
            }
        }
        PlayerPngDesc {
            cosmetics: cosmetics.into(),
            colors: self.player_color,
            invert_border: self.invert_border,
        }
    }

    pub fn mina_color_picker(
        &mut self,
        ui: &mut Ui,
        game_save_path: Option<PathBuf>,
        player_image: RgbaImage,
    ) {
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
                get_player_skin(
                    player_image.clone(),
                    self.create_png_desc(game_save_path.clone()),
                ),
                12.0,
            );
            player_select_current_color_slot(ui, self, game_save_path.clone());
            player_skin_display_color_picker(ui, &mut self.player_color, &self.player_picker);
        });
        if ui.button(tr("Reset-colors-to-default")).clicked() {
            self.hue = 0.0;
            self.player_color = Default::default();
        }
    }
}

impl Default for PlayerAppearance {
    fn default() -> Self {
        Self {
            player_color: PlayerColor::default(),
            player_picker: PlayerPicker::None,
            hue: 0.0,
            cosmetics: (true, true, true),
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
