use super::*;

#[test]
fn identify_agent_in_job_ignores_codex_mcp_server_child_of_claude() {
    let codex_service =
        foreground_process(11, "codex", &["codex", "--model", "gpt-5.5", "mcp-server"]);
    let claude = foreground_process(12, "claude", &["claude", "--model", "opus"]);

    for processes in [
        vec![codex_service.clone(), claude.clone()],
        vec![claude.clone(), codex_service.clone()],
    ] {
        let job = crate::platform::ForegroundJob {
            process_group_id: 10,
            processes,
        };

        assert_eq!(
            identify_agent_in_job(&job),
            Some((Agent::Claude, "claude".to_string()))
        );
    }
}

#[test]
fn identify_agent_in_job_ignores_bare_codex_mcp_server() {
    let job = crate::platform::ForegroundJob {
        process_group_id: 11,
        processes: vec![foreground_process(
            11,
            "codex",
            &["codex", "--model", "gpt-5.5", "mcp-server"],
        )],
    };

    assert_eq!(identify_agent_in_job(&job), None);
}

#[test]
fn identify_agent_in_job_ignores_codex_app_server_child_of_claude() {
    let job = crate::platform::ForegroundJob {
        process_group_id: 20,
        processes: vec![
            foreground_process(21, "codex", &["codex", "app-server"]),
            foreground_process(22, "claude", &["claude"]),
        ],
    };

    assert_eq!(
        identify_agent_in_job(&job),
        Some((Agent::Claude, "claude".to_string()))
    );
}
