//! Integration tests: sRGB <-> linear conversion, through sploot-core's public API.
//!
//! TODO: add tests for
//! - 8-bit round trip: all 256 values sRGB -> linear -> linear_to_srgb_u8 come back exactly
//! - linear_to_srgb_u8 clamps: HDR 4.0 -> 255, negative -> 0, NaN -> 0
//! - srgb_to_linear_rgba / linear_to_srgb_rgba keep alpha unchanged
//! - LinearColor::luminance: white = 1.0, pure green = 0.7152, decoded sRGB grey = 0.214
//! - matrices: white stays white through REC709_TO_ACESCG; REC709_TO_XYZ[1] matches REC709_LUMINANCE
//! - REC709_LUMINANCE sums to 1.0
//! - ColorSpace Display: "sRGB" / "linear"; format!("{:<8}", ..) pads (f.pad)
//! - Array versions: [0.5; 3].srgb_to_linear() = 0.214 per channel; [f32; 1] works too
//! - SrgbColor::to_linear matches converting each channel
//! - dot3([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]) == 32.0
//! - Linear segment: 0.01 -> 0.01 / 12.92; no jump across the 0.04045 / 0.0031308 thresholds
use sploot_core::{LinearToSrgb, SrgbToLinear, rgb_to_hex};

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
    assert_close(0.5f32.srgb_to_linear(), 0.214041f32);
}

#[test]
fn linear_mid_grey() {
    assert_close(0.214041f32.linear_to_srgb(), 0.5);
}

#[test]
fn hex_from_srgb8() {
    assert_eq!(rgb_to_hex([125, 78, 48]), "#7d4e30");
    assert_eq!(rgb_to_hex([0, 0, 0]), "#000000");
    assert_eq!(rgb_to_hex([255, 255, 255]), "#ffffff");
}
