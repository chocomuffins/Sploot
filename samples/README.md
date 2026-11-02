# Samples

## `dds/`: generated test textures (committed)

Procedural images made by `tools/make_test_dds.py`, so they carry no third-party license.
All are 256 x 256 with a full mip chain (9 levels), filtered in linear space, unless noted.

| File | Format | What it tests |
|---|---|---|
| `sploot_rgba8_srgb.dds` | R8G8B8A8_UNORM_SRGB | uncompressed reference, alpha |
| `sploot_bc1_srgb.dds` | BC1_UNORM_SRGB | BC1, no alpha |
| `sploot_bc3_srgb.dds` | BC3_UNORM_SRGB | BC3 with alpha (soft hole, transparent corner) |
| `sploot_bc7_srgb.dds` | BC7_UNORM_SRGB | BC7 (simple mode-6 encoder) |
| `sploot_dxt1_legacy.dds` | DXT1 (FourCC, no DX10 header) | old-style header |
| `sploot_bc1_srgb_badmip3.dds` | BC1_UNORM_SRGB | mip 3 is deliberately wrong (magenta), for `sploot mips` |
| `sploot_bc1_srgb_npot.dds` | BC1_UNORM_SRGB | 200 x 120, odd mip sizes down to 3 x 1 and 1 x 1 |
| `sploot_bc4_height.dds` | BC4_UNORM | single channel, linear (height / mask) |
| `sploot_bc5_normal.dds` | BC5_UNORM | tangent-space normal map, XY only, mips renormalized |
| `sploot_rgba16f_hdr.dds` | R16G16B16A16_FLOAT | linear HDR, sun disc at 50.0 |
| `sploot_albedo_source.png` | PNG, 8-bit sRGB RGBA | mip 0 of the albedo, to compare the DDS files against |

The image has smooth gradients (BC1 banding), 1 px and 2 px stripes (mip aliasing), a checker,
saturated discs and a soft-edged alpha hole.

The BC files come from simple encoders and are meant for testing loading, mips and formats,
not for judging compression quality. Use `image_dds` (Slow quality) for that.

## Test images (512 x 512 PNG, committed)

Made by `tools/make_test_images.py`. Handy because the right answers are known.

| File | Content | Decode as | Expected `sploot luma` |
|---|---|---|---|
| `sploot_rainbow.png` | full hue circle left to right, tiles horizontally | `--srgb` | average luminance 0.437 |
| `sploot_uv.png` | R = 0..1 left to right, G = 0..1 top to bottom, B = 0 (8-bit) | `--linear` | average color 0.5, 0.5, 0.0 |
| `sploot_uv_16bit.png` | same, 16-bit: one unique value per column / row | `--linear` | average color 0.5, 0.5, 0.0 |
| `sploot_checker.png` | 8 x 8 black / white squares of 64 px, black top-left | either | average luminance exactly 0.5 |

## Other textures

The Textures.com files used during development are not committed (their license doesn't allow
redistribution); `/samples/TCom_*` is in `.gitignore`.

To add real textures to the repo, use CC0 sources (no attribution needed, redistribution allowed):

- [Poly Haven](https://polyhaven.com/textures): PBR textures and HDRIs (PNG / JPG / EXR).
- [ambientCG](https://ambientcg.com): PBR materials, including 16-bit PNG and EXR maps.

Keep committed files small (1K or less); larger ones can stay local and be ignored.
