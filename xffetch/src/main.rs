mod display;
mod output;
mod system;
mod types;

use crate::display::display_system_info;
use crate::system::get_system_info;
use std::process::exit;

fn main() {
    match get_system_info() {
        Ok(info) => display_system_info(&info),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
