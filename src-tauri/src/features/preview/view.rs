//! Preview view hosting per ADR 0012.
//!
//! One `wry` WebView lives in a `GtkFixed` overlay child of the Tauri window,
//! outside the Tauri webview manager, so it has no Tauri IPC by construction.
//! It loads only the owned loopback origin, keeps an isolated profile under the
//! application data directory, and is destroyed on hide or application close.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use gtk::prelude::*;
use tauri::{AppHandle, Manager};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{Rect, WebContext, WebView, WebViewBuilder, WebViewBuilderExtUnix};

const MAIN_THREAD_TIMEOUT: Duration = Duration::from_secs(10);

thread_local! {
    static VIEW: RefCell<Option<PreviewView>> = const { RefCell::new(None) };
    static HOST: RefCell<Option<PreviewHost>> = const { RefCell::new(None) };
}

struct PreviewHost {
    #[allow(dead_code)]
    gtk_window: gtk::ApplicationWindow,
    #[allow(dead_code)]
    overlay: gtk::Overlay,
    #[allow(dead_code)]
    vbox: gtk::Box,
    fixed: gtk::Fixed,
}

struct PreviewView {
    webview: WebView,
    #[allow(dead_code)]
    context: WebContext,
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
    on_main_thread(app, move || {
        let webview_window = handle
            .get_webview_window("main")
            .ok_or("main window missing")?;
        let window = webview_window.as_ref().window();
        ensure_host(&window)?;
        destroy_current()?;
        let _ = std::fs::create_dir_all(&profile_root);
        HOST.with(|slot| {
            let host = slot.borrow();
            let host = host.as_ref().ok_or("preview host missing")?;
            let mut context = WebContext::new(Some(profile_root.join("preview-profile")));
            let allowed_port = port;
            let builder = WebViewBuilder::new_with_web_context(&mut context)
                .with_url(preview_url(allowed_port))
                .with_bounds(bounds)
                .with_navigation_handler(move |candidate| {
                    preview_origin_allowed(&candidate, allowed_port)
                });
            let webview = builder
                .build_gtk(&host.fixed)
                .map_err(|error| format!("preview view failed: {error}"))?;
            VIEW.with(|view| {
                *view.borrow_mut() = Some(PreviewView {
                    webview,
                    context,
                    port: allowed_port,
                });
            });
            Ok(())
        })
    })
}

pub fn set_bounds(app: &AppHandle, bounds: Rect) -> Result<(), String> {
    on_main_thread(app, move || {
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
    on_main_thread(app, destroy_current)
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

fn ensure_host(window: &tauri::Window) -> Result<(), String> {
    HOST.with(|slot| {
        if slot.borrow().is_some() {
            return Ok(());
        }
        let gtk_window = window.gtk_window().map_err(|error| error.to_string())?;
        let child = gtk_window.child().ok_or("window has no GTK child")?;
        let vbox = child
            .downcast::<gtk::Box>()
            .map_err(|_| "window child is not a gtk::Box".to_string())?;
        let overlay = gtk::Overlay::new();
        gtk_window.remove(&vbox);
        overlay.add(&vbox);
        let fixed = gtk::Fixed::new();
        overlay.add_overlay(&fixed);
        gtk_window.add(&overlay);
        gtk_window.show_all();
        *slot.borrow_mut() = Some(PreviewHost {
            gtk_window,
            overlay,
            vbox,
            fixed,
        });
        Ok(())
    })
}

fn on_main_thread<T, F>(app: &AppHandle, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = sender.send(task());
    })
    .map_err(|error| error.to_string())?;
    receiver
        .recv_timeout(MAIN_THREAD_TIMEOUT)
        .map_err(|_| "preview view operation timed out".to_string())?
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
