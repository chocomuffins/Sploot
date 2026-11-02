//! Comparisons between two images: max abs error, mean error, RMSE, PSNR, per channel.
//! Used by mip comparison and compression preview. Input is always linear.
//!
//! TODO:
//! 1. ImageMetrics { max_abs_error, mean_error, rmse, psnr } per channel (R, G, B, A),
//!    from two same-sized linear images; f64 sums like stats.rs.
//! 2. PSNR peak: 1.0 for LDR; for HDR use the source max (or report RMSE only).
//! 3. Normal maps: angular error in degrees after renormalizing (mean / max).
//! 4. Tests: identical images give 0 error and infinite PSNR; a known offset gives a known RMSE.
