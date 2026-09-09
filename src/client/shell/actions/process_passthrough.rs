use super::*;

impl ClientShellState {
    pub(in crate::client::shell) fn defer_passthrough_key(
        &mut self,
        key: &crate::input::TerminalKey,
        binding: crate::input::KeybindMatch,
        outcome: &mut ClientShellInput,
    ) -> bool {
        let processes = crate::config::passthrough_processes_for_key(
            &self.config.keybinds.keybinds.passthroughs,
            key,
        );
        if processes.is_empty() {
            return false;
        }
        let Some(pane_id) = self.focused_pane_id() else {
            return false;
        };
        self.push_endpoint_method_with_kind(
            crate::api::schema::Method::PaneProcessInfo(
                crate::api::schema::PaneProcessInfoParams {
                    pane_id: Some(pane_id.clone()),
                },
            ),
            PendingEndpointKind::Passthrough {
                pane_id,
                key: key.clone(),
                binding,
                processes,
            },
            outcome,
        )
    }

    pub(in crate::client::shell) fn complete_passthrough_key(
        &mut self,
        pane_id: String,
        key: crate::input::TerminalKey,
        binding: crate::input::KeybindMatch,
        processes: Vec<String>,
        result: &Result<crate::api::schema::ResponseResult, ClientShellEndpointError>,
    ) -> (bool, Vec<ClientShellAction>) {
        if matches!(
            result,
            Err(error) if error.code.as_deref() == Some("endpoint_cancelled")
        ) || self.mode != ClientShellMode::Terminal
            || self.overlay.is_some()
            || self.focused_pane_id().as_deref() != Some(pane_id.as_str())
        {
            return (false, Vec::new());
        }
        let foreground_process_names = match result {
            Ok(crate::api::schema::ResponseResult::PaneProcessInfo { process_info })
                if process_info.pane_id == pane_id =>
            {
                process_info
                    .foreground_processes
                    .iter()
                    .flat_map(|process| {
                        [
                            Some(process.name.to_ascii_lowercase()),
                            process
                                .argv0
                                .as_deref()
                                .map(|argv0| path_basename(argv0).to_ascii_lowercase()),
                        ]
                        .into_iter()
                        .flatten()
                    })
                    .collect::<Vec<_>>()
            }
            _ => Vec::new(),
        };
        let claimed = processes.iter().any(|expected| {
            foreground_process_names
                .iter()
                .any(|actual| actual == expected)
        });
        if claimed {
            let actions = crate::protocol::ClientPaneInputEvent::from_terminal_key(key)
                .map(|event| ClientShellAction::PaneInput {
                    endpoint_id: self.active_endpoint_id.clone(),
                    pane_id,
                    event,
                })
                .into_iter()
                .collect();
            return (false, actions);
        }
        let mut outcome = ClientShellInput::default();
        self.record_binding(binding, &mut outcome);
        (outcome.repaint, outcome.actions)
    }
}

fn path_basename(path: &str) -> &str {
    path.rsplit(['/', '\\'])
        .find(|component| !component.is_empty())
        .unwrap_or(path)
}
