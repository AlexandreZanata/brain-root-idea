//! Human Browser feature (ADR 0013).
//!
//! The first slice provides the engine: a typed navigation policy, one remote
//! view in the shared Canvas overlay with a persistent BrainRoot-owned
//! profile, denied popups and downloads, and typed commands for the shell.
//! The Canvas surface lands in B14-S02.

mod policy;
mod view;

pub use view::HumanStatus;

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::Manager;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::Rect;

#[derive(Debug, Clone, Serialize)]
pub struct HumanError {
    pub code: String,
    pub message: String,
}

impl HumanError {
    fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    fn from_code(code: &str) -> Self {
        let message = match code {
            "human_scheme_denied" => "This link type is not opened in the browser.",
            "human_userinfo_denied" => "Links with embedded credentials are not opened.",
            "human_url_invalid" => "That address is not valid.",
            "human_external_open" => "This link opens outside BrainRoot.",
            "human_popup_denied" => "Popups are blocked.",
            "human_download_denied" => "Downloads are blocked in this version.",
            "human_view_failed" => "The browser view could not be shown.",
            "human_view_not_visible" => "The browser is not visible.",
            "human_profile_unavailable" => "BrainRoot could not prepare the browser profile.",
            _ => "The browser request could not be completed.",
        };
        Self::new(code, message)
    }
}

#[derive(Default)]
pub struct HumanBrowserState {
    status: Arc<Mutex<HumanStatus>>,
    activity: Arc<Mutex<Option<std::time::Instant>>>,
    #[cfg(debug_assertions)]
    profile_override: Arc<Mutex<Option<std::path::PathBuf>>>,
}

impl HumanBrowserState {
    /// Record user activity for the governor idle clock. Called by mutating
    /// commands only; status reads never move the clock.
    pub fn touch(&self) {
        if let Ok(mut activity) = self.activity.lock() {
            *activity = Some(std::time::Instant::now());
        }
    }

    /// Seconds since the last mutating command, or `None` if never used.
    pub fn idle_secs(&self) -> Option<u64> {
        self.activity
            .lock()
            .ok()
            .and_then(|activity| activity.map(|at| at.elapsed().as_secs()))
    }
}

impl HumanBrowserState {
    fn status(&self) -> Arc<Mutex<HumanStatus>> {
        Arc::clone(&self.status)
    }

    /// Debug-only: points the harness at a probe profile instead of the real
    /// application data directory. Release builds never compile this.
    #[cfg(debug_assertions)]
    pub fn debug_set_profile_root(&self, root: std::path::PathBuf) {
        if let Ok(mut override_root) = self.profile_override.lock() {
            *override_root = Some(root);
        }
    }

    fn profile_root(&self, app: &tauri::AppHandle) -> Result<std::path::PathBuf, HumanError> {
        #[cfg(debug_assertions)]
        if let Ok(override_root) = self.profile_override.lock() {
            if let Some(root) = override_root.as_ref() {
                return Ok(root.clone());
            }
        }
        app.path()
            .app_data_dir()
            .map_err(|_| HumanError::from_code("human_profile_unavailable"))
    }

    /// Destroys the view and clears the status; used by the close path, which
    /// already runs on the GTK main thread or posts there.
    pub fn shutdown(&self, app: &tauri::AppHandle) {
        let handle = app.clone();
        let _ = handle.run_on_main_thread(view::destroy_on_main_thread);
        if let Ok(mut status) = self.status.lock() {
            *status = HumanStatus::default();
        }
    }
}

fn valid_bounds(bounds: [f64; 4]) -> bool {
    let [_, _, width, height] = bounds;
    bounds.iter().all(|value| value.is_finite())
        && width > 1.0
        && height > 1.0
        && width < 100_000.0
        && height < 100_000.0
}

fn bounds_rect(bounds: [f64; 4]) -> Rect {
    Rect {
        position: LogicalPosition::new(bounds[0], bounds[1]).into(),
        size: LogicalSize::new(bounds[2], bounds[3]).into(),
    }
}

#[tauri::command]
pub fn human_browser_show(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
    url: String,
    bounds: [f64; 4],
) -> Result<HumanStatus, HumanError> {
    if !valid_bounds(bounds) {
        return Err(HumanError::new(
            "human_bounds_invalid",
            "The browser area is not valid.",
        ));
    }
    let profile_root = app
        .path()
        .app_data_dir()
        .map_err(|_| HumanError::from_code("human_profile_unavailable"))?;
    state.touch();
    let status = state.status();
    view::show(
        &app,
        profile_root,
        url,
        bounds_rect(bounds),
        Arc::clone(&status),
    )
    .map_err(|error| HumanError::from_code(&error))?;
    let _ = view::refresh(&app, Arc::clone(&status));
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_navigate(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
    url: String,
) -> Result<HumanStatus, HumanError> {
    let status = state.status();
    view::navigate(&app, url, Arc::clone(&status))
        .map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let _ = view::refresh(&app, Arc::clone(&status));
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_back(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
) -> Result<HumanStatus, HumanError> {
    let status = state.status();
    view::back(&app).map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let _ = view::refresh(&app, Arc::clone(&status));
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_forward(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
) -> Result<HumanStatus, HumanError> {
    let status = state.status();
    view::forward(&app).map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let _ = view::refresh(&app, Arc::clone(&status));
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_reload(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
) -> Result<HumanStatus, HumanError> {
    let status = state.status();
    view::reload(&app).map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let _ = view::refresh(&app, Arc::clone(&status));
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_set_bounds(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
    bounds: [f64; 4],
) -> Result<HumanStatus, HumanError> {
    if !valid_bounds(bounds) {
        return Err(HumanError::new(
            "human_bounds_invalid",
            "The browser area is not valid.",
        ));
    }
    let status = state.status();
    view::set_bounds(&app, bounds_rect(bounds)).map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_hide(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
) -> Result<HumanStatus, HumanError> {
    let status = state.status();
    view::hide(&app, Arc::clone(&status)).map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_clear_data(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
) -> Result<HumanStatus, HumanError> {
    let profile_root = state.profile_root(&app)?;
    let status = state.status();
    view::clear_data(&app, profile_root, Arc::clone(&status))
        .map_err(|error| HumanError::from_code(&error))?;
    state.touch();
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

#[tauri::command]
pub fn human_browser_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, HumanBrowserState>,
) -> Result<HumanStatus, HumanError> {
    let status = state.status();
    let _ = view::refresh(&app, Arc::clone(&status));
    let snapshot = status.lock().expect("human status").clone();
    Ok(snapshot)
}

/// Debug-only: whether a live human browser view exists.
#[cfg(debug_assertions)]
pub fn debug_view_present(app: &tauri::AppHandle) -> bool {
    view::debug_present(app)
}

#[cfg(debug_assertions)]
pub fn debug_fixture(app: tauri::AppHandle) {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc as StdArc;
    use std::thread;
    use std::time::Duration;

    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(_) => return,
    };
    listener
        .set_nonblocking(true)
        .expect("human fixture nonblocking");
    let addr = listener.local_addr().expect("human fixture address");
    let base_url = format!("http://{}:{}/", addr.ip(), addr.port());
    let stop = StdArc::new(AtomicBool::new(false));
    let thread_stop = StdArc::clone(&stop);
    thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buffer = [0_u8; 2048];
                    let read = stream.read(&mut buffer).unwrap_or(0);
                    let request = String::from_utf8_lossy(&buffer[..read]);
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("/");
                    let title = if path == "/a" {
                        "page-a"
                    } else if path == "/b" {
                        "page-b"
                    } else {
                        "page-missing"
                    };
                    let body = format!("<!doctype html><title>{title}</title><p>{title}</p>");
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });

    thread::spawn(move || {
        let state = app.state::<HumanBrowserState>();
        let status = state.status();
        let probe_root = match app.path().app_data_dir() {
            Ok(root) => root.join("human-browser-probe"),
            Err(_) => return,
        };
        state.debug_set_profile_root(probe_root.clone());
        let mut report = serde_json::Map::new();
        let mut reasons: Vec<String> = Vec::new();
        let page_a = format!("{base_url}a");
        let page_b = format!("{base_url}b");

        let shown = view::show(
            &app,
            probe_root.clone(),
            page_a.clone(),
            bounds_rect([40.0, 80.0, 560.0, 380.0]),
            Arc::clone(&status),
        )
        .is_ok();
        let reached_a = wait_for(&app, &status, |status| {
            status.title.as_deref() == Some("page-a")
        });
        if !shown || !reached_a {
            reasons.push("page-a not reached".to_string());
        }

        let _ = view::navigate(&app, page_b.clone(), Arc::clone(&status));
        let reached_b = wait_for(&app, &status, |status| {
            status.title.as_deref() == Some("page-b")
        });
        if !reached_b {
            reasons.push("page-b not reached".to_string());
        }

        let _ = view::back(&app);
        let back_ok = wait_for(&app, &status, |status| {
            status.title.as_deref() == Some("page-a")
                && status.url.as_deref().is_some_and(|url| url.ends_with("/a"))
        });
        if !back_ok {
            reasons.push("back navigation failed".to_string());
        }

        let _ = view::forward(&app);
        let forward_ok = wait_for(&app, &status, |status| {
            status.title.as_deref() == Some("page-b")
        });
        if !forward_ok {
            reasons.push("forward navigation failed".to_string());
        }

        let _ = view::reload(&app);
        thread::sleep(Duration::from_millis(400));
        let _ = view::refresh(&app, Arc::clone(&status));

        let denial =
            view::navigate(&app, "file:///etc/passwd".to_string(), Arc::clone(&status)).err();
        let _ = view::refresh(&app, Arc::clone(&status));
        let (final_url, denial_code) = {
            let status = status.lock().expect("human status");
            (status.url.clone(), status.last_denial.clone())
        };
        let denied = denial_code.as_deref() == Some("human_scheme_denied") && denial.is_some();
        if !denied {
            reasons.push("file scheme was not denied".to_string());
        }

        // Permission denial: geolocation and user media must be denied and
        // recorded, never prompted.
        let _ = view::debug_evaluate(
            &app,
            "navigator.geolocation.getCurrentPosition(() => {}, () => {});              navigator.mediaDevices.getUserMedia({ audio: true }).catch(() => {}); 'probe'",
        );
        let permission_denied = wait_for(&app, &status, |status| {
            status
                .last_denial
                .as_deref()
                .map(|code| code.starts_with("human_permission_denied"))
                .unwrap_or(false)
        });
        if !permission_denied {
            reasons.push("permission request was not denied".to_string());
        }

        // Browser data: write a marker, clear the data, reload, and expect the
        // marker to be gone with a fresh profile.
        let _ = view::debug_evaluate(&app, "localStorage.setItem('probe', 'value'); 'ok'");
        let before_clear =
            view::debug_evaluate(&app, "localStorage.getItem('probe')").unwrap_or_default();
        let _ = human_browser_clear_data(app.clone(), app.state::<HumanBrowserState>());
        let _ = view::show(
            &app,
            probe_root.clone(),
            page_a.clone(),
            bounds_rect([40.0, 80.0, 560.0, 380.0]),
            Arc::clone(&status),
        );
        let _ = wait_for(&app, &status, |status| {
            status.title.as_deref() == Some("page-a")
        });
        let after_clear =
            view::debug_evaluate(&app, "localStorage.getItem('probe')").unwrap_or_default();
        let data_cleared = before_clear.contains("value") && after_clear.contains("null");
        if !data_cleared {
            reasons.push("browser data was not cleared".to_string());
        }

        let _ = view::hide(&app, Arc::clone(&status));
        thread::sleep(Duration::from_millis(300));
        let hidden = !view::debug_present(&app);
        if !hidden {
            reasons.push("view was not destroyed".to_string());
        }
        let _ = std::fs::remove_dir_all(&probe_root);

        report.insert(
            "decision".to_string(),
            serde_json::json!(if reasons.is_empty() { "go" } else { "no-go" }),
        );
        report.insert("page_a_reached".to_string(), serde_json::json!(reached_a));
        report.insert("page_b_reached".to_string(), serde_json::json!(reached_b));
        report.insert("back_ok".to_string(), serde_json::json!(back_ok));
        report.insert("forward_ok".to_string(), serde_json::json!(forward_ok));
        report.insert("denial_code".to_string(), serde_json::json!(denial_code));
        report.insert(
            "permission_denied".to_string(),
            serde_json::json!(permission_denied),
        );
        report.insert("data_cleared".to_string(), serde_json::json!(data_cleared));
        report.insert("final_url".to_string(), serde_json::json!(final_url));
        report.insert("hidden".to_string(), serde_json::json!(hidden));
        report.insert("reasons".to_string(), serde_json::json!(reasons));
        println!(
            "brainroot: human browser probe {}",
            serde_json::Value::Object(report)
        );
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        stop.store(true, Ordering::Relaxed);
        app.exit(0);
    });
}

#[cfg(debug_assertions)]
fn wait_for(
    app: &tauri::AppHandle,
    status: &Arc<Mutex<HumanStatus>>,
    predicate: impl Fn(&HumanStatus) -> bool,
) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let _ = view::refresh(app, Arc::clone(status));
        if status
            .lock()
            .map(|status| predicate(&status))
            .unwrap_or(false)
        {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
