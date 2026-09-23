//! Disposable probe for GtkFixed preview hosting inside the Tauri window (B09-S01).
//!
//! Built only with `--features probe`. It answers one question on the reference
//! environment: can a `wry` WebView hosted in a `GtkFixed` overlay inside the
//! Tauri window be created at exact bounds, resized, navigated under an
//! allowlist, and destroyed cleanly, without breaking the shell webview or the
//! default build? The shipped application never includes this binary.

use std::cell::{Cell, RefCell};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use serde_json::{json, Value};
use tauri::Manager;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{
    PageLoadEvent, Rect, WebContext, WebView, WebViewBuilder, WebViewBuilderExtUnix, WebViewExtUnix,
};

const TICK: Duration = Duration::from_millis(50);
const PHASE_TIMEOUT: Duration = Duration::from_secs(10);
const SETTLE: Duration = Duration::from_millis(300);

fn main() {
    let fixture = Fixture::start();
    let port = fixture.port();
    let base_url = fixture.base_url();
    let stop_flag = fixture.stop_flag();
    let profile_dir =
        std::env::temp_dir().join(format!("brainroot-hosting-probe-{}", std::process::id()));

    tauri::Builder::default()
        .setup(move |app| {
            let window = app.get_window("main").ok_or("main window missing")?;
            let gtk_window = window.gtk_window()?;
            let probe = Probe::new(gtk_window, base_url, port, profile_dir, stop_flag)?;
            let state = Rc::new(RefCell::new(probe));
            glib::timeout_add_local(TICK, move || {
                let mut probe = state.borrow_mut();
                if probe.step() {
                    ControlFlow::Break
                } else {
                    ControlFlow::Continue
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("preview hosting probe failed to run");
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    }
}

fn allocation_of<W: IsA<gtk::Widget>>(widget: &W) -> (i32, i32, i32, i32) {
    let (rectangle, _baseline) = widget.allocated_size();
    (
        rectangle.x(),
        rectangle.y(),
        rectangle.width(),
        rectangle.height(),
    )
}

fn environment() -> Value {
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    json!({
        "kernel": kernel,
        "session_type": std::env::var("XDG_SESSION_TYPE").ok(),
        "wayland_display": std::env::var("WAYLAND_DISPLAY").ok(),
        "software": "tauri 2.11.6, wry 0.55.1, gtk 0.18.2",
    })
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

    fn port(&self) -> u16 {
        self.addr.port()
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
    let (status, body) = if path == "/" {
        (
            200,
            "<!doctype html><title>hosting-fixture</title><p>brainroot-hosting-probe</p>"
                .to_string(),
        )
    } else {
        (
            404,
            "<!doctype html><title>hosting-missing</title>".to_string(),
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

#[derive(Clone, Copy)]
enum Phase {
    CreateFirst,
    WaitFirstLoad,
    CreateSecond,
    WaitSecondLoad,
    Geometry,
    Resize,
    WaitResize,
    WaitDenial,
    ShellCheck,
    Cleanup,
    Finished,
}

struct Probe {
    gtk_window: gtk::ApplicationWindow,
    overlay: gtk::Overlay,
    vbox: gtk::Box,
    fixed: gtk::Fixed,
    context: Option<WebContext>,
    base_url: String,
    port: u16,
    profile_dir: PathBuf,
    stop_flag: Arc<AtomicBool>,
    loads: Rc<RefCell<Vec<String>>>,
    denials: Rc<Cell<usize>>,
    nav_seen: Rc<RefCell<Vec<String>>>,
    views: Vec<WebView>,
    phase: Phase,
    deadline: Instant,
    started: Instant,
    settle_until: Instant,
    shell_before: (i32, i32, i32, i32),
    reasons: Vec<String>,
    result: serde_json::Map<String, Value>,
}

const FIRST: (f64, f64, f64, f64) = (0.0, 0.0, 320.0, 200.0);
const SECOND: (f64, f64, f64, f64) = (340.0, 0.0, 200.0, 300.0);
const RESIZED: (f64, f64, f64, f64) = (60.0, 30.0, 240.0, 180.0);

impl Probe {
    fn new(
        gtk_window: gtk::ApplicationWindow,
        base_url: String,
        port: u16,
        profile_dir: PathBuf,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let child = gtk_window.child().ok_or("window has no GTK child")?;
        let vbox = child
            .downcast::<gtk::Box>()
            .map_err(|_| "window child is not a gtk::Box".to_string())?;
        let shell_before = (0, 0, 0, 0);
        let overlay = gtk::Overlay::new();
        gtk_window.remove(&vbox);
        overlay.add(&vbox);
        let fixed = gtk::Fixed::new();
        overlay.add_overlay(&fixed);
        gtk_window.add(&overlay);
        gtk_window.show_all();
        let context = WebContext::new(Some(profile_dir.clone()));
        Ok(Self {
            gtk_window,
            overlay,
            vbox,
            fixed,
            context: Some(context),
            base_url,
            port,
            profile_dir,
            stop_flag,
            loads: Rc::new(RefCell::new(Vec::new())),
            denials: Rc::new(Cell::new(0)),
            nav_seen: Rc::new(RefCell::new(Vec::new())),
            views: Vec::new(),
            phase: Phase::CreateFirst,
            deadline: Instant::now() + PHASE_TIMEOUT,
            started: Instant::now(),
            settle_until: Instant::now(),
            shell_before,
            reasons: Vec::new(),
            result: serde_json::Map::new(),
        })
    }

    fn create_view(&mut self, label: &str, bounds: Rect) -> Result<(), String> {
        let loads = Rc::clone(&self.loads);
        let denials = Rc::clone(&self.denials);
        let nav_seen = Rc::clone(&self.nav_seen);
        let allow_prefix = self.base_url.trim_end_matches('/').to_string();
        let context = self.context.as_mut().ok_or("missing web context")?;
        let builder = WebViewBuilder::new_with_web_context(context)
            .with_url(self.base_url.clone())
            .with_bounds(bounds)
            .with_navigation_handler(move |url| {
                if nav_seen.borrow().len() < 8 {
                    nav_seen.borrow_mut().push(url.clone());
                }
                let allowed = url.starts_with(&allow_prefix);
                if !allowed {
                    denials.set(denials.get() + 1);
                }
                allowed
            })
            .with_on_page_load_handler(move |event, url| {
                if matches!(event, PageLoadEvent::Finished) {
                    loads.borrow_mut().push(url);
                }
            });
        let view = builder
            .build_gtk(&self.fixed)
            .map_err(|error| format!("{label}: {error}"))?;
        self.views.push(view);
        Ok(())
    }

    fn view_allocation(&self, index: usize) -> (i32, i32, i32, i32) {
        match self.views.get(index) {
            Some(view) => allocation_of(&view.webview()),
            None => (-1, -1, -1, -1),
        }
    }

    fn fail(&mut self, phase: &str, reason: String) {
        self.reasons.push(format!("{phase}: {reason}"));
    }

    fn step(&mut self) -> bool {
        match self.phase {
            Phase::CreateFirst => {
                self.shell_before = allocation_of(&self.vbox);
                match self.create_view("hosting-first", rect(FIRST.0, FIRST.1, FIRST.2, FIRST.3)) {
                    Ok(()) => {
                        self.result.insert(
                            "create_ms".into(),
                            json!(self.started.elapsed().as_millis() as u64),
                        );
                        self.phase = Phase::WaitFirstLoad;
                        self.deadline = Instant::now() + PHASE_TIMEOUT;
                    }
                    Err(error) => {
                        self.fail("create_first", error);
                        self.phase = Phase::Cleanup;
                        self.settle_until = Instant::now() + SETTLE;
                    }
                }
            }
            Phase::WaitFirstLoad => {
                if !self.loads.borrow().is_empty() {
                    self.result.insert(
                        "load_ms".into(),
                        json!(self.started.elapsed().as_millis() as u64),
                    );
                    self.result.insert("first_requested".into(), json!(FIRST));
                    self.phase = Phase::CreateSecond;
                } else if Instant::now() > self.deadline {
                    self.result.insert(
                        "first_url".into(),
                        json!(self
                            .views
                            .first()
                            .and_then(|view| view.url().ok())
                            .unwrap_or_default()),
                    );
                    self.result
                        .insert("denied_navigations".into(), json!(self.denials.get()));
                    self.result
                        .insert("nav_seen".into(), json!(self.nav_seen.borrow().clone()));
                    self.result
                        .insert("loads_len".into(), json!(self.loads.borrow().len()));
                    self.fail("wait_first_load", "timeout".to_string());
                    self.phase = Phase::Cleanup;
                    self.settle_until = Instant::now() + SETTLE;
                }
            }
            Phase::CreateSecond => {
                match self.create_view(
                    "hosting-second",
                    rect(SECOND.0, SECOND.1, SECOND.2, SECOND.3),
                ) {
                    Ok(()) => {
                        self.result.insert("second_requested".into(), json!(SECOND));
                        self.phase = Phase::WaitSecondLoad;
                        self.deadline = Instant::now() + PHASE_TIMEOUT;
                    }
                    Err(error) => {
                        self.fail("create_second", error);
                        self.phase = Phase::Cleanup;
                        self.settle_until = Instant::now() + SETTLE;
                    }
                }
            }
            Phase::WaitSecondLoad => {
                if self.loads.borrow().len() >= 2 {
                    self.phase = Phase::Geometry;
                    self.settle_until = Instant::now() + SETTLE;
                } else if Instant::now() > self.deadline {
                    self.fail("wait_second_load", "timeout".to_string());
                    self.phase = Phase::Cleanup;
                    self.settle_until = Instant::now() + SETTLE;
                }
            }
            Phase::Geometry => {
                if Instant::now() < self.settle_until {
                    return false;
                }
                let first = self.view_allocation(0);
                let second = self.view_allocation(1);
                self.result.insert("first_allocation".into(), json!(first));
                self.result
                    .insert("second_allocation".into(), json!(second));
                let first_ok = first == tuple_i32(FIRST);
                let second_ok = second == tuple_i32(SECOND);
                self.result
                    .insert("first_geometry_match".into(), json!(first_ok));
                self.result
                    .insert("second_geometry_match".into(), json!(second_ok));
                if !first_ok || !second_ok {
                    self.fail(
                        "geometry",
                        format!("requested {FIRST:?}/{SECOND:?}, got {first:?}/{second:?}"),
                    );
                }
                let wry_bounds = self
                    .views
                    .first()
                    .and_then(|view| view.bounds().ok())
                    .map(|bounds| {
                        json!({
                            "position": [bounds.position.to_logical::<f64>(1.0).x, bounds.position.to_logical::<f64>(1.0).y],
                            "size": [bounds.size.to_logical::<f64>(1.0).width, bounds.size.to_logical::<f64>(1.0).height],
                        })
                    })
                    .unwrap_or(json!(null));
                self.result.insert("wry_bounds_first".into(), wry_bounds);
                self.phase = Phase::Resize;
                self.settle_until = Instant::now() + SETTLE;
            }
            Phase::Resize => {
                if Instant::now() < self.settle_until {
                    return false;
                }
                let target = rect(RESIZED.0, RESIZED.1, RESIZED.2, RESIZED.3);
                let outcome = self
                    .views
                    .first()
                    .ok_or("first view missing".to_string())
                    .and_then(|view| view.set_bounds(target).map_err(|error| error.to_string()));
                match outcome {
                    Ok(()) => {
                        self.result.insert("resize_error".into(), json!(null));
                    }
                    Err(error) => {
                        self.fail("resize", error.clone());
                        self.result.insert("resize_error".into(), json!(error));
                    }
                }
                self.result
                    .insert("resize_requested".into(), json!(RESIZED));
                self.phase = Phase::WaitResize;
                self.settle_until = Instant::now() + SETTLE;
            }
            Phase::WaitResize => {
                if Instant::now() < self.settle_until {
                    return false;
                }
                let resized = self.view_allocation(0);
                self.result
                    .insert("resize_allocation".into(), json!(resized));
                let resized_ok = resized == tuple_i32(RESIZED);
                self.result
                    .insert("resize_geometry_match".into(), json!(resized_ok));
                if !resized_ok {
                    self.fail("resize", format!("requested {RESIZED:?}, got {resized:?}"));
                }
                let denial = self
                    .views
                    .first()
                    .ok_or("first view missing".to_string())
                    .and_then(|view| {
                        view.load_url("file:///etc/passwd")
                            .map_err(|error| error.to_string())
                    });
                if let Err(error) = denial {
                    self.fail("denial_request", error);
                }
                self.phase = Phase::WaitDenial;
                self.settle_until = Instant::now() + SETTLE;
            }
            Phase::WaitDenial => {
                if Instant::now() < self.settle_until {
                    return false;
                }
                let current = self
                    .views
                    .first()
                    .and_then(|view| view.url().ok())
                    .unwrap_or_default();
                let still_fixture = current.starts_with(&self.base_url);
                self.result
                    .insert("url_after_denial".into(), json!(current));
                self.result
                    .insert("denied_navigations".into(), json!(self.denials.get()));
                self.result
                    .insert("navigation_denied".into(), json!(still_fixture));
                if !still_fixture {
                    self.fail("denial", format!("loaded {current}"));
                }
                self.phase = Phase::ShellCheck;
            }
            Phase::ShellCheck => {
                let shell_after = allocation_of(&self.vbox);
                self.result
                    .insert("shell_allocation_after".into(), json!(shell_after));
                let unchanged = shell_after == self.shell_before;
                self.result
                    .insert("shell_unchanged".into(), json!(unchanged));
                if !unchanged {
                    self.fail(
                        "shell",
                        format!("before {:?}, after {shell_after:?}", self.shell_before),
                    );
                }
                self.phase = Phase::Cleanup;
                self.settle_until = Instant::now() + SETTLE;
            }
            Phase::Cleanup => {
                if Instant::now() < self.settle_until {
                    return false;
                }
                self.views.clear();
                let fixed_children = self.fixed.children().len();
                self.result
                    .insert("fixed_children_after_drop".into(), json!(fixed_children));
                if fixed_children != 0 {
                    self.fail("cleanup", format!("{fixed_children} children remained"));
                }
                self.overlay.remove(&self.fixed);
                self.overlay.remove(&self.vbox);
                self.gtk_window.remove(&self.overlay);
                self.gtk_window.add(&self.vbox);
                self.context = None;
                let _ = std::fs::remove_dir_all(&self.profile_dir);
                self.phase = Phase::Finished;
            }
            Phase::Finished => {
                self.result.insert(
                    "decision".into(),
                    json!(if self.reasons.is_empty() {
                        "go"
                    } else {
                        "no-go"
                    }),
                );
                self.result
                    .insert("reasons".into(), json!(self.reasons.clone()));
                let report = json!({
                    "environment": environment(),
                    "probe": "preview_hosting",
                    "result": Value::Object(self.result.clone()),
                });
                println!("{}", serde_json::to_string(&report).expect("report json"));
                let _ = std::io::stdout().flush();
                self.stop_flag.store(true, Ordering::Relaxed);
                std::process::exit(0);
            }
        }
        false
    }
}

fn tuple_i32(source: (f64, f64, f64, f64)) -> (i32, i32, i32, i32) {
    (
        source.0 as i32,
        source.1 as i32,
        source.2 as i32,
        source.3 as i32,
    )
}
