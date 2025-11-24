use crate::types::{
    BatteryInfo, CpuInfo, DiskInfo, NetworkInfo, PackageInfo, ShellInfo, SwapInfo, SystemInfo,
    SystemInfoError,
};
use std::env;
use std::fs;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::detection::cursor::detect_cursor_apple;

/// Cache for command outputs to avoid repeated executions
static COMMAND_CACHE: OnceLock<Mutex<std::collections::HashMap<String, (String, Instant)>>> =
    OnceLock::new();
const CACHE_DURATION: Duration = Duration::from_secs(5);

/// Executes a system command and returns its output as a string.
/// Implements caching to avoid repeated executions of the same command.
pub fn get_command_output(cmd: &str, args: &[&str]) -> Result<String, SystemInfoError> {
    let cache_key = format!("{} {}", cmd, args.join(" "));
    let cache = COMMAND_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));

    // Check cache first
    if let Some((cached_output, timestamp)) = cache.lock().unwrap().get(&cache_key) {
        if timestamp.elapsed() < CACHE_DURATION {
            return Ok(cached_output.clone());
        }
    }

    // Execute command if not in cache or cache expired
    let output: std::process::Output = Command::new(cmd).args(args).output().map_err(|e| {
        SystemInfoError::CommandExecutionError(format!("Failed to execute {}: {}", cmd, e))
    })?;

    if !output.status.success() {
        return Err(SystemInfoError::CommandExecutionError(format!(
            "Command '{}' failed with status {}: {}",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let result = String::from_utf8(output.stdout)
        .map_err(|e| {
            SystemInfoError::ParsingError(format!("Failed to parse command output: {}", e))
        })?
        .trim()
        .to_string();

    // Update cache
    cache
        .lock()
        .unwrap()
        .insert(cache_key, (result.clone(), Instant::now()));

    Ok(result)
}

/// Retrieves comprehensive system information.
pub fn get_system_info() -> Result<SystemInfo, SystemInfoError> {
    Ok(SystemInfo {
        username: get_command_output("whoami", &[])?,
        hostname: get_command_output("scutil", &["--get", "ComputerName"])?,
        os: get_command_output("sw_vers", &["-productName"])?,
        os_version: get_command_output("sw_vers", &["-productVersion"])?,
        architecture: env::consts::ARCH.to_string(),
        model: get_command_output("sysctl", &["-n", "hw.model"])?,
        kernel: get_command_output("uname", &["-r"])?,
        uptime: get_uptime()?,
        packages: get_package_info()?,
        shell: get_shell_info()?,
        display: get_display_info()?,
        cpu: get_cpu_info()?,
        gpu: get_gpu_info()?,
        memory: get_memory_info()?,
        swap: get_swap_info()?,
        disk: get_disk_info()?,
        network: get_network_info()?,
        battery: get_battery_info()?,
        locale: env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string()),
        power_adapter: get_power_adapter_info()?,
        os_release_name: get_os_release_name()?,
        terminal: get_terminal()?,
        cursor: detect_cursor_apple(&env::var("HOME").unwrap_or_else(|_| "/".to_string())),
        wifi: get_wifi_info()?,
    })
}

fn get_os_release_name() -> Result<String, SystemInfoError> {
    let os_version: String = get_command_output("sw_vers", &["-productVersion"])?;
    let version: &str = os_version.split(".").next().unwrap_or("");
    let num: u32 = version.parse::<u32>().unwrap_or(0);
    let name: &'static str = match num {
        26 => "Tahoe",
        15 => "Sequoia",
        14 => "Sonoma",
        13 => "Ventura",
        12 => "Monterey",
        11 => "Big Sur",
        10 => "Catalina",
        9 => "Mojave",
        8 => "High Sierra",
        7 => "Sierra",
        6 => "El Capitan",
        5 => "Yosemite",
        4 => "Mavericks",
        3 => "Mountain Lion",
        2 => "Jaguar",
        1 => "Puma",
        0 => "Cheetah",
        _ => "Unknown",
    };

    Ok(name.to_string())
}

/// Extracts uptime information from the system.
fn get_uptime() -> Result<String, SystemInfoError> {
    let uptime: String = get_command_output("uptime", &[])?;
    let up_section = uptime.splitn(2, " up ").nth(1).ok_or_else(|| {
        SystemInfoError::ParsingError("Could not find 'up' in uptime output".to_string())
    })?;
    let up_section = up_section.split(',').next().unwrap_or("").trim();
    let mut days = None;
    let mut hours = None;
    let mut mins = None;
    let mut parts = up_section.split_whitespace().peekable();
    while let Some(part) = parts.next() {
        if let Ok(num) = part.parse::<u64>() {
            if let Some(&"days") = parts.peek() {
                days = Some(num);
                parts.next(); // consume 'days'
            } else if part.contains(":") {
                let time_parts: Vec<&str> = part.split(':').collect();
                if time_parts.len() == 2 {
                    hours = time_parts[0].parse::<u64>().ok();
                    mins = time_parts[1].parse::<u64>().ok();
                }
            }
        } else if part.contains(":") {
            let time_parts: Vec<&str> = part.split(':').collect();
            if time_parts.len() == 2 {
                hours = time_parts[0].parse::<u64>().ok();
                mins = time_parts[1].parse::<u64>().ok();
            }
        }
    }
    let mut result = String::new();
    if let Some(d) = days {
        result.push_str(&format!("{} days, ", d));
    }
    if let Some(h) = hours {
        result.push_str(&format!("{} hours, ", h));
    }
    if let Some(m) = mins {
        result.push_str(&format!("{} mins", m));
    }
    if result.ends_with(", ") {
        result.truncate(result.len() - 2);
    }
    if result.is_empty() {
        return Err(SystemInfoError::ParsingError(
            "Could not parse uptime info".to_string(),
        ));
    }
    Ok(result)
}

/// Retrieves package information from Homebrew.
fn get_package_info() -> Result<PackageInfo, SystemInfoError> {
    let brew_path: String = "/opt/homebrew/".to_string();
    let brew_bin_path: String = brew_path.clone() + "Cellar";
    let brew_cask_path: String = brew_path.clone() + "Caskroom";
    let brew: usize = fs::read_dir(brew_bin_path)
        .map(|rd| {
            rd.filter(|e| e.as_ref().ok().map(|e| e.path().is_dir()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);
    let brew_cask: usize = fs::read_dir(brew_cask_path)
        .map(|rd| {
            rd.filter(|e| e.as_ref().ok().map(|e| e.path().is_dir()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    Ok(PackageInfo {
        brew_count: brew,
        brew_cask_count: brew_cask,
    })
}

/// Retrieves shell information.
fn get_shell_info() -> Result<ShellInfo, SystemInfoError> {
    let shell = env::var("SHELL")
        .map_err(|e| SystemInfoError::EnvironmentError(format!("Failed to get SHELL: {}", e)))?;
    let version = get_command_output(&shell, &["--version"])?;

    Ok(ShellInfo {
        version: version
            .lines()
            .next()
            .ok_or_else(|| {
                SystemInfoError::ParsingError("Failed to parse shell version".to_string())
            })?
            .to_string(),
    })
}

/// Retrieves display information.
fn get_display_info() -> Result<String, SystemInfoError> {
    let display = get_command_output("system_profiler", &["SPDisplaysDataType"])?;
    Ok(display
        .lines()
        .find(|l| l.contains("Resolution"))
        .ok_or_else(|| {
            SystemInfoError::ParsingError("Failed to find display resolution".to_string())
        })?
        .trim()
        .to_string())
}

/// Retrieves CPU information.
fn get_cpu_info() -> Result<CpuInfo, SystemInfoError> {
    Ok(CpuInfo {
        model: get_command_output("sysctl", &["-n", "machdep.cpu.brand_string"])?,
        cores: get_command_output("sysctl", &["-n", "hw.ncpu"])?,
    })
}

/// Retrieves GPU information.
fn get_gpu_info() -> Result<String, SystemInfoError> {
    let display = get_command_output("system_profiler", &["SPDisplaysDataType"])?;
    Ok(display
        .lines()
        .find(|l| l.contains("Chipset Model"))
        .ok_or_else(|| SystemInfoError::ParsingError("Failed to find GPU information".to_string()))?
        .trim()
        .replace("Chipset Model: ", ""))
}

/// Retrieves memory information in GiB.
fn get_memory_info() -> Result<f64, SystemInfoError> {
    let mem = get_command_output("sysctl", &["-n", "hw.memsize"])?;
    Ok(mem
        .parse::<u64>()
        .map_err(|e| SystemInfoError::ParsingError(format!("Failed to parse memory size: {}", e)))?
        as f64
        / (1024.0 * 1024.0 * 1024.0))
}

/// Retrieves swap information.
fn get_swap_info() -> Result<SwapInfo, SystemInfoError> {
    let swap_info = get_command_output("sysctl", &["-n", "vm.swapusage"])?;
    let parts: Vec<&str> = swap_info.split_whitespace().collect();

    if parts.len() < 6 {
        return Err(SystemInfoError::ParsingError(
            "Invalid swap information format".to_string(),
        ));
    }

    let used = parts[5]
        .strip_suffix("M")
        .ok_or_else(|| SystemInfoError::ParsingError("Invalid swap usage format".to_string()))?
        .parse::<f64>()
        .map_err(|e| SystemInfoError::ParsingError(format!("Failed to parse swap usage: {}", e)))?;

    let total = parts[2]
        .strip_suffix("M")
        .ok_or_else(|| SystemInfoError::ParsingError("Invalid swap total format".to_string()))?
        .parse::<f64>()
        .map_err(|e| SystemInfoError::ParsingError(format!("Failed to parse swap total: {}", e)))?;

    let percentage = (used / total) * 100.0;

    Ok(SwapInfo {
        used: format!("{:.2}GiB", used / 1024.0),
        total: format!("{:.2}GiB", total / 1024.0),
        percentage: format!("{:.0}%", percentage),
    })
}

/// Retrieves disk information.
fn get_disk_info() -> Result<DiskInfo, SystemInfoError> {
    let disk = get_command_output("df", &["-h", "/"])?;
    let disk_line = disk
        .lines()
        .nth(1)
        .ok_or_else(|| SystemInfoError::ParsingError("No disk information found".to_string()))?;

    let parts: Vec<&str> = disk_line.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(SystemInfoError::ParsingError(
            "Invalid disk information format".to_string(),
        ));
    }

    Ok(DiskInfo {
        used: parts[2].to_string(),
        total: parts[1].to_string(),
        percentage: parts[4].to_string(),
    })
}

/// Retrieves network information.
fn get_network_info() -> Result<NetworkInfo, SystemInfoError> {
    Ok(NetworkInfo {
        local_ip: get_command_output("ipconfig", &["getifaddr", "en0"])?,
    })
}

/// Retrieves battery information.
fn get_battery_info() -> Result<BatteryInfo, SystemInfoError> {
    let battery = get_command_output("pmset", &["-g", "batt"])?;
    let battery_line = battery
        .lines()
        .nth(1)
        .ok_or_else(|| SystemInfoError::ParsingError("No battery information found".to_string()))?;

    let parts: Vec<&str> = battery_line.split(';').collect();
    if parts.len() < 2 {
        return Err(SystemInfoError::ParsingError(
            "Invalid battery information format".to_string(),
        ));
    }

    let percentage = parts[0]
        .split_whitespace()
        .nth(2)
        .ok_or_else(|| {
            SystemInfoError::ParsingError("Failed to parse battery percentage".to_string())
        })?
        .to_string();

    let status = parts[1]
        .split_whitespace()
        .next()
        .ok_or_else(|| SystemInfoError::ParsingError("Failed to parse battery status".to_string()))?
        .to_string();

    Ok(BatteryInfo { percentage, status })
}

/// Retrieves power adapter information.
fn get_power_adapter_info() -> Result<String, SystemInfoError> {
    let power_adapter = get_command_output("ioreg", &["-r", "-c", "AppleSmartBattery"])?;

    let adapter_details_line = power_adapter
        .lines()
        .find(|l| l.contains("\"AdapterDetails\" = {"))
        .ok_or_else(|| SystemInfoError::ParsingError("No adapter details found".to_string()))?;

    // Extract Name field from the dictionary
    if let Some(name_start) = adapter_details_line.find("\"Name\"=\"") {
        let name_start = name_start + 8; // Length of "\"Name\"=\""
        if let Some(name_end) = adapter_details_line[name_start..].find('\"') {
            return Ok(adapter_details_line[name_start..name_start + name_end].to_string());
        }
    }

    Ok("No power adapter name found".to_string())
}

fn get_terminal() -> Result<String, SystemInfoError> {
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        return Ok(term_program);
    }
    if let Ok(lc_terminal) = env::var("LC_TERMINAL") {
        return Ok(lc_terminal);
    }
    if let Ok(term) = env::var("TERM") {
        return Ok(term);
    }
    Ok("Unknown".to_string())
}

/// Very basic XML key-value string parser for Apple's plist.
/// Only supports extracting flat keys with dict nesting one level deep. Not robust!
fn get_plist_dict_value<'a>(plist: &'a str, key: &str) -> Option<&'a str> {
    let key_tag = format!("<key>{}</key>", key);
    let idx = plist.find(&key_tag)?;
    let after = &plist[idx + key_tag.len()..];
    let val_start = after.find('<')?;
    let val_end = after[val_start..].find('>')?;
    let tag = &after[val_start + 1..val_start + val_end];
    let close_tag = format!("</{}>", tag);
    let content_start = val_start + val_end + 1;
    let content_end = after[content_start..].find(&close_tag)?;
    let value = &after[content_start..content_start + content_end];
    Some(value.trim())
}

/// Parse a color dictionary (as a string) and extract RGBA as f64.
/// Only works if the color dict is stored as XML inline.
fn parse_color_dict(dict_str: &str) -> Option<(u8, u8, u8, u8)> {
    let get_comp = |name| {
        dict_str
            .find(&format!("<key>{}</key>", name))
            .and_then(|idx| {
                let after = &dict_str[idx + format!("<key>{}</key>", name).len()..];
                let real_start = after.find("<real>")?;
                let real_end = after[real_start + 6..].find("</real>")?;
                let val = &after[real_start + 6..real_start + 6 + real_end];
                val.trim().parse::<f64>().ok()
            })
    };
    let r = get_comp("red")?;
    let g = get_comp("green")?;
    let b = get_comp("blue")?;
    let a = get_comp("alpha")?;
    Some((
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    ))
}

/// Format color like the C code: White/Black or #RRGGBBAA
fn format_color(r: u8, g: u8, b: u8, a: u8) -> String {
    match (r, g, b, a) {
        (255, 255, 255, 255) => "White".to_string(),
        (0, 0, 0, 255) => "Black".to_string(),
        _ => format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a),
    }
}
