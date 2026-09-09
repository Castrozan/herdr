use std::time::{Duration, Instant};

use bytes::Bytes;

use super::App;

const CHECK_INTERVAL: Duration = Duration::from_millis(100);
const SUBMIT_DELAY: Duration = Duration::from_millis(300);
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
            let Some(runtime) = self.terminal_runtimes.get(&terminal_id).filter(|runtime| {
                ready && super::agents::runtime_hosts_agent(runtime, pending.agent)
            }) else {
                continue;
            };
            if runtime
                .queue_user_input_submission(
                    Bytes::from(pending.prompt.clone()),
                    Bytes::from_static(b"\r"),
                    SUBMIT_DELAY,
                    None,
                )
                .is_ok()
            {
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
mod tests;
