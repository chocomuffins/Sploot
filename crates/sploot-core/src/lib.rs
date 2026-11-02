use std::fmt;
use std::path::PathBuf;

pub mod color;
pub use color::*; // re-export: sploot_core::SrgbToLinear, LinearColor, etc.
pub mod load;
pub mod metrics;
pub mod stats;
// Planned: TODOs only for now, see each file.
pub mod cache;
use serde::{Deserialize, Serialize};
pub mod compress;
pub mod heatmap;
pub mod mips;

pub const SPLOOT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub format: StorageFormat,
    pub width: u32,
    pub height: u32,
    pub sploot_version: &'static str,
    pub content_hash: [u8; 32],
    pub mip_level: u8,
    pub modified: std::time::SystemTime,
    pub color_space: ColorSpace,
    pub compression: Compression,
    pub quality: Quality,
    // pub alpha_stats: AlphaStats
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterType {
    Box,
    Triangle,
    Kaiser,
    Lanczos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Compression {
    Bc1,
    Bc3,
    Bc4Unorm,
    Bc4Snorm,
    Bc5Unorm,
    Bc5Snorm,
    Bc6hUf16,
    Bc6hSf16,
    Bc7,
}

/// BC encoder effort. Maps one-to-one onto image_dds::Quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quality {
    None,
    Fast,
    Normal,
    Slow,
}

/// How the texture's pixels are stored on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageFormat {
    Uncompressed {
        sample_type: SampleType,
        channels: ChannelLayout,
    },
    Block(Compression),
}

/// Only used if the texture is not compressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleType {
    U8,
    U16,
    F16,
    F32,
}

/// Which channels the file stores, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelLayout {
    Gray,
    GrayAlpha,
    Rg,
    Rgb,
    Rgba,
    Alpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureType {
    Color,
    Normal,
    SpecularLevel,
    SpecularColor,
    Metalness,
    Glossiness,
    Roughness,
    AmbientOcclusion,
    Curvature,
    Mask,
    Opacity,
    Height,
    Emissive,
    Subsurface,
    Thickness,
}

// --- Display ---

impl fmt::Display for SampleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(match self {
            SampleType::U8 => "8-bit",
            SampleType::U16 => "16-bit",
            SampleType::F16 => "16-bit float",
            SampleType::F32 => "32-bit float",
        })
    }
}

impl fmt::Display for ChannelLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(match self {
            ChannelLayout::Gray => "gray",
            ChannelLayout::GrayAlpha => "gray + alpha",
            ChannelLayout::Rg => "RG",
            ChannelLayout::Rgb => "RGB",
            ChannelLayout::Rgba => "RGBA",
            ChannelLayout::Alpha => "alpha",
        })
    }
}

impl fmt::Display for StorageFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            StorageFormat::Uncompressed {
                sample_type,
                channels,
            } => format!("{sample_type} {channels}"),
            StorageFormat::Block(compression) => format!("{compression:?}"),
        };
        f.pad(&text)
    }
}
