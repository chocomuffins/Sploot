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

    /// Machine-readable output
    #[arg(long)]
    json: bool,

    /// Skip cached data
    #[arg(long)]
    no_cache: bool,

    /// Leave out pixels with alpha at or below this value (0-1)
    #[arg(long)]
    alpha_cutoff: Option<f32>,
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
    let alpha_cutoff = args.alpha_cutoff;

    if let Some(c) = alpha_cutoff
        && !(0.0..=1.0).contains(&c)
    {
        return Err("--alpha-cutoff must be between 0 and 1".into());
    }

    // Header only: enough for the cache key and the table header, without decoding.
    let (width, height, format) = load::header(&args.input)?;
    let key = CacheKey::for_stats(&args.input, color_space, alpha_cutoff, width, height)?;
    let cache_dir = cache::default_dir();

    let cached = if skip_cache {
        None
    } else {
        cache_dir.as_deref().and_then(|dir| key.load(dir))
    };

    let from_cache = cached.is_some(); // before the match: `match cached` moves it

    let (stats, histograms) = match cached {
        Some(hit) => hit,
        None => {
            let img = load::load(&args.input, color_space)?;

            let stats =
                ImageStats::from_pixels(&img.pixels, alpha_cutoff).ok_or("image has no pixels")?;
            let histograms =
                Histograms::from_pixels(&img.pixels, alpha_cutoff, color_space, DEFAULT_BINS);

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

    print_histograms(&histograms, color_space);

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

/// Width of the histogram graphs: the full 50-character table minus `│ L │ ` and ` │`.
const GRAPH_WIDTH: usize = 42;

/// Luminance and R / G / B histograms as one-line bar graphs, 0 on the left, 1 on the right.
fn print_histograms(h: &Histograms, space: ColorSpace) {
    let title = match space {
        ColorSpace::Srgb => "Histogram (sRGB-encoded)",
        ColorSpace::Linear => "Histogram (linear)",
    };

    println!("┌{:─<48}┐", "");
    println!("│ {:<46} │", title);
    println!("├───┬{:─<44}┤", "");

    let rows = [
        ("L", &h.luminance, None),
        ("R", &h.rgb[0], Some([230, 70, 70])),
        ("G", &h.rgb[1], Some([80, 200, 90])),
        ("B", &h.rgb[2], Some([80, 130, 255])),
    ];
    for (label, counts, color) in rows {
        let graph = bar_graph(counts, GRAPH_WIDTH);
        let graph = match color {
            Some(rgb) => colored(&graph, rgb),
            None => graph,
        };
        println!("│ {label} │ {graph} │");
    }

    println!("│   │ {:<20}{:^3}{:>19} │", "0", "0.5", "1");
    println!("└───┴{:─<44}┘", "");
    println!();
}

/// Squeezes `counts` into `width` columns drawn with ▁..█. Each column is the average of its
/// bins: bins don't divide evenly into columns, and a sum would make 7-bin columns look taller.
/// Heights use a square-root scale, so one big spike doesn't flatten everything else.
/// Empty columns (no pixels at all) are drawn as `_`, so the baseline stays visible.
fn bar_graph(counts: &[u64], width: usize) -> String {
    const LEVELS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let columns: Vec<f64> = (0..width)
        .map(|c| {
            let start = c * counts.len() / width;
            let end = (c + 1) * counts.len() / width;
            let sum: u64 = counts[start..end].iter().sum();
            sum as f64 / (end - start).max(1) as f64
        })
        .collect();
    let max = columns.iter().copied().fold(0.0, f64::max);

    columns
        .iter()
        .map(|&v| {
            if v == 0.0 {
                return '_';
            }
            let level = ((v / max).sqrt() * 7.0).round() as usize;
            LEVELS[level.min(7)]
        })
        .collect()
}

/// True when the terminal can show colors (stdout is a terminal and NO_COLOR isn't set).
fn use_color() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

/// `text` in the given sRGB color, if the terminal can show it.
fn colored(text: &str, [r, g, b]: [u8; 3]) -> String {
    if !use_color() {
        return text.to_string();
    }
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

/// A small block in the given linear color, if the terminal can show it (and NO_COLOR isn't set).
fn swatch(linear_rgb: [f32; 3]) -> String {
    if !use_color() {
        return String::new();
    }
    let [r, g, b] = linear_rgb.map(linear_to_srgb_u8);
    format!("\x1b[48;2;{r};{g};{b}m    \x1b[0m")
}
