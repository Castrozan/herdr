use super::*;
use crate::api::schema::{ErrorResponse, SuccessResponse};
use crate::detect::{Agent, AgentState};

fn app_with_codex(record_session: bool) -> (App, crate::terminal::TerminalId) {
    let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(
        &crate::config::Config::default(),
        crate::app::AppPolicy::TEST,
        None,
        api_rx,
        crate::api::EventHub::default(),
    );
    app.state.workspaces = vec![crate::workspace::Workspace::test_new("agent")];
    app.state.ensure_test_terminals();
    let pane_id = app.state.workspaces[0].tabs[0].root_pane;
    let terminal_id = app.state.workspaces[0]
        .terminal_id(pane_id)
        .cloned()
        .unwrap();
    let terminal = app.state.terminals.get_mut(&terminal_id).unwrap();
    terminal.set_detected_state(Some(Agent::Codex), AgentState::Idle);
    if record_session {
        terminal.set_persisted_agent_session(crate::agent_resume::PersistedAgentSession {
            source: "herdr:codex".into(),
            agent: "codex".into(),
            session_ref: crate::agent_resume::AgentSessionRef::id("session-123").unwrap(),
        });
    }
    let (runtime, _receiver) = crate::terminal::TerminalRuntime::test_with_channel(80, 24);
    app.terminal_runtimes.insert(terminal_id.clone(), runtime);
    (app, terminal_id)
}

#[tokio::test]
async fn restart_queues_native_resume_for_the_exact_session() {
    let (mut app, terminal_id) = app_with_codex(true);

    let response = app.handle_agent_restart(
        "req".into(),
        AgentRestartParams {
            target: "codex".into(),
            prompt: Some("continue".into()),
        },
    );

    let success: SuccessResponse = serde_json::from_str(&response).unwrap();
    assert!(matches!(success.result, ResponseResult::Ok {}));
    let terminal = app.state.terminals.get(&terminal_id).unwrap();
    assert_eq!(
        terminal.pending_agent_resume_plan.as_ref().unwrap().argv,
        ["codex", "resume", "session-123"]
    );
    assert_eq!(terminal.state, AgentState::Unknown);
    assert_eq!(
        app.state.terminal_runtime_shutdowns.as_slice(),
        std::slice::from_ref(&terminal_id)
    );
    assert!(app.pending_agent_continuations.contains_key(&terminal_id));
}

#[tokio::test]
async fn restart_refuses_without_an_exact_session() {
    let (mut app, _terminal_id) = app_with_codex(false);

    let response = app.handle_agent_restart(
        "req".into(),
        AgentRestartParams {
            target: "codex".into(),
            prompt: None,
        },
    );
    let error: ErrorResponse = serde_json::from_str(&response).unwrap();

    assert_eq!(error.error.code, "agent_restart_unavailable");
    assert!(app.state.terminal_runtime_shutdowns.is_empty());
}

#[tokio::test]
async fn exit_queues_shell_respawn_in_the_same_pane() {
    let (mut app, terminal_id) = app_with_codex(true);

    let response = app.handle_agent_exit(
        "req".into(),
        AgentTarget {
            target: "codex".into(),
        },
    );

    let success: SuccessResponse = serde_json::from_str(&response).unwrap();
    assert!(matches!(success.result, ResponseResult::Ok {}));
    let terminal = app.state.terminals.get(&terminal_id).unwrap();
    assert!(terminal.respawn_shell_on_exit);
    assert_eq!(terminal.state, AgentState::Unknown);
    assert_eq!(app.state.terminal_runtime_shutdowns, [terminal_id]);
}

#[tokio::test]
async fn lifecycle_rejects_managed_agents() {
    let (mut app, terminal_id) = app_with_codex(true);
    app.state
        .terminals
        .get_mut(&terminal_id)
        .unwrap()
        .begin_managed_agent(
            "managed".into(),
            Agent::Codex,
            std::time::Instant::now(),
            std::time::Duration::ZERO,
            std::time::Duration::from_secs(10),
        );

    let restart = app.handle_agent_restart(
        "restart".into(),
        AgentRestartParams {
            target: "managed".into(),
            prompt: None,
        },
    );
    let exit = app.handle_agent_exit(
        "exit".into(),
        AgentTarget {
            target: "managed".into(),
        },
    );

    for response in [restart, exit] {
        let error: ErrorResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(error.error.code, "managed_agent_lifecycle_unavailable");
    }
    assert!(app.state.terminal_runtime_shutdowns.is_empty());
}
