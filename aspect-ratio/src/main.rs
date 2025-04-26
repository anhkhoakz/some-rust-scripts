use clap::CommandFactory;
use clap::Parser;
use std::error::Error;
use std::fmt;

/// Simple program to calculate the aspect ratio of a given width and height
///
/// ## Examples
/// ```sh
/// aspect-ratio 1920 1080
/// aspect-ratio 1920x1080
/// aspect-ratio 1920:1080
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Width and height, e.g. 1920 1080 or 1920x1080 or 1920:1080
    #[arg(required = true)]
    dims: Vec<String>,
}

mod aspect_ratio {
    /// Function to calculate the greatest common divisor (GCD) of two numbers
    pub fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let temp: u32 = b;
            b = a % b;
            a = temp;
        }
        a
    }

    /// Function to calculate the aspect ratio of a given width and height
    pub fn get_aspect_ratio(width: u32, height: u32) -> (u32, u32) {
        if width == 0 || height == 0 {
            return (width, height);
        }
        let divisor: u32 = gcd(width, height);
        (width / divisor, height / divisor)
    }
}

#[derive(Debug)]
/// Custom error type for parsing dimensions
enum ParseError {
    InvalidFormat,
    InvalidNumbers,
    NonPositiveNumbers,
}

/// Implementing the Display trait for ParseError
impl fmt::Display for ParseError {
    /// This trait is used to convert the error into a human-readable string
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

/// Implementing the Error trait for ParseError
impl Error for ParseError {}

/// Function to parse the command line arguments and extract width and height
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

/// Main function to parse command line arguments and calculate aspect ratio
fn main() {
    let args: Args = Args::parse();

    match parse_dimensions(&args.dims) {
        Ok((width, height)) => {
            let (w, h) = aspect_ratio::get_aspect_ratio(width, height);
            println!("{}:{}", w, h);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Args::command().print_help().unwrap();
            println!();
            std::process::exit(1);
        }
    }
}
