extern crate getopts;
extern crate photon;
use getopts::Options;
use photon::monochrome::grayscale;
use photon::{monochrome::monochrome, native::open_image, PhotonImage};
use std::{env, fs, fs::metadata, path::PathBuf, str::FromStr};

const SUPPORTED_FORMATS: [&'static str; 18] = [
    ".jpg", ".jpeg", ".bmp", ".dds", ".exif", ".gif", ".jps", ".jp2", ".jpx", ".pcx", ".png",
    ".pnm", ".ras", ".tga", ".tif", ".tiff", ".xbm", ".xpm",
];

fn main() {
    let mut input_f: String = String::from("");
    let mut output_f: String = String::from("");
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
    } else {
        for (index, file) in fs::read_dir(&input_path).unwrap().enumerate() {
            let mut image_path = input_path.clone();
            image_path.push(file.unwrap().file_name().clone());

            let mut image = open_image(&image_path.to_str().unwrap()).expect("File should open");
            let mut image_grayscale = image.clone();
            grayscale(&mut image_grayscale);

            let (left, top, right, bottom) = image_boundbox(&mut image_grayscale, tolerance);

            photon::native::save_image(image_grayscale, "./shit.jpg");
            println!(
                "left {} top {} right {} bottom {}",
                left, top, right, bottom
            );
        }
    }
}

fn image_boundbox(image: &mut PhotonImage, tolerance: u8) -> (u32, u32, u32, u32) {
    let data = image.get_raw_pixels();
    let width = image.get_width();
    let height = image.get_height();

    let t = ((255u8 - tolerance)..=255u8).collect::<Vec<u8>>();

    let mut left: u32 = 0;
    for y in 0..height {
        for x in 0..width {
            if !t.contains(&data[((y * width) + x) as usize]) {
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
            if !t.contains(&data[((y * width) + x) as usize]) {
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
            if !t.contains(&data[((y * width) + x) as usize]) {
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
            if !t.contains(&data[((y * width) + x) as usize]) {
                if (y < bottom && bottom == height) || (y > bottom && bottom != height) {
                    bottom = y;
                    break;
                }
            }
        }
    }

    (left, top, right, bottom)
}

fn support_extensions(input: &str) -> bool {
    SUPPORTED_FORMATS.contains(&input)
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
    opts.optflag(
        "l",
        "logs",
        "Enable's logging of information about the image. In ternimal and in output file.",
    );
    opts.optflag(
        "f",
        "force",
        "Overwrite output file in case it already exists. Without this the program does not replace images.");
    /*
    opts.optflag(
        "c",
        "color",
        "Preview each image after it's done being processed.",
    );
    opts.optflag("g", "grayscale", "Show grayscale image");
    opts.optflag(
        "m",
        "mark",
        "Keep track of collisions and show them in grayscale result.",
    );
    opts.optflag("w", "watch", "Run script as a watcher that notices file changes in input directory and outputs the result in the output directory.");
    */
    opts.optflag("h", "help", "Shows this manual.");

    return opts;
}

fn print_usage(opts: &Options) -> String {
    opts.usage("Utility for normalizing multiple image sizes for web listings")
}
