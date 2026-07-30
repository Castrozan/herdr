mod render_profile_harness;
mod support;

use std::fs;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use render_profile_harness::prof_log::{
    aggregate_render_prof_counters, aggregate_render_prof_durations,
};
use render_profile_harness::server_process::{
    server_cpu_seconds, spawn_profiled_server, unique_profile_base,
};
use render_profile_harness::synthetic_load::{
    drain_client_frames_until, report_agent_states_until, split_until_visible_pane_count,
    write_pane_output_generator,
};
use render_profile_harness::{
    CLIENT_TERMINAL_COLUMNS, CLIENT_TERMINAL_ROWS, MEASUREMENT_WINDOW, PANE_SETTLE_DELAY,
    VISIBLE_PANE_COUNT,
};
use support::{client_handshake, wait_for_socket, CURRENT_PROTOCOL};

#[test]
#[ignore = "long-running render loop measurement, run explicitly with --nocapture"]
fn render_loop_cost_under_agent_fleet_load() {
    let base = unique_profile_base();
    fs::create_dir_all(&base).unwrap();
    let config_home = base.join("config");
    let runtime_dir = base.join("run");
    let api_socket_path = base.join("herdr.sock");
    let client_socket_path = base.join("herdr-client.sock");
    let pane_program = write_pane_output_generator(&base);

    let server = spawn_profiled_server(&config_home, &runtime_dir, &api_socket_path, &pane_program);
    wait_for_socket(&api_socket_path, Duration::from_secs(20));

    let mut client = connect_profiling_client(&client_socket_path);
    client_handshake(
        &mut client,
        CURRENT_PROTOCOL,
        CLIENT_TERMINAL_COLUMNS,
        CLIENT_TERMINAL_ROWS,
    )
    .expect("profiling client handshake should succeed");

    let pane_ids = split_until_visible_pane_count(&api_socket_path, VISIBLE_PANE_COUNT);
    thread::sleep(PANE_SETTLE_DELAY);

    let stop = Arc::new(AtomicBool::new(false));
    let frame_reader = drain_client_frames_until(
        client.try_clone().expect("client stream should clone"),
        Arc::clone(&stop),
    );
    let agent_reporter = report_agent_states_until(&api_socket_path, &pane_ids, Arc::clone(&stop));

    let log_path = config_home.join("herdr/herdr-server.log");
    let counters_before = aggregate_render_prof_counters(&log_path);
    let cpu_before = server_cpu_seconds(server.process_id);
    let measurement_started = Instant::now();
    thread::sleep(MEASUREMENT_WINDOW);
    let cpu_after = server_cpu_seconds(server.process_id);
    let measured_window = measurement_started.elapsed();
    stop.store(true, Ordering::Relaxed);

    let counters_after = aggregate_render_prof_counters(&log_path);
    let (full_render_count, full_render_average_us) =
        aggregate_render_prof_durations(&log_path, "full_render.total");
    let (retained_count, retained_average_us) =
        aggregate_render_prof_durations(&log_path, "retained.total");
    let agent_reports = agent_reporter.join().unwrap_or(0);
    let frames_read = frame_reader.join().unwrap_or(0);

    let counter_delta = |name: &str| -> u64 {
        counters_after
            .get(name)
            .copied()
            .unwrap_or(0)
            .saturating_sub(counters_before.get(name).copied().unwrap_or(0))
    };

    let cpu_seconds = cpu_after - cpu_before;
    println!("RENDER_LOOP_PROFILE_BEGIN");
    println!("panes={}", pane_ids.len());
    println!("window_seconds={:.2}", measured_window.as_secs_f64());
    println!("server_cpu_seconds={cpu_seconds:.2}");
    println!(
        "server_cpu_percent={:.1}",
        100.0 * cpu_seconds / measured_window.as_secs_f64()
    );
    println!(
        "pty_dirty_requests={}",
        counter_delta("render.request.pty_dirty")
    );
    println!(
        "full_render_invocations={}",
        counter_delta("full_render.invoke")
    );
    println!("retained_successes={}", counter_delta("retained.success"));
    println!("full_render_total_count={full_render_count}");
    println!("full_render_avg_us={full_render_average_us:.0}");
    println!("retained_total_count={retained_count}");
    println!("retained_avg_us={retained_average_us:.0}");
    println!("agent_state_reports={agent_reports}");
    println!("client_frames_read={frames_read}");
    for (name, value) in dominant_causes(&counters_after, &counters_before) {
        println!("cause.{name}={value}");
    }
    println!("RENDER_LOOP_PROFILE_END");

    drop(server);
    let _ = fs::remove_dir_all(&base);
}

fn connect_profiling_client(client_socket_path: &std::path::Path) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match UnixStream::connect(client_socket_path) {
            Ok(stream) => return stream,
            Err(error) if Instant::now() >= deadline => {
                panic!("client socket never accepted a connection: {error}")
            }
            Err(_) => thread::sleep(Duration::from_millis(200)),
        }
    }
}

fn dominant_causes(
    after: &std::collections::BTreeMap<String, u64>,
    before: &std::collections::BTreeMap<String, u64>,
) -> Vec<(String, u64)> {
    let mut causes: Vec<(String, u64)> = after
        .iter()
        .filter(|(name, _)| {
            name.starts_with("retained_fallback.")
                || name.starts_with("full_render_cause.")
                || name.starts_with("retained_gate.")
                || name.starts_with("retained_success.")
                || name.starts_with("dirty_fallback.")
                || name.starts_with("dirty_collect.")
                || name.starts_with("full_render_skipped.")
        })
        .map(|(name, value)| {
            (
                name.clone(),
                value.saturating_sub(before.get(name).copied().unwrap_or(0)),
            )
        })
        .filter(|(_, value)| *value > 0)
        .collect();
    causes.sort_by(|left, right| right.1.cmp(&left.1));
    causes.truncate(20);
    causes
}
