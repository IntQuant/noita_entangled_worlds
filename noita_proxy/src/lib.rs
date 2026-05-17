use std::{
    ops::Deref,
    sync::{Arc, atomic::Ordering},
    thread::JoinHandle,
};

pub use app::App;
pub use cli::{Args, connect_cli, host_cli};
use tracing::error;
pub use util::{color, lang, steam_helper};

use audio_settings::AudioSettings;
use bookkeeping::{mod_manager, releases};
use game_map::ImageMap;
use game_settings::{DefaultSettings, GameSettings};
use lang::tr;

use crate::{asset::AssetManager, paths::Paths};

mod asset;
mod bookkeeping;
mod lobby_code;
pub mod net;
pub mod paths;
mod player_cosmetics;
mod runtime_dir;
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

fn init_assets(paths: &Paths) -> AssetManager {
    let mut asset_manager = AssetManager::default();
    player_cosmetics::extend_assets(paths.noita_quantew_install(), &mut asset_manager);
    let errors = asset_manager.fetch_auto();
    if !errors.is_empty() {
        error!("Some asset failed to automatically fetch:");
        for error in errors {
            error!("{error}")
        }
    }
    asset_manager
}
