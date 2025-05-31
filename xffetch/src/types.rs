use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct SystemInfo {
    pub username: String,
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub architecture: String,
    pub model: String,
    pub kernel: String,
    pub uptime: String,
    pub packages: PackageInfo,
    pub shell: ShellInfo,
    pub display: String,
    pub cpu: CpuInfo,
    pub gpu: String,
    pub memory: f64,
    pub swap: SwapInfo,
    pub disk: DiskInfo,
    pub network: NetworkInfo,
    pub battery: BatteryInfo,
    pub locale: String,
}

#[derive(Debug)]
pub struct PackageInfo {
    pub brew_count: usize,
    pub brew_cask_count: usize,
}

#[derive(Debug)]
pub struct ShellInfo {
    pub version: String,
}

#[derive(Debug)]
pub struct CpuInfo {
    pub model: String,
    pub cores: String,
}

#[derive(Debug)]
pub struct SwapInfo {
    pub used: String,
    pub total: String,
    pub percentage: String,
}

#[derive(Debug)]
pub struct DiskInfo {
    pub used: String,
    pub total: String,
    pub percentage: String,
}

#[derive(Debug)]
pub struct NetworkInfo {
    pub local_ip: String,
}

#[derive(Debug)]
pub struct BatteryInfo {
    pub percentage: String,
    pub status: String,
}

#[derive(Debug)]
pub enum SystemInfoError {
    CommandExecutionError(String),
    ParsingError(String),
    EnvironmentError(String),
}

impl Error for SystemInfoError {}

impl fmt::Display for SystemInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemInfoError::CommandExecutionError(msg) => {
                write!(f, "Command execution error: {}", msg)
            }
            SystemInfoError::ParsingError(msg) => write!(f, "Parsing error: {}", msg),
            SystemInfoError::EnvironmentError(msg) => write!(f, "Environment error: {}", msg),
        }
    }
}

pub const COLORS: [&str; 8] = [
    "\x1b[38;5;120m", // green
    "\x1b[38;5;179m", // yellow
    "\x1b[38;5;215m", // orange
    "\x1b[38;5;110m", // blue
    "\x1b[38;5;117m", // cyan
    "\x1b[38;5;139m", // magenta
    "\x1b[38;5;245m", // gray
    "\x1b[0m",        // reset
];
