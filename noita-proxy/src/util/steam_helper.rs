use std::{env, thread, time::Duration};

use eframe::egui::{self, RichText, TextureHandle, Ui};
use steamworks::{PersonaChange, PersonaStateChange, SteamAPIInitError, SteamId};
use tracing::{error, info};

pub struct SteamUserAvatar {
    avatar: TextureHandle,
}

impl SteamUserAvatar {
    pub fn display_with_labels(&self, ui: &mut Ui, label_top: &str, label_bottom: &str) {
        let image = egui::Image::new(&self.avatar).fit_to_exact_size([32.0, 32.0].into());
        ui.scope(|ui| {
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
}

impl SteamState {
    pub(crate) fn new(spacewars: bool) -> Result<Self, SteamAPIInitError> {
        if env::var_os("NP_DISABLE_STEAM").is_some() {
            return Err(SteamAPIInitError::FailedGeneric(
                "Disabled by env variable".to_string(),
            ));
        }
        let app_id = env::var("NP_APPID").ok().and_then(|x| x.parse().ok());
        info!("Initializing steam client...");
        let (client, single) =
            steamworks::Client::init_app(app_id.unwrap_or(if spacewars { 480 } else { 881100 }))?;
        info!("Initializing relay network accesss...");
        client.networking_utils().init_relay_network_access();
        if let Err(err) = client.networking_sockets().init_authentication() {
            error!("Failed to init_authentication: {}", err)
        }

        thread::spawn(move || {
            info!("Spawned steam callback thread");
            loop {
                single.run_callbacks();
                thread::sleep(Duration::from_millis(3));
            }
        });

        {
            client.register_callback(move |event: PersonaStateChange| {
                if event.flags.contains(PersonaChange::AVATAR) {
                    info!(
                        "Got PersonaStateChange for {:?}, removing from avatar cache.",
                        event.steam_id
                    );
                }
            });
        }
        Ok(SteamState { client })
    }

    pub fn get_user_name(&self, id: SteamId) -> String {
        let friends = self.client.friends();
        friends.get_friend(id).name()
    }

    pub(crate) fn get_my_id(&self) -> SteamId {
        self.client.user().steam_id()
    }
}
