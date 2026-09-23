//! Disposable Linux child-WebView lifecycle probe (B08-S01).
//!
//! Built only with `--features probe`, which enables `tauri/unstable`; the
//! shipped application never includes this binary. The probe serves a local
//! fixture from `127.0.0.1` on an OS-assigned port, drives child-WebView
//! lifecycle phases, prints one bounded JSON report, and exits.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::webview::{PageLoadEvent, WebviewBuilder};
use tauri::window::WindowBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl, Window, Wry};

const HOST_WINDOW: &str = "probe-host";
const LOAD_TIMEOUT: Duration = Duration::from_secs(10);
const TITLE_SETTLE: Duration = Duration::from_millis(400);
const CLOSE_SETTLE: Duration = Duration::from_millis(50);
const SAMPLE_EVERY: usize = 10;
const CHILD_WIDTH: f64 = 480.0;
const CHILD_HEIGHT: f64 = 320.0;

fn main() {
    let cycles = arg_cycles();
    let fixture = Fixture::start();
    let fixture_port = fixture.port();
    let base_url = fixture.base_url();
    let data_root =
        std::env::temp_dir().join(format!("brainroot-webview-probe-{}", std::process::id()));

    tauri::Builder::default()
        .setup(move |app| {
            WindowBuilder::new(app, HOST_WINDOW)
                .title("BrainRoot WebView probe")
                .inner_size(720.0, 480.0)
                .build()?;
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.close();
            }
            let handle = app.handle().clone();
            let data_root = data_root.clone();
            thread::spawn(move || {
                let report = run_probe(&handle, &base_url, fixture_port, &data_root, cycles);
                println!(
                    "{}",
                    serde_json::to_string(&report).expect("probe report serializes")
                );
                fixture.stop();
                let _ = std::fs::remove_dir_all(&data_root);
                handle.exit(0);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("webview probe failed to run");
}

fn arg_cycles() -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|argument| argument == "--cycles")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(100)
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

    fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
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
    let (status, body) = if path.starts_with("/iso") {
        (200, isolation_page(path))
    } else if path == "/" {
        (
            200,
            "<!doctype html><title>probe-fixture</title><p>brainroot-probe</p>".to_string(),
        )
    } else if path.starts_with("/cycle")
        || path.starts_with("/resize")
        || path.starts_with("/crash")
    {
        (
            200,
            format!("<!doctype html><title>{path}</title><p>brainroot-probe</p>"),
        )
    } else {
        (
            404,
            "<!doctype html><title>probe-missing</title>".to_string(),
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

fn isolation_page(_path: &str) -> String {
    "<!doctype html><title>iso</title><script>\
     const params = new URLSearchParams(location.search);\
     const key = params.get('key') || 'probe';\
     if (params.get('op') === 'write') {\
       localStorage.setItem(key, params.get('value') || 'alpha');\
       document.title = 'stored:' + localStorage.getItem(key);\
     } else {\
       document.title = 'read:' + (localStorage.getItem(key) || 'empty');\
     }\
     </script>"
        .to_string()
}

struct Child {
    webview: Webview<Wry>,
    label: String,
    create_ms: u64,
    load_ms: u64,
    titles: Receiver<String>,
}

fn create_child(
    window: &Window,
    label: &str,
    url: String,
    data_directory: Option<PathBuf>,
    fixture_port: u16,
) -> Result<Child, String> {
    let parsed = url
        .parse()
        .map_err(|error| format!("invalid fixture url for {label}: {error}"))?;
    let (load_sender, load_receiver) = mpsc::channel::<String>();
    let (title_sender, title_receiver) = mpsc::channel::<String>();
    let mut builder = WebviewBuilder::new(label, WebviewUrl::External(parsed))
        .on_page_load(move |_webview, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished) {
                let _ = load_sender.send(payload.url().to_string());
            }
        })
        .on_document_title_changed(move |_webview, title| {
            let _ = title_sender.send(title);
        })
        .on_navigation(move |navigation_url| {
            navigation_url.scheme() == "http"
                && navigation_url.host_str() == Some("127.0.0.1")
                && navigation_url.port() == Some(fixture_port)
        });
    if let Some(data_directory) = data_directory {
        builder = builder.data_directory(data_directory);
    }
    let started = Instant::now();
    let webview = window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(CHILD_WIDTH, CHILD_HEIGHT),
        )
        .map_err(|error| format!("create {label}: {error}"))?;
    let create_ms = started.elapsed().as_millis() as u64;
    load_receiver
        .recv_timeout(LOAD_TIMEOUT)
        .map_err(|_| format!("load timeout for {label}"))?;
    let load_ms = started.elapsed().as_millis() as u64;
    Ok(Child {
        webview,
        label: label.to_string(),
        create_ms,
        load_ms,
        titles: title_receiver,
    })
}

fn destroy_child(child: &Child) -> Result<u64, String> {
    let started = Instant::now();
    child
        .webview
        .close()
        .map_err(|error| format!("close {}: {error}", child.label))?;
    thread::sleep(CLOSE_SETTLE);
    Ok(started.elapsed().as_millis() as u64)
}

fn wait_title(child: &Child) -> String {
    let mut last = String::new();
    while let Ok(title) = child.titles.recv_timeout(TITLE_SETTLE) {
        if !title.is_empty() {
            last = title;
        }
    }
    last
}

fn navigation_denial(webview: &Webview<Wry>) -> Value {
    let attempted = "file:///etc/passwd"
        .parse()
        .ok()
        .map(|file_url| webview.navigate(file_url).is_ok())
        .unwrap_or(false);
    thread::sleep(Duration::from_millis(200));
    let current = webview.url().map(|url| url.to_string()).unwrap_or_default();
    json!({
        "attempted": attempted,
        "current_url": current,
        "still_fixture": current.contains("127.0.0.1"),
    })
}

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

fn stats(values: &mut [u64]) -> Value {
    if values.is_empty() {
        return json!({"samples": 0});
    }
    values.sort_unstable();
    json!({
        "samples": values.len(),
        "min": values[0],
        "p50": values[values.len() / 2],
        "max": values[values.len() - 1],
    })
}

fn environment() -> Value {
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let webkitgtk = std::process::Command::new("pkg-config")
        .args(["--modversion", "webkit2gtk-4.1"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    json!({
        "kernel": kernel,
        "session_type": std::env::var("XDG_SESSION_TYPE").ok(),
        "wayland_display": std::env::var("WAYLAND_DISPLAY").ok(),
        "webkitgtk": webkitgtk,
        "probe_version": env!("CARGO_PKG_VERSION"),
    })
}

fn resize_phase(window: &Window, base_url: &str, fixture_port: u16) -> Value {
    match create_child(
        window,
        "probe-resize",
        format!("{base_url}resize"),
        None,
        fixture_port,
    ) {
        Ok(child) => {
            let physical_before = child
                .webview
                .size()
                .map(|size| (size.width, size.height))
                .unwrap_or((0, 0));
            let set_error = child
                .webview
                .set_size(LogicalSize::new(640.0, 200.0))
                .err()
                .map(|error| error.to_string());
            thread::sleep(Duration::from_millis(300));
            let physical_after = child
                .webview
                .size()
                .map(|size| (size.width, size.height))
                .unwrap_or((0, 0));
            let scale_factor = window.scale_factor().unwrap_or(1.0);
            let resized = physical_after.0 > physical_before.0;
            let _ = destroy_child(&child);
            json!({
                "created": true,
                "logical_before": [CHILD_WIDTH, CHILD_HEIGHT],
                "logical_set": [640.0, 200.0],
                "physical_before": physical_before,
                "physical_after": physical_after,
                "scale_factor": scale_factor,
                "set_error": set_error,
                "resized": resized,
            })
        }
        Err(error) => json!({"created": false, "error": error, "resized": false}),
    }
}

fn isolation_phase(window: &Window, base_url: &str, fixture_port: u16, data_root: &Path) -> Value {
    let mut notes = Vec::new();
    let write = create_child(
        window,
        "probe-iso-a",
        format!("{base_url}iso?op=write&value=alpha"),
        Some(data_root.join("a")),
        fixture_port,
    );
    let stored = match &write {
        Ok(child) => {
            let title = wait_title(child);
            let result = destroy_child(child);
            if let Err(error) = result {
                notes.push(error);
            }
            title
        }
        Err(error) => {
            notes.push(error.clone());
            format!("error:{error}")
        }
    };
    let other = create_child(
        window,
        "probe-iso-b",
        format!("{base_url}iso?op=read"),
        Some(data_root.join("b")),
        fixture_port,
    );
    let other_read = match &other {
        Ok(child) => {
            let title = wait_title(child);
            if let Err(error) = destroy_child(child) {
                notes.push(error);
            }
            title
        }
        Err(error) => {
            notes.push(error.clone());
            format!("error:{error}")
        }
    };
    let again = create_child(
        window,
        "probe-iso-a2",
        format!("{base_url}iso?op=read"),
        Some(data_root.join("a")),
        fixture_port,
    );
    let again_read = match &again {
        Ok(child) => {
            let title = wait_title(child);
            if let Err(error) = destroy_child(child) {
                notes.push(error);
            }
            title
        }
        Err(error) => {
            notes.push(error.clone());
            format!("error:{error}")
        }
    };
    json!({
        "stored_title": stored,
        "fresh_profile_title": other_read,
        "persisted_title": again_read,
        "isolated": other_read == "read:empty",
        "persisted": again_read == "read:alpha",
        "notes": notes,
    })
}

fn layout_phase(window: &Window, base_url: &str, fixture_port: u16) -> Value {
    let first = create_child(
        window,
        "probe-layout-1",
        format!("{base_url}layout?slot=1"),
        None,
        fixture_port,
    );
    let second = create_child(
        window,
        "probe-layout-2",
        format!("{base_url}layout?slot=2"),
        None,
        fixture_port,
    );
    let mut report = json!({
        "created": [first.is_ok(), second.is_ok()],
    });
    if let (Ok(first), Ok(second)) = (&first, &second) {
        let first_size = first
            .webview
            .size()
            .map(|size| (size.width, size.height))
            .unwrap_or((0, 0));
        let second_size = second
            .webview
            .size()
            .map(|size| (size.width, size.height))
            .unwrap_or((0, 0));
        let first_position = first
            .webview
            .position()
            .map(|position| (position.x, position.y))
            .unwrap_or((0, 0));
        let second_position = second
            .webview
            .position()
            .map(|position| (position.x, position.y))
            .unwrap_or((0, 0));
        let window_size = window
            .inner_size()
            .map(|size| (size.width, size.height))
            .unwrap_or((0, 0));
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let requested_physical = (
            (CHILD_WIDTH * scale_factor) as u32,
            (CHILD_HEIGHT * scale_factor) as u32,
        );
        let geometry_honored =
            first_size == requested_physical && second_size == requested_physical;
        report = json!({
            "created": [true, true],
            "window_physical_size": window_size,
            "requested_logical": [CHILD_WIDTH, CHILD_HEIGHT],
            "requested_physical": requested_physical,
            "scale_factor": scale_factor,
            "first_size": first_size,
            "second_size": second_size,
            "first_position": first_position,
            "second_position": second_position,
            "both_full_window": first_size == window_size && second_size == window_size,
            "geometry_honored": geometry_honored,
        });
    }
    if let Ok(child) = &first {
        let _ = destroy_child(child);
    }
    if let Ok(child) = &second {
        let _ = destroy_child(child);
    }
    report
}

fn crash_recovery_phase(window: &Window, base_url: &str, fixture_port: u16) -> Value {
    let first = create_child(
        window,
        "probe-crash-1",
        format!("{base_url}crash"),
        None,
        fixture_port,
    );
    let (forced_close, navigation) = match &first {
        Ok(child) => {
            let navigation = navigation_denial(&child.webview);
            let forced_close = destroy_child(child).is_ok();
            (forced_close, navigation)
        }
        Err(_) => (false, json!({"attempted": false, "still_fixture": false})),
    };
    let started = Instant::now();
    let recovered = match create_child(
        window,
        "probe-crash-2",
        format!("{base_url}crash"),
        None,
        fixture_port,
    ) {
        Ok(child) => {
            let reload_ms = started.elapsed().as_millis() as u64;
            let closed = destroy_child(&child).is_ok();
            json!({"recovered": closed, "reload_ms": reload_ms})
        }
        Err(error) => json!({"recovered": false, "error": error}),
    };
    json!({
        "first_created": first.is_ok(),
        "forced_close": forced_close,
        "navigation_denial": navigation,
        "recovered": recovered["recovered"],
        "reload_ms": recovered["reload_ms"],
    })
}

fn run_probe(
    handle: &AppHandle,
    base_url: &str,
    fixture_port: u16,
    data_root: &Path,
    cycles: usize,
) -> Value {
    let Some(window) = handle.get_window(HOST_WINDOW) else {
        return json!({"decision": "no-go", "reasons": ["host window missing"]});
    };
    let baseline_rss = rss_kb();
    let baseline_children = child_process_count();

    let mut completed = 0_usize;
    let mut failures: Vec<String> = Vec::new();
    let mut create_ms = Vec::new();
    let mut load_ms = Vec::new();
    let mut destroy_ms = Vec::new();
    let mut samples = Vec::new();
    for index in 0..cycles {
        let label = format!("probe-cycle-{index}");
        match create_child(
            &window,
            &label,
            format!("{base_url}cycle?index={index}"),
            None,
            fixture_port,
        ) {
            Ok(child) => {
                create_ms.push(child.create_ms);
                load_ms.push(child.load_ms);
                match destroy_child(&child) {
                    Ok(destroyed_ms) => {
                        destroy_ms.push(destroyed_ms);
                        completed += 1;
                    }
                    Err(error) => failures.push(error),
                }
            }
            Err(error) => failures.push(error),
        }
        if (index + 1) % SAMPLE_EVERY == 0 {
            samples.push(json!({
                "cycle": index + 1,
                "rss_kb": rss_kb(),
                "children": child_process_count(),
            }));
        }
    }
    thread::sleep(Duration::from_millis(500));
    let children_after = child_process_count();

    let resize = resize_phase(&window, base_url, fixture_port);
    let layout = layout_phase(&window, base_url, fixture_port);
    let isolation = isolation_phase(&window, base_url, fixture_port, data_root);
    let crash = crash_recovery_phase(&window, base_url, fixture_port);

    let cycles_ok = completed == cycles && failures.is_empty();
    let resize_ok = resize["resized"].as_bool().unwrap_or(false);
    let layout_ok = layout["geometry_honored"].as_bool().unwrap_or(false);
    let isolation_ok = isolation["isolated"].as_bool().unwrap_or(false);
    let persisted_ok = isolation["persisted"].as_bool().unwrap_or(false);
    let crash_ok = crash["recovered"].as_bool().unwrap_or(false);
    let denial_ok = crash["navigation_denial"]["still_fixture"]
        .as_bool()
        .unwrap_or(false);

    let mut reasons = Vec::new();
    if !cycles_ok {
        reasons.push(format!(
            "{completed}/{cycles} create/load/destroy cycles completed"
        ));
    }
    if !resize_ok {
        reasons.push("resize was not observed on the child webview".to_string());
    }
    if !layout_ok {
        reasons
            .push("child webviews ignore the requested geometry and fill the window".to_string());
    }
    if !isolation_ok {
        reasons.push("fresh profile read the other profile storage".to_string());
    }
    if !persisted_ok {
        reasons.push("persisted profile did not retain its own storage".to_string());
    }
    if !crash_ok {
        reasons.push("forced-close recovery failed".to_string());
    }
    if !denial_ok {
        reasons.push("file navigation was not denied".to_string());
    }
    if children_after > 0 {
        reasons.push(format!(
            "{children_after} shared WebKit process(es) remained after all views closed"
        ));
    }

    json!({
        "environment": environment(),
        "cycles_requested": cycles,
        "baseline": {"rss_kb": baseline_rss, "children": baseline_children},
        "phase_create_destroy": {
            "completed": completed,
            "failed": failures,
            "create_ms": stats(&mut create_ms),
            "load_ms": stats(&mut load_ms),
            "destroy_ms": stats(&mut destroy_ms),
            "samples": samples,
        },
        "phase_resize": resize,
        "phase_layout": layout,
        "phase_isolation": isolation,
        "phase_crash_recovery": crash,
        "cleanup": {"children_after_phases": children_after},
        "decision": if reasons.is_empty() { "go" } else { "no-go" },
        "reasons": reasons,
    })
}
