//! Companion Canvas preview feature.
//!
//! The first slice owns the dev-server lifecycle (ADR 0011); the preview view
//! itself (ADR 0012) lands in a later microstep. The public interface is the
//! typed command set used by the shell and the managed state.

mod dev_server;

pub use dev_server::{
    PreviewError, PreviewStartRequest, PreviewStartResponse, PreviewStatus, PreviewStopResponse,
};

use dev_server::DevServerSupervisor;

#[derive(Default)]
pub struct PreviewState {
    supervisor: DevServerSupervisor,
}

impl PreviewState {
    /// Stops the owned dev server with a short grace, for window close and
    /// application exit paths where blocking is acceptable but bounded.
    pub fn stop_now(&self) {
        let _ = self.supervisor.stop(1_000);
    }
}

#[tauri::command]
pub fn preview_start(
    state: tauri::State<'_, PreviewState>,
    request: PreviewStartRequest,
) -> Result<PreviewStartResponse, PreviewError> {
    state.supervisor.start(request)
}

#[tauri::command]
pub fn preview_stop(
    state: tauri::State<'_, PreviewState>,
    grace_ms: Option<u64>,
) -> Result<PreviewStopResponse, PreviewError> {
    match grace_ms {
        Some(grace_ms) => state.supervisor.stop(grace_ms),
        None => state.supervisor.stop_default(),
    }
}

#[tauri::command]
pub fn preview_status(state: tauri::State<'_, PreviewState>) -> PreviewStatus {
    state.supervisor.status()
}
