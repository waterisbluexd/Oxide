mod commands;
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
}

fn main() {
    let args = Args::parse();
    commands::handle(args);
}
