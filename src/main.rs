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

    #[arg(short, long, value_name = "PATH", value_parser = validate_image_path)]
    pub image: Option<String>,

    #[arg(short = 'n', long = "count", value_name = "N")]
    pub count: Option<usize>,

    #[arg(long = "set-default", value_name = "N")]
    pub set_default: Option<usize>,

    #[arg(
        long = "threshold",
        short = 't',
        value_name = "F",
        default_value_t = 10.0
    )]
    pub threshold: f32,

    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    #[arg(long = "time")]
    pub time: bool,

    #[arg(long = "saturate", value_name = "0.0-1.0")]
    pub saturate: Option<f32>,

    #[arg(short = 'r', long = "refresh")]
    pub refresh: bool,
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

        if args.refresh {
            cache::invalidate(&path);
        }

        if let Some(cached) = cache::load(&path) {
            if cached.colors.len() >= count {
                let colors: Vec<(u8, u8, u8)> = cached
                    .colors
                    .iter()
                    .filter_map(|hex| {
                        if hex.len() == 7 && hex.starts_with('#') {
                            let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
                            let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
                            let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
                            Some((r, g, b))
                        } else {
                            None
                        }
                    })
                    .collect();

                let colors = if colors.len() > count {
                    colors[..count].to_vec()
                } else {
                    colors
                };

                commands::display_palette(colors, args.quiet, args.time);
                return;
            }
        }

        commands::handle(
            path,
            count,
            args.threshold,
            args.quiet,
            args.time,
            args.saturate,
            true,
        );
    } else {
        println!("Extract colors from images and generate palettes.");
    }
}
