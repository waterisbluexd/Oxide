use auto_palette::{Algorithm, ImageData, Palette};
use image::imageops::FilterType;
use std::io::Cursor;

fn luminance(r: u8, g: u8, b: u8) -> f32 {
    0.2126 * (r as f32 / 255.0) + 0.7152 * (g as f32 / 255.0) + 0.0722 * (b as f32 / 255.0)
}

fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f32 {
    let dr = r1 as f32 - r2 as f32;
    let dg = g1 as f32 - g2 as f32;
    let db = b1 as f32 - b2 as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

pub fn run(path: String, count: usize, threshold: f32) {
    let start = std::time::Instant::now();

    let filename = std::path::Path::new(&path)
        .file_name()
        .unwrap()
        .to_string_lossy();

    println!("[i] image: Using image {}.", filename);
    println!(
        "[i] colors: Extracting {} colors (threshold: {})...\n",
        count, threshold
    );

    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    let img = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        })
        .decode()
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    let (w, h) = (img.width(), img.height());
    let img = img.resize(w / 4, h / 4, FilterType::Nearest);
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let image_data = ImageData::new(width, height, rgba.as_raw()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let palette: Palette<f64> = Palette::builder()
        .algorithm(Algorithm::KMeans)
        .build(&image_data)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });

    let swatches = palette.find_swatches(count * 4).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    if swatches.is_empty() {
        eprintln!("[!] No colors found.");
        std::process::exit(1);
    }

    let raw_colors: Vec<(u8, u8, u8)> = swatches
        .iter()
        .map(|s| {
            let rgb = s.color().to_rgb();
            (rgb.r as u8, rgb.g as u8, rgb.b as u8)
        })
        .collect();

    let mut deduped: Vec<(u8, u8, u8)> = Vec::new();
    for color in &raw_colors {
        let too_close = deduped.iter().any(|kept| {
            color_distance(color.0, color.1, color.2, kept.0, kept.1, kept.2) < threshold
        });
        if !too_close {
            deduped.push(*color);
        }
    }

    if deduped.len() < count {
        for color in &raw_colors {
            if deduped.len() >= count {
                break;
            }
            if !deduped.contains(color) {
                deduped.push(*color);
            }
        }
    }

    deduped.truncate(count);

    deduped.sort_by(|a, b| {
        luminance(a.0, a.1, a.2)
            .partial_cmp(&luminance(b.0, b.1, b.2))
            .unwrap()
    });

    let mid = deduped.len() / 2;
    let dark = &deduped[..mid];
    let light = &deduped[mid..];

    let print_row = |row: &[(u8, u8, u8)]| {
        for (r, g, b) in row {
            print!("\x1b[48;2;{r};{g};{b}m        \x1b[0m ");
        }
        println!();
        for (r, g, b) in row {
            print!("#{:02X}{:02X}{:02X}  ", r, g, b);
        }
        println!("\n");
    };

    print_row(dark);
    print_row(light);

    println!("[i] Done in {:.2?}", start.elapsed());
}
