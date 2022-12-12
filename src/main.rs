extern crate getopts;
use getopts::Options;
use image::{io::Reader as ImageReader, DynamicImage, GenericImage};
use std::{env, fs, fs::metadata, path::PathBuf, str::FromStr};

fn main() {
    let input_f: String;
    let output_f: String;
    let mut image_size: (u32, u32) = (800u32, 800u32);
    let mut tolerance: u8 = 10;
    let mut padding: u32 = 50;

    let args: Vec<String> = env::args().collect();
    let opts = prepare_cmd_opts();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}\n\n{}", f.to_string(), print_usage(&opts)),
    };

    if matches.opt_present("h") {
        print_usage(&opts);
    }

    input_f = match matches.opt_str("i") {
        Some(i) => i,
        None => panic!("{}", print_usage(&opts)),
    };
    output_f = match matches.opt_str("o") {
        Some(o) => o,
        None => panic!("{}", print_usage(&opts)),
    };

    image_size = match matches.opt_str("s") {
        Some(s) => {
            let v: Vec<u32> = s
                .split(' ')
                .map(|x| u32::from_str(x).expect("Failed to prase number"))
                .collect();
            (v[0], v[1])
        }
        None => image_size,
    };

    padding = match matches.opt_str("p") {
        Some(p) => u32::from_str(&p).expect("Failed to parse number"),
        None => padding,
    };

    tolerance = match matches.opt_str("t") {
        Some(t) => u8::from_str(&t).expect("Failed to parse number"),
        None => tolerance,
    };

    let input_path = PathBuf::from_str(&input_f.to_owned()).unwrap();
    let output_path = PathBuf::from_str(&output_f.to_owned()).unwrap();

    let ip_md = metadata(&input_path).unwrap();

    if ip_md.is_file() {
        let mut image = ImageReader::open(input_path.to_str().unwrap())
            .unwrap()
            .decode()
            .unwrap();
        image = process_image(image, image_size, padding, tolerance);
        let mut save_location = output_path.clone();
        save_location.push(input_path.file_name().clone().unwrap());

        match image.save(&save_location.to_str().unwrap()) {
            Ok(what) => println!("{:?}", what),
            Err(e) => panic!("{}", e),
        };
    } else if ip_md.is_dir() {
        for file in fs::read_dir(&input_path).unwrap() {
            let mut image_path = input_path.clone();
            image_path.push(file.as_ref().unwrap().file_name().clone());

            let mut image = ImageReader::open(&image_path.to_str().unwrap())
                .unwrap()
                .decode()
                .unwrap();
            image = process_image(image, image_size, padding, tolerance);
            let mut save_location = output_path.clone();
            save_location.push(file.as_ref().unwrap().file_name().clone());
            match image.save(&save_location.to_str().unwrap()) {
                Ok(what) => println!("{:?}", what),
                Err(e) => panic!("{}", e),
            };
        }
    } else {
        print_usage(&opts);
    }
}

fn process_image(
    mut image: DynamicImage,
    size: (u32, u32),
    padding: u32,
    tolerance: u8,
) -> DynamicImage {
    let mut image_grayscale = image.clone().to_luma8();

    let (target_width, target_height) = size;
    let (padded_width, padded_height) =
        (target_width - (2 * padding), target_height - (2 * padding));

    let (left, top, right, bottom) = image_boundbox(&mut image_grayscale, tolerance);

    let mut actual_object = image.crop(left, top, right - left, bottom - top);

    let object_width: i32 = actual_object.width() as i32;
    let object_height: i32 = actual_object.height() as i32;
    let size_change_x: i32 = padded_width as i32 - object_width as i32;
    let size_change_y: i32 = padded_height as i32 - object_height as i32;
    let new_size_x: i32;
    let new_size_y: i32;

    if object_width > object_height {
        new_size_x = object_width as i32 + size_change_x as i32;
        let increment: i32 = new_size_x - object_width as i32;
        new_size_y =
            object_height + (object_height * (increment as f32 / object_width as f32) as i32);
    } else {
        new_size_y = object_height + size_change_y;
        let increment: i32 = new_size_y - object_height;
        new_size_x =
            object_width + (object_width * (increment as f32 / object_height as f32) as i32);
    }

    actual_object = actual_object.resize_exact(
        new_size_x as u32,
        new_size_y as u32,
        image::imageops::FilterType::Gaussian,
    );

    let mut final_image = image::DynamicImage::new_rgb8(target_width, target_height);
    for x in 0..final_image.width() {
        for y in 0..final_image.height() {
            final_image.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
    }

    let ps_x: i32 = ((target_width as f32 / 2f32) - (new_size_x as f32 / 2f32)) as i32;
    let ps_y: i32 = ((target_height as f32 / 2f32) - (new_size_y as f32 / 2f32)) as i32;
    final_image
        .copy_from(&actual_object, ps_x as u32, ps_y as u32)
        .unwrap();

    final_image
}

fn image_boundbox(image: &mut image::GrayImage, tolerance: u8) -> (u32, u32, u32, u32) {
    let width = image.width();
    let height = image.height();

    let t = ((255u8 - tolerance)..=255u8).collect::<Vec<u8>>();

    let mut left: u32 = 0;
    for y in 0..height {
        for x in 0..width {
            if !t.contains(&image.get_pixel(x, y)[0]) {
                if (x > left && left == 0) || (x < left && left != 0) {
                    left = x;
                    break;
                }
            }
        }
    }

    let mut right = width;
    for y in 0..height {
        for x in (left..width).rev() {
            if !t.contains(&image.get_pixel(x, y)[0]) {
                if (x < right && right == width) || (x > right && right != width) {
                    right = x;
                    break;
                }
            }
        }
    }

    let mut top = 0;
    for x in left..right {
        for y in 0..height {
            if !t.contains(&image.get_pixel(x, y)[0]) {
                if (y > top && top == 0) || y < top && top != 0 {
                    top = y;
                    break;
                }
            }
        }
    }

    let mut bottom = height;
    for x in left..right {
        for y in (top..height).rev() {
            if !t.contains(&image.get_pixel(x, y)[0]) {
                if (y < bottom && bottom == height) || (y > bottom && bottom != height) {
                    bottom = y;
                    break;
                }
            }
        }
    }

    (left, top, right, bottom)
}

fn prepare_cmd_opts() -> Options {
    let mut opts = Options::new();
    opts.reqopt(
        "i",
        "input",
        "Input file location. If file location is a file then the program will use single image mode.",
        "NAME");
    opts.reqopt(
        "o",
        "output",
        "Output can be either a directory where the image will be stored with the same name as the input file name or it can be a folder where all images will be stored at if we're doing bulk processing", 
        "OUTPUT");
    opts.optopt(
        "s",
        "size",
        "Used to set the target image size. Must be passed as a string with two values Eg: -s \"800 800\"",
        "\"Xsize Ysize\"");
    opts.optopt(
        "t",
        "tolerance",
        "Used to control how much tolerance in color values the algorithm will have. The bigger the tolerance the less ipxels will pass the algorithm's test.",
        "VALUE");
    opts.optopt(
        "p",
        "padding",
        "How much white space will the result image have around the object.",
        "VALUE",
    );
    opts.optflag("h", "help", "Shows this manual.");

    return opts;
}

fn print_usage(opts: &Options) -> String {
    opts.usage("Utility for normalizing multiple image sizes for web listings")
}
