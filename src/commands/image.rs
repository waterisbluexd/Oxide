use crate::cache;
use image::imageops::FilterType;
use kmeans_colors::get_kmeans;
use palette::{cast::from_component_slice, FromColor, Hsl, IntoColor, Lab, Srgb};
use std::io::Cursor;

fn darken(r: u8, g: u8, b: u8, amount: f32) -> (u8, u8, u8) {
    (
        (r as f32 * (1.0 - amount)) as u8,
        (g as f32 * (1.0 - amount)) as u8,
        (b as f32 * (1.0 - amount)) as u8,
    )
}

fn lighten(r: u8, g: u8, b: u8, amount: f32) -> (u8, u8, u8) {
    (
        (r as f32 + (255.0 - r as f32) * amount) as u8,
        (g as f32 + (255.0 - g as f32) * amount) as u8,
        (b as f32 + (255.0 - b as f32) * amount) as u8,
    )
}

fn blend(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> (u8, u8, u8) {
    (
        (0.5 * r1 as f32 + 0.5 * r2 as f32) as u8,
        (0.5 * g1 as f32 + 0.5 * g2 as f32) as u8,
        (0.5 * b1 as f32 + 0.5 * b2 as f32) as u8,
    )
}

fn saturate(r: u8, g: u8, b: u8, amount: f32) -> (u8, u8, u8) {
    let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let mut hsl: Hsl = Hsl::from_color(srgb);
    hsl.saturation = amount;
    let out: Srgb = Srgb::from_color(hsl);
    (
        (out.red * 255.0).clamp(0.0, 255.0) as u8,
        (out.green * 255.0).clamp(0.0, 255.0) as u8,
        (out.blue * 255.0).clamp(0.0, 255.0) as u8,
    )
}

fn luminance(r: u8, g: u8, b: u8) -> f32 {
    0.2126 * (r as f32 / 255.0) + 0.7152 * (g as f32 / 255.0) + 0.0722 * (b as f32 / 255.0)
}

fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f32 {
    let dr = r1 as f32 - r2 as f32;
    let dg = g1 as f32 - g2 as f32;
    let db = b1 as f32 - b2 as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

fn adjust(colors: &mut Vec<(u8, u8, u8)>, light: bool, sat: Option<f32>) {
    let len = colors.len();
    if len < 2 {
        return;
    }

    if light {
        if let Some(s) = sat {
            for c in colors.iter_mut() {
                *c = saturate(c.0, c.1, c.2, s);
            }
        }
        let last = colors[len - 1];
        let first = colors[0];
        colors[0] = lighten(last.0, last.1, last.2, 0.85);
        colors[len / 2 - 1] = first;
        let darkened = darken(last.0, last.1, last.2, 0.4);
        colors[len / 2] = darkened;
        colors[len - 1] = first;
    } else {
        let bg = colors[0];
        if bg.0 != 0 {
            colors[0] = darken(bg.0, bg.1, bg.2, 0.40);
        }
        let mid = len / 2 - 1;
        let c7 = colors[mid];
        colors[mid] = blend(c7.0, c7.1, c7.2, 0xEE, 0xEE, 0xEE);
        let c7_blended = colors[mid];
        colors[mid + 1] = darken(c7_blended.0, c7_blended.1, c7_blended.2, 0.30);
        let last_idx = len - 1;
        let fg = colors[last_idx];
        colors[last_idx] = blend(fg.0, fg.1, fg.2, 0xEE, 0xEE, 0xEE);

        if let Some(s) = sat {
            for c in colors.iter_mut() {
                *c = saturate(c.0, c.1, c.2, s);
            }
        }
    }
}

pub fn run(
    path: String,
    count: usize,
    threshold: f32,
    show_hex: bool,
    show_time: bool,
    sat: Option<f32>,
    reload: bool,
) {
    let start = std::time::Instant::now();

    let filename = std::path::Path::new(&path)
        .file_name()
        .unwrap()
        .to_string_lossy();

    if reload {
        cache::invalidate(&path);
        println!("[i] cache: Invalidated cache for {}.", filename);
    } else if let Some(cached) = cache::load(&path) {
        println!("[i] cache: Using cached palette for {}.", filename);
        let hex_colors: Vec<(u8, u8, u8)> = cached
            .colors
            .iter()
            .map(|s| {
                let s = s.trim_start_matches('#');
                let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
                (r, g, b)
            })
            .collect();

        print_palette(&hex_colors, show_hex);

        if show_time {
            println!("[i] Done in {:.2?}", start.elapsed());
        }
        return;
    }

    println!("[i] image: Using image {}.", filename);

    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("{}", e);
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
    let rgb_img = img.to_rgb8();
    let pixels = rgb_img.as_raw();

    let lab: Vec<Lab> = from_component_slice::<Srgb<u8>>(pixels)
        .iter()
        .map(|x| x.into_linear().into_color())
        .collect();

    let k = count + 8;
    let result = get_kmeans(k, 10, 1.0, false, &lab, 42);

    let colors: Vec<(u8, u8, u8)> = result
        .centroids
        .iter()
        .map(|&lab| {
            let srgb: Srgb = Srgb::from_linear(Lab::into_color(lab));
            let srgb8: Srgb<u8> = srgb.into_format();
            (srgb8.red, srgb8.green, srgb8.blue)
        })
        .collect();

    let mut deduped: Vec<(u8, u8, u8)> = Vec::new();
    for color in &colors {
        let too_close = deduped.iter().any(|kept| {
            color_distance(color.0, color.1, color.2, kept.0, kept.1, kept.2) < threshold
        });
        if !too_close {
            deduped.push(*color);
        }
    }

    if deduped.len() < count {
        for color in &colors {
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

    adjust(&mut deduped, false, sat);

    let hex_strings: Vec<String> = deduped
        .iter()
        .map(|(r, g, b)| format!("#{:02X}{:02X}{:02X}", r, g, b))
        .collect();
    cache::save(&path, &hex_strings);

    print_palette(&deduped, show_hex);

    if show_time {
        println!("[i] Done in {:.2?}", start.elapsed());
    }
}

fn print_palette(colors: &[(u8, u8, u8)], show_hex: bool) {
    let mid = colors.len() / 2;
    let dark = &colors[..mid];
    let light_row = &colors[mid..];

    let print_row = |row: &[(u8, u8, u8)]| {
        for (r, g, b) in row {
            print!("\x1b[48;2;{r};{g};{b}m    \x1b[0m");
        }
        println!();
        if show_hex {
            for (r, g, b) in row {
                print!("#{:02X}{:02X}{:02X} ", r, g, b);
            }
            println!();
        }
    };

    print_row(dark);
    print_row(light_row);
}
