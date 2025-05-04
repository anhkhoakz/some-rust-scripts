// src/timer.rs

use crate::{config::AppConfig, ui::BreakWindow};
use tokio::time::{Duration, sleep};

/**
 * Schedules breaks based on configuration.
 */
pub struct BreakScheduler {
    config: AppConfig,
}

impl BreakScheduler {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub async fn start(&mut self) {
        loop {
            sleep(Duration::from_secs(self.config.break_interval)).await;
            BreakWindow::show("Time for a break!").await;
        }
    }
}
