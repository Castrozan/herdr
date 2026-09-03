use std::time::{Duration, Instant};

use bytes::Bytes;

use super::App;

const CHECK_INTERVAL: Duration = Duration::from_millis(100);
const TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) struct PendingAgentContinuation {
    agent: crate::detect::Agent,
    prompt: String,
    expires_at: Instant,
}

impl App {
    pub(crate) fn queue_agent_continuation(
        &mut self,
        terminal_id: crate::terminal::TerminalId,
        agent: crate::detect::Agent,
        prompt: String,
    ) {
        let now = Instant::now();
        self.pending_agent_continuations.insert(
            terminal_id,
            PendingAgentContinuation {
                agent,
                prompt,
                expires_at: now + TIMEOUT,
            },
        );
        self.pending_agent_continuation_deadline = Some(now);
    }

    pub(crate) fn deliver_pending_agent_continuations(&mut self, now: Instant) -> bool {
        let terminal_ids = self
            .pending_agent_continuations
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut completed = Vec::new();
        let mut changed = false;

        for terminal_id in terminal_ids {
            let Some(pending) = self.pending_agent_continuations.get(&terminal_id) else {
                continue;
            };
            if now >= pending.expires_at {
                tracing::warn!(terminal = %terminal_id, agent = ?pending.agent, "agent continuation prompt timed out");
                completed.push(terminal_id);
                continue;
            }
            let ready = self
                .state
                .terminals
                .get(&terminal_id)
                .is_some_and(|terminal| {
                    terminal.state == crate::detect::AgentState::Idle
                        && terminal.effective_known_agent() == Some(pending.agent)
                });
            if !ready {
                continue;
            }
            let Some(runtime) = self.terminal_runtimes.get(&terminal_id) else {
                continue;
            };
            let mut input = pending.prompt.clone();
            input.push('\r');
            if runtime.try_send_bytes(Bytes::from(input)).is_ok() {
                completed.push(terminal_id);
                changed = true;
            }
        }

        for terminal_id in completed {
            self.pending_agent_continuations.remove(&terminal_id);
        }
        self.pending_agent_continuation_deadline = self
            .pending_agent_continuations
            .values()
            .map(|pending| pending.expires_at.min(now + CHECK_INTERVAL))
            .min();
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn continuation_is_sent_only_after_the_resumed_agent_is_idle() {
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(
            &crate::config::Config::default(),
            true,
            None,
            api_rx,
            crate::api::EventHub::default(),
        );
        let workspace = crate::workspace::Workspace::test_new("continuation");
        let pane_id = workspace.tabs[0].root_pane;
        let terminal_id = workspace.terminal_id(pane_id).cloned().unwrap();
        app.state.workspaces = vec![workspace];
        app.state.ensure_test_terminals();
        let (runtime, mut receiver) = crate::terminal::TerminalRuntime::test_with_channel(80, 24);
        app.terminal_runtimes.insert(terminal_id.clone(), runtime);
        app.queue_agent_continuation(
            terminal_id.clone(),
            crate::detect::Agent::Codex,
            "continue".into(),
        );

        assert!(!app.deliver_pending_agent_continuations(Instant::now()));
        assert!(receiver.try_recv().is_err());
        app.state
            .terminals
            .get_mut(&terminal_id)
            .unwrap()
            .set_detected_state(
                Some(crate::detect::Agent::Codex),
                crate::detect::AgentState::Idle,
            );

        assert!(app.deliver_pending_agent_continuations(Instant::now()));
        assert_eq!(receiver.recv().await.unwrap(), Bytes::from("continue\r"));
        assert!(!app.pending_agent_continuations.contains_key(&terminal_id));
    }
}
