#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod health;
mod provider;

fn main() {
    tauri::Builder::default()
        .manage(provider::credential::ProviderState::new())
        .invoke_handler(tauri::generate_handler![
            health::health,
            provider::credential::provider_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running BrainRoot");
}
