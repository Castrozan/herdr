use super::test_headless_server;
use crate::api;
use crate::app::Mode;
use crate::server::headless::HeadlessServer;
use crate::workspace::Workspace;

fn report_agent_over_api(
    server: &mut HeadlessServer,
    pane_id: crate::layout::PaneId,
    state: api::schema::PaneAgentState,
    custom_status: Option<&str>,
) -> bool {
    let (respond_to, _response_rx) = std::sync::mpsc::channel();
    server.handle_api_request_with_shutdown_check(api::ApiRequestMessage {
        request: api::schema::Request {
            id: "render-gate-report".into(),
            method: api::schema::Method::PaneReportAgent(api::schema::PaneReportAgentParams {
                pane_id: format!("p_{}", pane_id.raw()),
                source: "render-gate-test".into(),
                agent: "claude".into(),
                state,
                message: None,
                custom_status: custom_status.map(str::to_string),
                seq: None,
                agent_session_id: None,
                agent_session_path: None,
            }),
        },
        respond_to,
    })
}

fn server_with_one_agent_pane() -> (HeadlessServer, crate::layout::PaneId) {
    let mut server = test_headless_server();
    server.app.state.workspaces = vec![Workspace::test_new("render-gate")];
    server.app.state.ensure_test_terminals();
    server.app.state.active = Some(0);
    server.app.state.selected = 0;
    server.app.state.mode = Mode::Terminal;
    let pane_id = *server.app.state.workspaces[0].tabs[0]
        .panes
        .keys()
        .next()
        .expect("the test workspace should own exactly one pane");
    (server, pane_id)
}

#[test]
fn repeated_identical_agent_report_does_not_force_a_full_render() {
    let (mut server, pane_id) = server_with_one_agent_pane();

    assert!(
        report_agent_over_api(
            &mut server,
            pane_id,
            api::schema::PaneAgentState::Working,
            None
        ),
        "the first report moves the effective agent state and must force a full render"
    );
    assert!(
        !report_agent_over_api(
            &mut server,
            pane_id,
            api::schema::PaneAgentState::Working,
            None
        ),
        "a repeated identical report changes nothing rendered and must not force a full render"
    );
}

#[test]
fn agent_state_transition_still_forces_a_full_render() {
    let (mut server, pane_id) = server_with_one_agent_pane();
    report_agent_over_api(
        &mut server,
        pane_id,
        api::schema::PaneAgentState::Working,
        None,
    );

    assert!(
        report_agent_over_api(
            &mut server,
            pane_id,
            api::schema::PaneAgentState::Idle,
            None
        ),
        "an effective agent state transition must still force a full render"
    );
}

#[test]
fn custom_status_change_without_state_change_still_forces_a_full_render() {
    let (mut server, pane_id) = server_with_one_agent_pane();
    report_agent_over_api(
        &mut server,
        pane_id,
        api::schema::PaneAgentState::Working,
        Some("compiling"),
    );

    assert!(
        report_agent_over_api(
            &mut server,
            pane_id,
            api::schema::PaneAgentState::Working,
            Some("linking")
        ),
        "a presentation-only change must still force a full render"
    );
}

#[test]
fn agent_label_change_without_state_change_still_forces_a_full_render() {
    let (mut server, pane_id) = server_with_one_agent_pane();
    report_agent_over_api(
        &mut server,
        pane_id,
        api::schema::PaneAgentState::Working,
        None,
    );

    let (respond_to, _response_rx) = std::sync::mpsc::channel();
    let changed = server.handle_api_request_with_shutdown_check(api::ApiRequestMessage {
        request: api::schema::Request {
            id: "render-gate-relabel".into(),
            method: api::schema::Method::PaneReportAgent(api::schema::PaneReportAgentParams {
                pane_id: format!("p_{}", pane_id.raw()),
                source: "render-gate-test".into(),
                agent: "codex".into(),
                state: api::schema::PaneAgentState::Working,
                message: None,
                custom_status: None,
                seq: None,
                agent_session_id: None,
                agent_session_path: None,
            }),
        },
        respond_to,
    });

    assert!(
        changed,
        "an effective agent label change must still force a full render"
    );
}
