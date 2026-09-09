use super::Agent;

pub(super) fn is_background_agent_service(
    process: &crate::platform::ForegroundProcess,
    agent: Agent,
) -> bool {
    if agent != Agent::Codex {
        return false;
    }

    if let Some(arguments) = process.argv.as_deref() {
        return contains_service_subcommand(arguments.iter().map(String::as_str));
    }

    process
        .cmdline
        .as_deref()
        .is_some_and(|command_line| command_line.split_whitespace().any(is_service_token))
}

fn contains_service_subcommand<'a>(arguments: impl IntoIterator<Item = &'a str>) -> bool {
    let mut arguments = arguments.into_iter().skip(1);
    while let Some(argument) = arguments.next() {
        let argument = argument.trim_matches(['"', '\'']);
        if is_service_token(argument) {
            return true;
        }
        if argument == "--" || !argument.starts_with('-') {
            return false;
        }
        if option_takes_value(argument) {
            arguments.next();
        }
    }
    false
}

fn option_takes_value(option: &str) -> bool {
    !option.contains('=')
        && matches!(
            option,
            "-c" | "--config"
                | "--enable"
                | "--disable"
                | "--remote"
                | "--remote-auth-token-env"
                | "-i"
                | "--image"
                | "-m"
                | "--model"
                | "--local-provider"
                | "-p"
                | "--profile"
                | "-s"
                | "--sandbox"
                | "-C"
                | "--cd"
                | "--add-dir"
                | "-a"
                | "--ask-for-approval"
        )
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
