//! Companion Canvas preview feature.
//!
//! The feature owns the dev-server lifecycle (ADR 0011) and the preview view
//! hosting (ADR 0012). The public interface is the typed command set used by
//! the shell plus the managed state.

mod dev_server;
mod view;

pub use dev_server::{
    PreviewError, PreviewPhase, PreviewStartRequest, PreviewStartResponse, PreviewStatus,
    PreviewStopResponse,
};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use dev_server::DevServerSupervisor;
use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Clone, Default, Serialize)]
pub struct PreviewViewStatus {
    pub visible: bool,
    pub port: Option<u16>,
    pub bounds: Option<(f64, f64, f64, f64)>,
}

#[derive(Default)]
pub struct PreviewState {
    supervisor: DevServerSupervisor,
    view: Arc<Mutex<PreviewViewStatus>>,
}

impl PreviewState {
    /// Destroys the preview view and stops the owned dev server. Called from
    /// the window-close path; the view destruction is posted to the GTK main
    /// thread and the server stop is bounded by the short grace.
    pub fn shutdown(&self, app: &tauri::AppHandle) {
        let handle = app.clone();
        let _ = handle.run_on_main_thread(view::destroy_on_main_thread);
        let _ = self.supervisor.stop(1_000);
    }
}

fn bounds_tuple(bounds: [f64; 4]) -> (f64, f64, f64, f64) {
    (bounds[0], bounds[1], bounds[2], bounds[3])
}

fn valid_bounds(bounds: [f64; 4]) -> bool {
    let [_, _, width, height] = bounds;
    width.is_finite()
        && height.is_finite()
        && width > 1.0
        && height > 1.0
        && width < 100_000.0
        && height < 100_000.0
        && bounds.iter().all(|value| value.is_finite())
}

#[tauri::command]
pub fn preview_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, PreviewState>,
    request: PreviewStartRequest,
) -> Result<PreviewStartResponse, PreviewError> {
    state.supervisor.start(request, Some(app))
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

#[tauri::command]
pub fn preview_show(
    app: tauri::AppHandle,
    state: tauri::State<'_, PreviewState>,
    port: u16,
    bounds: [f64; 4],
) -> Result<PreviewViewStatus, PreviewError> {
    if port == 0 || !valid_bounds(bounds) {
        return Err(PreviewError::new(
            "preview_bounds_invalid",
            "The preview area is not valid.",
        ));
    }
    let profile_root = app.path().app_data_dir().map_err(|_| {
        PreviewError::new(
            "preview_profile_unavailable",
            "BrainRoot could not prepare the preview profile.",
        )
    })?;
    view::show(
        &app,
        port,
        view::rect(bounds[0], bounds[1], bounds[2], bounds[3]),
        profile_root,
    )
    .map_err(|_| PreviewError::new("preview_view_failed", "The preview could not be shown."))?;
    let status = PreviewViewStatus {
        visible: true,
        port: Some(port),
        bounds: Some(bounds_tuple(bounds)),
    };
    *state.view.lock().expect("preview view state") = status.clone();
    Ok(status)
}

#[tauri::command]
pub fn preview_set_bounds(
    app: tauri::AppHandle,
    state: tauri::State<'_, PreviewState>,
    bounds: [f64; 4],
) -> Result<PreviewViewStatus, PreviewError> {
    if !valid_bounds(bounds) {
        return Err(PreviewError::new(
            "preview_bounds_invalid",
            "The preview area is not valid.",
        ));
    }
    view::set_bounds(&app, view::rect(bounds[0], bounds[1], bounds[2], bounds[3])).map_err(
        |_| PreviewError::new("preview_view_not_visible", "The preview is not visible."),
    )?;
    let mut status = state.view.lock().expect("preview view state");
    status.bounds = Some(bounds_tuple(bounds));
    Ok(status.clone())
}

#[tauri::command]
pub fn preview_view_status(state: tauri::State<'_, PreviewState>) -> PreviewViewStatus {
    state.view.lock().expect("preview view state").clone()
}

#[tauri::command]
pub fn preview_hide(
    app: tauri::AppHandle,
    state: tauri::State<'_, PreviewState>,
) -> Result<PreviewViewStatus, PreviewError> {
    view::destroy(&app).map_err(|_| {
        PreviewError::new(
            "preview_view_hide_failed",
            "The preview could not be hidden.",
        )
    })?;
    let mut status = state.view.lock().expect("preview view state");
    *status = PreviewViewStatus::default();
    Ok(status.clone())
}

#[cfg(debug_assertions)]
fn rss_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("VmRSS:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0)
}

#[cfg(debug_assertions)]
fn child_process_count() -> usize {
    let own_pid = std::process::id();
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(pid) = name.to_str().and_then(|value| value.parse::<u32>().ok()) else {
                continue;
            };
            if pid == own_pid {
                continue;
            }
            let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
                continue;
            };
            let Some(after_command) = stat.rfind(')').map(|index| &stat[index + 1..]) else {
                continue;
            };
            let parent = after_command
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u32>().ok());
            if parent == Some(own_pid) {
                count += 1;
            }
        }
    }
    count
}

/// Debug-only: whether a live preview view exists.
#[cfg(debug_assertions)]
pub fn debug_view_present(app: &tauri::AppHandle) -> bool {
    view::debug_present(app)
}

/// Debug-only fixture harness: starts the owned server with a local HTTP
/// fixture, then shows the preview view at a fixed rectangle and prints one
/// marker once it is created. Release builds never compile or run this path.
#[cfg(debug_assertions)]
pub fn debug_fixture(app: tauri::AppHandle) {
    thread::spawn(move || {
        let state = app.state::<PreviewState>();
        let request = PreviewStartRequest {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "python3 -m http.server \"$PORT\" --bind 127.0.0.1".to_string(),
            ],
            cwd: std::env::temp_dir().to_string_lossy().to_string(),
            readiness_timeout_ms: Some(15_000),
            stop_grace_ms: Some(1_000),
        };
        let Ok(started) = state.supervisor.start(request, Some(app.clone())) else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if state.supervisor.status().phase == dev_server::PreviewPhase::Ready {
                break;
            }
            if Instant::now() >= deadline {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        let Ok(profile_root) = app.path().app_data_dir() else {
            return;
        };
        let bounds = (40.0, 80.0, 480.0, 320.0);
        if view::show(
            &app,
            started.port,
            view::rect(bounds.0, bounds.1, bounds.2, bounds.3),
            profile_root,
        )
        .is_ok()
        {
            *state.view.lock().expect("preview view state") = PreviewViewStatus {
                visible: true,
                port: Some(started.port),
                bounds: Some(bounds),
            };
            println!("brainroot: preview view ready");

            let mut inner_width_base = None;
            let deadline = Instant::now() + Duration::from_secs(8);
            while inner_width_base.is_none() && Instant::now() < deadline {
                inner_width_base = view::debug_inner_width(&app).ok();
                if inner_width_base.is_none() {
                    thread::sleep(Duration::from_millis(200));
                }
            }
            let mut environment = None;
            let deadline = Instant::now() + Duration::from_secs(4);
            while environment.is_none() && Instant::now() < deadline {
                if let Ok(measured) = view::debug_environment(&app) {
                    if measured.1 != (0, 0, 0, 0) {
                        environment = Some(measured);
                    }
                }
                if environment.is_none() {
                    thread::sleep(Duration::from_millis(150));
                }
            }
            let focus = view::debug_focus(&app).ok();
            let device_pixel_ratio = view::debug_device_pixel_ratio(&app).ok();
            let _ = view::debug_zoom(&app, 2.0);
            thread::sleep(Duration::from_millis(400));
            let inner_width_zoomed = view::debug_inner_width(&app).ok();
            let _ = view::debug_zoom(&app, 1.0);
            thread::sleep(Duration::from_millis(400));
            let inner_width_restored = view::debug_inner_width(&app).ok();
            let rss_before = rss_kb();
            let children_before = child_process_count();
            let soak = view::debug_soak(&app, 100).ok();
            thread::sleep(Duration::from_millis(300));
            let rss_after = rss_kb();
            let children_after = child_process_count();
            let quality = serde_json::json!({
                "scale_factor": environment.as_ref().map(|(scale, _)| *scale),
                "allocated_physical": environment.as_ref().map(|(_, allocation)| *allocation),
                "requested_logical": [bounds.0, bounds.1, bounds.2, bounds.3],
                "widget_focused": focus.map(|(focused, _)| focused),
                "window_active": focus.map(|(_, active)| active),
                "device_pixel_ratio": device_pixel_ratio,
                "inner_width_base": inner_width_base,
                "inner_width_zoomed": inner_width_zoomed,
                "inner_width_restored": inner_width_restored,
                "soak_updates": soak.map(|_| 100),
                "soak_p50_us": soak.map(|(p50, _)| p50),
                "soak_max_us": soak.map(|(_, max)| max),
                "rss_kb_before": rss_before,
                "rss_kb_after": rss_after,
                "children_before": children_before,
                "children_after": children_after,
            });
            println!("brainroot: preview quality {quality}");

            thread::sleep(Duration::from_secs(2));
            state.shutdown(&app);
            thread::sleep(Duration::from_secs(1));
            app.exit(0);
        }
    });
}
