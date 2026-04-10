use crate::Args;

mod image;

pub fn handle(args: Args) {
    if let Some(path) = args.image {
        image::run(path);
    } else {
        println!("Extract colors from images and generate palettes.");
    }
}
