// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;
use tauri::Window;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_timer(window: Window, minutes: u64) {
    let duration = Duration::from_secs(minutes * 60);
    thread::spawn(move || {
        thread::sleep(duration);
        window.emit("timer-complete", ()).unwrap();
    });
}

#[tauri::command]
fn show_reminder(window: Window) {
    window.show().unwrap();
    window.set_focus().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_timer, show_reminder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
