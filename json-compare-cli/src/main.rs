use clap::Parser;
use colored::*;
use serde_json::Value;
use similar::{Algorithm, ChangeTag, TextDiff};
use std::{fs, path::PathBuf, process};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The first JSON file to compare
    #[arg(value_name = "FILE1")]
    file1: PathBuf,
    /// The second JSON file to compare
    #[arg(value_name = "FILE2")]
    file2: PathBuf,
}

fn read_input(file: &PathBuf) -> Result<String, String> {
    fs::read_to_string(file)
        .map_err(|e: std::io::Error| format!("Failed to read file {}: {}", file.display(), e))
}

fn parse_json(input: &str) -> String {
    serde_json::from_str::<Value>(input)
        .map(|v: Value| serde_json::to_string_pretty(&v).unwrap_or_else(|_| input.to_string()))
        .unwrap_or_else(|_| input.to_string())
}

fn print_diff(original: &str, changed: &str) {
    let diff: TextDiff<'_, '_, '_, str> = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_lines(original, changed);

    for group in diff.grouped_ops(3) {
        for op in group {
            for change in diff.iter_changes(&op) {
                let sign: &str = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                let colored_line: String = match change.tag() {
                    ChangeTag::Delete => format!("{}{}", sign, change).red().to_string(),
                    ChangeTag::Insert => format!("{}{}", sign, change).green().to_string(),
                    ChangeTag::Equal => format!("{}{}", sign, change),
                };
                print!("{}", colored_line);
            }
        }
        println!();
    }
}

fn main() {
    let args: Args = Args::parse();

    let text1: String = match read_input(&args.file1) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let text2: String = match read_input(&args.file2) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let formatted1: String = parse_json(&text1);
    let formatted2: String = parse_json(&text2);

    print_diff(&formatted1, &formatted2);
}
