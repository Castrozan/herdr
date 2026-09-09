use tracing::warn;

use super::{parse_binding_string, ActionKeybinds, ParsedBinding, PassthroughKeybindConfig};
use crate::input::TerminalKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledPassthroughBinding {
    pub bindings: ActionKeybinds,
    pub processes: Vec<String>,
}

pub fn passthrough_processes_for_key(
    passthroughs: &[CompiledPassthroughBinding],
    key: &TerminalKey,
) -> Vec<String> {
    let mut processes = passthroughs
        .iter()
        .filter(|entry| entry.bindings.matches_direct_key(key))
        .flat_map(|entry| entry.processes.iter().cloned())
        .collect::<Vec<_>>();
    processes.sort();
    processes.dedup();
    processes
}

pub(super) fn compile_passthrough_bindings(
    entries: &[PassthroughKeybindConfig],
    diagnostics: &mut Vec<String>,
) -> Vec<CompiledPassthroughBinding> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let field = format!("keys.passthrough[{index}]");
            let mut processes = entry
                .processes
                .iter()
                .map(|process| process.trim().to_ascii_lowercase())
                .filter(|process| !process.is_empty())
                .collect::<Vec<_>>();
            processes.sort();
            processes.dedup();
            if processes.is_empty() {
                let diagnostic = format!(
                    "passthrough has no processes: {field}.processes; disabling passthrough"
                );
                warn!(message = %diagnostic, "config diagnostic");
                diagnostics.push(diagnostic);
                return None;
            }
            let mut bindings = Vec::new();
            for raw in entry.key.values() {
                let raw = raw.trim();
                if raw.is_empty() {
                    continue;
                }
                match parse_binding_string(raw) {
                    Some(ParsedBinding::Single(binding)) if binding.trigger.is_direct() => {
                        bindings.push(binding);
                    }
                    Some(ParsedBinding::Single(_)) => {
                        let diagnostic = format!(
                            "passthrough keybinding cannot use prefix: {field}.key = {raw:?}; disabling binding"
                        );
                        warn!(message = %diagnostic, "config diagnostic");
                        diagnostics.push(diagnostic);
                    }
                    Some(ParsedBinding::Range(_)) | None => {
                        let diagnostic = format!(
                            "invalid passthrough keybinding: {field}.key = {raw:?}; disabling binding"
                        );
                        warn!(message = %diagnostic, "config diagnostic");
                        diagnostics.push(diagnostic);
                    }
                }
            }
            (!bindings.is_empty()).then_some(CompiledPassthroughBinding {
                bindings: ActionKeybinds { bindings },
                processes,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use super::*;
    use crate::config::Config;

    #[test]
    fn passthrough_binding_overlaps_an_action_and_normalizes_processes() {
        let config: Config = toml::from_str(
            r#"
[keys]
previous_tab = "ctrl+pageup"

[[keys.passthrough]]
key = "ctrl+pageup"
processes = ["NVim", " nvim ", "vim"]
"#,
        )
        .unwrap();
        let keybinds = config.keybinds();
        let key = TerminalKey::new(KeyCode::PageUp, KeyModifiers::CONTROL);

        assert!(keybinds.previous_tab.matches_direct_key(&key));
        assert_eq!(
            passthrough_processes_for_key(&keybinds.passthroughs, &key),
            ["nvim", "vim"]
        );
    }

    #[test]
    fn passthrough_rejects_prefix_keys_and_empty_processes() {
        let config: Config = toml::from_str(
            r#"
[[keys.passthrough]]
key = "prefix+pageup"
processes = ["nvim"]

[[keys.passthrough]]
key = "ctrl+pagedown"
processes = []
"#,
        )
        .unwrap();

        assert!(config.keybinds().passthroughs.is_empty());
        let diagnostics = config.collect_diagnostics();
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("cannot use prefix")));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("has no processes")));
    }
}
