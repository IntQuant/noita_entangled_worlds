use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use tracing::{info, warn};

use crate::Settings;

pub(crate) struct SavePaths {
    settings_path: PathBuf,
    pub save_state_path: PathBuf,
}

impl SavePaths {
    pub fn new() -> Self {
        if Self::settings_next_to_exe_path().exists() {
            Self::new_next_to_exe()
        } else if let Some(project_dirs) = Self::project_dirs() {
            info!("Using 'system' paths to store things");
            let me = Self {
                settings_path: project_dirs.config_dir().join("proxy.ron"),
                save_state_path: project_dirs.data_dir().join("save_state"),
            };
            info!("Settings path: {}", me.settings_path.display());
            let _ = fs::create_dir_all(project_dirs.config_dir());
            let _ = fs::create_dir_all(&me.save_state_path);
            me
        } else {
            warn!("Failed to get project dirst!");
            Self::new_next_to_exe()
        }
    }

    fn new_next_to_exe() -> Self {
        info!("Using 'next to exe' path to store things");
        Self {
            settings_path: Self::settings_next_to_exe_path(),
            save_state_path: Self::next_to_exe_path().join("save_state"),
        }
    }

    fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("", "quant", "entangledworlds")
    }

    fn next_to_exe_path() -> PathBuf {
        let base_path = std::env::current_exe()
            .map(|p| p.parent().unwrap().to_path_buf())
            .unwrap_or(".".into());
        base_path
    }

    fn settings_next_to_exe_path() -> PathBuf {
        let base_path = std::env::current_exe()
            .map(|p| p.parent().unwrap().to_path_buf())
            .unwrap_or(".".into());
        let config_name = "proxy.ron";
        base_path.join(config_name)
    }

    pub fn load_settings(&self) -> Settings {
        if let Ok(mut file) = File::open(&self.settings_path) {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);
            ron::from_str::<Settings>(&s).unwrap_or_default()
        } else {
            info!("Failed to load settings file, returing default settings");
            Settings::default()
        }
    }

    pub fn save_settings(&self, settings: Settings) {
        let settings = ron::to_string(&settings).unwrap();
        if let Ok(mut file) = File::create(&self.settings_path) {
            file.write_all(settings.as_bytes()).unwrap();
        }
    }
}
