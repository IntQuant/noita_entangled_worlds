use std::{
    ops::Deref,
    sync::{Arc, atomic::Ordering},
    thread::JoinHandle,
};

pub use app::App;
pub use cli::{Args, connect_cli, host_cli};
pub use util::{lang, steam_helper};

use app::AppSavedState;
use audio_settings::AudioSettings;
use bookkeeping::{mod_manager, releases};
use game_map::ImageMap;
use game_settings::{DefaultSettings, GameSettings};
use lang::tr;
use mod_manager::ModmanagerSettings;
use player_cosmetics::player_path;
use player_settings::{PlayerAppearance, PlayerColor, PlayerPicker};

use serde::{Deserialize, Serialize};

mod bookkeeping;
mod lobby_code;
pub mod net;
pub mod paths;
mod player_cosmetics;
mod util;

mod app;
mod audio_settings;
mod game_map;
mod game_settings;
mod player_settings;

mod cli;

const DEFAULT_PORT: u16 = 5123;

pub struct NetManStopOnDrop(pub Arc<net::NetManager>, Option<JoinHandle<()>>);

impl Deref for NetManStopOnDrop {
    type Target = Arc<net::NetManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for NetManStopOnDrop {
    fn drop(&mut self) {
        self.0.continue_running.store(false, Ordering::Relaxed);
        self.1.take().unwrap().join().unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    color: PlayerAppearance,
    app: AppSavedState,
    modmanager: ModmanagerSettings,
    audio: AudioSettings,
}
