use image;
use std::env;
use std::io::{self, Cursor, Read, Write};
use std::path::Path;

#[derive(Debug)]
enum Input<'a> {
    Pipe,
    Path(Option<&'a Path>),
}

#[derive(Debug)]
enum Output {
    Dump,
    Path(String),
}

#[derive(Debug)]
struct Configuration<'a> {
    input: Input<'a>,
    output: Output,
}

#[derive(Debug)]
enum Effects {
    Pass,
    Blur(f32),
    Contrast(f32),
    Invert,
}

#[derive(Debug)]
struct Layer {
    image: image::DynamicImage,
    effect_chain: Vec<Effects>,
}

impl Layer {
    fn new() -> Self {
        Self {
            image: image::DynamicImage::new_rgb32f(1, 1),
            effect_chain: Vec::new(),
        }
    }
}

fn main() {
    let mut config = Configuration {
        input: Input::Path(None),
        output: Output::Dump,
    };

    let mut layers = Vec::new();
    layers.push(Layer::new());
    let mut args = env::args();

    let _ = args.next(); // ignore first item

    let mut input_path_arg: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.trim() {
            "-i" => {
                input_path_arg = Some(String::from(args.next().unwrap().trim()));
            }
            "-o" => {
                config.output = Output::Path(String::from(args.next().unwrap().trim()));
            }
            // effects
            "-pass" => {
                layers.last_mut().unwrap().effect_chain.push(Effects::Pass);
            }
            "-blur" => {
                layers.last_mut().unwrap().effect_chain.push(Effects::Blur(
                    args.next()
                        .expect("blur needs an argument")
                        .trim()
                        .parse::<f32>()
                        .expect("blur argument must be an f32"),
                ));
            }
            "-contrast" => {
                layers.last_mut().unwrap().effect_chain.push(Effects::Contrast(
                    args.next()
                        .expect("contrast needs an argument")
                        .trim()
                        .parse::<f32>()
                        .expect("contrast argument must be an f32"),
                ));
            }
            "-invert" => {
                layers
                    .last_mut()
                    .unwrap()
                    .effect_chain
                    .push(Effects::Invert);
            }
            "-layer" => {
                layers.push(Layer::new());
            }
            "-pipe" => {
                config.input = Input::Pipe;
            }
            "-dump" => {
                config.output = Output::Dump;
            }
            _ => panic!("couldn't parse input"),
        }
    }

    let input_path;
    match input_path_arg {
        None => (),
        Some(path) => {
            input_path = path;
            config.input = Input::Path(Some(Path::new(&input_path)));
        }
    }

    layers.first_mut().unwrap().image = match config.input {
        Input::Pipe => {
            let mut buf: Vec<u8> = Vec::new();
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();

            match handle.read_to_end(&mut buf) {
                Result::Err(_x) => (),
                Result::Ok(x) => {
                    eprintln!("read {} bytes", x)
                }
            };

            let buf = Cursor::new(buf);

            image::ImageReader::new(buf)
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
        }
        Input::Path(None) => panic!("input needed"),
        Input::Path(Some(path)) => image::ImageReader::open(path).unwrap().decode().unwrap(),
    };

    for layer in layers.iter_mut() {
        eprintln!("layer:");
        for effect in layer.effect_chain.iter() {
            eprintln!("    {:?}", effect);
            match effect {
                Effects::Pass => (),
                Effects::Blur(s) => layer.image = layer.image.blur(*s),
                Effects::Contrast(x) => layer.image = layer.image.adjust_contrast(*x),
                Effects::Invert => layer.image.invert(),
            }
        }
    }

    let output_image = layers.first().unwrap().image.clone();

    match config.output {
        Output::Dump => {
            let mut bytes: Vec<u8> = Vec::new();
            output_image
                .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                .unwrap();
            io::stdout().write_all(bytes.as_slice()).unwrap()
        }
        Output::Path(path) => output_image.save(path).unwrap(),
    }
}
