//! Integration tests: loading files from samples/ (load::load).
//! Paths: concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/<file>").
//!
//! TODO: add tests for
//! - samples/sploot_checker.png: 512 x 512, "8-bit RGB", 262,144 pixels.
//! - samples/sploot_uv_16bit.png: 16-bit RGB (SampleType::U16).
//! - Srgb vs Linear: the same file decodes differently, except pure black / white.
//! - Alpha is never decoded: samples/dds/sploot_albedo_source.png alpha matches the raw value.
//! - A missing file returns an error (not a panic), and the error names the path.
//! - Once DDS loading exists, every samples/dds file:
//!   - mip count (9 for 256 x 256, 8 for the 200 x 120 npot file),
//!   - StorageFormat (BC1 / BC3 / BC4 / BC5 / BC7, RGBA8, RGBA16F),
//!   - legacy DXT1 header loads the same pixels as the DX10 BC1 file.
