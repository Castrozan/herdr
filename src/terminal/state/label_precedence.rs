use super::*;

#[test]
fn terminal_title_falls_between_manual_and_agent_labels() {
    let mut terminal = TerminalState::new(TerminalId::alloc(), "/tmp".into());
    terminal.set_detected_state(Some(Agent::Claude), AgentState::Idle);
    terminal.set_terminal_title(Some("⠋ terminal task".into()));

    assert_eq!(
        terminal.border_label(false).as_deref(),
        Some("terminal task")
    );
    assert_eq!(
        terminal.border_label(true).as_deref(),
        Some("terminal task")
    );

    terminal.set_manual_label(" reviewer ".into());
    assert_eq!(terminal.border_label(false).as_deref(), Some("reviewer"));
    assert_eq!(terminal.border_label(true).as_deref(), Some("reviewer"));

    terminal.set_manual_label("   ".into());
    assert_eq!(
        terminal.border_label(true).as_deref(),
        Some("terminal task")
    );

    terminal.set_manual_label("reviewer".into());
    terminal.clear_manual_label();
    assert_eq!(
        terminal.border_label(true).as_deref(),
        Some("terminal task")
    );

    terminal.set_terminal_title(None);
    assert_eq!(terminal.border_label(true).as_deref(), Some("claude"));
}
