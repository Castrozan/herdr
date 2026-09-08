use super::*;

#[test]
fn pane_exit_preserves_a_pane_awaiting_native_agent_resume() {
    let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(
        &crate::config::Config::default(),
        crate::app::AppPolicy::TEST,
        None,
        api_rx,
        crate::api::EventHub::default(),
    );
    let workspace = crate::workspace::Workspace::test_new("restart");
    let pane_id = workspace.tabs[0].root_pane;
    let terminal_id = workspace.terminal_id(pane_id).cloned().unwrap();
    app.state.workspaces = vec![workspace];
    app.state.ensure_test_terminals();
    app.state
        .terminals
        .get_mut(&terminal_id)
        .unwrap()
        .pending_agent_resume_plan = Some(crate::agent_resume::AgentResumePlan {
        agent: "codex".into(),
        argv: vec!["codex".into(), "resume".into(), "session-123".into()],
        dedupe_key: "session-123".into(),
    });

    app.handle_internal_event(AppEvent::PaneDied {
        pane_id,
        exit_reason: crate::platform::ChildExitReason::Exited,
    });

    assert!(app.find_pane(pane_id).is_some());
    assert!(app.state.terminals[&terminal_id]
        .pending_agent_resume_plan
        .is_some());
}
