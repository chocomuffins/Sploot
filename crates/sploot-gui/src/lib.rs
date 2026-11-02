//! sploot-gui: the viewer behind `sploot view`. A library, called from sploot-cli.
//!
//! TODO (eframe / egui):
//! - Window: `sploot view <file>`; with no file, an empty window waiting for a drop.
//! - Drag and drop: a file opens it, a second file starts an A/B compare, a folder browses its textures.
//! - Zoom and pan, nearest-neighbor filtering when zoomed in.
//! - Channels: RGB, R, G, B, A, luminance. Step through mip levels. Exposure slider for HDR.
//! - Pixel readout under the cursor (raw value + linear value).
//! - A/B compare: split slider or flip between the two.
//! - Show diff heatmaps and compression results from `sploot mips` / `sploot compress`.
//! - Quick in-terminal thumbnail (viuer) when no window is wanted.
