//! Integration tests: sRGB <-> linear conversion, through sploot-core's public API.
//!
//! TODO: add tests for
//! - 8-bit round trip: all 256 values sRGB -> linear -> linear_to_srgb_u8 come back exactly
//! - linear_to_srgb_u8 clamps: HDR 4.0 -> 255, negative -> 0, never NaN
//! - srgb_to_linear_rgba / linear_to_srgb_rgba keep alpha unchanged
//! - LinearColor::luminance: white = 1.0, pure green = 0.7152, decoded sRGB grey = 0.214
//! - matrices: white stays white through REC709_TO_ACESCG; REC709_TO_XYZ[1] matches REC709_LUMINANCE
use sploot_core::{LinearToSrgb, SrgbToLinear};

/// Compare floats within a tolerance.
fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-5, "{a} != {b}");
}

#[test]
fn srgb_endpoints() {
    assert_close(0.0f32.srgb_to_linear(), 0.0);
    assert_close(1.0f32.srgb_to_linear(), 1.0);
}

#[test]
fn srgb_mid_grey() {
    // sRGB 0.5 is ~21.4% linear, not 50%
    assert_close(0.5f32.srgb_to_linear(), 0.214041);
}

#[test]
fn linear_mid_grey() {
    // linear 0.214 encodes back to sRGB 0.5
    assert_close(0.214041f32.linear_to_srgb(), 0.5);
}
