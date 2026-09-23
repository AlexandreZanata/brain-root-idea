//! Debug-only deck harness (B15-S01).
//!
//! Drives the production Preview and Human Browser commands through a
//! destroy/recreate switch cycle and records presence flags, switch latencies,
//! child-process counts, and RSS. Release builds never compile this module.

use std::thread;
use std::time::{Duration, Instant};

use tauri::Manager;

use crate::features::human_browser::{human_browser_hide, human_browser_show, HumanBrowserState};
use crate::features::preview::{
    preview_hide, preview_show, preview_start, preview_status, preview_stop, PreviewPhase,
    PreviewStartRequest, PreviewState,
};

#[cfg(debug_assertions)]
pub fn debug_fixture(app: tauri::AppHandle) {
    thread::spawn(move || {
        let mut report = serde_json::Map::new();
        let mut reasons: Vec<String> = Vec::new();
        let bounds = [40.0, 80.0, 560.0, 380.0];

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
        let started = match preview_start(app.clone(), app.state::<PreviewState>(), request) {
            Ok(started) => started,
            Err(_) => {
                reasons.push("preview server did not start".to_string());
                finish(&app, report, reasons);
                return;
            }
        };
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if preview_status(app.state::<PreviewState>()).phase == PreviewPhase::Ready {
                break;
            }
            if Instant::now() >= deadline {
                reasons.push("preview server did not become ready".to_string());
                let _ = preview_stop(app.state::<PreviewState>(), None);
                finish(&app, report, reasons);
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }

        let host = "127.0.0.1";
        let root_url = format!("http://{}:{}/", host, started.port);
        let baseline_children = child_process_count();
        let baseline_rss = rss_kb();

        // Phase 1: preview.
        let switch = Instant::now();
        let _ = preview_show(
            app.clone(),
            app.state::<PreviewState>(),
            started.port,
            bounds,
        );
        let show_preview_ms = switch.elapsed().as_millis() as u64;
        thread::sleep(Duration::from_millis(400));
        let preview_present = crate::features::preview::debug_view_present(&app);
        let human_present = crate::features::human_browser::debug_view_present(&app);
        let children_after_preview = child_process_count();
        if !preview_present || human_present {
            reasons.push("preview phase presence is wrong".to_string());
        }

        // Phase 2: switch to the browser.
        let switch = Instant::now();
        let _ = preview_hide(app.clone(), app.state::<PreviewState>());
        let hide_preview_ms = switch.elapsed().as_millis() as u64;
        let switch = Instant::now();
        let _ = human_browser_show(
            app.clone(),
            app.state::<HumanBrowserState>(),
            root_url.clone(),
            bounds,
        );
        let show_human_ms = switch.elapsed().as_millis() as u64;
        thread::sleep(Duration::from_millis(500));
        let preview_present = crate::features::preview::debug_view_present(&app);
        let human_present = crate::features::human_browser::debug_view_present(&app);
        let children_after_human = child_process_count();
        if preview_present || !human_present {
            reasons.push("browser phase presence is wrong".to_string());
        }

        // Phase 3: switch back to the preview.
        let switch = Instant::now();
        let _ = human_browser_hide(app.clone(), app.state::<HumanBrowserState>());
        let hide_human_ms = switch.elapsed().as_millis() as u64;
        let switch = Instant::now();
        let _ = preview_show(
            app.clone(),
            app.state::<PreviewState>(),
            started.port,
            bounds,
        );
        let re_show_preview_ms = switch.elapsed().as_millis() as u64;
        thread::sleep(Duration::from_millis(400));
        let preview_present = crate::features::preview::debug_view_present(&app);
        let human_present = crate::features::human_browser::debug_view_present(&app);
        let children_after_switch_back = child_process_count();
        let rss_after_switch_back = rss_kb();
        if !preview_present || human_present {
            reasons.push("return phase presence is wrong".to_string());
        }
        // Phase 4: a second cycle must not grow the footprint beyond the
        // two-role contexts created by the first cycle.
        let switch = Instant::now();
        let _ = preview_hide(app.clone(), app.state::<PreviewState>());
        let _ = human_browser_show(
            app.clone(),
            app.state::<HumanBrowserState>(),
            root_url.clone(),
            bounds,
        );
        let second_show_human_ms = switch.elapsed().as_millis() as u64;
        let switch = Instant::now();
        let _ = human_browser_hide(app.clone(), app.state::<HumanBrowserState>());
        let _ = preview_show(
            app.clone(),
            app.state::<PreviewState>(),
            started.port,
            bounds,
        );
        let second_show_preview_ms = switch.elapsed().as_millis() as u64;
        thread::sleep(Duration::from_millis(500));
        let children_after_second_cycle = child_process_count();
        if children_after_second_cycle > children_after_human {
            reasons.push("child processes accumulated across cycles".to_string());
        }

        // Cleanup.
        let _ = preview_hide(app.clone(), app.state::<PreviewState>());
        let _ = human_browser_hide(app.clone(), app.state::<HumanBrowserState>());
        let _ = preview_stop(app.state::<PreviewState>(), None);
        thread::sleep(Duration::from_millis(600));
        let children_after_cleanup = child_process_count();

        report.insert(
            "show_preview_ms".to_string(),
            serde_json::json!(show_preview_ms),
        );
        report.insert(
            "hide_preview_ms".to_string(),
            serde_json::json!(hide_preview_ms),
        );
        report.insert(
            "show_human_ms".to_string(),
            serde_json::json!(show_human_ms),
        );
        report.insert(
            "hide_human_ms".to_string(),
            serde_json::json!(hide_human_ms),
        );
        report.insert(
            "re_show_preview_ms".to_string(),
            serde_json::json!(re_show_preview_ms),
        );
        report.insert(
            "baseline_children".to_string(),
            serde_json::json!(baseline_children),
        );
        report.insert(
            "baseline_rss_kb".to_string(),
            serde_json::json!(baseline_rss),
        );
        report.insert(
            "children_after_preview".to_string(),
            serde_json::json!(children_after_preview),
        );
        report.insert(
            "children_after_human".to_string(),
            serde_json::json!(children_after_human),
        );
        report.insert(
            "children_after_switch_back".to_string(),
            serde_json::json!(children_after_switch_back),
        );
        report.insert(
            "second_show_human_ms".to_string(),
            serde_json::json!(second_show_human_ms),
        );
        report.insert(
            "second_show_preview_ms".to_string(),
            serde_json::json!(second_show_preview_ms),
        );
        report.insert(
            "children_after_second_cycle".to_string(),
            serde_json::json!(children_after_second_cycle),
        );
        report.insert(
            "rss_after_switch_back_kb".to_string(),
            serde_json::json!(rss_after_switch_back),
        );
        report.insert(
            "children_after_cleanup".to_string(),
            serde_json::json!(children_after_cleanup),
        );
        finish(&app, report, reasons);
    });
}

#[cfg(debug_assertions)]
fn finish(
    app: &tauri::AppHandle,
    mut report: serde_json::Map<String, serde_json::Value>,
    reasons: Vec<String>,
) {
    report.insert(
        "decision".to_string(),
        serde_json::json!(if reasons.is_empty() { "go" } else { "no-go" }),
    );
    report.insert("reasons".to_string(), serde_json::json!(reasons));
    println!(
        "brainroot: deck probe {}",
        serde_json::Value::Object(report)
    );
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
    app.exit(0);
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
