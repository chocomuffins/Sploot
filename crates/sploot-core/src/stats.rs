//! Statistics over one image: average color, luminance stats (average, log-average,
//! min / max / median), histogram, optional alpha mask.
//! Input is always linear; decoding happens before anything reaches this module.
//!
//! TODO:
//! 1. histogram: only a stub; "luminance histogram output" is on the README roadmap.
//!    The tested sketch is commented out at the bottom of this file.
//! 2. noise_residual: estimate_noise_sigma needs a residual as input; the function that
//!    makes one is commented out at the bottom, so there's no noise number from an image yet.
//! 3. Image dimensions: ImageStats::from_pixels only gets a flat pixel list. Noise needs
//!    width / height (which pixels are neighbours); pass them in once noise joins the report.
//! 4. White balance: compare rgb.average[0..3], e.g. red / green and blue / green.
//! 5. Tests: add tests/stats.rs (mixed pixels, fully transparent, empty input,
//!    odd / even medians, u8 / u16 input).
//! 6. Bit-depth-aware alpha cutoff: desaturate_visible_pixels hard-codes 1/255,
//!    too coarse for 16-bit alpha.
//!
//! Optional:
//! - Percentiles (1st / 99th): a robust min / max that ignores hot or dead pixels (HDR).
//! - Infinity count next to nan_count; HDR files can contain both.
//! - serde::Serialize on the stats structs for --json (needs serde as a dependency).
use crate::color::LinearColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LuminanceStats {
    pub average: f64, // Average luminance
    pub log_average: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub std_dev: f64,
    pub nan_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbStats {
    pub average: [f64; 3], // Average color
    pub log_average: [f64; 3],
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub median: [f64; 3],
    pub std_dev: [f64; 3],
    pub nan_count: [usize; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlphaStats {
    pub opaque_count: usize,      // alpha == 1.0
    pub transparent_count: usize, // alpha <= cutoff
    pub coverage: f64,            // fraction of pixels above the cutoff
    pub min: f64,
    pub max: f64,
}

// --- Layer 1: statistics of a list of numbers ---

pub trait Stats {
    fn average(&self) -> Option<f64>;
    fn log_average(&self, epsilon: f64) -> Option<f64>;
    fn min_value(&self) -> Option<f64>;
    fn max_value(&self) -> Option<f64>;
    fn median(&self) -> Option<f64>;
    fn variance(&self) -> Option<f64>;
    fn std_dev(&self) -> Option<f64> {
        self.variance().map(f64::sqrt)
    }
    fn count_nan(&self) -> usize;
}

/// Works on any numeric slice: f32 pixels (0-1), or raw u8 / u16 data.
/// Results are on the input's own scale: 0-1 for f32, 0-255 for u8, 0-65535 for u16.
impl<T: Copy + Into<f64>> Stats for [T] {
    fn average(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        Some(self.iter().map(|&v| v.into()).sum::<f64>() / self.len() as f64)
    }

    /// Geometric mean: exp(mean(ln(v + epsilon))). Epsilon keeps black pixels from giving ln(0) = -inf.
    // TODO(bug): epsilon assumes 0-1 data; on raw u8 / u16 it's meaningless ([0u8, 255] gives 0.16).
    //            Scale it to the input range, or document that log_average expects 0-1 values.
    fn log_average(&self, epsilon: f64) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        let sum_ln: f64 = self
            .iter()
            .map(|&v| (v.into().max(0.0) + epsilon).ln())
            .sum();
        Some((sum_ln / self.len() as f64).exp())
    }

    fn min_value(&self) -> Option<f64> {
        self.iter().map(|&v| v.into()).reduce(f64::min)
    }

    fn max_value(&self) -> Option<f64> {
        self.iter().map(|&v| v.into()).reduce(f64::max)
    }

    /// Uses select_nth_unstable on an f64 copy (no full sort).
    fn median(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        let mut v: Vec<f64> = self.iter().map(|&x| x.into()).collect();
        let mid = v.len() / 2;
        let (lower_half, upper_mid, _) = v.select_nth_unstable_by(mid, f64::total_cmp);
        let upper_mid = *upper_mid;
        if self.len() % 2 == 1 {
            return Some(upper_mid);
        }
        let lower_mid = lower_half.iter().copied().reduce(f64::max)?;
        Some((lower_mid + upper_mid) / 2.0)
    }

    fn variance(&self) -> Option<f64> {
        let mean = self.average()?;
        Some(
            self.iter()
                .map(|&v| {
                    let d = v.into() - mean;
                    d * d
                })
                .sum::<f64>()
                / self.len() as f64,
        )
    }

    fn count_nan(&self) -> usize {
        self.iter().filter(|&&v| v.into().is_nan()).count()
    }
}

// --- Layer 2: statistics of an image ---

impl LuminanceStats {
    pub fn from_values(values: &[f32]) -> Option<Self> {
        Some(Self {
            average: values.average()?,
            log_average: values.log_average(1e-4)?,
            min: values.min_value()?,
            max: values.max_value()?,
            median: values.median()?,
            std_dev: values.std_dev()?,
            nan_count: values.count_nan(),
        })
    }
}

/// Runs `f` on each of the three channels; None if any channel has no answer.
fn per_channel<T>(ch: [&[f32]; 3], f: impl Fn(&[f32]) -> Option<T>) -> Option<[T; 3]> {
    Some([f(ch[0])?, f(ch[1])?, f(ch[2])?])
}

impl RgbStats {
    pub fn from_channels(r: &[f32], g: &[f32], b: &[f32]) -> Option<Self> {
        let ch = [r, g, b];
        Some(Self {
            average: per_channel(ch, |c| c.average())?,
            log_average: per_channel(ch, |c| c.log_average(1e-4))?,
            min: per_channel(ch, |c| c.min_value())?,
            max: per_channel(ch, |c| c.max_value())?,
            median: per_channel(ch, |c| c.median())?,
            std_dev: per_channel(ch, |c| c.std_dev())?,
            nan_count: ch.map(|c| c.count_nan()),
        })
    }

    /// Average linear color (the channel averages).
    pub fn average_color(&self) -> LinearColor {
        LinearColor(self.average.map(|a| a as f32))
    }
}

impl AlphaStats {
    /// Measured over all pixels, including transparent ones.
    pub fn from_pixels(pixels: &[[f32; 4]], alpha_cutoff: f32) -> Option<Self> {
        let alpha: Vec<f32> = pixels.iter().map(|p| p[3]).collect();
        let transparent_count = alpha.iter().filter(|&&a| a <= alpha_cutoff).count();
        Some(Self {
            opaque_count: alpha.iter().filter(|&&a| a >= 1.0).count(),
            transparent_count,
            coverage: (alpha.len() - transparent_count) as f64 / alpha.len() as f64,
            min: alpha.min_value()?,
            max: alpha.max_value()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageStats {
    pub visible_count: usize, // pixels that passed the alpha cutoff; shared by rgb and luminance
    pub rgb: Option<RgbStats>, // None if every pixel was masked out
    pub luminance: Option<LuminanceStats>, // None if every pixel was masked out
    pub alpha: AlphaStats,
}

impl ImageStats {
    /// `alpha_cutoff`: pixels with alpha at or below it are left out of rgb / luminance.
    // TODO(3): take width / height once noise is part of the report.
    // TODO(bug): with alpha_cutoff None, rgb / luminance include every pixel (visible_count = all),
    //            but AlphaStats gets a 0.0 cutoff, so coverage drops alpha-0 pixels. Use one rule for both.
    pub fn from_pixels(pixels: &[[f32; 4]], alpha_cutoff: Option<f32>) -> Option<Self> {
        let kept: Vec<&[f32; 4]> = pixels
            .iter()
            .filter(|p| alpha_cutoff.is_none_or(|c| p[3] > c))
            .collect();
        let channel = |i: usize| kept.iter().map(|p| p[i]).collect::<Vec<f32>>();
        let lum: Vec<f32> = kept
            .iter()
            .map(|p| LinearColor([p[0], p[1], p[2]]).luminance())
            .collect();
        Some(Self {
            visible_count: kept.len(),
            rgb: RgbStats::from_channels(&channel(0), &channel(1), &channel(2)),
            luminance: LuminanceStats::from_values(&lum),
            alpha: AlphaStats::from_pixels(pixels, alpha_cutoff.unwrap_or(0.0))?,
        })
    }
}

/// Mean of the channels of *one* pixel, e.g. [r, g, b] -> one number.
/// For a whole list of values (an image), use `Stats::average`.
pub trait ChannelMean {
    fn channel_mean(self) -> f32;
}

impl<const N: usize> ChannelMean for [f32; N] {
    fn channel_mean(self) -> f32 {
        (self.iter().map(|&v| v as f64).sum::<f64>() / N as f64) as f32
    }
}

/// Noise sigma from a high-pass residual: MAD * 1.4826, corrected for the 8-neighbour filter.
// TODO(2): needs noise_residual (commented out at the bottom) to be usable on an image.
// TODO(bug): panics on empty input (index out of bounds); return Option<f32> like the Stats methods.
pub fn estimate_noise_sigma(residual: &[f32]) -> f32 {
    let mut v: Vec<f32> = residual.to_vec();
    v.sort_by(f32::total_cmp);
    let med = v[v.len() / 2];
    let mut dev: Vec<f32> = v.iter().map(|x| (x - med).abs()).collect();
    dev.sort_by(f32::total_cmp);
    dev[dev.len() / 2] * 1.4826 / (9.0f32 / 8.0).sqrt()
}

/// True if a pixel's alpha is above the cutoff, i.e. it counts in the statistics.
pub fn is_visible(alpha: f32, cutoff: f32) -> bool {
    alpha > cutoff
}

/// Grey pixel from the plain mean of R, G, B (alpha kept). `luminance()` is the weighted version.
pub fn desaturate_mean([r, g, b, a]: [f32; 4]) -> [f32; 4] {
    let avg = [r, g, b].channel_mean();
    [avg, avg, avg, a]
}

/// Like `desaturate_mean`, but None for pixels that aren't visible, so callers can skip them.
pub fn desaturate_mean_visible([r, g, b, a]: [f32; 4], alpha_cutoff: f32) -> Option<[f32; 4]> {
    if !is_visible(a, alpha_cutoff) {
        return None;
    }
    let avg = [r, g, b].channel_mean();
    Some([avg, avg, avg, a])
}

/// `desaturate_mean` for every visible pixel; alpha at or below one 8-bit step (1/255) is dropped.
// TODO(6): derive the cutoff from the source bit depth (1/255 for 8-bit, 1/65535 for 16-bit).
pub fn desaturate_visible_pixels(pixels: &[[f32; 4]]) -> Vec<[f32; 4]> {
    let kept: Vec<[f32; 4]> = pixels
        .iter()
        .filter_map(|&p| desaturate_mean_visible(p, 1.0 / 255.0))
        .collect();
    kept
}

// TODO(1): implement from the sketch below: histogram(values, bins, lo, hi) -> Vec<u64>.
pub fn histogram() {
    todo!()
}

// /// Fixed-range histogram: `bins` buckets between lo and hi; values outside are clamped into the end buckets.
// pub fn histogram(values: &[f32], bins: usize, lo: f32, hi: f32) -> Vec<u64> {
//     let mut h = vec![0u64; bins];
//     let scale = bins as f32 / (hi - lo);
//     for &v in values.iter().filter(|v| v.is_finite()) {
//         let i = (((v - lo) * scale) as isize).clamp(0, bins as isize - 1) as usize;
//         h[i] += 1;
//     }
//     h
// }

// /// High-pass residual: each pixel minus the mean of its 8 neighbours (edges skipped).
// pub fn noise_residual(values: &[f32], width: usize, height: usize) -> Vec<f32> {
//     let mut out = Vec::with_capacity(width * height);
//     for y in 1..height - 1 {
//         for x in 1..width - 1 {
//             let mut sum = 0.0f32;
//             for dy in [-1isize, 0, 1] {
//                 for dx in [-1isize, 0, 1] {
//                     if dx != 0 || dy != 0 {
//                         sum += values[(y as isize + dy) as usize * width + (x as isize + dx) as usize];
//                     }
//                 }
//             }
//             out.push(values[y * width + x] - sum / 8.0);
//         }
//     }
//     out
// }
