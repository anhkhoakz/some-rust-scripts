use clap::CommandFactory;
use clap::{Args as ClapArgs, Parser, Subcommand};
use clap_complete::generate;
use std::error::Error;
use std::fmt;
use std::io;

// Simple aspect ratio calculator
#[derive(Parser, Debug)]
#[command(author, version, about = "Calculate and convert aspect ratios.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert aspect ratio to target width or height
    #[command(
        about = "Convert an aspect ratio to a target width or height.",
        long_about = "Convert an aspect ratio to a target width or height.\n\
Examples:\n  aspect-ratio convert 16:9 --width 1920\n  aspect-ratio convert 4:3 --height 720"
    )]
    Convert(ConvertArgs),
    /// Show info about an aspect ratio
    #[command(
        about = "Show info about an aspect ratio.",
        long_about = "Show info about an aspect ratio.\n\
Examples:\n  aspect-ratio info 1920x1080\n  aspect-ratio info 4:3"
    )]
    Info(InfoArgs),
    /// Generate shell completions
    #[command(
        about = "Generate shell completions.",
        long_about = "Generate shell completions for supported shells.\n\
Examples:\n  aspect-ratio completions bash\n  aspect-ratio completions zsh\n\
Supported shells: bash, zsh, fish, powershell, elvish"
    )]
    Completions {
        /// The shell to generate completions for
        #[arg(default_value = "bash")]
        shell: String,
    },
    /// Calculate the reduced aspect ratio
    #[command(
        about = "Reduce an aspect ratio.",
        long_about = "Reduce an aspect ratio to its simplest form.\n\
Examples:\n  aspect-ratio calc 1920x1080\n  aspect-ratio calc 16:9\n  aspect-ratio calc 1920 1080"
    )]
    Calc(CalcArgs),
}

#[derive(ClapArgs, Debug)]
struct ConvertArgs {
    /// Aspect ratio, e.g. 16:9 or 1920x1080
    ratio: String,
    /// Target width (optional)
    #[arg(long)]
    width: Option<u32>,
    /// Target height (optional)
    #[arg(long)]
    height: Option<u32>,
}

#[derive(ClapArgs, Debug)]
struct InfoArgs {
    /// Aspect ratio, e.g. 16:9 or 1920x1080
    ratio: String,
}

#[derive(ClapArgs, Debug)]
struct CalcArgs {
    /// Aspect ratio (e.g. 1920x1080, 16:9) or width
    arg1: String,
    /// Height (optional, if arg1 is width)
    arg2: Option<String>,
}

mod aspect_ratio {
    pub fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let temp: u32 = b;
            b = a % b;
            a = temp;
        }
        a
    }
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
    TooLarge,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid input format"),
            ParseError::InvalidNumbers => write!(f, "Invalid numbers provided"),
            ParseError::NonPositiveNumbers => {
                write!(f, "Width and height must be positive numbers")
            }
            ParseError::TooLarge => write!(f, "Numbers are too large"),
        }
    }
}

impl Error for ParseError {}

fn parse_ratio(input: &str) -> Result<(u32, u32), ParseError> {
    let (w, h) = if let Some((w, h)) = input.split_once(['x', ':', '×']) {
        (w.trim(), h.trim())
    } else {
        return Err(ParseError::InvalidFormat);
    };
    let width: u32 = w.parse().map_err(|_| ParseError::InvalidNumbers)?;
    let height: u32 = h.parse().map_err(|_| ParseError::InvalidNumbers)?;
    if width == 0 || height == 0 {
        return Err(ParseError::NonPositiveNumbers);
    }
    if width > 100_000_000 || height > 100_000_000 {
        return Err(ParseError::TooLarge);
    }
    Ok((width, height))
}

#[derive(Debug, Clone, Copy)]
enum SupportedShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl SupportedShell {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "powershell" => Some(Self::PowerShell),
            "elvish" => Some(Self::Elvish),
            _ => None,
        }
    }
    fn as_clap_shell(self) -> clap_complete::Shell {
        match self {
            Self::Bash => clap_complete::Shell::Bash,
            Self::Zsh => clap_complete::Shell::Zsh,
            Self::Fish => clap_complete::Shell::Fish,
            Self::PowerShell => clap_complete::Shell::PowerShell,
            Self::Elvish => clap_complete::Shell::Elvish,
        }
    }
    fn all() -> &'static [&'static str] {
        &["bash", "zsh", "fish", "powershell", "elvish"]
    }
}

fn handle_convert(args: &ConvertArgs) -> Result<(), Box<dyn Error>> {
    let (w, h) = parse_ratio(&args.ratio)?;
    if let Some(width) = args.width {
        if width > 100_000_000 {
            return Err(Box::new(ParseError::TooLarge));
        }
        let height = width
            .checked_mul(h)
            .and_then(|v| v.checked_div(w))
            .ok_or(ParseError::TooLarge)?;
        println!("{}x{}", width, height);
    } else if let Some(height) = args.height {
        if height > 100_000_000 {
            return Err(Box::new(ParseError::TooLarge));
        }
        let width = height
            .checked_mul(w)
            .and_then(|v| v.checked_div(h))
            .ok_or(ParseError::TooLarge)?;
        println!("{}x{}", width, height);
    } else {
        return Err("Please provide either --width or --height".into());
    }
    Ok(())
}

fn handle_info(args: &InfoArgs) -> Result<(), Box<dyn Error>> {
    let (w, h) = parse_ratio(&args.ratio)?;
    let decimal: f64 = w as f64 / h as f64;
    println!("Aspect Ratio: {}:{}", w, h);
    println!("Decimal: {:.6}", decimal);
    Ok(())
}

fn handle_completions(shell: &str) -> Result<(), Box<dyn Error>> {
    let mut cmd = Cli::command();
    let shell_enum = SupportedShell::from_str(shell).ok_or_else(|| {
        format!(
            "Unsupported shell: {}. Supported shells: {}",
            shell,
            SupportedShell::all().join(", ")
        )
    })?;
    generate(
        shell_enum.as_clap_shell(),
        &mut cmd,
        "aspect-ratio",
        &mut io::stdout(),
    );
    Ok(())
}

fn handle_calc(args: &CalcArgs) -> Result<(), Box<dyn Error>> {
    if let Some(arg2) = &args.arg2 {
        let w: u32 = args.arg1.parse().map_err(|_| ParseError::InvalidNumbers)?;
        let h: u32 = arg2.parse().map_err(|_| ParseError::InvalidNumbers)?;
        if w == 0 || h == 0 {
            return Err(Box::new(ParseError::NonPositiveNumbers));
        }
        if w > 100_000_000 || h > 100_000_000 {
            return Err(Box::new(ParseError::TooLarge));
        }
        let (rw, rh) = aspect_ratio::get_aspect_ratio(w, h);
        println!("{}:{}", rw, rh);
    } else {
        let (w, h) = parse_ratio(&args.arg1)?;
        let (rw, rh) = aspect_ratio::get_aspect_ratio(w, h);
        println!("{}:{}", rw, rh);
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        Some(Commands::Convert(args)) => handle_convert(args),
        Some(Commands::Info(args)) => handle_info(args),
        Some(Commands::Completions { shell }) => handle_completions(shell),
        Some(Commands::Calc(args)) => handle_calc(args),
        None => {
            Cli::command()
                .print_help()
                .unwrap_or_else(|e| eprintln!("{}", e));
            println!();
            return;
        }
    };
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
