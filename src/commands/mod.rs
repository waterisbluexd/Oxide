mod image;

pub fn handle(
    path: String,
    count: usize,
    threshold: f32,
    quiet: bool,
    time: bool,
    sat: Option<f32>,
) -> Vec<String> {
    image::run(path, count, threshold, quiet, time, sat)
}

pub fn display_palette(colors: Vec<(u8, u8, u8)>, show_hex: bool, show_time: bool) {
    let mid = colors.len() / 2;
    let dark = &colors[..mid];
    let light_row = &colors[mid..];

    let print_row = |row: &[(u8, u8, u8)]| {
        for (r, g, b) in row {
            print!("\x1b[48;2;{r};{g};{b}m    \x1b[0m");
        }
        if show_hex {
            for (r, g, b) in row {
                print!("#{:02X}{:02X}{:02X} ", r, g, b);
            }
            println!();
        }
        println!();
    };

    print_row(dark);
    print_row(light_row);

    if show_time {
        println!("[i] Cached");
    }
}
