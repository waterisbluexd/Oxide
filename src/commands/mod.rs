mod image;

pub fn handle(
    path: String,
    count: usize,
    threshold: f32,
    quiet: bool,
    time: bool,
    sat: Option<f32>,
    reload: bool,
) {
    image::run(path, count, threshold, quiet, time, sat, reload);
}
