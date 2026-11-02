//! Compression preview: what BC formats and quality presets do to a texture.
//!
//! TODO:
//! - Encode to BC1 / BC3 / BC4 / BC5 / BC7, and BC6H for HDR, with image_dds per Quality preset.
//! - Decode back and compare against the source with metrics.rs.
//! - Report file size and bits per pixel; rank settings by quality vs size.
//! - Output: tiled side-by-side image of all settings + an error heatmap per setting (heatmap.rs).
//! - Later: ASTC block sizes.
