use std::env;
use std::fs::File;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input: String = String::new();
    if args.len() > 1 {
        let file_path: &String = &args[1];
        let mut file: File = File::open(file_path).expect("Failed to open file");
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
    let lines: Vec<&str> = input
        .lines()
        .skip_while(|line: &&str| line.trim().is_empty())
        .collect();
    let lines: Vec<&str> = lines
        .into_iter()
        .rev()
        .skip_while(|line: &&str| line.trim().is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let filtered_input: String = lines.join("\n");
    let char_count: usize = filtered_input.chars().count();
    println!("Input contains {} characters.", char_count);
}
