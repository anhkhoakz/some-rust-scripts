use clap::Parser;
use std::fs::File;
use std::io::{self, Read};

/// Simple program to count characters in a file or from standard input
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file. If not provided, reads from stdin.
    file: Option<String>,
}

/// Reads input from a file or stdin, trims leading/trailing empty lines, and counts characters.
fn main() {
    let args: Args = Args::parse();
    let mut input: String = String::new();
    if let Some(file_path) = args.file {
        let mut file: File = File::open(&file_path).expect("Failed to open file");
        file.read_to_string(&mut input)
            .expect("Failed to read file");
    } else {
        println!(
            "Paste your text, then press Ctrl+D (on Mac/Linux) or Ctrl+Z (on Windows) to finish:"
        );
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read input");
    }

    // Trim leading and trailing empty lines
    let filtered_input: Vec<&str> = input
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<_>>();
    let filtered_input: Vec<&&str> = filtered_input
        .iter()
        .rev()
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<_>>();
    let filtered_input: String = filtered_input
        .iter()
        .rev()
        .map(|&&line| line)
        .collect::<Vec<_>>()
        .join("\n");

    let char_count: usize = filtered_input.chars().count();
    println!("Input contains {} characters.", char_count);
}
