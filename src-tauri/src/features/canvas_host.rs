//! Shared Canvas host.
//!
//! Every content role that hosts a `wry` WebView outside the Tauri webview
//! manager (the Preview and the Human Browser) draws into one `GtkOverlay`
//! whose overlay child is a `GtkFixed`; the shell webview keeps filling the
//! window as the overlay's main child. The host is installed once per window
//! on the GTK main thread and reused by every role.

use std::cell::RefCell;
use std::sync::mpsc;
use std::time::Duration;

use gtk::prelude::*;

pub const MAIN_THREAD_TIMEOUT: Duration = Duration::from_secs(10);

thread_local! {
    static HOST: RefCell<Option<CanvasHost>> = const { RefCell::new(None) };
}

struct CanvasHost {
    gtk_window: gtk::ApplicationWindow,
    #[allow(dead_code)]
    overlay: gtk::Overlay,
    #[allow(dead_code)]
    vbox: gtk::Box,
    fixed: gtk::Fixed,
}

/// Installs the overlay/fixed container once and returns the fixed where every
/// content view is built.
pub fn ensure_fixed(window: &tauri::Window) -> Result<gtk::Fixed, String> {
    HOST.with(|slot| {
        if let Some(host) = slot.borrow().as_ref() {
            return Ok(host.fixed.clone());
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
        *slot.borrow_mut() = Some(CanvasHost {
            gtk_window,
            overlay,
            vbox,
            fixed: fixed.clone(),
        });
        Ok(fixed)
    })
}

/// The display scale factor of the host window.
pub fn scale_factor() -> Result<i32, String> {
    HOST.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|host| host.gtk_window.scale_factor())
            .ok_or_else(|| "canvas host missing".to_string())
    })
}

/// Whether the host window is active.
pub fn window_active() -> Result<bool, String> {
    HOST.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|host| host.gtk_window.is_active())
            .ok_or_else(|| "canvas host missing".to_string())
    })
}

/// Runs a closure on the GTK main thread and waits for its result.
pub fn on_main_thread<T, F>(app: &tauri::AppHandle, task: F) -> Result<T, String>
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
        .map_err(|_| "canvas host operation timed out".to_string())?
}
