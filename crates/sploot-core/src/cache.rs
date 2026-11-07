//! Result cache: one JSON file per key, written via `.tmp` + rename.
//!
//! TODO:
//! - Pixel results (mips, BC encodes) as DDS.

use crate::{
    ColorSpace, Compression, Quality, SPLOOT_VERSION,
    stats::{DEFAULT_BINS, Histograms, ImageStats},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub path: PathBuf,
    pub file_size: u64,
    pub modified: SystemTime,
    pub width: u32,
    pub height: u32,
    pub sploot_version: String,
    pub color_space: ColorSpace,
    pub compression: Compression,
    pub quality: Quality,
    pub mip_level: u8,
    pub bins: usize,
    /// Alpha cutoff as `f32::to_bits`, or None; f32 can't be Eq / Hash.
    pub alpha_cutoff: Option<u32>,
}

/// The default cache folder for this version, or None if the OS doesn't have one.
pub fn default_dir() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "chocomuffins", "Sploot")?;

    Some(dirs.cache_dir().join(SPLOOT_VERSION))
}

/// On-disk file: key + results.
#[derive(Serialize, Deserialize)]
struct CacheEntry {
    key: CacheKey,
    stats: ImageStats,
    histograms: Histograms,
}

impl CacheKey {
    /// Canonical path, size and modified time, without opening the image.
    pub fn fingerprint(path: &Path) -> std::io::Result<(PathBuf, u64, SystemTime)> {
        let path = std::fs::canonicalize(path)?;
        let meta = std::fs::metadata(&path)?;
        Ok((path, meta.len(), meta.modified()?))
    }

    /// BLAKE3 of the key's JSON, as hex.
    pub fn file_stem(&self) -> String {
        let json = serde_json::to_vec(self).expect("CacheKey serializes");
        blake3::hash(&json).to_hex().to_string()
    }

    /// Key for `sploot luma` stats of a whole image (mip 0, default histogram bins).
    pub fn for_stats(
        path: &Path,
        color_space: ColorSpace,
        alpha_cutoff: Option<f32>,
        width: u32,
        height: u32,
    ) -> std::io::Result<Self> {
        let (path, file_size, modified) = Self::fingerprint(path)?;

        Ok(Self {
            path,
            file_size,
            modified,
            width,
            height,
            sploot_version: SPLOOT_VERSION.to_string(),
            color_space,
            // Placeholders: stats aren't compressed.
            compression: Compression::Bc1,
            quality: Quality::Normal,
            mip_level: 0,
            bins: DEFAULT_BINS,
            alpha_cutoff: alpha_cutoff.map(f32::to_bits),
        })
    }

    pub fn save(
        &self,
        dir: &Path,
        stats: &ImageStats,
        histograms: &Histograms,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.json", self.file_stem()));
        let tmp = path.with_extension("json.tmp");
        let entry = CacheEntry {
            key: self.clone(),
            stats: stats.clone(),
            histograms: histograms.clone(),
        };
        std::fs::write(&tmp, serde_json::to_string_pretty(&entry)?)?;
        std::fs::rename(tmp, path)
    }

    pub fn load(&self, dir: &Path) -> Option<(ImageStats, Histograms)> {
        let path = dir.join(format!("{}.json", self.file_stem()));
        let text = std::fs::read_to_string(path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&text).ok()?;
        (entry.key == *self).then_some((entry.stats, entry.histograms))
    }
}
