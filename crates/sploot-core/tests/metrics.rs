//! Integration tests: error metrics between two images (metrics.rs).
//!
//! TODO: add tests once ImageMetrics exists:
//! - Identical images: max abs error, mean error and RMSE are 0; PSNR is infinite.
//! - Known offset: every pixel +0.1 gives max abs error 0.1, mean error 0.1, RMSE 0.1.
//! - Per channel: an error in R only shows up in R, not in G / B / A.
//! - Different sizes: an error, not a panic.
//! - Normal maps: a 90 degree difference reports 90 degrees of angular error.
