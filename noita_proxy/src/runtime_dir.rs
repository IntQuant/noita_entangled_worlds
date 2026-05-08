use std::path::{Path, PathBuf};

use crate::{net::omni::OmniPeerId, paths::Paths};

/// A directory that store data that will be changed at runtime by the proxy
/// and read by quant.ew
#[derive(Debug, Clone)]
pub struct RuntimeDir {
    /// Runtime path relative to the noita install folder, used by lua and entity xml files.
    noita_runtime_path: PathBuf,
    /// Full path to the runtime dir, should be the same as [`crate::paths::Paths::noita_quantew_runtime`].
    /// Must be accessible by the proxy.
    full_runtime_path: PathBuf,
}

impl RuntimeDir {
    pub fn from_paths(paths: &Paths) -> Option<RuntimeDir> {
        let noita_quantew_runtime = paths.noita_quantew_runtime.as_ref()?;
        Some(RuntimeDir {
            full_runtime_path: noita_quantew_runtime.clone(),
            noita_runtime_path: PathBuf::from(crate::paths::NOITA_QUANTEW_RUNTIME),
        })
    }
    pub fn full_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.full_runtime_path.join(path)
    }
    pub fn noita_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.noita_runtime_path.join(path)
    }
    pub fn for_peer(&self, id: OmniPeerId) -> PeerRuntimeDir<'_> {
        PeerRuntimeDir {
            runtime_dir: self,
            id: id.as_hex(),
        }
    }
}

/// Helper to deal with runtime dir of a specific player
pub struct PeerRuntimeDir<'a> {
    runtime_dir: &'a RuntimeDir,
    id: String,
}

impl<'a> PeerRuntimeDir<'a> {
    pub fn full_path(&self, suffix: &str) -> PathBuf {
        self.runtime_dir
            .full_runtime_path
            .join(format!("{}{}", self.id, suffix))
    }
    pub fn noita_path(&self, suffix: &str) -> PathBuf {
        self.runtime_dir
            .noita_runtime_path
            .join(format!("{}{}", self.id, suffix))
    }
}
