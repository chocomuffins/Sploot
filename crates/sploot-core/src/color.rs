//! Color conversions: sRGB <-> linear, luminance.
//! All math in sploot-core happens on linear values.

/// How a source's stored values are encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    Srgb,
    Linear,
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.pad(match self {
            ColorSpace::Srgb => "sRGB",
            ColorSpace::Linear => "linear",
        })
    }
}

/// Rec. 709 luminance weights (`REC709_TO_XYZ[1]`, rounded); they sum to 1.0.
pub const REC709_LUMINANCE: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// A 3x3 color matrix, row-major: each row is one output channel.
pub type Mat3 = [[f32; 3]; 3];

pub const REC709_TO_XYZ: Mat3 = [
    [0.4124564, 0.3575761, 0.1804375],
    [0.2126729, 0.7151522, 0.0721750],
    [0.0193339, 0.1191920, 0.9503041],
];

pub const REC709_TO_ACESCG: Mat3 = [
    [0.6130973, 0.3395229, 0.0473793],
    [0.0701942, 0.9163556, 0.0134526],
    [0.0206156, 0.1095698, 0.8698151],
];

pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// --- Color conversion: sRGB -> linear ---
pub trait SrgbToLinear {
    fn srgb_to_linear(self) -> Self;
}

impl SrgbToLinear for f32 {
    fn srgb_to_linear(self) -> f32 {
        if self <= 0.04045 {
            self / 12.92
        } else {
            ((self + 0.055) / 1.055).powf(2.4)
        }
    }
}

impl<const N: usize> SrgbToLinear for [f32; N] {
    fn srgb_to_linear(self) -> Self {
        self.map(f32::srgb_to_linear)
    }
}

pub fn srgb_to_linear_rgba([r, g, b, a]: [f32; 4]) -> [f32; 4] {
    [
        r.srgb_to_linear(),
        g.srgb_to_linear(),
        b.srgb_to_linear(),
        a,
    ]
}

// --- Color conversion: linear -> sRGB ---
pub trait LinearToSrgb {
    fn linear_to_srgb(self) -> Self;
}

impl LinearToSrgb for f32 {
    fn linear_to_srgb(self) -> f32 {
        if self <= 0.0031308 {
            self * 12.92
        } else {
            1.055 * self.powf(1.0 / 2.4) - 0.055
        }
    }
}

impl<const N: usize> LinearToSrgb for [f32; N] {
    fn linear_to_srgb(self) -> Self {
        self.map(f32::linear_to_srgb)
    }
}

pub fn linear_to_srgb_rgba([r, g, b, a]: [f32; 4]) -> [f32; 4] {
    [
        r.linear_to_srgb(),
        g.linear_to_srgb(),
        b.linear_to_srgb(),
        a,
    ]
}

pub fn linear_to_srgb_u8(c: f32) -> u8 {
    (c.linear_to_srgb().clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A color as stored: sRGB-encoded. Decode with `to_linear` before doing math on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SrgbColor(pub [f32; 3]);

/// A linear color in Rec.709/sRGB primaries: safe for luminance, filtering and error metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearColor(pub [f32; 3]);

impl SrgbColor {
    pub fn to_linear(self) -> LinearColor {
        LinearColor(self.0.srgb_to_linear())
    }
}

impl LinearColor {
    /// Relative luminance Y: normalized (white = 1.0), not cd/m².
    #[inline]
    pub fn luminance(self) -> f32 {
        dot3(self.0, REC709_LUMINANCE)
    }
}
