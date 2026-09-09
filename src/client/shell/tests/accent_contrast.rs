use super::*;

#[test]
fn focused_tab_foreground_contrasts_with_light_and_dark_accents() {
    fn focused_tab_foreground(accent: ratatui::style::Color) -> Option<ratatui::style::Color> {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.palette.accent = accent;
        let mut state = ClientShellState::new(config);
        state.set_snapshot(Box::new(snapshot()));
        state.set_pane_surface(surface());
        let frame = state.compose(106, 20).expect("tab frame");
        let tab = state.hits.tabs[0].0;
        frame.to_ratatui_buffer().expect("tab buffer")[(tab.x + 1, tab.y)]
            .style()
            .fg
    }

    assert_eq!(
        focused_tab_foreground(ratatui::style::Color::Rgb(230, 230, 230)),
        Some(ratatui::style::Color::Black)
    );
    assert_eq!(
        focused_tab_foreground(ratatui::style::Color::Rgb(20, 25, 40)),
        Some(ratatui::style::Color::White)
    );
}
