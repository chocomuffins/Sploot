//! Result cache: one JSON file per key, written via `.tmp` + rename.
//!
//! TODO:
//! - Cache folder via `directories`: ProjectDirs::from("com", "Choco", "Sploot")?.cache_dir().
//! - Pixel results (mips, BC encodes) as DDS.
//! - Opt-in sidecar files next to the texture.

use crate::{ColorSpace, Compression, Quality, stats::ImageStats};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Which file (and version of it) plus the settings; any change is a cache miss.
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
    /// 0 = full resolution.
    pub mip_level: u8,
}

/// On-disk file: key + result.
#[derive(Serialize, Deserialize)]
struct CacheEntry {
    key: CacheKey,
    stats: ImageStats,
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
        blake3::hash(&json)
            .to_hex()
            .to_string()
    }
}

/// Writes `stats` for `key` into `dir`.
pub fn save(dir: &Path, key: &CacheKey, stats: &ImageStats) -> std::io::Result<()> {
    let path = dir.join(format!("{}.json", key.file_stem()));
    let tmp = path.with_extension("json.tmp");
    let entry = CacheEntry {
        key: key.clone(),
        stats: stats.clone(),
    };
    std::fs::write(&tmp, serde_json::to_string_pretty(&entry)?)?;
    std::fs::rename(tmp, path)
}

/// Cached stats for `key`, or None on a miss.
pub fn load(dir: &Path, key: &CacheKey) -> Option<ImageStats> {
    let path = dir.join(format!("{}.json", key.file_stem()));
    let text = std::fs::read_to_string(path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&text).ok()?;
    (entry.key == *key).then_some(entry.stats)
}
