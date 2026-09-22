#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod conversation;
mod health;
mod provider;
mod session;

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
        session::ConversationSession::debug_fake()
    } else {
        session::ConversationSession::live(provider_state.clone())
    };
    #[cfg(not(debug_assertions))]
    let conversation_session = session::ConversationSession::live(provider_state.clone());

    tauri::Builder::default()
        .manage(provider_state)
        .manage(conversation_session)
        .invoke_handler(tauri::generate_handler![
            health::health,
            provider::credential::provider_status,
            session::conversation_send
        ])
        .run(tauri::generate_context!())
        .expect("error while running BrainRoot");
}
