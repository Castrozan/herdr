use super::test_headless_server;
use crate::app::Mode;
use crate::protocol::RenderEncoding;
use crate::server::clients::ClientConnection;
use crate::server::headless::HeadlessServer;
use crate::workspace::Workspace;

fn insert_test_app_client(server: &mut HeadlessServer, client_id: u64) {
    server.clients.insert(
        client_id,
        ClientConnection::new(
            (80, 24),
            crate::kitty_graphics::HostCellSize::default(),
            crate::terminal_theme::TerminalTheme::default(),
            Some(true),
            client_id,
            RenderEncoding::SemanticFrame,
            None,
        ),
    );
}

fn server_with_workspaces_and_clients(workspace_names: &[&str]) -> HeadlessServer {
    let mut server = test_headless_server();
    for name in workspace_names {
        server.app.state.workspaces.push(Workspace::test_new(name));
    }
    server.app.state.ensure_test_terminals();
    server.app.state.active = Some(0);
    server.app.state.selected = 0;
    server.app.state.mode = Mode::Terminal;
    insert_test_app_client(&mut server, 1);
    insert_test_app_client(&mut server, 2);
    server.foreground_client_id = Some(1);
    server
}

#[test]
fn two_clients_view_different_workspaces_without_mirroring() {
    let mut server = server_with_workspaces_and_clients(&["one", "two"]);

    server.focus_client_view(1);
    assert_eq!(server.app.state.active, Some(0));

    server.focus_client_view(2);
    server.app.state.switch_workspace(1);

    server.focus_client_view(1);
    assert_eq!(server.app.state.active, Some(0));
    assert_eq!(server.app.state.selected, 0);

    server.focus_client_view(2);
    assert_eq!(server.app.state.active, Some(1));
    assert_eq!(server.app.state.selected, 1);
}

#[test]
fn modal_state_stays_with_the_client_that_opened_it() {
    let mut server = server_with_workspaces_and_clients(&["one"]);

    server.focus_client_view(2);
    assert_eq!(server.app.state.mode, Mode::Terminal);

    server.focus_client_view(1);
    server.app.state.mode = Mode::KeybindHelp;
    server.app.state.navigator.query = "help".to_owned();

    server.focus_client_view(2);
    assert_eq!(server.app.state.mode, Mode::Terminal);
    assert!(server.app.state.navigator.query.is_empty());

    server.focus_client_view(1);
    assert_eq!(server.app.state.mode, Mode::KeybindHelp);
    assert_eq!(server.app.state.navigator.query, "help");
}

#[test]
fn workspace_removal_reconciles_other_clients_saved_views() {
    let mut server = server_with_workspaces_and_clients(&["one", "two"]);
    let removed_workspace_id = server.app.state.workspaces[1].id.clone();
    let remaining_workspace_id = server.app.state.workspaces[0].id.clone();

    server.focus_client_view(2);
    server.app.state.switch_workspace(1);
    server.focus_client_view(1);
    assert_eq!(
        server.clients[&2]
            .view
            .as_ref()
            .and_then(|view| view.active_workspace_id.clone()),
        Some(removed_workspace_id)
    );

    server.app.state.workspaces.remove(1);
    server.app.state.active = Some(0);
    server.app.state.selected = 0;
    server.reconcile_client_views_with_workspaces();

    assert_eq!(
        server.clients[&2]
            .view
            .as_ref()
            .and_then(|view| view.active_workspace_id.clone()),
        Some(remaining_workspace_id)
    );

    server.focus_client_view(2);
    assert_eq!(server.app.state.active, Some(0));
    assert_eq!(server.app.state.mode, Mode::Terminal);
}

#[test]
fn removing_the_view_owner_clears_ownership_and_refocuses_survivor() {
    let mut server = server_with_workspaces_and_clients(&["one", "two"]);

    server.focus_client_view(1);
    server.promote_client_to_foreground(2);
    server.focus_client_view(2);
    server.app.state.switch_workspace(1);
    assert_eq!(server.client_view_owner, Some(2));

    server.remove_client(2);
    assert_eq!(server.client_view_owner, Some(1));
    assert_eq!(server.foreground_client_id, Some(1));
    assert_eq!(server.app.state.active, Some(0));
}
