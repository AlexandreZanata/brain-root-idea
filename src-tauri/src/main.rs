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
        .manage(features::agent_host::AgentHostState::default())
        .manage(features::preview::PreviewState::default())
        .manage(features::human_browser::HumanBrowserState::default())
        .setup(|app| {
            #[cfg(debug_assertions)]
            if std::env::var("BRAINROOT_PREVIEW_FIXTURE").as_deref() == Ok("1") {
                features::preview::debug_fixture(app.handle().clone());
            }
            #[cfg(debug_assertions)]
            if std::env::var("BRAINROOT_HUMAN_FIXTURE").as_deref() == Ok("1") {
                features::human_browser::debug_fixture(app.handle().clone());
            }
            #[cfg(debug_assertions)]
            if std::env::var("BRAINROOT_DECK_FIXTURE").as_deref() == Ok("1") {
                features::deck::debug_fixture(app.handle().clone());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                use tauri::Manager;
                let session = window.state::<features::conversation::ConversationSession>();
                if session.is_active() {
                    let _ = session.cancel();
                }
                let host = window.state::<features::agent_host::AgentHostState>();
                host.shutdown();
                let preview = window.state::<features::preview::PreviewState>();
                preview.shutdown(&window.app_handle().clone());
                let human = window.state::<features::human_browser::HumanBrowserState>();
                human.shutdown(&window.app_handle().clone());
            }
        })
        .invoke_handler(tauri::generate_handler![
            features::health::health,
            provider::credential::provider_status,
            features::conversation::conversation_send,
            features::conversation::conversation_cancel,
            features::agent_host::agent_host_start,
            features::agent_host::agent_host_status,
            features::agent_host::agent_host_stop,
            features::agent_host::agent_host_models,
            features::agent_host::agent_host_select_model,
            features::agent_host::agent_host_send,
            features::agent_host::agent_host_cancel_send,
            features::agent_host::agent_host_catalog,
            features::agent_host::agent_host_set_agent,
            features::preview::preview_start,
            features::preview::preview_stop,
            features::preview::preview_status,
            features::preview::preview_show,
            features::preview::preview_set_bounds,
            features::preview::preview_view_status,
            features::preview::preview_hide,
            features::human_browser::human_browser_show,
            features::human_browser::human_browser_navigate,
            features::human_browser::human_browser_back,
            features::human_browser::human_browser_forward,
            features::human_browser::human_browser_reload,
            features::human_browser::human_browser_set_bounds,
            features::human_browser::human_browser_hide,
            features::human_browser::human_browser_status,
            features::human_browser::human_browser_clear_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running BrainRoot");
}
