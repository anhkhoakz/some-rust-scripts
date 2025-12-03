use clap::Parser;
use serde_json::Value;
use similar::{Algorithm, ChangeTag, TextDiff};
use std::{error::Error, fs, path::PathBuf};

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

fn read_input(file: &PathBuf) -> Result<String, Box<dyn Error>> {
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()).into());
    }
    if !file.is_file() {
        return Err(format!("Not a file: {}", file.display()).into());
    }
    let content: String = fs::read_to_string(file)?;
    Ok(content)
}

fn parse_json(input: &str) -> Result<String, Box<dyn Error>> {
    let v: Value = serde_json::from_str(input)?;
    let pretty: String = serde_json::to_string_pretty(&v)?;
    Ok(pretty)
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
                print!("{}{}", sign, change);
            }
        }
        println!();
    }
}

fn handle_compare(args: Args) -> Result<(), Box<dyn Error>> {
    let text1: String = read_input(&args.file1)?;
    let text2: String = read_input(&args.file2)?;
    let formatted1: String = parse_json(&text1).unwrap_or(text1);
    let formatted2: String = parse_json(&text2).unwrap_or(text2);
    print_diff(&formatted1, &formatted2);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if let Err(e) = handle_compare(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
