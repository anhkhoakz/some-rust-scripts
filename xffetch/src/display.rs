use crate::output::{OutputHelper, OutputOptions, OutputType};
use crate::types::{COLORS, SystemInfo};

pub fn display_system_info(info: &SystemInfo) {
    let green: &str = COLORS[0];
    let yellow: &str = COLORS[1];
    let orange: &str = COLORS[2];
    let blue: &str = COLORS[3];
    let cyan: &str = COLORS[4];
    let magenta: &str = COLORS[5];
    let gray: &str = COLORS[6];
    let reset: &str = COLORS[7];

    // Create header
    println!(
        "{magenta}{username}@{hostname}{reset}",
        magenta = magenta,
        username = info.username,
        hostname = info.hostname,
        reset = reset
    );
    println!(
        "{gray}-----------------------------{reset}",
        gray = gray,
        reset = reset
    );

    // Create output helper with default options
    let mut output: OutputHelper = OutputHelper::new(OutputOptions {
        output_type: OutputType::Rsfetch,
        caps: true,
        bold: true,
        use_borders: true,
        borders: '┃',
    });

    // Add system information
    output.add(
        "OS",
        &format!(
            "{} {} {} {}",
            info.os, info.os_release_name, info.os_version, info.architecture
        ),
    );
    output.add("Host", &info.model);
    output.add("Kernel", &format!("Darwin {}", info.kernel));
    output.add("Uptime", &info.uptime);
    output.add(
        "Packages",
        &format!(
            "{} (brew), {} (brew-cask)",
            info.packages.brew_count, info.packages.brew_cask_count
        ),
    );
    output.add("Shell", &info.shell.version);
    output.add("Display", &info.display);
    output.add("DE", "_");
    output.add("WM", "_");
    output.add("WM Theme", "_");
    output.add("Font", "_");
    output.add("Cursor", &info.cursor.theme);
    output.add("Terminal", &info.terminal);
    output.add("CPU", &format!("{} ({})", info.cpu.model, info.cpu.cores));
    output.add("GPU", &info.gpu);
    output.add("Memory", &format!("{:.2} GiB", info.memory));
    output.add(
        "Swap",
        &format!(
            "{} / {} ({})",
            info.swap.used, info.swap.total, info.swap.percentage
        ),
    );
    output.add(
        "Disk (/)",
        &format!(
            "{} / {} ({})",
            info.disk.used, info.disk.total, info.disk.percentage
        ),
    );
    output.add("Local IP (en0)", &format!("{}/_", info.network.local_ip));
    output.add(
        "Battery",
        &format!("{} [{}]", info.battery.percentage, info.battery.status),
    );
    output.add("Power Adapter", &info.power_adapter);
    output.add("Locale", &info.locale);

    // Output the information
    output.output();

    // Color blocks
    println!(
        "\n{green}███{yellow}███{orange}███{blue}███{cyan}███{magenta}███{gray}███{reset}",
        green = green,
        yellow = yellow,
        orange = orange,
        blue = blue,
        cyan = cyan,
        magenta = magenta,
        gray = gray,
        reset = reset
    );
}
