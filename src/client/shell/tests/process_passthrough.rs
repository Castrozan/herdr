use super::*;
use crossterm::event::{KeyCode, KeyModifiers};

fn passthrough_state() -> ClientShellState {
    let mut config = Config::default();
    config.keys.previous_tab = crate::config::BindingConfig::one("ctrl+pageup");
    config.keys.passthrough = vec![crate::config::PassthroughKeybindConfig {
        key: crate::config::BindingConfig::one("ctrl+pageup"),
        processes: vec!["nvim".into()],
    }];
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&config));
    let mut projection = snapshot();
    projection.tabs.push(ClientShellTab {
        tab_id: "tab_2".into(),
        workspace_id: "ws_1".into(),
        number: 2,
        label: "2".into(),
        custom_label: false,
        zoomed: false,
        focused: false,
        agent_status: AgentStatus::Idle,
    });
    state.set_snapshot(Box::new(projection));
    state.set_pane_surface(surface());
    state
}

fn passthrough_process_result(name: &str, argv0: &str) -> crate::api::schema::ResponseResult {
    crate::api::schema::ResponseResult::PaneProcessInfo {
        process_info: crate::api::schema::PaneProcessInfo {
            pane_id: "pane_1".into(),
            shell_pid: Some(10),
            foreground_process_group_id: Some(20),
            tty: None,
            foreground_processes: vec![crate::api::schema::PaneProcessInfoProcess {
                pid: 20,
                name: name.into(),
                argv0: Some(argv0.into()),
                argv: None,
                cmdline: None,
                cwd: None,
            }],
        },
    }
}

#[test]
fn passthrough_match_forwards_the_key_to_the_original_pane() {
    let mut state = passthrough_state();
    let pressed = state.handle_raw_events(vec![RawInputEvent::Key(
        crate::input::TerminalKey::new(KeyCode::PageUp, KeyModifiers::CONTROL),
    )]);
    let [ClientShellAction::Endpoint { request, .. }] = &pressed.actions[..] else {
        panic!("passthrough should inspect the focused pane");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::PaneProcessInfo(params)
            if params.pane_id.as_deref() == Some("pane_1")
    ));
    let request_id = request.id.clone();

    let (_, actions) = state.handle_endpoint_result(
        "boot-1",
        &request_id,
        Ok(passthrough_process_result("python", "/opt/bin/nvim")),
    );

    assert!(matches!(
        &actions[..],
        [ClientShellAction::PaneInput { pane_id, event, .. }]
            if pane_id == "pane_1"
                && matches!(event, crate::protocol::ClientPaneInputEvent::Key { .. })
    ));
}

#[test]
fn passthrough_miss_runs_the_original_client_local_binding() {
    let mut state = passthrough_state();
    let pressed = state.handle_raw_events(vec![RawInputEvent::Key(
        crate::input::TerminalKey::new(KeyCode::PageUp, KeyModifiers::CONTROL),
    )]);
    let [ClientShellAction::Endpoint { request, .. }] = &pressed.actions[..] else {
        panic!("passthrough should inspect the focused pane");
    };
    let request_id = request.id.clone();

    let (_, actions) = state.handle_endpoint_result(
        "boot-1",
        &request_id,
        Ok(passthrough_process_result("bash", "/bin/bash")),
    );

    assert!(matches!(
        &actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                crate::api::schema::Method::TabFocus(target) if target.tab_id == "tab_2"
            )
    ));
}
