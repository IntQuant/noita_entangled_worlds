use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    app::AppSavedState, audio_settings::AudioSettings, paths::Paths,
    player_settings::PlayerAppearance,
};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub color: PlayerAppearance,
    pub app: AppSavedState,
    pub audio: AudioSettings,
    pub paths: Paths,
}

impl Settings {
    pub fn load(path: &Path) -> io::Result<Settings> {
        let mut file = File::open(path)?;
        let mut s = String::new();
        let _ = file.read_to_string(&mut s);
        let settings = ron::from_str::<Settings>(&s).unwrap_or_default();
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let settings = ron::to_string(&self).unwrap();
        let mut file = File::create(path)?;
        file.write_all(settings.as_bytes())?;
        Ok(())
    }
}
