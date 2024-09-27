use std::{collections::HashMap, env, thread, time::Duration};

use eframe::egui::{self, ColorImage, RichText, TextureHandle, TextureOptions, Ui};
use steamworks::{SteamAPIInitError, SteamId};
use tracing::{error, info};

pub struct SteamUserAvatar {
    avatar: TextureHandle,
}

impl SteamUserAvatar {
    pub fn display_with_labels(&self, ui: &mut Ui, label_top: &str, label_bottom: &str) {
        let image = egui::Image::new(&self.avatar).fit_to_exact_size([32.0, 32.0].into());
        ui.group(|ui| {
            ui.set_min_width(200.0);
            ui.horizontal(|ui| {
                ui.add(image);
                ui.vertical(|ui| {
                    ui.label(RichText::new(label_top).size(14.0));
                    ui.label(RichText::new(label_bottom).size(11.0));
                });
            });
        });
    }
}

pub struct SteamState {
    pub client: steamworks::Client,
    avatar_cache: HashMap<SteamId, SteamUserAvatar>,
}

impl SteamState {
    pub(crate) fn new() -> Result<Self, SteamAPIInitError> {
        if env::var_os("NP_DISABLE_STEAM").is_some() {
            return Err(SteamAPIInitError::FailedGeneric(
                "Disabled by env variable".to_string(),
            ));
        }
        let app_id = env::var("NP_APPID").ok().and_then(|x| x.parse().ok());
        let client = steamworks::Client::init_app(app_id.unwrap_or(881100))?;
        client.networking_utils().init_relay_network_access();
        if let Err(err) = client.networking_sockets().init_authentication() {
            error!("Failed to init_authentication: {}", err)
        }
        let client_c = client.clone();
        thread::spawn(move || {
            info!("Spawned steam callback thread");
            loop {
                client_c.run_callbacks();
                thread::sleep(Duration::from_millis(3));
            }
        });
        Ok(SteamState {
            client,
            avatar_cache: HashMap::new(),
        })
    }

    pub fn get_user_name(&self, id: SteamId) -> String {
        let friends = self.client.friends();
        friends.get_friend(id).name()
    }

    pub fn get_avatar(&mut self, ctx: &egui::Context, id: SteamId) -> Option<&SteamUserAvatar> {
        let friends = self.client.friends();

        if self.avatar_cache.contains_key(&id) {
            self.avatar_cache.get(&id)
        } else {
            // Check that we already have the avatar, as otherwise small_avatar will return a placeholder image.
            if friends.request_user_information(id, false) {
                return None;
            };
            let friend = friends.get_friend(id);
            friend
                .small_avatar()
                .map(|data| {
                    ctx.load_texture(
                        format!("steam_avatar_for_{:?}", id),
                        ColorImage::from_rgba_unmultiplied([32, 32], &data),
                        TextureOptions::LINEAR,
                    )
                })
                .map(|avatar| {
                    let avatar = SteamUserAvatar { avatar };
                    &*self.avatar_cache.entry(id).or_insert(avatar)
                })
        }

        // ctx.load_texture(name, image, options)
    }

    pub(crate) fn get_my_id(&self) -> SteamId {
        self.client.user().steam_id()
    }
}
