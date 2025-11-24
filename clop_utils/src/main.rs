use clap::Parser;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

const CLOP_BINARY: &str = "/Applications/Clop.app/Contents/SharedSupport/ClopCLI";

/// Simple program to optimise files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the file to check
    #[arg(value_name = "FILE_PATH")]
    path: PathBuf,

    /// Action to perform
    #[arg(short, long)]
    action: String,

    /// The size to crop the file to
    #[arg(short, long, required = false)]
    size: Option<String>,
}

/// Check if a MIME type is an image
fn is_image(mime: &str) -> bool {
    mime.starts_with("image/")
}

/// Check if a MIME type is a video
fn is_video(mime: &str) -> bool {
    mime.starts_with("video/")
}

/// Check if a MIME type is a PDF
fn is_pdf(mime: &str) -> bool {
    mime.starts_with("application/pdf")
}

fn detect_mime(path: &PathBuf) -> &'static str {
    match tree_magic_mini::from_filepath(path) {
        Some(m) => m,
        None => {
            eprintln!("Could not determine MIME type for file: {}", path.display());
            process::exit(1);
        }
    }
}

/// Check if dependencies are installed
fn check_dependencies() {
    if !which::which(CLOP_BINARY).is_ok() {
        eprintln!("clop is not installed. Please install it with `cargo install clop`");
        process::exit(1);
    }
}

/// Optimize the file
fn optimize_file(path: &PathBuf) {
    let output: process::Output = Command::new(CLOP_BINARY)
        .args(["optimise", "-g", path.to_str().unwrap()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

/// Optimize the file aggressively
fn optimize_aggressively(path: &PathBuf) {
    let output: process::Output = Command::new(CLOP_BINARY)
        .args(["optimise", "-g", "-a", path.to_str().unwrap()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

/// Optimize and crop
fn optimize_and_crop(path: &PathBuf, size: &str) {
    let output: process::Output = Command::new(CLOP_BINARY)
        .args(["crop", "-g", "--size", size, path.to_str().unwrap()])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    check_dependencies();

    let args: Args = Args::parse();

    if !args.path.exists() {
        eprintln!("File does not exist: {}", args.path.display());
        process::exit(1);
    }

    if args.action == "optimize" {
        optimize_file(&args.path);
        process::exit(0);
    }

    if args.action == "aggressive" {
        optimize_aggressively(&args.path);
        process::exit(0);
    }

    if args.action == "crop" {
        match args.size {
            Some(size) => {
                optimize_and_crop(&args.path, &size);
                process::exit(0);
            }
            None => {
                eprintln!("The --size argument is required for the 'crop' action.");
                process::exit(1);
            }
        }
    }
}
