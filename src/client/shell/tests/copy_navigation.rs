use super::*;
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn control_shift_arrows_jump_seven_rows_in_copy_mode() {
    for (code, initial_row, initial_offset, expected_row, expected_offset) in
        [(KeyCode::Up, 21, 0, 14, 6), (KeyCode::Down, 0, 20, 7, 14)]
    {
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
        state.set_snapshot(Box::new(snapshot()));
        let mut pane_surface = surface();
        pane_surface.panes[0].scroll = Some(crate::protocol::PaneSurfaceScrollMetrics {
            offset_from_bottom: initial_offset,
            max_offset_from_bottom: 20,
            viewport_rows: 2,
        });
        state.set_pane_surface(pane_surface);
        state.compose(106, 20).expect("composed frame");
        assert!(state.enter_copy_mode(&mut ClientShellInput::default()));
        state.copy_mode.as_mut().expect("copy mode").cursor.row = initial_row;

        let outcome = state.handle_raw_events(vec![RawInputEvent::Key(
            crate::input::TerminalKey::new(code, KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        )]);

        assert_eq!(
            state.copy_mode.as_ref().map(|mode| mode.cursor.row),
            Some(expected_row)
        );
        assert!(matches!(
            &outcome.actions[..],
            [ClientShellAction::Endpoint { request, .. }]
                if matches!(
                    &request.method,
                    crate::api::schema::Method::PaneScroll(params)
                        if params.offset_from_bottom == expected_offset
                )
        ));
    }
}

#[test]
fn copy_mode_control_arrows_use_word_motions() {
    for (code, expected) in [
        (
            KeyCode::Left,
            crate::api::schema::PaneCopyMotion::PreviousWordStart,
        ),
        (
            KeyCode::Right,
            crate::api::schema::PaneCopyMotion::NextWordStart,
        ),
    ] {
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
        state.set_snapshot(Box::new(snapshot()));
        let mut pane_surface = surface();
        pane_surface.panes[0].scroll = Some(crate::protocol::PaneSurfaceScrollMetrics {
            offset_from_bottom: 0,
            max_offset_from_bottom: 0,
            viewport_rows: 2,
        });
        state.set_pane_surface(pane_surface);
        state.compose(106, 20).expect("composed frame");
        let mut enter = ClientShellInput::default();
        state.record_binding(
            crate::input::KeybindMatch::Action(crate::input::KeybindAction::CopyMode),
            &mut enter,
        );
        let origin = state.copy_mode.as_ref().expect("copy mode").cursor;

        let outcome = state.handle_raw_events(vec![RawInputEvent::Key(
            crate::input::TerminalKey::new(code, KeyModifiers::CONTROL),
        )]);

        assert!(matches!(
            &outcome.actions[..],
            [ClientShellAction::Endpoint { request, .. }]
                if matches!(
                    &request.method,
                    crate::api::schema::Method::PaneCopyMotion(params)
                        if params.cursor == origin && params.motion == expected
                )
        ));
    }
}
