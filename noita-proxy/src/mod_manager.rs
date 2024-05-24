use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use eframe::egui::{Align2, Context, Ui};
use egui_file_dialog::{DialogState, FileDialog};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use steamworks::AppId;
use tracing::{error, info};

use crate::{
    releases::{get_release_by_tag, Downloader, ReleasesError, Version},
    SteamState,
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
                .title("Select path to noita.exe"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModmanagerSettings {
    game_path: PathBuf,
}

impl ModmanagerSettings {
    fn try_find_game_path(&mut self, steam_state: Option<&mut SteamState>) {
        info!("Trying to find game path");
        if let Some(state) = steam_state {
            let apps = state.client.apps();
            let app_id = AppId::from(881100);
            if apps.is_app_installed(app_id) {
                let app_install_dir = apps.app_install_dir(app_id);
                self.game_path = PathBuf::from(app_install_dir).join("noita.exe");
                info!("Found game path with steam: {}", self.game_path.display())
            } else {
                info!("App not installed");
            }
        }
    }
    fn mod_path(&self) -> PathBuf {
        let mut path = self.game_path.clone();
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
            if check_path_valid(&settings.game_path) {
                info!("Path is valid, checking mod now");
                self.state = State::PreCheckMod;
            } else {
                settings.try_find_game_path(steam_state);
                let could_find_automatically = check_path_valid(&settings.game_path);
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
                ui.heading("Found a path automatically:");
                ui.label(settings.game_path.display().to_string());
                if ui.button("Use this one").clicked() {
                    self.state = State::PreCheckMod;
                    ctx.request_repaint();
                }
                if ui.button("Select manually").clicked() {
                    self.select_noita_file();
                }
            }
            State::SelectPath => {
                if let Some(path) = self.file_dialog.update(ctx).selected() {
                    settings.game_path = path.to_path_buf();
                    if !check_path_valid(&settings.game_path) {
                        self.state = State::InvalidPath;
                    }
                }
                if self.file_dialog.state() == DialogState::Cancelled {
                    // self.select_noita_file()
                    self.state = State::JustStarted
                }
            }
            State::InvalidPath => {
                ui.label("This path is not valid");
                if ui.button("Select again").clicked() {
                    self.select_noita_file();
                }
            }
            State::PreCheckMod => {
                // settings.game_path = PathBuf::new();
                // self.state = State::JustStarted;
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
                ui.label(format!(
                    "Proxy will install the mod to {}",
                    mod_path.display()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
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
                    if ui.button("Select a different path").clicked() {
                        self.select_noita_file()
                    }
                });
            }
            State::DownloadMod(promise) => {
                ui.label("Downloading mod...");
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
                        ui.label("Receiving release info...");
                        ui.spinner();
                    }
                }
            }
            State::UnpackMod(promise) => match promise.ready() {
                Some(Ok(_)) => {
                    ui.label("Mod has been installed!");
                    if ui.button("Continue").clicked() {
                        self.state = State::Done;
                    };
                }
                Some(Err(err)) => {
                    self.state = State::ReleasesError(err.clone());
                }
                None => {
                    ui.label("Unpacking mod");
                }
            },
            State::Error(err) => {
                ui.label(format!("Encountered an error: {}", err));
                if ui.button("Retry").clicked() {
                    self.state = State::JustStarted;
                }
            }
            State::ReleasesError(err) => {
                ui.label(format!("Encountered an error: {}", err));
                if ui.button("Retry").clicked() {
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
        if let State::Done = self.state {
            true
        } else {
            false
        }
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
    let version = fs::read_to_string(&version_path)
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
