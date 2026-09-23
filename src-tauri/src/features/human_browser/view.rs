//! Human Browser view hosting per ADR 0013.
//!
//! The view is a `wry` WebView built into the shared Canvas overlay, outside
//! the Tauri webview manager, so it has no Tauri IPC by construction. It uses
//! a persistent BrainRoot-owned `human-profile`, enforces the typed navigation
//! policy, denies popups and downloads, and keeps a title/URL/history status
//! for the shell. One view exists at most.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};
use webkit2gtk::WebViewExt as _;
use wry::{
    NewWindowResponse, Rect, WebContext, WebView, WebViewBuilder, WebViewBuilderExtUnix,
    WebViewExtUnix,
};

use super::policy::{decide_navigation, NavigationDecision};
use crate::features::canvas_host as host;

thread_local! {
    static VIEW: RefCell<Option<HumanView>> = const { RefCell::new(None) };
    static CONTEXT: RefCell<Option<(PathBuf, WebContext)>> = const { RefCell::new(None) };
}

struct HumanView {
    webview: WebView,
}

/// The shell-visible status of the Human Browser.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HumanStatus {
    pub role: super::policy::BrowserRole,
    pub visible: bool,
    pub url: Option<String>,
    pub title: Option<String>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub last_denial: Option<String>,
}

fn emit_status(app: &AppHandle, status: &Arc<Mutex<HumanStatus>>) {
    if let Ok(status) = status.lock() {
        let _ = app.emit("human-browser-status", &*status);
    }
}

fn record_denial(status: &Arc<Mutex<HumanStatus>>, code: &str) {
    if let Ok(mut status) = status.lock() {
        status.last_denial = Some(code.to_string());
    }
}

fn record_allowed(status: &Arc<Mutex<HumanStatus>>, candidate: &str) {
    if let Ok(mut status) = status.lock() {
        status.url = Some(candidate.to_string());
        status.last_denial = None;
    }
}

pub fn show(
    app: &AppHandle,
    profile_root: PathBuf,
    url: String,
    bounds: Rect,
    status: Arc<Mutex<HumanStatus>>,
) -> Result<(), String> {
    match decide_navigation(&url) {
        NavigationDecision::Allow => {}
        NavigationDecision::OpenExternal => {
            record_denial(&status, "human_external_open");
            return Err("human_external_open".to_string());
        }
        NavigationDecision::Deny { code } => {
            record_denial(&status, &code);
            return Err(code);
        }
    }
    let handle = app.clone();
    host::on_main_thread(app, move || {
        let webview_window = handle
            .get_webview_window("main")
            .ok_or("main window missing")?;
        let window = webview_window.as_ref().window();
        let fixed = host::ensure_fixed(&window)?;
        destroy_current()?;

        let nav_status = Arc::clone(&status);
        let popup_status = Arc::clone(&status);
        let download_status = Arc::clone(&status);
        let title_status = Arc::clone(&status);
        let nav_app = handle.clone();
        let popup_app = handle.clone();
        let download_app = handle.clone();
        let title_app = handle.clone();
        let initial_url = url.clone();
        CONTEXT.with::<_, Result<(), String>>(|slot| {
            let mut slot = slot.borrow_mut();
            let needs_context = !matches!(
                slot.as_ref(),
                Some((root, _)) if root == &profile_root
            );
            if needs_context {
                let _ = std::fs::create_dir_all(&profile_root);
                *slot = Some((
                    profile_root.clone(),
                    WebContext::new(Some(profile_root.join("human-profile"))),
                ));
            }
            let context = &mut slot.as_mut().expect("human context").1;
            let builder = WebViewBuilder::new_with_web_context(context)
                .with_url(url)
                .with_bounds(bounds)
                .with_navigation_handler(move |candidate| {
                    let allowed = match decide_navigation(&candidate) {
                        NavigationDecision::Allow => {
                            record_allowed(&nav_status, &candidate);
                            true
                        }
                        NavigationDecision::OpenExternal => {
                            record_denial(&nav_status, "human_external_open");
                            false
                        }
                        NavigationDecision::Deny { code } => {
                            record_denial(&nav_status, &code);
                            false
                        }
                    };
                    emit_status(&nav_app, &nav_status);
                    allowed
                })
                .with_new_window_req_handler(move |_url, _features| {
                    record_denial(&popup_status, "human_popup_denied");
                    emit_status(&popup_app, &popup_status);
                    NewWindowResponse::Deny
                })
                .with_download_started_handler(move |_url, _path| {
                    record_denial(&download_status, "human_download_denied");
                    emit_status(&download_app, &download_status);
                    false
                })
                .with_document_title_changed_handler(move |title| {
                    if let Ok(mut status) = title_status.lock() {
                        status.title = Some(title);
                    }
                    emit_status(&title_app, &title_status);
                });
            let webview = builder
                .build_gtk(&fixed)
                .map_err(|error| format!("human view failed: {error}"))?;
            VIEW.with(|slot| {
                *slot.borrow_mut() = Some(HumanView { webview });
            });
            Ok(())
        })?;
        if let Ok(mut status) = status.lock() {
            status.visible = true;
            status.url = Some(initial_url);
        }
        emit_status(&handle, &status);
        Ok(())
    })
}

pub fn navigate(
    app: &AppHandle,
    url: String,
    status: Arc<Mutex<HumanStatus>>,
) -> Result<(), String> {
    match decide_navigation(&url) {
        NavigationDecision::Allow => {}
        NavigationDecision::OpenExternal => {
            record_denial(&status, "human_external_open");
            return Err("human_external_open".to_string());
        }
        NavigationDecision::Deny { code } => {
            record_denial(&status, &code);
            return Err(code);
        }
    }
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("human view is not visible")?;
            view.webview
                .load_url(&url)
                .map_err(|error| error.to_string())
        })
    })
}

pub fn back(app: &AppHandle) -> Result<(), String> {
    host::on_main_thread(app, || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("human view is not visible")?;
            view.webview.webview().go_back();
            Ok(())
        })
    })
}

pub fn forward(app: &AppHandle) -> Result<(), String> {
    host::on_main_thread(app, || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("human view is not visible")?;
            view.webview.webview().go_forward();
            Ok(())
        })
    })
}

pub fn reload(app: &AppHandle) -> Result<(), String> {
    host::on_main_thread(app, || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("human view is not visible")?;
            view.webview.reload().map_err(|error| error.to_string())
        })
    })
}

pub fn set_bounds(app: &AppHandle, bounds: Rect) -> Result<(), String> {
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("human view is not visible")?;
            view.webview
                .set_bounds(bounds)
                .map_err(|error| error.to_string())
        })
    })
}

pub fn hide(app: &AppHandle, status: Arc<Mutex<HumanStatus>>) -> Result<(), String> {
    host::on_main_thread(app, destroy_current)?;
    if let Ok(mut status) = status.lock() {
        *status = HumanStatus::default();
    }
    Ok(())
}

/// Reads the live URL and history state into the shared status.
pub fn refresh(app: &AppHandle, status: Arc<Mutex<HumanStatus>>) -> Result<(), String> {
    let snapshot = host::on_main_thread(app, || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let Some(view) = view.as_ref() else {
                return Ok(None);
            };
            let url = view.webview.url().ok();
            let can_go_back = view.webview.webview().can_go_back();
            let can_go_forward = view.webview.webview().can_go_forward();
            Ok(Some((url, can_go_back, can_go_forward)))
        })
    })?;
    if let Some((url, can_go_back, can_go_forward)) = snapshot {
        if let Ok(mut status) = status.lock() {
            status.visible = true;
            status.url = url;
            status.can_go_back = can_go_back;
            status.can_go_forward = can_go_forward;
        }
    }
    Ok(())
}

/// Destroys the human view. Must run on the GTK main thread; used by the
/// window-close path, which already runs there.
pub fn destroy_on_main_thread() {
    let _ = destroy_current();
}

fn destroy_current() -> Result<(), String> {
    VIEW.with(|slot| {
        slot.borrow_mut().take();
    });
    Ok(())
}

#[cfg(debug_assertions)]
pub fn debug_present(app: &AppHandle) -> bool {
    host::on_main_thread(app, || VIEW.with(|slot| Ok(slot.borrow().is_some()))).unwrap_or(false)
}
