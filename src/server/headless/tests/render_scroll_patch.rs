use std::time::Duration;

use super::{assert_frame_data_eq, read_server_frame, retained_test_server};

fn assert_retained_matches_full_render(initial_screen: &[u8], update: &[u8]) {
    let (mut retained_server, retained_rx, retained_pane_id) = retained_test_server(initial_screen);
    let (mut full_server, full_rx, full_pane_id) = retained_test_server(initial_screen);

    retained_server.render_and_stream();
    let _ = retained_rx
        .recv_timeout(Duration::from_millis(500))
        .expect("initial retained baseline");
    full_server.render_and_stream();
    let _ = full_rx
        .recv_timeout(Duration::from_millis(500))
        .expect("initial full baseline");

    retained_server
        .app
        .state
        .runtime_for_pane_in_workspace(&retained_server.app.terminal_runtimes, 0, retained_pane_id)
        .expect("retained runtime")
        .test_process_pty_bytes(update);
    full_server
        .app
        .state
        .runtime_for_pane_in_workspace(&full_server.app.terminal_runtimes, 0, full_pane_id)
        .expect("full runtime")
        .test_process_pty_bytes(update);

    assert!(
        retained_server.render_retained_pty_update_and_stream(),
        "the retained path must absorb a full-screen dirty update instead of falling back"
    );
    full_server.render_and_stream();

    let retained_frame = read_server_frame(
        retained_rx
            .recv_timeout(Duration::from_millis(500))
            .expect("retained frame"),
    );
    let full_frame = read_server_frame(
        full_rx
            .recv_timeout(Duration::from_millis(500))
            .expect("full frame"),
    );
    assert_frame_data_eq(&retained_frame, &full_frame);
}

#[tokio::test]
async fn scrolled_pane_retained_update_matches_full_render_frame() {
    let mut update = Vec::new();
    for line in 0..60 {
        update.extend_from_slice(format!("scrolled output line {line}\r\n").as_bytes());
    }
    assert_retained_matches_full_render(b"first screen contents", &update);
}

#[tokio::test]
async fn alternate_screen_switch_retained_update_matches_full_render_frame() {
    let mut update = Vec::new();
    update.extend_from_slice(b"\x1b[?1049h");
    update.extend_from_slice(b"alternate screen body\r\nsecond alternate row\r\n");
    assert_retained_matches_full_render(b"primary screen contents", &update);
}

#[tokio::test]
async fn styled_scroll_retained_update_matches_full_render_frame() {
    let mut update = Vec::new();
    for line in 0..40 {
        update.extend_from_slice(
            format!(
                "\x1b[3{}mstyled row {line}\x1b[0m padding text\r\n",
                line % 8
            )
            .as_bytes(),
        );
    }
    assert_retained_matches_full_render(b"\x1b[1mbold first screen\x1b[0m", &update);
}

#[tokio::test]
async fn wide_glyph_scroll_retained_update_matches_full_render_frame() {
    let mut update = Vec::new();
    for line in 0..40 {
        update.extend_from_slice(format!("行 {line} wide glyph row\r\n").as_bytes());
    }
    assert_retained_matches_full_render("初期画面".as_bytes(), &update);
}

#[tokio::test]
async fn screen_clear_retained_update_matches_full_render_frame() {
    assert_retained_matches_full_render(b"contents before the clear", b"\x1b[2J\x1b[Hafter clear");
}
