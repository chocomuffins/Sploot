//! Mipmap generation: a reference mip chain from mip 0, to compare stored mips against.
//!
//! TODO:
//! - Filters: box / triangle / Kaiser / Lanczos (FilterType in lib.rs).
//! - Filter in linear space (sRGB sources are decoded first; alpha is never decoded).
//! - Wrap edges for tiling textures (option: clamp for non-tiling).
//! - Odd sizes: 5 x 3 -> 2 x 1 -> 1 x 1 (see samples/dds/sploot_bc1_srgb_npot.dds).
//! - Normal maps: renormalize every level.
//! - Alpha-tested textures (foliage, fences): plain filtering lowers alpha coverage at every mip,
//!   so leaves thin out and shimmer in the distance.
//!   - Report alpha coverage per mip at a given cutoff (fraction of texels with alpha > cutoff),
//!     next to mip 0's coverage, so the loss is visible per level.
//!   - Coverage-preserving mips: per level, scale alpha (or find a per-mip cutoff) so coverage
//!     matches mip 0 (Ignacio Castaño, "Computing Alpha Mipmaps"; NVTT's scale-alpha-to-coverage).
//!   - Sample texture: generate a leaf / fence alpha-test pattern in tools/make_test_images.py.
