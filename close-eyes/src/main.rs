// src/main.rs

mod config;
mod timer;
mod ui;

use config::AppConfig;
use timer::BreakScheduler;
use tokio::runtime::Runtime;
use ui::BreakWindow;

/**
 * Main entry point of Stretchly-RS.
 */
fn main() {
    let config = AppConfig::load().unwrap_or_default();
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        let mut scheduler = BreakScheduler::new(config);
        scheduler.start().await;
    });
}
