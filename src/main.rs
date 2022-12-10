extern crate getopts;
extern crate photon;
use getopts::Options;
use std::env;
use std::fs;
use std::fs::metadata;
use std::fs::read_dir;
use std::path::PathBuf;
use std::str::FromStr;

const SUPPORTED_FORMATS: [&'static str; 18] = [
    ".jpg", ".jpeg", ".bmp", ".dds", ".exif", ".gif", ".jps", ".jp2", ".jpx", ".pcx", ".png",
    ".pnm", ".ras", ".tga", ".tif", ".tiff", ".xbm", ".xpm",
];

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
    opts.optflag("h", "help", "Shows this manual.");
    opts.optflag("w", "watch", "Run script as a watcher that notices file changes in input directory and outputs the result in the output directory.");

    return opts;
}

fn print_usage(opts: &Options) -> String {
    opts.usage("Utility for normalizing multiple image sizes for web listings")
}

fn main() {
    let mut input_f: String = String::from("");
    let mut output_f: String = String::from("");
    let mut image_size: (u32, u32) = (800u32, 800u32);
    let mut tolerance: u32 = 10;
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
        Some(t) => u32::from_str(&t).expect("Failed to parse number"),
        None => tolerance,
    };

    let input_path = PathBuf::from_str(&input_f.to_owned()).unwrap();
    let output_path = PathBuf::from_str(&output_f.to_owned()).unwrap();

    let ip_md = metadata(&input_path).unwrap();

    if ip_md.is_file() {
    } else {
        for (index, file) in fs::read_dir(&input_path).unwrap().enumerate() {
            let mut image_path = input_path.clone();
            image_path.push(file.unwrap());
        }
    }
}
