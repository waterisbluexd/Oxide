mod cache;
mod commands;
mod config;
use clap::Parser;

fn validate_image_path(s: &str) -> Result<String, String> {
    if !std::path::Path::new(s).exists() {
        return Err(format!("File not found: '{}'", s));
    }

    let kind = infer::get_from_path(s)
        .map_err(|e| format!("Cannot read file: {e}"))?
        .ok_or_else(|| format!("'{}' is not a recognized file type", s))?;

    if kind.matcher_type() != infer::MatcherType::Image {
        return Err(format!(
            "'{}' is not an image (detected: {})",
            s,
            kind.mime_type()
        ));
    }

    Ok(s.to_string())
}

#[derive(Parser, Debug)]
#[command(
    name = "oxide",
    about = "Extract colors from images and generate palettes.",
    version = "oxide 0.1.0",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub _version: (),

    /// Path to the image to extract colors from
    #[arg(short, long, value_name = "PATH", value_parser = validate_image_path)]
    pub image: Option<String>,

    /// Number of colors to extract (overrides set-default for this run)
    #[arg(short = 'n', long = "count", value_name = "N")]
    pub count: Option<usize>,

    /// Set the default palette size saved to config
    #[arg(long = "set-default", value_name = "N")]
    pub set_default: Option<usize>,

    /// Minimum perceptual distance between colors (higher = fewer similar colors)
    #[arg(
        long = "threshold",
        short = 't',
        value_name = "F",
        default_value_t = 10.0
    )]
    pub threshold: f32,

    /// Show hex values alongside swatches
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Show how long extraction took
    #[arg(long = "time")]
    pub time: bool,

    /// Override saturation of extracted colors (0.0–1.0)
    #[arg(long = "saturate", value_name = "0.0-1.0")]
    pub saturate: Option<f32>,

    /// Bypass cache and re-extract colors from the image
    #[arg(long = "reload", short = 'r')]
    pub reload: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(size) = args.set_default {
        let mut cfg = config::load();
        cfg.palette_size = size;
        config::save(&cfg);
        return;
    }

    if let Some(path) = args.image {
        let cfg = config::load();
        let count = args.count.unwrap_or(cfg.palette_size);
        commands::handle(
            path,
            count,
            args.threshold,
            args.quiet,
            args.time,
            args.saturate,
            args.reload,
        );
    } else {
        println!("Extract colors from images and generate palettes.");
        println!("Usage: oxide --image <PATH> [OPTIONS]");
        println!("       oxide --help");
    }
}
