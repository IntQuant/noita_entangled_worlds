use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use eframe::egui::{Align2, Context, Ui};
use egui_file_dialog::{DialogState, FileDialog};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use steamworks::AppId;
use tracing::{error, info, warn};

use crate::{
    lang::tr,
    releases::{get_release_by_tag, Downloader, ReleasesError, Version},
    steam_helper::SteamState,
};

#[derive(Default)]
enum State {
    #[default]
    JustStarted,
    IsAutomaticPathOk,
    SelectPath,
    PreCheckMod,
    InvalidPath,
    CheckMod,
    Done,
    DownloadMod(Promise<Result<Downloader, ReleasesError>>),
    Error(io::Error),
    ReleasesError(ReleasesError),
    UnpackMod(Promise<Result<(), ReleasesError>>),
    ConfirmInstall,
}

pub struct Modmanager {
    state: State,
    file_dialog: FileDialog,
}

impl Default for Modmanager {
    fn default() -> Self {
        Self {
            state: Default::default(),
            file_dialog: FileDialog::default()
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .title(&tr("modman_path_to_exe")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModmanagerSettings {
    pub game_exe_path: PathBuf,
    pub game_save_path: Option<PathBuf>,
}

impl ModmanagerSettings {
    fn try_find_game_path(&mut self, steam_state: Option<&mut SteamState>) {
        info!("Trying to find game path");
        if let Some(state) = steam_state {
            let apps = state.client.apps();
            let app_id = AppId::from(881100);
            if apps.is_app_installed(app_id) {
                let app_install_dir = apps.app_install_dir(app_id);
                self.game_exe_path = PathBuf::from(app_install_dir).join("noita.exe");
                info!(
                    "Found game path with steam: {}",
                    self.game_exe_path.display()
                )
            } else {
                info!("App not installed");
            }
        }
    }

    fn try_find_save_path(&mut self) {
        if cfg!(target_os = "windows") {
            let appdata_path = PathBuf::from(
                std::env::var_os("APPDATA").expect("appdata to be defined on windows"),
            );
            info!("Appdata path: {}", appdata_path.display());
            let save_path = appdata_path.join("LocalLow/Nolla_Games_Noita/");
            if save_path.exists() {
                info!("Save path exists");
                self.game_save_path = Some(save_path);
            }
        }
        if cfg!(target_os = "linux") {
            let mut save_path = self.game_exe_path.clone();
            // Reach steamapps/
            save_path.pop();
            save_path.pop();
            save_path.pop();
            save_path.push(
                "compatdata/881100/pfx/drive_c/users/steamuser/AppData/LocalLow/Nolla_Games_Noita/",
            );
            info!("Probable save_path: {}", save_path.display());
            if save_path.exists() {
                info!("Save path exists");
                self.game_save_path = Some(save_path);
            }
        }

        match &self.game_save_path {
            Some(path) => info!("Found game save path: {}", path.display()),
            None => warn!("Could not find game save path"),
        }
    }

    fn mod_path(&self) -> PathBuf {
        let mut path = self.game_exe_path.clone();
        path.pop();
        path.push("mods");
        path.push("quant.ew");
        path
    }
}

impl Modmanager {
    pub fn update(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        settings: &mut ModmanagerSettings,
        steam_state: Option<&mut SteamState>,
    ) {
        if let State::JustStarted = self.state {
            if check_path_valid(&settings.game_exe_path) {
                info!("Path is valid, checking mod now");
                self.state = State::PreCheckMod;
            } else {
                settings.try_find_game_path(steam_state);
                let could_find_automatically = check_path_valid(&settings.game_exe_path);
                if could_find_automatically {
                    self.state = State::IsAutomaticPathOk;
                } else {
                    self.select_noita_file();
                }
            }
        }

        match &self.state {
            State::JustStarted => unreachable!(),
            State::IsAutomaticPathOk => {
                ui.heading(tr("modman_found_automatically"));
                ui.label(settings.game_exe_path.display().to_string());
                if ui.button(tr("modman_use_this")).clicked() {
                    self.state = State::PreCheckMod;
                    ctx.request_repaint();
                }
                if ui.button(tr("modman_select_manually")).clicked() {
                    self.select_noita_file();
                }
            }
            State::SelectPath => {
                if let Some(path) = self.file_dialog.update(ctx).selected() {
                    settings.game_exe_path = path.to_path_buf();
                    if !check_path_valid(&settings.game_exe_path) {
                        self.state = State::InvalidPath;
                    } else {
                        self.state = State::PreCheckMod;
                    }
                }
                if self.file_dialog.state() == DialogState::Cancelled {
                    self.state = State::JustStarted
                }
            }
            State::InvalidPath => {
                ui.label(tr("modman_invalid_path"));
                if ui.button(tr("button_select_again")).clicked() {
                    self.select_noita_file();
                }
            }
            State::PreCheckMod => {
                if settings.game_save_path.is_none() {
                    settings.try_find_save_path();
                }
                if let Some(path) = &settings.game_save_path {
                    info!("Trying to enable mod: {:?}", enable_mod(path));
                }
                ui.label("Will check mod install now...");
                self.state = State::CheckMod;
                ctx.request_repaint();
            }
            State::CheckMod => {
                ctx.request_repaint();
                let mod_path = settings.mod_path();
                info!("Mod path: {}", mod_path.display());

                self.state = match is_mod_ok(&mod_path) {
                    Ok(true) => State::Done,
                    Ok(false) => State::ConfirmInstall,
                    Err(err) => {
                        error!("Could not check if mod is ok: {}", err);
                        State::Error(err)
                    }
                }
            }
            State::ConfirmInstall => {
                let mod_path = settings.mod_path();
                ui.label(tr("modman_will_install_to"));
                ui.label(mod_path.display().to_string());
                ui.horizontal(|ui| {
                    if ui.button(tr("button_confirm")).clicked() {
                        let download_path = PathBuf::from("mod.zip");
                        let tag = Version::current().into();
                        let promise = Promise::spawn_thread("release-request", move || {
                            mod_downloader_for(tag, download_path)
                        });
                        // Make sure we are deleting the right thing
                        assert!(mod_path.ends_with("quant.ew"));
                        fs::remove_dir_all(mod_path).ok();
                        info!("Current mod deleted");

                        self.state = State::DownloadMod(promise)
                    }
                    if ui.button(tr("modman_another_path")).clicked() {
                        self.select_noita_file()
                    }
                });
            }
            State::DownloadMod(promise) => {
                ui.label(tr("modman_downloading"));
                match promise.ready() {
                    Some(Ok(downloader)) => {
                        downloader.show_progress(ui);
                        match downloader.ready() {
                            Some(Ok(_)) => {
                                let path = downloader.path().to_path_buf();
                                let directory = settings.mod_path();
                                let promise: Promise<Result<(), ReleasesError>> =
                                    Promise::spawn_thread("unpack", move || {
                                        extract_and_remove_zip(path, directory)
                                    });
                                self.state = State::UnpackMod(promise);
                            }
                            Some(Err(err)) => self.state = State::ReleasesError(err.clone()),
                            None => {}
                        }
                    }
                    Some(Err(err)) => self.state = State::ReleasesError(err.clone()),
                    None => {
                        ui.label(tr("modman_receiving_rel_info"));
                        ui.spinner();
                    }
                }
            }
            State::UnpackMod(promise) => match promise.ready() {
                Some(Ok(_)) => {
                    ui.label(tr("modman_installed"));
                    if ui.button(tr("button_continue")).clicked() {
                        self.state = State::Done;
                    };
                }
                Some(Err(err)) => {
                    self.state = State::ReleasesError(err.clone());
                }
                None => {
                    ui.label(tr("modman_unpacking"));
                }
            },
            State::Error(err) => {
                ui.label(format!("Encountered an error: {}", err));
                if ui.button(tr("button_retry")).clicked() {
                    self.state = State::JustStarted;
                }
            }
            State::ReleasesError(err) => {
                ui.label(format!("Encountered an error: {}", err));
                if ui.button(tr("button_retry")).clicked() {
                    self.state = State::JustStarted;
                }
            }
            State::Done => {}
        }
    }

    fn select_noita_file(&mut self) {
        self.state = State::SelectPath;
        self.file_dialog.select_file();
    }

    pub fn is_done(&self) -> bool {
        matches!(self.state, State::Done)
    }
}

fn mod_downloader_for(
    tag: crate::releases::Tag,
    download_path: PathBuf,
) -> Result<Downloader, ReleasesError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap();
    get_release_by_tag(&client, tag)
        .and_then(|release| release.get_release_assets(&client))
        .and_then(|asset_list| asset_list.find_by_name("quant.ew.zip").cloned())
        .and_then(|asset| asset.download(&client, &download_path))
}

fn extract_and_remove_zip(zip_file: PathBuf, extract_to: PathBuf) -> Result<(), ReleasesError> {
    let reader = File::open(&zip_file)?;
    let mut zip = zip::ZipArchive::new(reader)?;
    info!("Extracting zip file");
    zip.extract(extract_to)?;
    info!("Zip file extracted");
    fs::remove_file(&zip_file).ok();
    Ok(())
}

fn is_mod_ok(mod_path: &Path) -> io::Result<bool> {
    if env::var_os("NP_SKIP_MOD_CHECK").is_some() {
        return Ok(true);
    }
    if !mod_path.try_exists()? {
        return Ok(false);
    }
    let version_path = mod_path.join("files/version.lua");
    let version = fs::read_to_string(version_path)
        .ok()
        .and_then(|v| Version::parse_from_mod(&v));

    info!("Mod version: {:?}", version);

    if Some(Version::current()) != version {
        info!("Mod version differs");
        return Ok(false);
    }

    info!("Mod is ok");

    Ok(true)
}

fn check_path_valid(game_path: &Path) -> bool {
    game_path.ends_with("noita.exe") && game_path.exists()
}

// Mod enabled="0" name="daily_practice" settings_fold_open="0" workshop_item_id="0"
#[derive(Serialize, Deserialize)]
struct ModEntry {
    #[serde(rename = "@enabled")]
    enabled: u8,
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@settings_fold_open")]
    settings_fold_open: u8,
    #[serde(rename = "@workshop_item_id")]
    workshop_item_id: u64,
}

#[derive(Serialize, Deserialize)]
struct Mods {
    #[serde(rename = "Mod")]
    mod_entries: Vec<ModEntry>,
}

impl Mods {
    fn entry(&mut self, name: &str) -> &mut ModEntry {
        let index = self.mod_entries.iter().position(|ent| ent.name == name);
        if let Some(index) = index {
            &mut self.mod_entries[index]
        } else {
            self.mod_entries.push(ModEntry {
                enabled: 0,
                name: name.to_owned(),
                settings_fold_open: 0,
                workshop_item_id: 0,
            });
            self.mod_entries.last_mut().unwrap()
        }
    }
}

fn enable_mod(saves_path: &Path) -> Result<(), Box<dyn Error>> {
    let config_path = saves_path.join("save00/mod_config.xml");
    let mut data: Mods = quick_xml::de::from_reader(BufReader::new(File::open(&config_path)?))?;
    data.entry("quant.ew").enabled = 1;
    let xml = quick_xml::se::to_string(&data)?;
    fs::write(saves_path.join("save00/mod_config.xml"), xml)?;
    Ok(())
}
