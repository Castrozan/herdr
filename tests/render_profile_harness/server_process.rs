use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

use crate::support::{
    register_runtime_dir, register_spawned_herdr_pid, unregister_spawned_herdr_pid,
};

use super::{CLIENT_TERMINAL_COLUMNS, CLIENT_TERMINAL_ROWS};

pub struct SpawnedProfiledServer {
    master: Option<Box<dyn MasterPty + Send>>,
    child: Box<dyn Child + Send + Sync>,
    pub process_id: u32,
}

impl Drop for SpawnedProfiledServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        drop(self.master.take());
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            let mut status = 0;
            let waited = unsafe {
                libc::waitpid(self.process_id as libc::pid_t, &mut status, libc::WNOHANG)
            };
            if waited == self.process_id as libc::pid_t || waited == -1 {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        unregister_spawned_herdr_pid(Some(self.process_id));
    }
}

pub fn unique_profile_base() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    PathBuf::from(format!(
        "/tmp/herdr-render-profile-{}-{nanos}",
        std::process::id()
    ))
}

pub fn spawn_profiled_server(
    config_home: &Path,
    runtime_dir: &Path,
    api_socket_path: &Path,
    pane_program: &Path,
) -> SpawnedProfiledServer {
    fs::create_dir_all(config_home.join("herdr")).unwrap();
    fs::create_dir_all(runtime_dir).unwrap();
    register_runtime_dir(runtime_dir);
    fs::write(
        config_home.join("herdr/config.toml"),
        "onboarding = false\n",
    )
    .unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: CLIENT_TERMINAL_ROWS,
            cols: CLIENT_TERMINAL_COLUMNS,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_herdr"));
    command.arg("server");
    command.env("XDG_CONFIG_HOME", config_home);
    command.env("XDG_RUNTIME_DIR", runtime_dir);
    command.env("HERDR_SOCKET_PATH", api_socket_path);
    command.env_remove("HERDR_CLIENT_SOCKET_PATH");
    command.env_remove("HERDR_ENV");
    command.env("SHELL", pane_program);
    command.env("HERDR_RENDER_PROF", "1");

    let child = pair.slave.spawn_command(command).unwrap();
    let process_id = child.process_id().expect("server pid should be known");
    register_spawned_herdr_pid(Some(process_id));
    drop(pair.slave);

    SpawnedProfiledServer {
        master: Some(pair.master),
        child,
        process_id,
    }
}

pub fn server_cpu_seconds(process_id: u32) -> f64 {
    let output = Command::new("ps")
        .args(["-p", &process_id.to_string(), "-o", "time="])
        .output()
        .expect("ps should run");
    parse_ps_cpu_time(String::from_utf8_lossy(&output.stdout).trim())
}

pub fn parse_ps_cpu_time(raw: &str) -> f64 {
    if raw.is_empty() {
        return 0.0;
    }
    let (days, clock) = match raw.split_once('-') {
        Some((days, clock)) => (days.parse::<f64>().unwrap_or(0.0), clock),
        None => (0.0, raw),
    };
    let mut seconds = days * 86_400.0;
    let mut multiplier = 1.0;
    for component in clock.rsplit(':') {
        seconds += component.parse::<f64>().unwrap_or(0.0) * multiplier;
        multiplier *= 60.0;
    }
    seconds
}
