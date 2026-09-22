#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod features;
mod provider;

#[cfg(debug_assertions)]
fn debug_fake_requested() -> bool {
    std::env::var("BRAINROOT_PROVIDER_MODE").as_deref() == Ok("fake")
}

fn main() {
    let provider_state = provider::credential::ProviderState::with_boxed_store(
        provider::secret_service::platform_store(),
    );
    #[cfg(debug_assertions)]
    let conversation_session = if debug_fake_requested() {
        features::conversation::ConversationSession::debug_fake()
    } else {
        features::conversation::ConversationSession::live(provider_state.clone())
    };
    #[cfg(not(debug_assertions))]
    let conversation_session =
        features::conversation::ConversationSession::live(provider_state.clone());

    tauri::Builder::default()
        .manage(provider_state)
        .manage(conversation_session)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                use tauri::Manager;
                let session = window.state::<features::conversation::ConversationSession>();
                if session.is_active() {
                    let _ = session.cancel();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            features::health::health,
            provider::credential::provider_status,
            features::conversation::conversation_send,
            features::conversation::conversation_cancel
        ])
        .run(tauri::generate_context!())
        .expect("error while running BrainRoot");
}
