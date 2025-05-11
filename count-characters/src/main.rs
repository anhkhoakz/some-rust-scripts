use clap::{ArgGroup, Parser};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// Simple program to count characters in a file or from standard input
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("count_opts").args(["bytes", "lines", "chars", "words", "longest_line"])))]
struct Args {
    /// Path(s) to the input file(s). If not provided, reads from stdin.
    #[arg(value_name = "FILE", required = false)]
    files: Vec<String>,

    /// Write the length of the line containing the most bytes (default) or characters (when -m is provided)
    #[arg(short = 'L', long = "longest-line")]
    longest_line: bool,

    /// The number of bytes in each input file
    #[arg(short = 'c', long = "bytes", long_help = "Count characters (bytes)")]
    bytes: bool,

    /// The number of lines in each input file
    #[arg(short = 'l', long = "lines")]
    lines: bool,

    /// The number of characters in each input file
    #[arg(short = 'm', long = "chars")]
    chars: bool,

    /// The number of words in each input file
    #[arg(short = 'w', long = "words")]
    words: bool,
}

#[derive(Default, Debug, Clone, Copy)]
struct WcResult {
    lines: usize,
    words: usize,
    bytes: usize,
    chars: usize,
    longest_line: usize,
}

impl WcResult {
    fn add(&mut self, other: &WcResult) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
        self.chars += other.chars;
        self.longest_line = self.longest_line.max(other.longest_line);
    }
}

fn count_stats<R: BufRead>(
    reader: R,
    count_bytes: bool,
    count_lines: bool,
    count_chars: bool,
    count_words: bool,
    count_longest_line: bool,
    use_chars_for_longest: bool,
) -> WcResult {
    let mut res: WcResult = WcResult::default();
    for line in reader.lines() {
        let line: String = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if count_lines || count_longest_line {
            res.lines += 1;
        }
        if count_words {
            res.words += line.split_whitespace().count();
        }
        if count_bytes {
            res.bytes += line.len() + 1;
        }
        if count_chars || use_chars_for_longest {
            res.chars += line.chars().count() + 1;
        }
        if count_longest_line {
            let len = if use_chars_for_longest {
                line.chars().count()
            } else {
                line.len()
            };
            res.longest_line = res.longest_line.max(len);
        }
    }
    res
}

fn handle_wc(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.files.iter().any(|f: &String| f.is_empty()) {
        return Err("Empty file name provided".into());
    }
    let (count_bytes, count_chars) = match (args.bytes, args.chars) {
        (true, true) => (false, true),
        (true, false) => (true, false),
        (false, true) => (false, true),
        (false, false) => (true, false),
    };
    let count_lines: bool =
        args.lines || !args.words && !args.bytes && !args.chars && !args.longest_line;
    let count_words: bool =
        args.words || !args.lines && !args.bytes && !args.chars && !args.longest_line;
    let count_longest_line: bool = args.longest_line;
    let use_chars_for_longest: bool = args.chars;
    let mut total: WcResult = WcResult::default();
    let files: Vec<String> = if args.files.is_empty() {
        vec!["-".to_string()]
    } else {
        args.files.clone()
    };
    let mut headers: Vec<&str> = vec![];
    if count_lines {
        headers.push("lines");
    }
    if count_words {
        headers.push("words");
    }
    if count_bytes {
        headers.push("bytes");
    }
    if count_chars {
        headers.push("chars");
    }
    if count_longest_line {
        headers.push(if use_chars_for_longest {
            "longest_line (chars)"
        } else {
            "longest_line (bytes)"
        });
    }
    if args.files.len() > 1 || !args.files.is_empty() {
        headers.push("file");
    }
    if !headers.is_empty() {
        println!("{}", headers.join("  "));
    }
    for (i, file_path) in files.iter().enumerate() {
        let reader: Box<dyn BufRead> = if file_path == "-" {
            if i == 0 && args.files.is_empty() {
                eprintln!("Paste your text, then press Ctrl+D (on Mac/Linux) or Ctrl+Z (on Windows) to finish:");
            }
            Box::new(BufReader::new(io::stdin()))
        } else {
            let file: File = File::open(file_path)?;
            Box::new(BufReader::new(file))
        };
        let res = count_stats(
            reader,
            count_bytes,
            count_lines,
            count_chars,
            count_words,
            count_longest_line,
            use_chars_for_longest,
        );
        total.add(&res);
        let mut output: Vec<String> = vec![];
        if count_lines {
            output.push(format!("{:>8}", res.lines));
        }
        if count_words {
            output.push(format!("{:>8}", res.words));
        }
        if count_bytes {
            output.push(format!("{:>8}", res.bytes));
        }
        if count_chars {
            output.push(format!("{:>8}", res.chars));
        }
        if count_longest_line {
            output.push(format!("{:>8}", res.longest_line));
        }
        if args.files.len() > 1 || !args.files.is_empty() {
            output.push(file_path.to_string());
        }
        println!("{}", output.join("  "));
    }
    if files.len() > 1 {
        let mut output: Vec<String> = vec![];
        if count_lines {
            output.push(format!("{:>8}", total.lines));
        }
        if count_words {
            output.push(format!("{:>8}", total.words));
        }
        if count_bytes {
            output.push(format!("{:>8}", total.bytes));
        }
        if count_chars {
            output.push(format!("{:>8}", total.chars));
        }
        if count_longest_line {
            output.push(format!("{:>8}", total.longest_line));
        }
        output.push("total".to_string());
        println!("{}", output.join("  "));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    handle_wc(&args)
}
