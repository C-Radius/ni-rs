
// main.rs
extern crate getopts;
use getopts::Options;
use image::{io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};
use std::{env, fs, fs::metadata, path::PathBuf, str::FromStr};

fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = prepare_cmd_opts();
    let matches = opts.parse(&args[1..]).unwrap_or_else(|f| {
        eprintln!("{}", f.to_string());
        std::process::exit(1);
    });

    if matches.opt_present("h") {
        println!("{}", opts.usage("Usage: normalize -i INPUT -o OUTPUT [options]"));
        return;
    }

    let input_f = matches.opt_str("i").expect("Missing input path.");
    let output_f = matches.opt_str("o").expect("Missing output path.");
    let image_size = matches.opt_str("s")
        .map(|s| {
            let parts: Vec<u32> = s
                .split_whitespace()
                .map(|x| u32::from_str(x).expect("Invalid number in size"))
                .collect();
            (parts[0], parts[1])
        })
        .unwrap_or((800, 800));
    let tolerance = matches.opt_str("t")
        .map(|t| u8::from_str(&t).expect("Invalid tolerance value"))
        .unwrap_or(10);
    let padding = matches.opt_str("p")
        .map(|p| u32::from_str(&p).expect("Invalid padding value"))
        .unwrap_or(50);

    let input_path = PathBuf::from(&input_f);
    let output_path = PathBuf::from(&output_f);

    if metadata(&input_path).unwrap().is_file() {
        process_and_save_image(&input_path, &output_path, image_size, padding, tolerance);
    } else {
        fs::create_dir_all(&output_path).ok();
        for entry in fs::read_dir(&input_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                process_and_save_image(&path, &output_path, image_size, padding, tolerance);
            }
        }
    }
}

fn process_and_save_image(
    path: &PathBuf,
    output_dir: &PathBuf,
    image_size: (u32, u32),
    padding: u32,
    tolerance: u8,
) {
    let image = ImageReader::open(path).unwrap().decode().unwrap();
    let processed = process_image(image, image_size, padding, tolerance);
    let mut save_path = output_dir.clone();
    save_path.push(path.file_name().unwrap());
    processed.save(&save_path).unwrap();
    println!("Saved: {:?}", save_path);
}

fn process_image(
    image: DynamicImage,
    size: (u32, u32),
    padding: u32,
    tolerance: u8,
) -> DynamicImage {
    let grayscale = image.to_luma8();
    let (left, top, right, bottom) = image_boundbox(&grayscale, tolerance);
    let cropped = image.crop_imm(left, top, right - left, bottom - top);
    let (target_w, target_h) = (size.0 - 2 * padding, size.1 - 2 * padding);
    let (w, h) = cropped.dimensions();
    let (new_w, new_h) = if w > h {
        let scale = target_w as f32 / w as f32;
        (target_w, (h as f32 * scale) as u32)
    } else {
        let scale = target_h as f32 / h as f32;
        ((w as f32 * scale) as u32, target_h)
    };
    let resized = cropped.resize_exact(new_w, new_h, image::imageops::FilterType::Gaussian);
    let mut canvas = DynamicImage::new_rgb8(size.0, size.1);
    for x in 0..canvas.width() {
        for y in 0..canvas.height() {
            canvas.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }
    let offset_x = (size.0 - new_w) / 2;
    let offset_y = (size.1 - new_h) / 2;
    canvas.copy_from(&resized, offset_x, offset_y).unwrap();
    canvas
}

fn image_boundbox(image: &image::GrayImage, tolerance: u8) -> (u32, u32, u32, u32) {
    let (w, h) = image.dimensions();
    let threshold = 255 - tolerance;
    let mut left = w;
    let mut right = 0;
    let mut top = h;
    let mut bottom = 0;
    for y in 0..h {
        for x in 0..w {
            if image.get_pixel(x, y)[0] < threshold {
                if x < left { left = x; }
                if x > right { right = x; }
                if y < top { top = y; }
                if y > bottom { bottom = y; }
            }
        }
    }
    (
        left.min(w - 1),
        top.min(h - 1),
        (right + 1).min(w),
        (bottom + 1).min(h),
    )
}

fn prepare_cmd_opts() -> Options {
    let mut opts = Options::new();
    opts.reqopt("i", "input", "Path to the input file or folder.", "PATH");
    opts.reqopt("o", "output", "Output folder.", "PATH");
    opts.optopt("s", "size", "Target image size in pixels, e.g., \"800 800\"", "\"WIDTH HEIGHT\"");
    opts.optopt("t", "tolerance", "Background tolerance (0–255).", "VALUE");
    opts.optopt("p", "padding", "Amount of white padding around the object.", "VALUE");
    opts.optflag("h", "help", "Show help message and exit.");
    opts
}

