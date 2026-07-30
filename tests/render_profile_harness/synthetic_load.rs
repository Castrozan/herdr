use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::support::read_server_message;

use super::{AGENT_STATE_REPORT_INTERVAL, AGENT_STATE_TRANSITION_EVERY};

const PANE_OUTPUT_GENERATOR: &str = "#!/bin/sh\nemitted_line_count=0\nwhile :; do\n  emitted_line_count=$((emitted_line_count + 1))\n  printf 'pane output line %d carrying enough text to occupy a realistic share of the row\\n' \"$emitted_line_count\"\n  sleep 0.05\ndone\n";

pub fn write_pane_output_generator(base: &Path) -> PathBuf {
    let path = base.join("pane-output-generator.sh");
    fs::write(&path, PANE_OUTPUT_GENERATOR).expect("pane output generator should be writable");
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

pub fn run_herdr_cli(api_socket_path: &Path, arguments: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_herdr"))
        .args(arguments)
        .env("HERDR_SOCKET_PATH", api_socket_path)
        .env_remove("HERDR_CLIENT_SOCKET_PATH")
        .env_remove("HERDR_ENV")
        .output()
        .expect("herdr cli should run");
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn pane_identifiers(api_socket_path: &Path) -> Vec<String> {
    let listing = run_herdr_cli(api_socket_path, &["pane", "list"]);
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&listing) else {
        return Vec::new();
    };
    let mut identifiers = Vec::new();
    collect_pane_identifiers(&parsed, &mut identifiers);
    identifiers.sort();
    identifiers.dedup();
    identifiers
}

fn collect_pane_identifiers(value: &serde_json::Value, identifiers: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(entries) => {
            for (key, entry) in entries {
                if key == "pane_id" {
                    if let Some(identifier) = entry.as_str() {
                        identifiers.push(identifier.to_string());
                    }
                }
                collect_pane_identifiers(entry, identifiers);
            }
        }
        serde_json::Value::Array(entries) => {
            for entry in entries {
                collect_pane_identifiers(entry, identifiers);
            }
        }
        _ => {}
    }
}

pub fn split_until_visible_pane_count(api_socket_path: &Path, target_count: usize) -> Vec<String> {
    let mut directions = ["right", "down"].into_iter().cycle();
    let mut identifiers = pane_identifiers(api_socket_path);
    let mut attempts = 0;
    while identifiers.len() < target_count && attempts < target_count * 4 {
        attempts += 1;
        let Some(anchor) = identifiers.get(identifiers.len() / 2).cloned() else {
            break;
        };
        let direction = directions.next().unwrap();
        run_herdr_cli(
            api_socket_path,
            &[
                "pane",
                "split",
                &anchor,
                "--direction",
                direction,
                "--no-focus",
            ],
        );
        thread::sleep(Duration::from_millis(150));
        identifiers = pane_identifiers(api_socket_path);
    }
    identifiers
}

pub fn report_agent_states_until(
    api_socket_path: &Path,
    pane_ids: &[String],
    stop: Arc<AtomicBool>,
) -> thread::JoinHandle<u64> {
    let api_socket_path = api_socket_path.to_path_buf();
    let pane_ids = pane_ids.to_vec();
    thread::spawn(move || {
        let mut report_count = 0u64;
        let mut rotation = 0usize;
        while !stop.load(Ordering::Relaxed) {
            let state = if rotation % AGENT_STATE_TRANSITION_EVERY == 0 {
                "idle"
            } else {
                "working"
            };
            for pane_id in &pane_ids {
                run_herdr_cli(
                    &api_socket_path,
                    &[
                        "pane",
                        "report-agent",
                        pane_id,
                        "--source",
                        "render-loop-profile",
                        "--agent",
                        "claude",
                        "--state",
                        state,
                    ],
                );
                report_count += 1;
            }
            rotation += 1;
            thread::sleep(AGENT_STATE_REPORT_INTERVAL);
        }
        report_count
    })
}

pub fn drain_client_frames_until(
    mut stream: UnixStream,
    stop: Arc<AtomicBool>,
) -> thread::JoinHandle<u64> {
    thread::spawn(move || {
        stream
            .set_read_timeout(Some(Duration::from_millis(250)))
            .expect("client stream timeout should be settable");
        let mut frame_count = 0u64;
        while !stop.load(Ordering::Relaxed) {
            if read_server_message(&mut stream).is_ok() {
                frame_count += 1;
            }
        }
        frame_count
    })
}
