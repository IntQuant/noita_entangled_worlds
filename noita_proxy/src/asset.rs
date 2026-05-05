#![expect(dead_code)]

use std::{collections::HashMap, fs, io::BufReader, path::PathBuf};
use tracing::warn;

// Maybe should be arc but the memory usage aint that big of a deal
// and there's no asset that change between netman and proxy at runtime
// at the momment

#[derive(Debug)]
pub enum AssetError {
    OpenFailed(AssetInfo, std::io::Error),
    FormatNotSpecified(AssetInfo),
    ParseFailed(AssetInfo, Box<dyn std::error::Error>),
}

impl std::error::Error for AssetError {}
impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::FormatNotSpecified(asset) => {
                write!(
                    f,
                    "{asset} cannot be parsed because its format have not been specified"
                )
            }
            AssetError::OpenFailed(asset, error) => {
                write!(f, "{asset} failed to open: {error}")
            }
            AssetError::ParseFailed(asset, error) => {
                write!(f, "{asset} failed to parse: {error}")
            }
        }
    }
}
impl AssetError {
    pub fn with_name(mut self, name: String) -> AssetError {
        match &mut self {
            AssetError::OpenFailed(asset_info, _)
            | AssetError::FormatNotSpecified(asset_info)
            | AssetError::ParseFailed(asset_info, _) => {
                asset_info.set_name(name);
            }
        }
        self
    }
}

type Result<T> = std::result::Result<T, AssetError>;

#[derive(Debug, Default, Clone)]
pub struct AssetManager {
    map: HashMap<String, Asset>,
}

impl AssetManager {
    pub fn extend(&mut self, iter: impl IntoIterator<Item = (String, Asset)>) {
        self.map.extend(iter);
    }

    pub fn get(&self, name: &str) -> Option<&Asset> {
        self.map.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Asset> {
        self.map.get_mut(name)
    }

    #[track_caller]
    pub fn get_parsed(&self, name: &str) -> &Parsed {
        let asset = if let Some(asset) = self.get(name) {
            asset
        } else {
            panic!("{name} does not exists");
        };
        if let Some(parsed) = asset.parsed().as_ref() {
            parsed
        } else {
            panic!("{name} is not parsed");
        }
    }

    pub fn fetch_all(&mut self) -> Vec<AssetError> {
        let mut errors = vec![];
        for (name, asset) in self.map.iter_mut() {
            let result = asset.fetch();
            match result {
                Ok(()) => {}
                Err(e) => errors.push(e.with_name(name.to_string())),
            }
        }
        errors
    }
}

#[derive(Debug, Default)]
pub struct AssetInfo {
    name: Option<String>,
    path: PathBuf,
}

impl std::fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "asset named {name} at {}", self.path.display())
        } else {
            write!(f, "asset at {}", self.path.display())
        }
    }
}

impl AssetInfo {
    pub fn with_name(self, name: String) -> AssetInfo {
        AssetInfo {
            name: Some(name),
            path: self.path,
        }
    }
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Format {
    #[default]
    Unknown,
    Image(image::ImageFormat),
}

#[derive(Debug, Clone)]
pub enum Parsed {
    Image(image::DynamicImage),
}

impl Parsed {
    #[allow(irrefutable_let_patterns)]
    pub fn as_image(&self) -> &image::DynamicImage {
        if let Parsed::Image(image) = self {
            image
        } else {
            panic!("is not image");
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Asset {
    path: PathBuf,
    format: Format,
    parsed: Option<Parsed>,
}

impl Asset {
    pub fn new(path: PathBuf) -> Asset {
        Asset {
            path,
            ..Default::default()
        }
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn parsed(&self) -> &Option<Parsed> {
        &self.parsed
    }

    pub fn get_info(&self) -> AssetInfo {
        AssetInfo {
            name: None,
            path: self.path.clone(),
        }
    }

    pub fn fetch(&mut self) -> Result<()> {
        if matches!(self.format, Format::Unknown) {
            return Err(AssetError::FormatNotSpecified(self.get_info()));
        }
        let file = fs::OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(|e| AssetError::OpenFailed(self.get_info(), e))?;
        match self.format {
            Format::Image(format) => {
                let file_reader = BufReader::new(file);
                let image = image::load(file_reader, format)
                    .map_err(|e| AssetError::ParseFailed(self.get_info(), Box::new(e)))?;
                self.parsed = Some(Parsed::Image(image));
            }
            Format::Unknown => unreachable!(),
        }
        Ok(())
    }

    pub fn guess_format_from_extension(&mut self) {
        let ext = if let Some(ext) = self.path.extension() {
            ext
        } else {
            return;
        };
        if let Some(format) = image::ImageFormat::from_extension(ext) {
            self.format = Format::Image(format);
        } else {
            warn!("Found {} with unimplemented format", self.get_info());
        }
    }

    pub fn with_format_guessed(mut self) -> Self {
        self.guess_format_from_extension();
        self
    }
}
