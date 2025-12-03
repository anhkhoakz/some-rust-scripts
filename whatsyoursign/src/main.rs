use clap::Parser;
use std::io;
use std::process::{Command, ExitCode, Output};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the file to inspect
    #[arg(short, long)]
    path: String,
}

fn inspect_signature(path: &str) -> io::Result<()> {
    // codesign -dvvv --verbose=4 <path>
    let codesign_out = Command::new("codesign")
        .args(&["-dvvv", "--verbose=4", path])
        .output()?;

    println!(
        "codesign stdout:\n{}",
        String::from_utf8_lossy(&codesign_out.stdout)
    );
    println!(
        "codesign stderr:\n{}",
        String::from_utf8_lossy(&codesign_out.stderr)
    );

    // spctl -a -v <path>
    let spctl_out: Output = Command::new("spctl").args(&["-a", "-v", path]).output()?;

    println!(
        "spctl stdout:\n{}",
        String::from_utf8_lossy(&spctl_out.stdout)
    );
    println!(
        "spctl stderr:\n{}",
        String::from_utf8_lossy(&spctl_out.stderr)
    );

    Ok(())
}

fn check_dependencies() -> io::Result<()> {
    let codesign: Output = Command::new("type").arg("codesign").output()?;
    if !codesign.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "codesign not found"));
    }

    let spctl: Output = Command::new("type").arg("spctl").output()?;
    if !spctl.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "spctl not found"));
    }

    Ok(())
}

fn output() -> String {
    

}

fn main() -> ExitCode {
    if !cfg!(target_os = "macos") {
        return ExitCode::FAILURE;
    }
    let args: Args = Args::parse();
    println!("Inspecting signature of {}", args.path);

    check_dependencies().expect("Failed to check dependencies");

    let path: &String = &args.path;
    inspect_signature(path).expect("Failed to inspect signature");
    ExitCode::SUCCESS
}
