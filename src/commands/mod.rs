mod image;

pub fn handle(path: String, count: usize, threshold: f32) {
    image::run(path, count, threshold);
}
