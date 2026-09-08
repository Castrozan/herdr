use super::Agent;

pub(super) fn is_background_agent_service(
    process: &crate::platform::ForegroundProcess,
    agent: Agent,
) -> bool {
    if agent != Agent::Codex {
        return false;
    }

    process
        .argv
        .as_deref()
        .is_some_and(|arguments| arguments.iter().any(|argument| is_service_token(argument)))
        || process
            .cmdline
            .as_deref()
            .is_some_and(|command_line| command_line.split_whitespace().any(is_service_token))
}

fn is_service_token(token: &str) -> bool {
    matches!(
        token
            .trim_matches(['"', '\''])
            .to_ascii_lowercase()
            .as_str(),
        "mcp-server" | "app-server"
    )
}
