use super::*;
use crate::api::schema::{EventData, ResponseResult, SuccessResponse, TabRenameParams};
use crate::config::Config;
use crate::workspace::Workspace;

#[test]
fn clearing_tab_name_restores_the_single_pane_terminal_title() {
    let event_hub = crate::api::EventHub::default();
    let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(
        &Config::default(),
        crate::app::AppPolicy::TEST,
        None,
        api_rx,
        event_hub.clone(),
    );
    app.state.workspaces = vec![Workspace::test_new("tabs")];
    app.state.ensure_test_terminals();
    let tab_id = app.public_tab_id(0, 0).unwrap();
    let pane_id = app.state.workspaces[0].tabs[0].root_pane;
    let terminal_id = app.state.workspaces[0]
        .terminal_id(pane_id)
        .cloned()
        .unwrap();
    app.state
        .terminals
        .get_mut(&terminal_id)
        .unwrap()
        .set_terminal_title(Some("⠋ terminal task".into()));
    app.handle_tab_rename(
        "name".into(),
        TabRenameParams {
            tab_id: tab_id.clone(),
            label: "mine".into(),
        },
    );

    let response = app.handle_tab_rename(
        "clear".into(),
        TabRenameParams {
            tab_id,
            label: String::new(),
        },
    );

    let success: SuccessResponse = serde_json::from_str(&response).unwrap();
    let ResponseResult::TabInfo { tab } = success.result else {
        panic!("expected tab info");
    };
    assert_eq!(tab.label, "terminal task");
    assert!(app.state.workspaces[0].tabs[0].is_auto_named());
    assert!(event_hub.events_after(0).iter().any(|(_, event)| matches!(
        &event.data,
        EventData::TabRenamed { label, .. } if label == "terminal task"
    )));
}
