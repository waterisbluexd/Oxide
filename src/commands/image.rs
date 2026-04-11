use crate::cache;
use color_quant::NeuQuant;
use image::imageops::FilterType;
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
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return (r, g, b);
    }

    let d = max - min;
    let _s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == rf {
        (gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }
    } else if max == gf {
        (bf - rf) / d + 2.0
    } else {
        (rf - gf) / d + 4.0
    };
    let h = h / 6.0;

    let s = amount.clamp(0.0, 1.0);

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    };

    (
        (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0).clamp(0.0, 255.0) as u8,
        (hue_to_rgb(p, q, h) * 255.0).clamp(0.0, 255.0) as u8,
        (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0).clamp(0.0, 255.0) as u8,
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

fn maxmin_select(colors: &[(u8, u8, u8)], count: usize) -> Vec<(u8, u8, u8)> {
    if colors.len() <= count {
        return colors.to_vec();
    }

    let mut selected = vec![colors[0]];

    while selected.len() < count {
        let next = colors
            .iter()
            .filter(|c| !selected.contains(c))
            .max_by(|a, b| {
                let da = selected
                    .iter()
                    .map(|s| color_distance(a.0, a.1, a.2, s.0, s.1, s.2))
                    .fold(f32::MAX, f32::min);
                let db = selected
                    .iter()
                    .map(|s| color_distance(b.0, b.1, b.2, s.0, s.1, s.2))
                    .fold(f32::MAX, f32::min);
                da.partial_cmp(&db).unwrap()
            });

        if let Some(&c) = next {
            selected.push(c);
        } else {
            break;
        }
    }

    selected
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
        colors[0] = darken(colors[0].0, colors[0].1, colors[0].2, 0.40);
        let mid = len / 2 - 1;
        colors[mid] = lighten(colors[0].0, colors[0].1, colors[0].2, 0.75);
        let last = len - 1;
        colors[last] = blend(
            colors[last].0,
            colors[last].1,
            colors[last].2,
            0xEE,
            0xEE,
            0xEE,
        );
        if let Some(s) = sat {
            for i in 1..mid {
                colors[i] = saturate(colors[i].0, colors[i].1, colors[i].2, s);
            }
        }
    }
}

pub fn run(
    path: String,
    count: usize,
    _threshold: f32,
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

    let rgba_img = img.to_rgba8();
    let pixels = rgba_img.as_raw();

    let nq = NeuQuant::new(10, 256, pixels);
    let color_map = nq.color_map_rgba();

    let mut all_colors: Vec<(u8, u8, u8)> =
        color_map.chunks(4).map(|c| (c[0], c[1], c[2])).collect();

    all_colors.sort_by(|a, b| {
        luminance(a.0, a.1, a.2)
            .partial_cmp(&luminance(b.0, b.1, b.2))
            .unwrap()
    });

    let mut deduped: Vec<(u8, u8, u8)> = Vec::new();
    for color in &all_colors {
        let too_close = deduped
            .iter()
            .any(|kept| color_distance(color.0, color.1, color.2, kept.0, kept.1, kept.2) < 10.0);
        if !too_close {
            deduped.push(*color);
        }
    }

    let mut selected = maxmin_select(&deduped, count);

    selected.sort_by(|a, b| {
        luminance(a.0, a.1, a.2)
            .partial_cmp(&luminance(b.0, b.1, b.2))
            .unwrap()
    });

    adjust(&mut selected, false, sat);

    let hex_strings: Vec<String> = selected
        .iter()
        .map(|(r, g, b)| format!("#{:02X}{:02X}{:02X}", r, g, b))
        .collect();
    cache::save(&path, &hex_strings);

    print_palette(&selected, show_hex);

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
