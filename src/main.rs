mod commands;
use clap::Parser;
use std::path::Path;

fn validate_image_path(s: &str) -> Result<String, String> {
    let path = Path::new(s);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some("png") | Some("jpg") | Some("jpeg") | Some("bmp") | Some("webp") | Some("tiff")
        | Some("tif") => Ok(s.to_string()),
        Some(_) => Err(format!(
            "Unsupported image format: '{}'. Supported: png, jpg, jpeg, webp, tiff",
            path.extension().and_then(|e| e.to_str()).unwrap_or("")
        )),
        None => Err("File must have an image extension (e.g., .png, .jpg)".to_string()),
    }
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
