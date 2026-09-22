#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod health;
mod provider;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![health::health])
        .run(tauri::generate_context!())
        .expect("error while running BrainRoot");
}
