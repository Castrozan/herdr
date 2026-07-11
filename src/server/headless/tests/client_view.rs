use super::test_headless_server;
use crate::app::Mode;
use crate::protocol::RenderEncoding;
use crate::server::client_transport::ServerEvent;
use crate::server::clients::{ClientConnection, ClientConnectionMode};
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

fn insert_test_terminal_attach_client(
    server: &mut HeadlessServer,
    client_id: u64,
    terminal_id: &str,
) {
    server.clients.insert(
        client_id,
        ClientConnection::new_with_mode(
            ClientConnectionMode::TerminalAttach {
                terminal_id: terminal_id.to_owned(),
            },
            None,
            (80, 24),
            crate::kitty_graphics::HostCellSize::default(),
            crate::terminal_theme::TerminalTheme::default(),
            Some(true),
            client_id,
            RenderEncoding::SemanticFrame,
            false,
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

#[test]
fn rename_modal_payload_stays_with_the_client_that_opened_it() {
    let mut server = server_with_workspaces_and_clients(&["one"]);
    let renamed_pane = *server.app.state.workspaces[0]
        .active_tab()
        .expect("workspace must have an active tab")
        .panes
        .keys()
        .next()
        .expect("active tab must have a pane");

    server.focus_client_view(2);
    server.focus_client_view(1);
    server.app.state.mode = Mode::RenamePane;
    server.app.state.rename_pane_target = Some(renamed_pane);
    server.app.state.name_input = "foo".to_owned();

    server.focus_client_view(2);
    assert_eq!(server.app.state.mode, Mode::Terminal);
    assert_eq!(server.app.state.rename_pane_target, None);
    assert!(server.app.state.name_input.is_empty());

    server.focus_client_view(1);
    assert_eq!(server.app.state.mode, Mode::RenamePane);
    assert_eq!(server.app.state.rename_pane_target, Some(renamed_pane));
    assert_eq!(server.app.state.name_input, "foo");
}

#[test]
fn same_workspace_clients_share_workspace_scoped_state() {
    let mut server = server_with_workspaces_and_clients(&["one"]);

    server.focus_client_view(2);
    server.focus_client_view(1);
    assert_eq!(server.app.state.active, Some(0));

    let split_pane =
        server.app.state.workspaces[0].test_split(ratatui::layout::Direction::Horizontal);
    assert_eq!(
        server.app.state.workspaces[0].focused_pane_id(),
        Some(split_pane)
    );

    server.focus_client_view(2);
    assert_eq!(server.app.state.active, Some(0));
    assert_eq!(
        server.app.state.workspaces[0].focused_pane_id(),
        Some(split_pane)
    );
    assert_eq!(
        server.app.state.workspaces[0]
            .active_tab()
            .expect("workspace must have an active tab")
            .panes
            .len(),
        2
    );
}

#[test]
fn focus_client_view_for_input_swaps_app_clients_and_skips_terminal_attach() {
    let mut server = server_with_workspaces_and_clients(&["one", "two"]);

    server.focus_client_view(1);
    server.focus_client_view(2);
    server.app.state.switch_workspace(1);
    server.focus_client_view(1);
    assert_eq!(server.client_view_owner, Some(1));
    assert_eq!(server.app.state.active, Some(0));

    server.focus_client_view_for_input(2);
    assert_eq!(server.client_view_owner, Some(2));
    assert_eq!(server.app.state.active, Some(1));

    insert_test_terminal_attach_client(&mut server, 3, "term");
    server.focus_client_view_for_input(3);
    assert_eq!(server.client_view_owner, Some(2));
    assert_eq!(server.app.state.active, Some(1));
}

#[test]
fn client_resize_from_background_client_does_not_mark_owners_tabs_seen() {
    let mut server = server_with_workspaces_and_clients(&["one", "two"]);

    server.focus_client_view(1);
    server.focus_client_view(2);
    server.app.state.switch_workspace(1);
    server.focus_client_view(1);
    assert_eq!(server.client_view_owner, Some(1));
    assert_eq!(server.foreground_client_id, Some(1));
    assert_eq!(server.app.state.active, Some(0));

    let owner_pane = *server.app.state.workspaces[0]
        .active_tab()
        .expect("workspace must have an active tab")
        .panes
        .keys()
        .next()
        .expect("active tab must have a pane");
    server.app.state.workspaces[0]
        .active_tab_mut()
        .expect("workspace must have an active tab")
        .panes
        .get_mut(&owner_pane)
        .expect("pane must exist")
        .seen = false;

    server.handle_server_event(ServerEvent::ClientResize {
        client_id: 2,
        cols: 100,
        rows: 30,
        cell_width_px: 0,
        cell_height_px: 0,
    });

    assert!(
        !server.app.state.workspaces[0]
            .active_tab()
            .expect("workspace must have an active tab")
            .panes
            .get(&owner_pane)
            .expect("pane must exist")
            .seen
    );
}

#[test]
fn single_client_focus_adopts_view_and_takes_ownership() {
    let mut server = server_with_workspaces_and_clients(&["one"]);
    server.clients.remove(&2);
    assert!(server.clients[&1].view.is_none());
    assert_eq!(server.client_view_owner, None);

    server.focus_client_view(1);

    assert_eq!(server.client_view_owner, Some(1));
    assert_eq!(
        server.clients[&1]
            .view
            .as_ref()
            .and_then(|view| view.active_workspace_id.clone()),
        server.app.state.workspaces.first().map(|ws| ws.id.clone())
    );
}

#[test]
fn removing_last_client_clears_view_owner_and_foreground() {
    let mut server = server_with_workspaces_and_clients(&["one"]);
    server.clients.remove(&2);
    server.focus_client_view(1);
    assert_eq!(server.client_view_owner, Some(1));
    assert_eq!(server.foreground_client_id, Some(1));

    server.remove_client(1);

    assert_eq!(server.client_view_owner, None);
    assert_eq!(server.foreground_client_id, None);
}
