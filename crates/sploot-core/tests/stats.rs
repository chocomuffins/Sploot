//! Integration tests: the Stats trait and ImageStats, on small hand-made pixel lists.
//! No files needed; the expected answers can be worked out by hand.
//!
//! TODO: add tests for
//! - Stats on a slice: average, min / max, variance / std_dev of [1.0, 2.0, 3.0, 4.0].
//! - median: odd length (middle value) and even length (mean of the two middle values).
//! - Empty input: every Stats method returns None; ImageStats::from_pixels(&[], ..) returns None.
//! - log_average: [1.0, 1.0] == 1.0; black pixels don't give -inf (epsilon).
//! - u8 / u16 input: results on the input's own scale (0-255, 0-65535).
//! - count_nan: NaNs are counted and don't break min / max.
//! - Alpha: mixed pixels with a cutoff (only visible pixels in rgb / luminance);
//!   fully transparent image gives rgb / luminance None and coverage 0.0.
//! - Known bug (see stats.rs): with alpha_cutoff None, coverage and visible_count disagree.
