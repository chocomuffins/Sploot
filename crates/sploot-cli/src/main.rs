use clap::{Args, Parser, Subcommand};
use sploot_core::cache::{self, CacheKey};
use sploot_core::stats::{DEFAULT_BINS, Histograms, ImageStats};
use sploot_core::{ColorSpace, linear_to_srgb_u8, load, rgb_to_hex};
use std::io::IsTerminal;
use std::path::PathBuf;

/// Low to the ground, close to the pixels.
#[derive(Parser, Debug)]
#[command(name = "sploot", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Measure average / log-average luminance
    Luma(LumaArgs),
    // TODO: `sploot mips <a> [b]`: compare stored mips against a reference chain (core mips.rs),
    //       or two textures' chains against each other. --filter, --heatmap <dir>, --normal
    //       (renormalize, report angular error in degrees). Prints metrics per mip and channel.
    // TODO: `sploot compress <file>`: --formats bc1,bc7 --quality fast,slow --out <dir>
    //       (core compress.rs). Prints the ranking table, writes the tiled image and heatmaps.
    // TODO: `sploot view [a] [b]`: opens the previewer (sploot-gui), behind the `gui` feature.
}

#[derive(Args, Debug)]
struct LumaArgs {
    /// Texture file or folder
    // TODO: batch mode: when input is a folder, run every texture in it (one row per file)
    //       and add --csv for a spreadsheet-friendly table. Only single files work today.
    // TODO: compare: accept several files (or mips) and print their stats side by side,
    //       one column per image plus the difference to the first (luminance, average color).
    input: PathBuf,

    #[command(flatten)]
    color: ColorSpaceArgs,

    /// Geometric mean, for HDR / exposure work
    // TODO: unused; the table always shows log-average. Remove it, or make it print only
    //       that number (handy in scripts).
    #[arg(long)]
    log_average: bool,

    /// Machine-readable output
    #[arg(long)]
    json: bool,

    /// Skip cached data
    #[arg(long)]
    no_cache: bool,
}

// --srgb / --linear: exactly one is required.
#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct ColorSpaceArgs {
    /// Source is sRGB-encoded (albedo, UI)
    #[arg(long)]
    srgb: bool,
    /// Source is already linear (HDR, masks, normals)
    #[arg(long)]
    linear: bool,
}

impl ColorSpaceArgs {
    fn color_space(&self) -> ColorSpace {
        if self.srgb {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Luma(args) => luma(&args)?,
    }
    Ok(())
}

fn luma(args: &LumaArgs) -> Result<(), Box<dyn std::error::Error>> {
    let color_space = args.color.color_space();
    let skip_cache = args.no_cache;

    // Header only: enough for the cache key and the table header, without decoding.
    let (width, height, format) = load::header(&args.input)?;
    let key = CacheKey::for_stats(&args.input, color_space, width, height)?;
    let cache_dir = cache::default_dir();

    let cached = if skip_cache {
        None
    } else {
        cache_dir
            .as_deref()
            .and_then(|dir| key.load(dir))
    };

    let from_cache = cached.is_some(); // before the match: `match cached` moves it

    let (stats, _histograms) = match cached {
        Some(hit) => hit,
        None => {
            let img = load::load(&args.input, color_space)?;

            // TODO: alpha mask: add --alpha-cutoff <value> and pass it here instead of None,
            //       so transparent pixels can be left out of luminance / color (see TODO(bug) in stats.rs).
            let stats = ImageStats::from_pixels(&img.pixels, None).ok_or("image has no pixels")?;
            let histograms = Histograms::from_pixels(&img.pixels, DEFAULT_BINS, 0.0, 1.0);

            if let Some(dir) = &cache_dir
                && let Err(e) = key.save(dir, &stats, &histograms)
            {
                eprintln!(
                    "warning: couldn't write the cache in {}: {e}",
                    dir.display()
                );
            }

            (stats, histograms)
        }
    };

    println!("{}", args.input.display());
    println!(
        "{} x {} | {} | decoded as {} | {} pixels{}",
        width,
        height,
        format,
        color_space,
        thousands(width as usize * height as usize),
        if from_cache { " | cached" } else { "" }
    );
    println!();

    match stats.luminance {
        Some(l) => {
            println!("┌{:─<15}┬{:─>15}┬{:─>16}┐", "", "", "");
            println!(
                "│ {:<13} │ {:>13} │ {:>14} │",
                "Luminance", "linear", "sRGB 8-bit"
            );
            println!("├{:─<15}┼{:─>15}┼{:─>16}┤", "", "", "");
            for (label, v) in [
                ("average", l.average),
                ("log-average", l.log_average),
                ("median", l.median),
                ("min", l.min),
                ("max", l.max),
            ] {
                println!(
                    "│ {:<13} │ {:>13.4} │ {:>14} │",
                    label,
                    v,
                    linear_to_srgb_u8(v as f32)
                );
            }
            println!("│ {:<13} │ {:>13.4} │ {:>14} │", "std dev", l.std_dev, "");
            println!("└{:─<15}┴{:─>15}┴{:─>16}┘", "", "", "");
        }
        None => println!("Luminance: no visible pixels"),
    }
    println!();

    if let Some(rgb) = stats.rgb {
        let avg = rgb.average_color().0;
        let [r8, g8, b8] = avg.map(linear_to_srgb_u8);
        println!("┌{:─<15}┬{:─>10}┬{:─>10}┬{:─>10}┐", "", "", "", "");
        println!(
            "│ {:<13} │ {:>8} │ {:>8} │ {:>8} │",
            "Average color", "R", "G", "B"
        );
        println!("├{:─<15}┼{:─>10}┼{:─>10}┼{:─>10}┤", "", "", "", "");
        println!(
            "│ {:<13} │ {:>8.4} │ {:>8.4} │ {:>8.4} │",
            "linear", avg[0], avg[1], avg[2]
        );
        println!(
            "│ {:<13} │ {:>8} │ {:>8} │ {:>8} │",
            "sRGB 8-bit", r8, g8, b8
        );

        let block = swatch(avg);
        if !block.is_empty() {
            println!(
                "│ {:<13} │ {block}{:>4} │ {:>8} │ {:>8} │",
                "sRGB color", "", "", ""
            );
        }
        println!(
            "│ {:<13} │ {:>8} │ {:>8} │ {:>8} │",
            "Hex",
            rgb_to_hex([r8, g8, b8]),
            "",
            ""
        );
        println!("└{:─<15}┴{:─>10}┴{:─>10}┴{:─>10}┘", "", "", "", "");
        println!();
    }

    println!("┌{:─<15}┬{:─>32}┐", "", "");
    println!("│ {:<13} │ {:>30} │", "Alpha", "");
    println!("├{:─<15}┼{:─>32}┤", "", "");
    println!(
        "│ {:<13} │ {:>29.1}% │",
        "coverage",
        stats.alpha.coverage * 100.0
    );
    println!(
        "│ {:<13} │ {:>30} │",
        "opaque",
        thousands(stats.alpha.opaque_count)
    );
    println!(
        "│ {:<13} │ {:>30} │",
        "transparent",
        thousands(stats.alpha.transparent_count)
    );
    println!("└{:─<15}┴{:─>32}┘", "", "");
    println!();

    Ok(())
}

/// 1048576 -> "1,048,576"
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// A small block in the given linear color, if the terminal can show it (and NO_COLOR isn't set).
fn swatch(linear_rgb: [f32; 3]) -> String {
    if !std::io::stdout().is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        return String::new();
    }
    let [r, g, b] = linear_rgb.map(linear_to_srgb_u8);
    format!("\x1b[48;2;{r};{g};{b}m    \x1b[0m")
}
