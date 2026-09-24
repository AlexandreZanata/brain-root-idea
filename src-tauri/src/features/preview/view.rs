//! Preview view hosting per ADR 0012.
//!
//! One `wry` WebView lives in a `GtkFixed` overlay child of the Tauri window,
//! outside the Tauri webview manager, so it has no Tauri IPC by construction.
//! It loads only the owned loopback origin, keeps an isolated profile under the
//! application data directory, and is destroyed on hide or application close.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Instant;

use gtk::prelude::*;
use tauri::{AppHandle, Manager};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{Rect, WebContext, WebView, WebViewBuilder, WebViewBuilderExtUnix, WebViewExtUnix};

use crate::features::canvas_host as host;

thread_local! {
    static VIEW: RefCell<Option<PreviewView>> = const { RefCell::new(None) };
    static CONTEXT: RefCell<Option<(PathBuf, WebContext)>> = const { RefCell::new(None) };
}

struct PreviewView {
    webview: WebView,
    #[allow(dead_code)]
    port: u16,
}

/// The approved Preview origin: canonical loopback host on the owned port,
/// fail-closed on everything else (see `docs/specs/preview-origin-policy.md`).
pub fn preview_origin_allowed(candidate: &str, owned_port: u16) -> bool {
    let Ok(url) = tauri::Url::parse(candidate) else {
        return false;
    };
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    let host_allowed = matches!(
        url.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("::1") | Some("[::1]")
    );
    host_allowed && url.port() == Some(owned_port)
}

pub fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    }
}

pub fn show(app: &AppHandle, port: u16, bounds: Rect, profile_root: PathBuf) -> Result<(), String> {
    let handle = app.clone();
    host::on_main_thread(app, move || {
        let webview_window = handle
            .get_webview_window("main")
            .ok_or("main window missing")?;
        let window = webview_window.as_ref().window();
        let fixed = host::ensure_fixed(&window)?;
        destroy_current()?;
        CONTEXT.with(|slot| {
            let mut slot = slot.borrow_mut();
            let needs_context = !matches!(
                slot.as_ref(),
                Some((root, _)) if root == &profile_root
            );
            if needs_context {
                let _ = std::fs::create_dir_all(&profile_root);
                *slot = Some((
                    profile_root.clone(),
                    WebContext::new(Some(profile_root.join("preview-profile"))),
                ));
            }
            let context = &mut slot.as_mut().expect("preview context").1;
            let allowed_port = port;
            let builder = WebViewBuilder::new_with_web_context(context)
                .with_url(preview_url(allowed_port))
                .with_bounds(bounds)
                .with_navigation_handler(move |candidate| {
                    preview_origin_allowed(&candidate, allowed_port)
                });
            let webview = builder
                .build_gtk(&fixed)
                .map_err(|error| format!("preview view failed: {error}"))?;
            VIEW.with(|view| {
                *view.borrow_mut() = Some(PreviewView {
                    webview,
                    port: allowed_port,
                });
            });
            Ok(())
        })
    })
}

pub fn set_bounds(app: &AppHandle, bounds: Rect) -> Result<(), String> {
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("preview view is not visible")?;
            view.webview
                .set_bounds(bounds)
                .map_err(|error| error.to_string())
        })
    })
}

pub fn destroy(app: &AppHandle) -> Result<(), String> {
    host::on_main_thread(app, destroy_current)
}

/// Destroys the preview view. Must run on the GTK main thread; used by the
/// window-close path, which already runs there.
pub fn destroy_on_main_thread() {
    let _ = destroy_current();
}

fn preview_url(port: u16) -> String {
    let host = "127.0.0.1";
    format!("http://{host}:{port}/")
}

fn destroy_current() -> Result<(), String> {
    VIEW.with(|slot| {
        slot.borrow_mut().take();
    });
    Ok(())
}

/// Debug-only allocation rectangle: x, y, width, height in physical pixels.
#[cfg(debug_assertions)]
pub type DebugAllocation = (i32, i32, i32, i32);

/// Debug-only measurements used by the fixture harness: scale factor and the
/// preview widget's physical allocation. Release builds never compile this.
#[cfg(debug_assertions)]
pub fn debug_present(app: &AppHandle) -> bool {
    host::on_main_thread(app, || VIEW.with(|slot| Ok(slot.borrow().is_some()))).unwrap_or(false)
}

#[cfg(debug_assertions)]
pub fn debug_environment(app: &AppHandle) -> Result<(i32, DebugAllocation), String> {
    host::on_main_thread(app, || {
        let scale = host::scale_factor()?;
        let allocation = VIEW
            .with(|view| {
                view.borrow().as_ref().map(|view| {
                    let (rectangle, _) = view.webview.webview().allocated_size();
                    (
                        rectangle.x(),
                        rectangle.y(),
                        rectangle.width(),
                        rectangle.height(),
                    )
                })
            })
            .ok_or("preview view missing")?;
        Ok((scale, allocation))
    })
}

/// Debug-only focus probe: focuses the preview and reads the GTK widget focus
/// plus the host window's active state.
#[cfg(debug_assertions)]
pub fn debug_focus(app: &AppHandle) -> Result<(bool, bool), String> {
    host::on_main_thread(app, || {
        let window_active = host::window_active()?;
        let focused = VIEW
            .with(|view| {
                view.borrow().as_ref().map(|view| {
                    let _ = view.webview.focus();
                    view.webview.webview().has_focus()
                })
            })
            .unwrap_or(false);
        Ok((focused, window_active))
    })
}

/// Debug-only page-zoom readback: returns `window.innerWidth` in CSS pixels.
#[cfg(debug_assertions)]
pub fn debug_inner_width(app: &AppHandle) -> Result<f64, String> {
    let (sender, receiver) = mpsc::channel::<String>();
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("preview view missing")?;
            view.webview
                .evaluate_script_with_callback("window.innerWidth", move |value| {
                    let _ = sender.send(value);
                })
                .map_err(|error| error.to_string())
        })
    })?;
    let value = receiver
        .recv_timeout(host::MAIN_THREAD_TIMEOUT)
        .map_err(|_| "innerWidth readback timed out".to_string())?;
    let cleaned = value.trim().trim_matches('"');
    cleaned
        .parse::<f64>()
        .map_err(|_| "unexpected innerWidth value".to_string())
}

/// Debug-only device-pixel-ratio readback from the preview content.
#[cfg(debug_assertions)]
pub fn debug_device_pixel_ratio(app: &AppHandle) -> Result<f64, String> {
    let (sender, receiver) = mpsc::channel::<String>();
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("preview view missing")?;
            view.webview
                .evaluate_script_with_callback("window.devicePixelRatio", move |value| {
                    let _ = sender.send(value);
                })
                .map_err(|error| error.to_string())
        })
    })?;
    let value = receiver
        .recv_timeout(host::MAIN_THREAD_TIMEOUT)
        .map_err(|_| "devicePixelRatio readback timed out".to_string())?;
    value
        .trim()
        .trim_matches('"')
        .parse::<f64>()
        .map_err(|_| "unexpected devicePixelRatio value".to_string())
}

/// Debug-only zoom setter.
#[cfg(debug_assertions)]
pub fn debug_zoom(app: &AppHandle, scale: f64) -> Result<(), String> {
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("preview view missing")?;
            view.webview.zoom(scale).map_err(|error| error.to_string())
        })
    })
}

/// Debug-only bounds soak: runs `updates` alternating resizes on the preview
/// and returns the p50 and max per-update latency in microseconds.
#[cfg(debug_assertions)]
pub fn debug_soak(app: &AppHandle, updates: usize) -> Result<(u128, u128), String> {
    host::on_main_thread(app, move || {
        VIEW.with(|slot| {
            let view = slot.borrow();
            let view = view.as_ref().ok_or("preview view missing")?;
            let mut timings = Vec::with_capacity(updates);
            for index in 0..updates {
                let bounds = if index % 2 == 0 {
                    rect(40.0, 80.0, 420.0, 320.0)
                } else {
                    rect(40.0, 80.0, 520.0, 380.0)
                };
                let started = Instant::now();
                view.webview
                    .set_bounds(bounds)
                    .map_err(|error| error.to_string())?;
                timings.push(started.elapsed().as_micros());
            }
            timings.sort_unstable();
            let p50 = timings[timings.len() / 2];
            let max = *timings.last().unwrap_or(&0);
            Ok((p50, max))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_corpus_matches_the_policy() {
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../../../tests/preview-origin-corpus.json"))
                .expect("origin corpus json");
        let port = corpus["port"].as_u64().expect("corpus port") as u16;
        for entry in corpus["allowed"].as_array().expect("allowed entries") {
            let candidate = entry.as_str().expect("allowed candidate");
            assert!(
                preview_origin_allowed(candidate, port),
                "denied {candidate}"
            );
        }
        for entry in corpus["denied"].as_array().expect("denied entries") {
            let candidate = entry.as_str().expect("denied candidate");
            assert!(
                !preview_origin_allowed(candidate, port),
                "allowed {candidate}"
            );
        }
    }
}
