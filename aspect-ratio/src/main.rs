use std::env;
use std::error::Error;
use std::fmt;
use std::process;

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp: u32 = b;
        b = a % b;
        a = temp;
    }
    a
}

mod aspect_ratio {
    use super::gcd;

    pub fn get_aspect_ratio(width: u32, height: u32) -> (u32, u32) {
        if width == 0 || height == 0 {
            return (width, height);
        }
        let divisor: u32 = gcd(width, height);
        (width / divisor, height / divisor)
    }
}

#[derive(Debug)]
enum ParseError {
    InvalidFormat,
    InvalidNumbers,
    NonPositiveNumbers,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid input format"),
            ParseError::InvalidNumbers => write!(f, "Invalid numbers provided"),
            ParseError::NonPositiveNumbers => {
                write!(f, "Width and height must be positive numbers")
            }
        }
    }
}

impl Error for ParseError {}

fn parse_dimensions(args: &[String]) -> Result<(u32, u32), ParseError> {
    match args {
        [input] => {
            let input: &str = input.as_str();
            let (w, h) = if let Some((w, h)) = input.split_once(['x', ':']) {
                (w.trim(), h.trim())
            } else {
                return Err(ParseError::InvalidFormat);
            };
            let width: u32 = w.parse().map_err(|_| ParseError::InvalidNumbers)?;
            let height: u32 = h.parse().map_err(|_| ParseError::InvalidNumbers)?;
            if width == 0 || height == 0 {
                return Err(ParseError::NonPositiveNumbers);
            }
            Ok((width, height))
        }
        [w, h] => {
            let width: u32 = w.parse().map_err(|_| ParseError::InvalidNumbers)?;
            let height: u32 = h.parse().map_err(|_| ParseError::InvalidNumbers)?;
            if width == 0 || height == 0 {
                return Err(ParseError::NonPositiveNumbers);
            }
            Ok((width, height))
        }
        _ => Err(ParseError::InvalidFormat),
    }
}

fn print_usage() {
    eprintln!("Usage: aspect-ratio <width> <height>");
    eprintln!("Examples: aspect-ratio 1920 1080 | aspect-ratio 1920x1080 | aspect-ratio 1920:1080");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match parse_dimensions(&args) {
        Ok((width, height)) => {
            let (w, h) = aspect_ratio::get_aspect_ratio(width, height);
            println!("{}:{}", w, h);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            print_usage();
            process::exit(1);
        }
    }
}
