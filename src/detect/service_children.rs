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
        .is_some_and(|arguments| contains_service_subcommand(arguments.iter().map(String::as_str)))
        || process.cmdline.as_deref().is_some_and(|command_line| {
            contains_service_subcommand(command_line.split_whitespace())
        })
}

fn contains_service_subcommand<'a>(arguments: impl IntoIterator<Item = &'a str>) -> bool {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let mut index = 1;
    while let Some(argument) = arguments.get(index) {
        let argument = argument.trim_matches(['"', '\'']);
        if is_service_token(argument) {
            return true;
        }
        if argument == "--" || !argument.starts_with('-') {
            return false;
        }
        index += if option_takes_value(argument) { 2 } else { 1 };
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
