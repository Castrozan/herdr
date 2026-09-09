use super::*;

impl Workspace {
    pub fn active_tab_display_name_from(
        &self,
        terminals: &HashMap<TerminalId, TerminalState>,
    ) -> Option<String> {
        if self.active_tab()?.custom_name.is_some() {
            self.active_tab_display_name()
        } else {
            self.tab_display_name_from(self.active_tab, terminals)
        }
    }

    pub fn tab_display_name_from(
        &self,
        tab_idx: usize,
        terminals: &HashMap<TerminalId, TerminalState>,
    ) -> Option<String> {
        let tab = self.tabs.get(tab_idx)?;
        if let Some(name) = &tab.custom_name {
            return Some(name.clone());
        }
        let mut panes = tab.panes.values();
        let terminal_title = panes.next().and_then(|pane| {
            panes
                .next()
                .is_none()
                .then(|| {
                    terminals
                        .get(&pane.attached_terminal_id)?
                        .border_label(false)
                })
                .flatten()
        });
        Some(terminal_title.unwrap_or_else(|| (tab_idx + 1).to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_pane_tab_uses_terminal_title_until_manually_named() {
        let mut workspace = Workspace::test_new("test");
        let pane_id = workspace.tabs[0].root_pane;
        let terminal_id = workspace.terminal_id(pane_id).cloned().unwrap();
        let mut terminal = TerminalState::new(terminal_id.clone(), PathBuf::from("/tmp"));
        terminal.set_terminal_title(Some("⠋ terminal task".into()));
        let terminals = HashMap::from([(terminal_id, terminal)]);

        assert_eq!(
            workspace.tab_display_name_from(0, &terminals).as_deref(),
            Some("terminal task")
        );
        workspace.tabs[0].set_custom_name("mine".into());
        assert_eq!(
            workspace.tab_display_name_from(0, &terminals).as_deref(),
            Some("mine")
        );
        workspace.tabs[0].set_custom_name(String::new());
        assert!(workspace.tabs[0].is_auto_named());
        assert_eq!(
            workspace.tab_display_name_from(0, &terminals).as_deref(),
            Some("terminal task")
        );
        workspace.test_split(Direction::Vertical);
        assert_eq!(
            workspace.tab_display_name_from(0, &terminals).as_deref(),
            Some("1")
        );
    }
}
