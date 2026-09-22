//! Conversation feature.
//!
//! Private submodules with a narrow interface: the runtime session and the two
//! Tauri commands defined here. `scripts/check-modules.sh` enforces this
//! registry.

mod catalog;
mod runner;
mod runtime;
mod state;
mod wire;

use tauri::{AppHandle, Emitter};

use self::wire::{ConversationAccepted, ConversationSendRequest, CONVERSATION_EVENT_NAME};
use crate::provider::contract::{NormalizedError, PROVIDER_CONTRACT_VERSION};

pub use runtime::ConversationSession;

#[tauri::command]
pub fn conversation_send(
    request: ConversationSendRequest,
    app: AppHandle,
    state: tauri::State<'_, ConversationSession>,
) -> Result<ConversationAccepted, NormalizedError> {
    let (accepted, _worker) = state.start(request, move |envelope| {
        let _ = app.emit(CONVERSATION_EVENT_NAME, envelope);
    })?;
    Ok(accepted)
}

#[tauri::command]
pub fn conversation_cancel(
    state: tauri::State<'_, ConversationSession>,
) -> Result<ConversationAccepted, NormalizedError> {
    state.cancel()?;
    Ok(ConversationAccepted {
        contract_version: PROVIDER_CONTRACT_VERSION,
        conversation: state.conversation_id(),
    })
}
