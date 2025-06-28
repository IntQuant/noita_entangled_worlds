use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};

use eframe::egui::{self, Ui};
use eyre::{Context, eyre};
use poll_promise::Promise;
use reqwest::blocking::Client;
use serde::Deserialize;

pub type ReleasesError = eyre::Report;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: Tag,
    assets_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    url: String,
    pub name: String,
    pub size: u64,
}

impl Asset {
    pub fn download(&self, client: &Client, path: &Path) -> Result<Downloader, ReleasesError> {
        let shared = Arc::new(DownloaderSharedState {
            progress: AtomicU64::new(0),
        });
        let client = client.clone();
        let url = self.url.clone();
        let file = File::create(path)?;
        let handle = {
            let shared = shared.clone();
            Promise::spawn_thread("downloader", move || {
                download_thread(client, &url, shared, file)
            })
        };

        Ok(Downloader {
            shared,
            handle,
            path: path.to_path_buf(),
            size: self.size,
        })
    }
}

fn download_thread(
    client: Client,
    url: &str,
    shared: Arc<DownloaderSharedState>,
    mut file: File,
) -> Result<(), ReleasesError> {
    let mut response = client
        .get(url)
        .header("Accept", "application/octet-stream")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-agent", "noita proxy")
        .send()
        .wrap_err_with(|| format!("Failed to download from {url}"))?;
    let mut buf = [0; 4096];

    loop {
        let len = response.read(&mut buf).wrap_err_with(|| {
            format!("Failed to download from {url}: couldn't read from response")
        })?;
        shared
            .progress
            .fetch_add(len as u64, std::sync::atomic::Ordering::Relaxed);
        if len == 0 {
            break;
        }
        file.write_all(&buf[..len])
            .wrap_err_with(|| format!("Failed to download from {url}: couldn't write to file"))?;
    }

    Ok(())
}

struct DownloaderSharedState {
    progress: AtomicU64,
}

pub struct Downloader {
    shared: Arc<DownloaderSharedState>,
    size: u64,
    handle: Promise<Result<(), ReleasesError>>,
    path: PathBuf,
}

impl Downloader {
    pub fn progress(&self) -> (u64, u64) {
        let written = self
            .shared
            .progress
            .load(std::sync::atomic::Ordering::Relaxed);
        (written, self.size)
    }

    pub fn show_progress(&self, ui: &mut Ui) {
        let (current, max) = self.progress();
        ui.label(format!("{current} out of {max} bytes"));
        ui.add(egui::ProgressBar::new(current as f32 / max as f32));
        ui.ctx().request_repaint_after(Duration::from_millis(200));
    }

    pub fn ready(&self) -> Option<&Result<(), ReleasesError>> {
        self.handle.ready()
    }

    pub fn into_ready(self) -> Result<(), ReleasesError> {
        self.handle.block_and_take()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Deserialize)]
pub struct AssetList(Vec<Asset>);

impl AssetList {
    pub fn find_by_name(&self, name: &str) -> Result<&Asset, ReleasesError> {
        self.0
            .iter()
            .find(|asset| asset.name == name)
            .ok_or_else(|| eyre!("Asset not found: {}", name))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tag(String);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn parse_from_mod(version: &str) -> Option<Self> {
        let strip_suffix = version.strip_prefix("return \"")?.strip_suffix('"')?;
        Self::parse_from_string(strip_suffix)
    }
    pub fn parse_from_diplay(version: &str) -> Option<Self> {
        Self::parse_from_string(version.strip_prefix('v')?)
    }
    fn parse_from_string(version: &str) -> Option<Self> {
        let mut nums = version.split('.');
        let major = nums.next()?.parse().ok()?;
        let minor = nums.next()?.parse().ok()?;
        let patch = nums.next()?.parse().ok()?;
        Some(Self {
            major,
            minor,
            patch,
        })
    }
    pub fn current() -> Self {
        Self::parse_from_string(env!("CARGO_PKG_VERSION")).expect("can always parse crate version")
    }
    pub fn parse_from_tag(tag: Tag) -> Option<Self> {
        Self::parse_from_string(tag.0.strip_prefix('v')?)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl From<Version> for Tag {
    fn from(value: Version) -> Self {
        Self(format!("v{}.{}.{}", value.major, value.minor, value.patch))
    }
}

impl Release {
    pub fn get_release_assets(&self, client: &Client) -> Result<AssetList, ReleasesError> {
        let response = client
            .get(&self.assets_url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-agent", "noita proxy")
            .send()
            .wrap_err_with(|| format!("Failed to request asset list from {}", self.assets_url))?;

        response.json().wrap_err_with(|| {
            format!(
                "Failed to request asset list from {}: couldn't parse json",
                self.assets_url
            )
        })
    }
}

pub fn get_latest_release(client: &Client) -> Result<Release, ReleasesError> {
    let response = client
        .get("https://api.github.com/repos/IntQuant/noita_entangled_worlds/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-agent", "noita proxy")
        .send()
        .wrap_err("Failed to request latest release")?;

    Ok(response.json()?)
}

pub fn get_release_by_tag(client: &Client, tag: Tag) -> Result<Release, ReleasesError> {
    let url = format!(
        "https://api.github.com/repos/IntQuant/noita_entangled_worlds/releases/tags/{}",
        tag.0
    );
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-agent", "noita proxy")
        .send()
        .wrap_err_with(|| format!("Failed to get release by tag from {url}"))?;
    let response = response.error_for_status().wrap_err_with(|| {
        format!("Failed to get release by tag from {url}: response returned an error")
    })?;
    Ok(response.json()?)
}

#[cfg(test)]
#[serial_test::serial]
mod test {
    use crate::releases::{Tag, get_release_by_tag};

    #[test]
    fn release_assets() {
        let client = reqwest::blocking::Client::new();
        // let release = get_latest_release(&client).unwrap();
        let release = get_release_by_tag(&client, Tag("v0.4.1".to_string())).unwrap();
        let assets = release.get_release_assets(&client).unwrap();
        //println!("{:?}", release);
        //println!("{:?}", assets);
        let mod_asset = assets.find_by_name("quant.ew.zip").unwrap();
        //println!("{:?}", mod_asset);
        assert_eq!(mod_asset.name, "quant.ew.zip")
    }
}
