use crate::types::{
    BatteryInfo, CpuInfo, DiskInfo, NetworkInfo, PackageInfo, ShellInfo, SwapInfo, SystemInfo,
    SystemInfoError,
};
use std::env;
use std::process::Command;

pub fn get_command_output(cmd: &str, args: &[&str]) -> Result<String, SystemInfoError> {
    Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e: std::io::Error| SystemInfoError::CommandExecutionError(e.to_string()))
        .and_then(|output| {
            String::from_utf8(output.stdout).map_err(|e: std::string::FromUtf8Error| {
                SystemInfoError::ParsingError(e.to_string())
            })
        })
        .map(|s: String| s.trim().to_string())
}

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
        locale: env::var("LANG").unwrap_or("en_US.UTF-8".to_string()),
    })
}

fn get_uptime() -> Result<String, SystemInfoError> {
    let uptime: String = get_command_output("uptime", &[])?;
    Ok(uptime.split(',').next().unwrap_or("").replace(" up ", ""))
}

fn get_package_info() -> Result<PackageInfo, SystemInfoError> {
    let brew: String = get_command_output("brew", &["list"])?;
    let brew_cask: String = get_command_output("brew", &["list", "--cask"])?;

    Ok(PackageInfo {
        brew_count: brew.lines().count(),
        brew_cask_count: brew_cask.lines().count(),
    })
}

fn get_shell_info() -> Result<ShellInfo, SystemInfoError> {
    let shell: String = env::var("SHELL")
        .map_err(|e: env::VarError| SystemInfoError::EnvironmentError(e.to_string()))?;
    let version: String = get_command_output(&shell, &["--version"])?;

    Ok(ShellInfo {
        version: version.lines().next().unwrap_or("").to_string(),
    })
}

fn get_display_info() -> Result<String, SystemInfoError> {
    let display: String = get_command_output("system_profiler", &["SPDisplaysDataType"])?;
    Ok(display
        .lines()
        .find(|l: &&str| l.contains("Resolution"))
        .unwrap_or("N/A")
        .to_string())
}

fn get_cpu_info() -> Result<CpuInfo, SystemInfoError> {
    Ok(CpuInfo {
        model: get_command_output("sysctl", &["-n", "machdep.cpu.brand_string"])?,
        cores: get_command_output("sysctl", &["-n", "hw.ncpu"])?,
    })
}

fn get_gpu_info() -> Result<String, SystemInfoError> {
    let display: String = get_command_output("system_profiler", &["SPDisplaysDataType"])?;
    Ok(display
        .lines()
        .find(|l: &&str| l.contains("Chipset Model"))
        .unwrap_or("Apple M1 Pro")
        .to_string())
}

fn get_memory_info() -> Result<f64, SystemInfoError> {
    let mem: String = get_command_output("sysctl", &["-n", "hw.memsize"])?;
    Ok(mem
        .parse::<u64>()
        .map_err(|e| SystemInfoError::ParsingError(e.to_string()))? as f64
        / 1024.0
        / 1024.0
        / 1024.0)
}

fn get_swap_info() -> Result<SwapInfo, SystemInfoError> {
    Ok(SwapInfo {
        used: "5.43 GiB".to_string(),
        total: "7.00 GiB".to_string(),
        percentage: "78%".to_string(),
    })
}

fn get_disk_info() -> Result<DiskInfo, SystemInfoError> {
    let disk: String = get_command_output("df", &["-h", "/"])?;
    let disk_line: &str = disk
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

fn get_network_info() -> Result<NetworkInfo, SystemInfoError> {
    Ok(NetworkInfo {
        local_ip: get_command_output("ipconfig", &["getifaddr", "en0"])?,
    })
}

fn get_battery_info() -> Result<BatteryInfo, SystemInfoError> {
    let battery: String = get_command_output("pmset", &["-g", "batt"])?;
    let battery_line: &str = battery
        .lines()
        .nth(1)
        .ok_or_else(|| SystemInfoError::ParsingError("No battery information found".to_string()))?;

    let parts: Vec<&str> = battery_line.split(';').collect();
    Ok(BatteryInfo {
        percentage: parts.get(0).unwrap_or(&"").trim().to_string(),
        status: "AC connected".to_string(),
    })
}
