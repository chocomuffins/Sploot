//! Integration tests: reference mip chains and mip comparison (mips.rs).
//!
//! TODO: add tests once mips.rs has code:
//! - Chain length: 256 x 256 gives 9 levels; 200 x 120 gives 8 (200x120 ... 3x1, 1x1).
//! - Box filter on a 2 x 2 image: mip 1 is the average, computed in linear space.
//! - sRGB source: black / white checker downsamples to linear 0.5 (sRGB 188), not sRGB 128.
//! - Alpha is filtered as-is, never decoded.
//! - samples/dds/sploot_bc1_srgb_badmip3.dds: mip 3 stands out; the other levels don't.
//! - Wrap vs clamp edges give different results on the border pixels only.
//! - Normal maps: every level is renormalized (length 1.0).
//! - Alpha coverage per mip: coverage-preserving mips keep mip 0's coverage at the cutoff.
