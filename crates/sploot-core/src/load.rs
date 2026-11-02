use std::path::Path;

use crate::{ChannelLayout, ColorSpace, SampleType, StorageFormat, srgb_to_linear_rgba};

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub format: StorageFormat,
    pub color_space: ColorSpace,
    pub pixels: Vec<[f32; 4]>,
}

// TODO: assess sRGB or linear from the file itself (PNG sRGB / gAMA / iCCP chunks, DDS *_SRGB formats;
//       EXR / HDR are linear). Use it when neither --srgb nor --linear is given, print which was used,
//       and let an explicit flag always win (the CLI's ColorSpaceArgs group then stops being required).
// TODO: DDS: route .dds files through image_dds and keep every stored mip level
//       (Vec of mips instead of one pixel list), with StorageFormat::Block(..) for BC files.
//       Needed by `sploot mips` and the compression preview.
// TODO: images larger than 4K (longest side > 4096): reduce to 4096 for stats, to save memory
//       (8K as [f32; 4] is ~1 GB) and time; the effect on the averages is negligible.
//       - Downsample after decoding, in linear space, with a box filter: the average stays the same
//         (exactly so for 2x steps), but min / max / median / std dev shift a little, and a
//         single-pixel hot spot (an HDR sun) gets averaged away.
//       - Only for luma / stats: mips and compress need the full-resolution pixels.
//       - Print it in the header ("8192 x 8192, measured at 4096 x 4096"), and add --full-res to skip it.
pub fn load(path: &Path, color_space: ColorSpace) -> Result<LoadedImage, image::ImageError> {
    let img = image::open(path)?;
    let width = img.width();
    let height = img.height();
    let format = storage_format(img.color());

    let pixels = img
        .to_rgba32f()
        .pixels()
        .map(|p| match color_space {
            ColorSpace::Srgb => srgb_to_linear_rgba(p.0),
            ColorSpace::Linear => p.0,
        })
        .collect();

    Ok(LoadedImage {
        width,
        height,
        format,
        color_space,
        pixels,
    })
}

// TODO: EXR / HDR are on the roadmap as untested: try a real .exr and .hdr.
//       The image crate has no f16 ColorType (f16 EXRs arrive as Rgb32F / Rgba32F), so half-float
//       files report as 32-bit float and SampleType::F16 is never used; read the EXR header to tell.
fn storage_format(color: image::ColorType) -> StorageFormat {
    use image::ColorType as C;
    let sample_type = match color {
        C::L8 | C::La8 | C::Rgb8 | C::Rgba8 => SampleType::U8,
        C::L16 | C::La16 | C::Rgb16 | C::Rgba16 => SampleType::U16,
        _ => SampleType::F32,
    };

    let channels = match color.channel_count() {
        1 => ChannelLayout::Gray,
        2 => ChannelLayout::GrayAlpha,
        3 => ChannelLayout::Rgb,
        _ => ChannelLayout::Rgba,
    };

    StorageFormat::Uncompressed {
        sample_type,
        channels,
    }
}
