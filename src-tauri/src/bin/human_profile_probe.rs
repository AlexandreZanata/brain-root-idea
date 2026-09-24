//! Disposable Human Browser profile isolation probe (B13-S02).
//!
//! Built only with `--features probe`. It verifies, through real `wry` views
//! with separate `WebContext` data directories under the application data
//! directory, that the Human Browser profile and the Preview profile are
//! isolated in both directions and persist across close/reopen. The shipped
//! application never includes this binary.

use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use serde_json::json;
use tauri::Manager;
use wry::{Rect, WebContext, WebView, WebViewBuilder, WebViewBuilderExtUnix};

const TICK: Duration = Duration::from_millis(50);
const PHASE_TIMEOUT: Duration = Duration::from_secs(12);
const CHILD_WIDTH: f64 = 420.0;
const CHILD_HEIGHT: f64 = 320.0;

fn main() {
    let fixture = Fixture::start();
    let base_url = fixture.base_url();
    let stop_flag = fixture.stop_flag();

    tauri::Builder::default()
        .setup(move |app| {
            let host = tauri::window::WindowBuilder::new(app, "probe-host")
                .inner_size(680.0, 520.0)
                .build()?;
            install_host(&host)?;
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.close();
            }
            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("app data dir: {error}"))?;
            let handle = app.handle().clone();
            let state = Rc::new(RefCell::new(Probe::new(
                app_data,
                base_url.clone(),
                stop_flag.clone(),
            )));
            glib::timeout_add_local(TICK, move || {
                let mut probe = state.borrow_mut();
                if probe.step(&handle) {
                    ControlFlow::Break
                } else {
                    ControlFlow::Continue
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("human profile probe failed to run");
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: wry::dpi::LogicalPosition::new(x, y).into(),
        size: wry::dpi::LogicalSize::new(width, height).into(),
    }
}

fn directory_size_bytes(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total += directory_size_bytes(&entry_path);
            } else if let Ok(metadata) = entry.metadata() {
                total += metadata.len();
            }
        }
    }
    total
}

struct Fixture {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
}

impl Fixture {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("fixture binds");
        let addr = listener.local_addr().expect("fixture address");
        listener.set_nonblocking(true).expect("fixture nonblocking");
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = serve(stream);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => break,
                }
            }
        });
        Self { addr, stop }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}/", self.addr.ip(), self.addr.port())
    }

    fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }
}

fn serve(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut buffer = [0_u8; 4096];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let (status, body) = if path.starts_with("/profile") {
        (200, profile_page())
    } else {
        (
            404,
            "<!doctype html><title>profile-missing</title>".to_string(),
        )
    };
    let reason = if status == 200 { "OK" } else { "Not Found" };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn profile_page() -> String {
    "<!doctype html><title>profile</title><script>\
     const params = new URLSearchParams(location.search);\
     const key = params.get('key');\
     const value = params.get('value');\
     if (value && key) { localStorage.setItem(key, value); }\
     const base = value && key\
       ? 'stored:' + (localStorage.getItem(key) ?? 'empty')\
       : (key ? 'read:' + (localStorage.getItem(key) ?? 'empty') : 'ready');\
     const read = params.get('read');\
     const read2 = params.get('read2');\
     document.title = base\
       + (read ? '|' + (localStorage.getItem(read) ?? 'empty') : '')\
       + (read2 ? '|' + (localStorage.getItem(read2) ?? 'empty') : '');\
     </script>"
        .to_string()
}

enum Phase {
    HumanWrite,
    PreviewWrite,
    HumanRead,
    Finish,
}

struct Probe {
    probe_root: PathBuf,
    preview_dir: PathBuf,
    human_dir: PathBuf,
    app_data_ok: bool,
    base_url: String,
    stop_flag: Arc<AtomicBool>,
    phase: Phase,
    deadline: Instant,
    view: Option<WebView>,
    titles: Option<Receiver<String>>,
    last_title: String,
    human_write_title: Option<String>,
    preview_write_title: Option<String>,
    human_read_title: Option<String>,
    reasons: Vec<String>,
}

impl Probe {
    fn new(app_data: PathBuf, base_url: String, stop_flag: Arc<AtomicBool>) -> Self {
        let probe_root = app_data.join("human-profile-probe");
        let preview_dir = probe_root.join("preview-profile-probe");
        let human_dir = probe_root.join("human-profile-probe");
        let app_data_ok = probe_root.starts_with(&app_data)
            && preview_dir.starts_with(&app_data)
            && human_dir.starts_with(&app_data)
            && preview_dir != human_dir;
        let _ = std::fs::create_dir_all(&preview_dir);
        let _ = std::fs::create_dir_all(&human_dir);
        Self {
            probe_root,
            preview_dir,
            human_dir,
            app_data_ok,
            base_url,
            stop_flag,
            phase: Phase::HumanWrite,
            deadline: Instant::now() + PHASE_TIMEOUT,
            view: None,
            titles: None,
            last_title: String::new(),
            human_write_title: None,
            preview_write_title: None,
            human_read_title: None,
            reasons: Vec::new(),
        }
    }

    fn create_view(&mut self, data_dir: &Path, url: String) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel::<String>();
        let mut context = WebContext::new(Some(data_dir.to_path_buf()));
        let builder = WebViewBuilder::new_with_web_context(&mut context)
            .with_url(url)
            .with_bounds(rect(0.0, 0.0, CHILD_WIDTH, CHILD_HEIGHT))
            .with_document_title_changed_handler(move |title| {
                let _ = sender.send(title);
            });
        let vbox = HOST
            .with(|slot| slot.borrow().as_ref().map(|host| host.vbox.clone()))
            .ok_or("probe host window missing")?;
        let webview = builder
            .build_gtk(&vbox)
            .map_err(|error| format!("view create failed: {error}"))?;
        self.view = Some(webview);
        self.titles = Some(receiver);
        self.last_title.clear();
        self.deadline = Instant::now() + PHASE_TIMEOUT;
        Ok(())
    }

    fn drain_title(&mut self) {
        if let Some(receiver) = self.titles.as_ref() {
            loop {
                match receiver.try_recv() {
                    Ok(title) => {
                        if !title.is_empty() {
                            self.last_title = title;
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        }
    }

    fn step(&mut self, app: &tauri::AppHandle) -> bool {
        match self.phase {
            Phase::HumanWrite => {
                if self.view.is_none() {
                    if let Err(error) = self.create_view(
                        &self.human_dir.clone(),
                        format!("{}profile?key=hb&value=human-marker&read=pv", self.base_url),
                    ) {
                        self.reasons.push(error);
                        self.phase = Phase::Finish;
                        return false;
                    }
                }
                self.drain_title();
                if self.last_title == "stored:human-marker|empty" {
                    self.human_write_title = Some(self.last_title.clone());
                    self.view = None;
                    self.titles = None;
                    self.phase = Phase::PreviewWrite;
                } else if Instant::now() > self.deadline {
                    self.reasons
                        .push(format!("human write title: {}", self.last_title));
                    self.phase = Phase::Finish;
                }
            }
            Phase::PreviewWrite => {
                if self.view.is_none() {
                    if let Err(error) = self.create_view(
                        &self.preview_dir.clone(),
                        format!(
                            "{}profile?key=pv&value=preview-marker&read=hb",
                            self.base_url
                        ),
                    ) {
                        self.reasons.push(error);
                        self.phase = Phase::Finish;
                        return false;
                    }
                }
                self.drain_title();
                if self.last_title == "stored:preview-marker|empty" {
                    self.preview_write_title = Some(self.last_title.clone());
                    self.view = None;
                    self.titles = None;
                    self.phase = Phase::HumanRead;
                } else if Instant::now() > self.deadline {
                    self.reasons
                        .push(format!("preview write title: {}", self.last_title));
                    self.phase = Phase::Finish;
                }
            }
            Phase::HumanRead => {
                if self.view.is_none() {
                    if let Err(error) = self.create_view(
                        &self.human_dir.clone(),
                        format!("{}profile?key=hb&read2=pv", self.base_url),
                    ) {
                        self.reasons.push(error);
                        self.phase = Phase::Finish;
                        return false;
                    }
                }
                self.drain_title();
                if self.last_title == "read:human-marker|empty" {
                    self.human_read_title = Some(self.last_title.clone());
                    self.view = None;
                    self.titles = None;
                    self.phase = Phase::Finish;
                } else if Instant::now() > self.deadline {
                    self.reasons
                        .push(format!("human read title: {}", self.last_title));
                    self.phase = Phase::Finish;
                }
            }
            Phase::Finish => {
                self.view = None;
                self.titles = None;
                let size_bytes = directory_size_bytes(&self.probe_root);
                let _ = std::fs::remove_dir_all(&self.probe_root);
                let removed = !self.probe_root.exists();
                let isolation = self.human_write_title.as_deref()
                    == Some("stored:human-marker|empty")
                    && self.preview_write_title.as_deref() == Some("stored:preview-marker|empty");
                let persistence =
                    self.human_read_title.as_deref() == Some("read:human-marker|empty");
                if !isolation {
                    self.reasons.push("isolation check failed".to_string());
                }
                if !persistence {
                    self.reasons.push("persistence check failed".to_string());
                }
                if !self.app_data_ok {
                    self.reasons
                        .push("profile directories are not BrainRoot-owned".to_string());
                }
                if !removed {
                    self.reasons
                        .push("probe directory was not removed".to_string());
                }
                let report = json!({
                    "decision": if self.reasons.is_empty() { "go" } else { "no-go" },
                    "isolation": isolation,
                    "persistence": persistence,
                    "within_app_data": self.app_data_ok,
                    "probe_dir_size_bytes": size_bytes,
                    "probe_dir_removed": removed,
                    "human_write_title": self.human_write_title,
                    "preview_write_title": self.preview_write_title,
                    "human_read_after_reopen": self.human_read_title,
                    "reasons": self.reasons,
                });
                println!("brainroot: human profile probe {report}");
                let _ = std::io::stdout().flush();
                self.stop_flag.store(true, Ordering::Relaxed);
                app.exit(0);
                return true;
            }
        }
        false
    }
}

thread_local! {
    static HOST: RefCell<Option<ProbeHost>> = const { RefCell::new(None) };
}

struct ProbeHost {
    vbox: gtk::Box,
}

/// The probe installs its host container from `setup` on the main thread.
pub fn install_host(window: &tauri::Window) -> Result<(), String> {
    let gtk_window = window.gtk_window().map_err(|error| error.to_string())?;
    let child = gtk_window.child().ok_or("window has no GTK child")?;
    let vbox = child
        .downcast::<gtk::Box>()
        .map_err(|_| "window child is not a gtk::Box".to_string())?;
    HOST.with(|slot| *slot.borrow_mut() = Some(ProbeHost { vbox }));
    Ok(())
}
