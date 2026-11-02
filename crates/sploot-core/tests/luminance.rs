//! Integration tests: luminance and color stats on sample images with known answers.
//! Loads through sploot-core's public API (load::load + ImageStats::from_pixels), like the CLI.
//!
//! TODO: add tests for
//! - samples/sploot_checker.png, loaded as Srgb: luminance average == 0.5 exactly
//!   (black and white decode the same in sRGB and linear).
//! - samples/sploot_uv.png, loaded as Linear: RGB average ~= (0.5, 0.5, 0.0).
//! - samples/sploot_uv_16bit.png: same average, and format is 16-bit RGB.
//! - samples/sploot_rainbow.png, loaded as Srgb: luminance average ~= 0.437.
//!
//! Sample paths: concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/<file>"),
//! so the tests work on any machine and in CI.
