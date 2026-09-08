use crate::api::schema::{AgentRestartParams, AgentTarget, ResponseResult};
use crate::app::App;

use super::responses::{encode_error, encode_error_body, encode_success};

impl App {
    pub(super) fn handle_agent_restart(
        &mut self,
        id: String,
        params: AgentRestartParams,
    ) -> String {
        let resolved = match self.resolve_terminal_target(&params.target) {
            Ok(resolved) => resolved,
            Err(error) => return encode_error_body(id, self.agent_target_error_body(error)),
        };
        let Some(terminal_id) = self
            .state
            .workspaces
            .get(resolved.ws_idx)
            .and_then(|workspace| workspace.terminal_id(resolved.pane_id))
            .cloned()
        else {
            return agent_not_found(id, &params.target);
        };
        let Some(terminal) = self.state.terminals.get(&terminal_id) else {
            return agent_not_found(id, &params.target);
        };
        if terminal.managed_agent_kind().is_some() {
            return managed_agent_lifecycle_unavailable(id);
        }
        if self.terminal_runtimes.get(&terminal_id).is_none() {
            return agent_not_found(id, &params.target);
        }
        if self.state.terminal_runtime_shutdowns.contains(&terminal_id) {
            return lifecycle_in_progress(id);
        }
        let Some(session) = resumable_agent_session(terminal) else {
            return encode_error(
                id,
                "agent_restart_unavailable",
                "the agent does not have an exact recorded session",
            );
        };
        let Some(plan) =
            crate::agent_resume::plan(&session.source, &session.agent, &session.session_ref)
        else {
            return encode_error(
                id,
                "agent_restart_unavailable",
                "the recorded agent session cannot be resumed",
            );
        };
        let Some(agent) = crate::detect::parse_agent_label(&plan.agent) else {
            return encode_error(
                id,
                "agent_restart_unavailable",
                "the recorded agent is not recognized",
            );
        };

        let terminal = self.state.terminals.get_mut(&terminal_id).unwrap();
        terminal.clear_agent_runtime_identity_after_respawn();
        terminal.pending_agent_resume_plan = Some(plan);
        if let Some(prompt) = params.prompt.filter(|prompt| !prompt.is_empty()) {
            self.queue_agent_continuation(terminal_id.clone(), agent, prompt);
        }
        self.state.terminal_runtime_shutdowns.push(terminal_id);
        self.schedule_session_save();

        encode_success(id, ResponseResult::Ok {})
    }

    pub(super) fn handle_agent_exit(&mut self, id: String, target: AgentTarget) -> String {
        let resolved = match self.resolve_terminal_target(&target.target) {
            Ok(resolved) => resolved,
            Err(error) => return encode_error_body(id, self.agent_target_error_body(error)),
        };
        let Some(terminal_id) = self
            .state
            .workspaces
            .get(resolved.ws_idx)
            .and_then(|workspace| workspace.terminal_id(resolved.pane_id))
            .cloned()
        else {
            return agent_not_found(id, &target.target);
        };
        let Some(terminal) = self.state.terminals.get_mut(&terminal_id) else {
            return agent_not_found(id, &target.target);
        };
        if terminal.managed_agent_kind().is_some() {
            return managed_agent_lifecycle_unavailable(id);
        }
        if self.terminal_runtimes.get(&terminal_id).is_none() {
            return agent_not_found(id, &target.target);
        }
        if self.state.terminal_runtime_shutdowns.contains(&terminal_id) {
            return lifecycle_in_progress(id);
        }
        if terminal.effective_known_agent().is_none() && terminal.detected_agent.is_none() {
            return encode_error(
                id,
                "agent_exit_unavailable",
                "the target pane does not contain a recognized agent",
            );
        }

        terminal.clear_agent_runtime_identity_after_respawn();
        terminal.respawn_shell_on_exit = true;
        self.pending_agent_continuations.remove(&terminal_id);
        self.state.terminal_runtime_shutdowns.push(terminal_id);
        self.schedule_session_save();

        encode_success(id, ResponseResult::Ok {})
    }
}

fn lifecycle_in_progress(id: String) -> String {
    encode_error(
        id,
        "agent_lifecycle_in_progress",
        "an agent lifecycle operation is already pending",
    )
}

fn managed_agent_lifecycle_unavailable(id: String) -> String {
    encode_error(
        id,
        "managed_agent_lifecycle_unavailable",
        "managed agents must be controlled by their lifecycle owner",
    )
}

fn agent_not_found(id: String, target: &str) -> String {
    encode_error(
        id,
        "agent_not_found",
        format!("agent target {target} not found"),
    )
}

fn resumable_agent_session(
    terminal: &crate::terminal::TerminalState,
) -> Option<crate::agent_resume::PersistedAgentSession> {
    terminal
        .hook_authority
        .as_ref()
        .and_then(|authority| {
            Some(crate::agent_resume::PersistedAgentSession {
                source: authority.source.clone(),
                agent: authority.agent_label.clone(),
                session_ref: authority.session_ref.clone()?,
            })
        })
        .or_else(|| terminal.persisted_agent_session.clone())
}

#[cfg(test)]
mod tests;
