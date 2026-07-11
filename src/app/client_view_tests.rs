use super::super::state::{AppState, Mode};
use crate::workspace::Workspace;

fn app_state_with_workspaces(names: &[&str]) -> AppState {
    let mut state = AppState::test_new();
    for name in names {
        state.workspaces.push(Workspace::test_new(name));
    }
    state.ensure_test_terminals();
    if !state.workspaces.is_empty() {
        state.active = Some(0);
        state.selected = 0;
        state.mode = Mode::Terminal;
    }
    state
}

#[test]
fn snapshot_restore_roundtrips_active_and_selected_workspace() {
    let mut state = app_state_with_workspaces(&["one", "two", "three"]);
    state.active = Some(2);
    state.selected = 0;
    let client_view = state.snapshot_client_view();

    state.active = Some(1);
    state.selected = 1;

    state.restore_client_view(&client_view);
    assert_eq!(state.active, Some(2));
    assert_eq!(state.selected, 0);
}

#[test]
fn modal_mode_stays_per_client_view() {
    let mut state = app_state_with_workspaces(&["one", "two"]);
    let terminal_client_view = state.snapshot_client_view();

    state.mode = Mode::Navigate;
    state.navigator.query = "two".to_owned();
    let navigating_client_view = state.snapshot_client_view();

    state.restore_client_view(&terminal_client_view);
    assert_eq!(state.mode, Mode::Terminal);
    assert!(state.navigator.query.is_empty());

    state.restore_client_view(&navigating_client_view);
    assert_eq!(state.mode, Mode::Navigate);
    assert_eq!(state.navigator.query, "two");
}

#[test]
fn restore_falls_back_to_first_workspace_when_active_id_is_gone() {
    let mut state = app_state_with_workspaces(&["one", "two"]);
    state.switch_workspace(1);
    let client_view = state.snapshot_client_view();

    state.workspaces.remove(1);
    state.active = Some(0);
    state.selected = 0;

    state.restore_client_view(&client_view);
    assert_eq!(state.active, Some(0));
    assert_eq!(state.selected, 0);
}

#[test]
fn restore_with_no_workspaces_downgrades_terminal_mode_to_navigate() {
    let mut state = app_state_with_workspaces(&["one"]);
    let client_view = state.snapshot_client_view();

    state.workspaces.clear();
    state.active = None;
    state.selected = 0;

    state.restore_client_view(&client_view);
    assert_eq!(state.active, None);
    assert_eq!(state.mode, Mode::Navigate);
}
