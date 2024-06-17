use std::{
    cmp::Ordering,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use eframe::egui::{Align, Layout, Ui};
use poll_promise::Promise;
use reqwest::blocking::Client;
use tracing::info;

use crate::{
    lang::{tr, tr_a},
    releases::{get_latest_release, Downloader, ReleasesError, Version},
};

struct VersionCheckResult {
    newest: Version,
    ord: Ordering,
}

enum State {
    Initial,
    Download(Promise<Result<Downloader, ReleasesError>>),
    ReleasesError(ReleasesError),
    Unpack(Promise<Result<(), ReleasesError>>),
}

pub struct SelfUpdateManager {
    latest_check: Promise<Option<VersionCheckResult>>,
    pub request_update: bool,
    state: State,
}

impl SelfUpdateManager {
    pub fn new() -> Self {
        let latest_check = Promise::spawn_thread("version check", || {
            let client = Client::new();
            get_latest_release(&client)
                .map(|release| release.tag_name)
                .ok()
                .and_then(Version::parse_from_tag)
                .map(|ver| VersionCheckResult {
                    ord: ver.cmp(&Version::current()),
                    newest: ver,
                })
        });
        Self {
            latest_check,
            request_update: false,
            state: State::Initial,
        }
    }

    pub fn display_version(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            match self.latest_check.ready() {
                Some(&Some(VersionCheckResult {
                    newest: _,
                    ord: Ordering::Equal,
                })) => {
                    ui.label(tr("version_latest"));
                }
                Some(&Some(VersionCheckResult { newest, ord: _ })) => {
                    if ui
                        .small_button(tr_a(
                            "version_new_available",
                            &[("new_version", newest.to_string().into())],
                        ))
                        .clicked()
                    {
                        self.request_update = true;
                    }
                }
                Some(None) => {
                    ui.label(tr("version_check_failed"));
                }
                None => {
                    ui.label(tr("version_checking"));
                }
            }
            ui.label(concat!("Noita Proxy v", env!("CARGO_PKG_VERSION"),));
        });
    }

    pub fn self_update(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx();
        match &self.state {
            State::Initial => {
                if ui.button(tr("selfupdate_confirm")).clicked() {
                    let promise = Promise::spawn_thread("get_release", || {
                        proxy_downloader_for("newer.zip".into())
                    });
                    self.state = State::Download(promise)
                }
            }
            State::Download(promise) => match promise.ready() {
                Some(Ok(downloader)) => {
                    downloader.show_progress(ui);
                    match downloader.ready() {
                        Some(Ok(_)) => {
                            let path = downloader.path().to_path_buf();
                            let promise: Promise<Result<(), ReleasesError>> =
                                Promise::spawn_thread("unpack", move || {
                                    extract_and_remove_zip(path)
                                });
                            self.state = State::Unpack(promise);
                        }
                        Some(Err(err)) => self.state = State::ReleasesError(err.clone()),
                        None => {}
                    }
                }
                Some(Err(err)) => self.state = State::ReleasesError(err.clone()),
                None => {
                    ui.label(tr("selfupdate_receiving_rel_info"));
                    ui.spinner();
                }
            },
            State::Unpack(promise) => match promise.ready() {
                Some(Ok(_)) => {
                    ui.label(tr("selfupdate_updated"));
                }
                Some(Err(err)) => {
                    ui.label(format!("Could not update proxy: {}", err));
                }
                None => {
                    ctx.request_repaint();
                    ui.label(tr("selfupdate_unpacking"));
                    ui.spinner();
                }
            },
            State::ReleasesError(err) => {
                ui.label(format!("Encountered an error: {}", err));
            }
        }
    }
}

fn proxy_asset_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "noita-proxy-win.zip"
    } else {
        "noita-proxy-linux.zip"
    }
}

fn proxy_bin_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "noita_proxy.exe"
    } else {
        "noita_proxy.x86_64"
    }
}

fn proxy_downloader_for(download_path: PathBuf) -> Result<Downloader, ReleasesError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        .build()
        .unwrap();
    get_latest_release(&client)
        .and_then(|release| release.get_release_assets(&client))
        .and_then(|asset_list| asset_list.find_by_name(proxy_asset_name()).cloned())
        .and_then(|asset| asset.download(&client, &download_path))
}

fn extract_and_remove_zip(zip_file: PathBuf) -> Result<(), ReleasesError> {
    let extract_to = Path::new("tmp.exec");
    let bin_name = proxy_bin_name();
    let reader = File::open(&zip_file)?;
    let mut zip = zip::ZipArchive::new(reader)?;
    info!("Extracting zip file");
    let mut src = zip.by_name(bin_name)?;
    let mut dst = File::create(extract_to)?;
    io::copy(&mut src, &mut dst)?;

    self_replace::self_replace(extract_to)?;

    info!("Zip file extracted");
    fs::remove_file(&zip_file).ok();
    fs::remove_file(extract_to).ok();
    Ok(())
}
