use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use tracing::{info, warn};

use crate::Settings;

const DEFAULT_SETTINGS_NAME: &str = "proxy.ron";
const DEFAULT_SAVE_STATE_NAME: &str = "save_state";

const PROJECT_DIRS_ORGANIZATION: &str = "quant";
const PROJECT_DIRS_APPLICATION: &str = "entangledworlds";

pub(crate) struct SavePaths {
    settings_path: PathBuf,
    pub save_state_path: PathBuf,
}

impl SavePaths {
    pub fn new(settings_path: PathBuf, save_state_path: PathBuf) -> Self {
        SavePaths {
            settings_path,
            save_state_path,
        }
    }

    pub fn new_with_maybe_override(
        settings_path: Option<PathBuf>,
        save_state_path: Option<PathBuf>,
    ) -> SavePaths {
        use Prefer::*;
        enum Prefer {
            Custom,
            NextToExe,
            ProjectDirs,
        }

        let project_dirs = Self::project_dirs();
        let settings_next_to_exe_path = Self::default_settings_next_to_exe_path();

        let settings_prefer: Prefer;
        let settings_path = if let Some(settings_path) = settings_path {
            settings_prefer = Custom;
            settings_path
        } else if settings_next_to_exe_path.exists() {
            settings_prefer = NextToExe;
            settings_next_to_exe_path
        } else if let Some(project_dirs) = &project_dirs {
            settings_prefer = ProjectDirs;
            project_dirs.config_dir().join(DEFAULT_SETTINGS_NAME)
        } else {
            warn!(
                "There is no path override and failed to get project dirs. Falling back to 'next to exe' to store settings and save states."
            );
            settings_prefer = NextToExe;
            settings_next_to_exe_path
        };

        let save_state_path = if let Some(save_state_path) = save_state_path {
            save_state_path
        } else {
            let get_project_dirs_path = || {
                project_dirs
                    .as_ref()
                    .expect("project_dirs is already checked to be some")
                    .data_dir()
                    .join(DEFAULT_SAVE_STATE_NAME)
            };
            match settings_prefer {
                Custom if project_dirs.is_some() => get_project_dirs_path(),
                Custom => Self::default_save_state_next_to_exe_path(),
                ProjectDirs => get_project_dirs_path(),
                NextToExe => Self::default_save_state_next_to_exe_path(),
            }
        };

        if matches!(settings_prefer, ProjectDirs) {
            let _ = fs::create_dir_all(
                project_dirs
                    .expect("project_dirs is already checked to be some")
                    .config_dir(),
            );
        }
        if let Some(save_state_path_parent) = &save_state_path.parent() {
            let _ = fs::create_dir_all(save_state_path_parent);
        }

        info!("Settings path: {}", settings_path.display());
        info!("Save state path: {}", save_state_path.display());

        Self::new(settings_path, save_state_path)
    }

    fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("", PROJECT_DIRS_ORGANIZATION, PROJECT_DIRS_APPLICATION)
    }

    fn next_to_exe_path() -> PathBuf {
        std::env::current_exe()
            .map(|p| p.parent().unwrap().to_path_buf())
            .unwrap_or(".".into())
    }

    fn default_settings_next_to_exe_path() -> PathBuf {
        Self::next_to_exe_path().join(DEFAULT_SETTINGS_NAME)
    }

    fn default_save_state_next_to_exe_path() -> PathBuf {
        Self::next_to_exe_path().join(DEFAULT_SAVE_STATE_NAME)
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
