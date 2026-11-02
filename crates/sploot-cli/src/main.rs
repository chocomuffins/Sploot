use clap::{Args, Parser, Subcommand};
use sploot_core::{ColorSpace, linear_to_srgb_u8, load, rgb_to_hex, stats::ImageStats};
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
    // TODO: unused; print ImageStats (plus width / height / format) as JSON instead of the tables.
    //       Needs serde::Serialize on the stats structs (see stats.rs) and serde_json.
    #[arg(long)]
    json: bool,
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
    let img = load::load(&args.input, args.color.color_space())?;
    // TODO: alpha mask: add --alpha-cutoff <value> and pass it here instead of None,
    //       so transparent pixels can be left out of luminance / color (see TODO(bug) in stats.rs).
    let stats = ImageStats::from_pixels(&img.pixels, None).ok_or("image has no pixels")?;

    println!("{}", args.input.display());
    println!(
        "{} x {} | {} | decoded as {} | {} pixels",
        img.width,
        img.height,
        img.format,
        img.color_space,
        thousands(img.pixels.len())
    );
    println!();

    match stats.luminance {
        Some(l) => {
            println!(
                "| {:<13} | {:>8} | {:>10} |",
                "Luminance", "linear", "sRGB 8-bit"
            );
            println!("|{:-<15}|{:->10}|{:->12}|", "", "", "");
            for (label, v) in [
                ("average", l.average),
                ("log-average", l.log_average),
                ("median", l.median),
                ("min", l.min),
                ("max", l.max),
            ] {
                println!(
                    "| {:<13} | {:>8.4} | {:>10} |",
                    label,
                    v,
                    linear_to_srgb_u8(v as f32)
                );
            }
            println!("| {:<13} | {:>8.4} | {:>10} |", "std dev", l.std_dev, "");
        }
        None => println!("Luminance: no visible pixels"),
    }
    println!();

    if let Some(rgb) = stats.rgb {
        let avg = rgb.average_color().0;
        let [r8, g8, b8] = avg.map(linear_to_srgb_u8);
        println!(
            "| {:<13} | {:>8} | {:>8} | {:>8} |",
            "Average color", "R", "G", "B"
        );
        println!("|{:-<15}|{:->10}|{:->10}|{:->10}|", "", "", "", "");
        println!(
            "| {:<13} | {:>8.4} | {:>8.4} | {:>8.4} |",
            "linear", avg[0], avg[1], avg[2]
        );
        // Build the row first, so the HEX line below can be indented to the swatch column.
        let row = format!(
            "| {:<13} | {:>8} | {:>8} | {:>8} | ",
            "sRGB 8-bit", r8, g8, b8
        );
        println!("{row}{}", swatch(avg));
        println!(
            "{:width$}{}",
            "",
            rgb_to_hex([r8, g8, b8]),
            width = row.len()
        );
        println!();
    }

    println!("| {:<13} | {:>11} |", "Alpha", "");
    println!("|{:-<15}|{:->13}|", "", "");
    println!(
        "| {:<13} | {:>10.1}% |",
        "coverage",
        stats.alpha.coverage * 100.0
    );
    println!(
        "| {:<13} | {:>11} |",
        "opaque",
        thousands(stats.alpha.opaque_count)
    );
    println!(
        "| {:<13} | {:>11} |",
        "transparent",
        thousands(stats.alpha.transparent_count)
    );

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
