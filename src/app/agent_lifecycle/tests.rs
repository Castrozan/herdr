use super::*;

#[tokio::test]
async fn continuation_is_submitted_once_after_the_resumed_agent_is_idle() {
    let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(
        &crate::config::Config::default(),
        crate::app::AppPolicy::TEST,
        None,
        api_rx,
        crate::api::EventHub::default(),
    );
    let workspace = crate::workspace::Workspace::test_new("continuation");
    let pane_id = workspace.tabs[0].root_pane;
    let terminal_id = workspace.terminal_id(pane_id).cloned().unwrap();
    app.state.workspaces = vec![workspace];
    app.state.ensure_test_terminals();
    let (runtime, mut receiver) = crate::terminal::TerminalRuntime::test_with_channel(80, 24);
    app.terminal_runtimes.insert(terminal_id.clone(), runtime);
    app.queue_agent_continuation(
        terminal_id.clone(),
        crate::detect::Agent::Codex,
        "continue".into(),
    );

    let delivery_time = Instant::now();
    assert!(!app.deliver_pending_agent_continuations(delivery_time));
    assert!(receiver.try_recv().is_err());
    app.state
        .terminals
        .get_mut(&terminal_id)
        .unwrap()
        .set_detected_state(
            Some(crate::detect::Agent::Codex),
            crate::detect::AgentState::Idle,
        );

    assert!(app.deliver_pending_agent_continuations(delivery_time));
    assert_eq!(
        receiver
            .recv()
            .await
            .expect("continuation text should be queued"),
        Bytes::from_static(b"continue")
    );
    assert_eq!(
        receiver
            .recv()
            .await
            .expect("continuation submit should be queued"),
        Bytes::from_static(b"\r")
    );
    assert!(!app.deliver_pending_agent_continuations(delivery_time + TIMEOUT));
    assert!(receiver.try_recv().is_err());
}
