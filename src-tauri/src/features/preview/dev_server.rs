//! Owned dev-server lifecycle per ADR 0011.
//!
//! One supervisor per process: it assigns a loopback port, spawns the declared
//! command in its own process group, waits for readiness with a bounded
//! timeout, bounds the child output, and stops the whole group with SIGTERM,
//! a bounded grace, and SIGKILL. It never signals a process it did not spawn
//! and never restarts on its own.

use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const READINESS_MIN_MS: u64 = 1_000;
const READINESS_MAX_MS: u64 = 60_000;
const READINESS_DEFAULT_MS: u64 = 30_000;
const GRACE_MIN_MS: u64 = 200;
const GRACE_MAX_MS: u64 = 10_000;
const GRACE_DEFAULT_MS: u64 = 5_000;
const POLL: Duration = Duration::from_millis(100);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(250);
const OUTPUT_CAP_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewPhase {
    Idle,
    Starting,
    Ready,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewStatus {
    pub phase: PreviewPhase,
    pub port: Option<u16>,
    pub reason: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewStartResponse {
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewStopResponse {
    pub stopped: bool,
}

#[derive(Debug, Deserialize)]
pub struct PreviewStartRequest {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: String,
    #[serde(default)]
    pub readiness_timeout_ms: Option<u64>,
    #[serde(default)]
    pub stop_grace_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewError {
    pub code: String,
    pub message: String,
}

impl PreviewError {
    pub(crate) fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug)]
struct BoundedBuffer {
    bytes: Vec<u8>,
    total: usize,
}

impl Default for BoundedBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundedBuffer {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            total: 0,
        }
    }

    fn push(&mut self, data: &[u8]) {
        self.total += data.len();
        self.bytes.extend_from_slice(data);
        if self.bytes.len() > OUTPUT_CAP_BYTES {
            let excess = self.bytes.len() - OUTPUT_CAP_BYTES;
            self.bytes.drain(0..excess);
        }
    }
}

#[derive(Debug, Default)]
struct CapturedOutput {
    stdout: BoundedBuffer,
    stderr: BoundedBuffer,
}

impl CapturedOutput {
    fn new() -> Self {
        Self {
            stdout: BoundedBuffer::new(),
            stderr: BoundedBuffer::new(),
        }
    }
}

enum Stream {
    Stdout,
    Stderr,
}

struct Inner {
    phase: PreviewPhase,
    port: Option<u16>,
    reason: Option<String>,
    exit_code: Option<i32>,
    child: Option<Child>,
    pgid: Option<i32>,
    grace_ms: u64,
}

pub struct DevServerSupervisor {
    inner: Arc<Mutex<Inner>>,
    output: Arc<Mutex<CapturedOutput>>,
}

impl Default for DevServerSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl DevServerSupervisor {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                phase: PreviewPhase::Idle,
                port: None,
                reason: None,
                exit_code: None,
                child: None,
                pgid: None,
                grace_ms: GRACE_DEFAULT_MS,
            })),
            output: Arc::new(Mutex::new(CapturedOutput::new())),
        }
    }

    pub fn start(
        &self,
        request: PreviewStartRequest,
    ) -> Result<PreviewStartResponse, PreviewError> {
        let command = request.command.trim().to_string();
        if command.is_empty() {
            return Err(PreviewError::new(
                "preview_command_missing",
                "Choose a command to start the preview.",
            ));
        }
        let cwd = PathBuf::from(request.cwd.trim());
        if !cwd.is_absolute() {
            return Err(PreviewError::new(
                "preview_cwd_invalid",
                "Choose an absolute project folder for the preview.",
            ));
        }
        let readiness_ms = clamp(
            request.readiness_timeout_ms.unwrap_or(READINESS_DEFAULT_MS),
            READINESS_MIN_MS,
            READINESS_MAX_MS,
        );
        let grace_ms = clamp(
            request.stop_grace_ms.unwrap_or(GRACE_DEFAULT_MS),
            GRACE_MIN_MS,
            GRACE_MAX_MS,
        );

        let port = free_loopback_port()?;
        let mut inner = self.inner.lock().expect("preview state");
        if matches!(inner.phase, PreviewPhase::Starting | PreviewPhase::Ready) {
            return Err(PreviewError::new(
                "preview_already_active",
                "A preview is already running. Stop it before starting another.",
            ));
        }
        let mut command_builder = Command::new(&command);
        command_builder
            .args(&request.args)
            .current_dir(&cwd)
            .env("PORT", port.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command_builder.process_group(0);
        }
        let mut child = command_builder.spawn().map_err(|_| {
            PreviewError::new(
                "preview_spawn_failed",
                "The preview command could not be started.",
            )
        })?;
        let pgid = child.id() as i32;
        if let Some(stdout) = child.stdout.take() {
            spawn_reader(stdout, Arc::clone(&self.output), Stream::Stdout);
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_reader(stderr, Arc::clone(&self.output), Stream::Stderr);
        }
        inner.phase = PreviewPhase::Starting;
        inner.port = Some(port);
        inner.reason = None;
        inner.exit_code = None;
        inner.child = Some(child);
        inner.pgid = Some(pgid);
        inner.grace_ms = grace_ms;
        drop(inner);

        spawn_readiness_worker(
            Arc::clone(&self.inner),
            port,
            Duration::from_millis(readiness_ms),
        );
        Ok(PreviewStartResponse { port })
    }

    pub fn stop_default(&self) -> Result<PreviewStopResponse, PreviewError> {
        self.stop(GRACE_DEFAULT_MS)
    }

    pub fn stop(&self, grace_ms: u64) -> Result<PreviewStopResponse, PreviewError> {
        let grace = clamp(grace_ms, GRACE_MIN_MS, GRACE_MAX_MS);
        let (pgid, mut child) = {
            let mut inner = self.inner.lock().expect("preview state");
            match inner.phase {
                PreviewPhase::Idle | PreviewPhase::Stopped => {
                    return Ok(PreviewStopResponse { stopped: true });
                }
                PreviewPhase::Failed => {
                    inner.phase = PreviewPhase::Stopped;
                    inner.reason = None;
                    inner.exit_code = None;
                    return Ok(PreviewStopResponse { stopped: true });
                }
                PreviewPhase::Starting | PreviewPhase::Ready | PreviewPhase::Stopping => {}
            }
            inner.phase = PreviewPhase::Stopping;
            (inner.pgid, inner.child.take())
        };
        if let Some(pgid) = pgid {
            terminate_group(pgid, Duration::from_millis(grace));
        }
        if let Some(child) = child.as_mut() {
            let _ = child.wait();
        }
        let mut inner = self.inner.lock().expect("preview state");
        inner.phase = PreviewPhase::Stopped;
        inner.port = None;
        inner.reason = None;
        inner.exit_code = None;
        inner.child = None;
        inner.pgid = None;
        Ok(PreviewStopResponse { stopped: true })
    }

    pub fn status(&self) -> PreviewStatus {
        let mut inner = self.inner.lock().expect("preview state");
        if matches!(inner.phase, PreviewPhase::Starting | PreviewPhase::Ready) {
            if let Some(child) = inner.child.as_mut() {
                if let Ok(Some(exit)) = child.try_wait() {
                    inner.phase = PreviewPhase::Failed;
                    inner.reason = Some("preview_exited_early".to_string());
                    inner.exit_code = exit.code();
                    inner.child = None;
                    inner.pgid = None;
                    inner.port = None;
                }
            }
        }
        PreviewStatus {
            phase: inner.phase,
            port: inner.port,
            reason: inner.reason.clone(),
            exit_code: inner.exit_code,
        }
    }

    #[cfg(test)]
    fn active_pgid(&self) -> Option<i32> {
        self.inner.lock().expect("preview state").pgid
    }

    #[cfg(test)]
    fn captured_lengths(&self) -> (usize, usize, usize, usize) {
        let output = self.output.lock().expect("preview output");
        (
            output.stdout.total,
            output.stdout.bytes.len(),
            output.stderr.total,
            output.stderr.bytes.len(),
        )
    }
}

fn clamp(value: u64, min: u64, max: u64) -> u64 {
    value.clamp(min, max)
}

fn free_loopback_port() -> Result<u16, PreviewError> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| {
        PreviewError::new(
            "preview_port_unavailable",
            "BrainRoot could not reserve a local port for the preview.",
        )
    })?;
    let port = listener
        .local_addr()
        .map_err(|_| {
            PreviewError::new(
                "preview_port_unavailable",
                "BrainRoot could not reserve a local port for the preview.",
            )
        })?
        .port();
    drop(listener);
    Ok(port)
}

fn loopback_ready(port: u16) -> bool {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).is_ok()
}

fn spawn_reader(
    mut reader: impl Read + Send + 'static,
    output: Arc<Mutex<CapturedOutput>>,
    stream: Stream,
) {
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    if let Ok(mut captured) = output.lock() {
                        match stream {
                            Stream::Stdout => captured.stdout.push(&buffer[..read]),
                            Stream::Stderr => captured.stderr.push(&buffer[..read]),
                        }
                    }
                }
            }
        }
    });
}

fn spawn_readiness_worker(inner: Arc<Mutex<Inner>>, port: u16, timeout: Duration) {
    thread::spawn(move || {
        let deadline = Instant::now() + timeout;
        loop {
            {
                let mut state = inner.lock().expect("preview state");
                if state.phase != PreviewPhase::Starting {
                    return;
                }
                let exited = match state.child.as_mut() {
                    Some(child) => match child.try_wait() {
                        Ok(Some(exit)) => Some(exit),
                        Ok(None) => None,
                        Err(_) => None,
                    },
                    None => return,
                };
                if let Some(exit) = exited {
                    state.phase = PreviewPhase::Failed;
                    state.reason = Some("preview_exited_early".to_string());
                    state.exit_code = exit.code();
                    state.child = None;
                    state.pgid = None;
                    state.port = None;
                    return;
                }
            }
            if loopback_ready(port) {
                let mut state = inner.lock().expect("preview state");
                if state.phase == PreviewPhase::Starting {
                    state.phase = PreviewPhase::Ready;
                }
                return;
            }
            if Instant::now() >= deadline {
                let pgid = {
                    let mut state = inner.lock().expect("preview state");
                    if state.phase != PreviewPhase::Starting {
                        return;
                    }
                    state.phase = PreviewPhase::Stopping;
                    state.pgid
                };
                if let Some(pgid) = pgid {
                    terminate_group(pgid, Duration::from_millis(GRACE_MIN_MS));
                }
                let mut state = inner.lock().expect("preview state");
                if let Some(child) = state.child.as_mut() {
                    let _ = child.wait();
                }
                state.phase = PreviewPhase::Failed;
                state.reason = Some("preview_not_ready".to_string());
                state.exit_code = None;
                state.child = None;
                state.pgid = None;
                state.port = None;
                return;
            }
            thread::sleep(POLL);
        }
    });
}

fn terminate_group(pgid: i32, grace: Duration) {
    signal_group(pgid, libc::SIGTERM);
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if !group_alive(pgid) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    signal_group(pgid, libc::SIGKILL);
}

fn signal_group(pgid: i32, signal: i32) {
    if pgid <= 1 {
        return;
    }
    unsafe {
        libc::kill(-pgid, signal);
    }
}

fn group_alive(pgid: i32) -> bool {
    if pgid <= 1 {
        return false;
    }
    unsafe { libc::kill(-pgid, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(command: &str, args: &[&str]) -> PreviewStartRequest {
        PreviewStartRequest {
            command: command.to_string(),
            args: args.iter().map(|argument| argument.to_string()).collect(),
            cwd: std::env::temp_dir().to_string_lossy().to_string(),
            readiness_timeout_ms: Some(8_000),
            stop_grace_ms: Some(500),
        }
    }

    fn wait_for_phase(supervisor: &DevServerSupervisor, phases: &[PreviewPhase]) -> PreviewStatus {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let status = supervisor.status();
            if phases.contains(&status.phase) {
                return status;
            }
            if Instant::now() >= deadline {
                return status;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn readiness_succeeds_and_stop_cleans_up() {
        let supervisor = DevServerSupervisor::new();
        let response = supervisor
            .start(request(
                "sh",
                &["-c", "python3 -m http.server \"$PORT\" --bind 127.0.0.1"],
            ))
            .expect("start");
        assert!(response.port > 0);
        let status = wait_for_phase(&supervisor, &[PreviewPhase::Ready, PreviewPhase::Failed]);
        assert_eq!(status.phase, PreviewPhase::Ready, "status: {status:?}");
        let port = status.port.expect("port");
        assert!(loopback_ready(port));
        let pgid = supervisor.active_pgid().expect("pgid");
        supervisor.stop(500).expect("stop");
        assert_eq!(supervisor.status().phase, PreviewPhase::Stopped);
        assert!(!group_alive(pgid), "process group {pgid} is still alive");
        assert!(
            TcpListener::bind(("127.0.0.1", port)).is_ok(),
            "port {port} is still bound"
        );
    }

    #[test]
    fn grandchild_is_cleaned_up_with_the_group() {
        let supervisor = DevServerSupervisor::new();
        supervisor
            .start(request(
                "sh",
                &[
                    "-c",
                    "python3 -m http.server \"$PORT\" --bind 127.0.0.1 & wait",
                ],
            ))
            .expect("start");
        let status = wait_for_phase(&supervisor, &[PreviewPhase::Ready, PreviewPhase::Failed]);
        assert_eq!(status.phase, PreviewPhase::Ready, "status: {status:?}");
        let port = status.port.expect("port");
        let pgid = supervisor.active_pgid().expect("pgid");
        supervisor.stop(500).expect("stop");
        assert!(!group_alive(pgid), "process group {pgid} is still alive");
        assert!(
            TcpListener::bind(("127.0.0.1", port)).is_ok(),
            "grandchild still holds port {port}"
        );
    }

    #[test]
    fn readiness_timeout_fails_and_kills_the_group() {
        let supervisor = DevServerSupervisor::new();
        let mut start = request("sh", &["-c", "sleep 63"]);
        start.readiness_timeout_ms = Some(1_000);
        supervisor.start(start).expect("start");
        let status = wait_for_phase(&supervisor, &[PreviewPhase::Failed]);
        assert_eq!(status.phase, PreviewPhase::Failed, "status: {status:?}");
        assert_eq!(status.reason.as_deref(), Some("preview_not_ready"));
        assert!(
            !process_command_contains("sleep 63"),
            "timed-out child survived"
        );
    }

    #[test]
    fn early_exit_is_detected() {
        let supervisor = DevServerSupervisor::new();
        supervisor
            .start(request("sh", &["-c", "exit 3"]))
            .expect("start");
        let status = wait_for_phase(&supervisor, &[PreviewPhase::Failed]);
        assert_eq!(status.phase, PreviewPhase::Failed, "status: {status:?}");
        assert_eq!(status.reason.as_deref(), Some("preview_exited_early"));
        assert_eq!(status.exit_code, Some(3));
    }

    #[test]
    fn duplicate_start_is_rejected() {
        let supervisor = DevServerSupervisor::new();
        supervisor
            .start(request(
                "sh",
                &["-c", "python3 -m http.server \"$PORT\" --bind 127.0.0.1"],
            ))
            .expect("start");
        let duplicate = supervisor.start(request("sh", &["-c", "sleep 30"]));
        assert!(duplicate.is_err(), "duplicate start was accepted");
        assert_eq!(duplicate.expect_err("error").code, "preview_already_active");
        supervisor.stop(500).expect("stop");
    }

    #[test]
    fn invalidation_is_plain_and_bounded() {
        let supervisor = DevServerSupervisor::new();
        let missing = supervisor.start(PreviewStartRequest {
            command: "  ".to_string(),
            args: Vec::new(),
            cwd: "/tmp".to_string(),
            readiness_timeout_ms: None,
            stop_grace_ms: None,
        });
        assert_eq!(missing.expect_err("error").code, "preview_command_missing");
        let relative = supervisor.start(PreviewStartRequest {
            command: "sh".to_string(),
            args: Vec::new(),
            cwd: "relative".to_string(),
            readiness_timeout_ms: None,
            stop_grace_ms: None,
        });
        assert_eq!(relative.expect_err("error").code, "preview_cwd_invalid");
    }

    #[test]
    fn output_is_bounded_and_never_logged() {
        let supervisor = DevServerSupervisor::new();
        let mut start = request(
            "sh",
            &[
                "-c",
                "i=0; while [ $i -lt 4000 ]; do echo bounded-output-line; i=$((i+1)); done; sleep 30",
            ],
        );
        start.readiness_timeout_ms = Some(1_500);
        supervisor.start(start).expect("start");
        let status = wait_for_phase(&supervisor, &[PreviewPhase::Failed]);
        assert_eq!(status.phase, PreviewPhase::Failed, "status: {status:?}");
        let (stdout_total, stdout_buffered, _, _) = supervisor.captured_lengths();
        assert!(stdout_total > OUTPUT_CAP_BYTES, "total: {stdout_total}");
        assert!(
            stdout_buffered <= OUTPUT_CAP_BYTES,
            "buffered: {stdout_buffered}"
        );
    }

    fn process_command_contains(needle: &str) -> bool {
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return false;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(pid) = name.to_str().and_then(|value| value.parse::<u32>().ok()) else {
                continue;
            };
            let Ok(command) = std::fs::read(format!("/proc/{pid}/cmdline")) else {
                continue;
            };
            let text = String::from_utf8_lossy(&command);
            if text.contains(needle) {
                return true;
            }
        }
        false
    }
}
