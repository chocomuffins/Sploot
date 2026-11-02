//! Diff / error heatmap images, written to disk.
//!
//! TODO:
//! - |a - b| per pixel (per channel or max over channels), mapped through a color ramp.
//! - Used per mip (`sploot mips --heatmap`) and per compression setting (`sploot compress`).
//! - Save as PNG; fixed or auto scale, with the scale printed so images are comparable.
