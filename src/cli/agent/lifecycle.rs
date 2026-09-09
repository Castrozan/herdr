use crate::api::schema::{AgentRestartParams, AgentTarget, Method, Request};

pub(super) fn restart(args: &[String]) -> std::io::Result<i32> {
    let mut prompt = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--prompt" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --prompt");
                    return Ok(2);
                };
                prompt = Some(value.clone());
                index += 2;
            }
            "help" | "--help" | "-h" => {
                eprintln!("usage: herdr agent restart [--prompt TEXT]");
                return Ok(0);
            }
            value => {
                eprintln!("unknown option: {value}");
                return Ok(2);
            }
        }
    }
    let Some(target) = current_agent_target() else {
        return Ok(1);
    };

    super::super::print_response(&super::super::send_request(&Request {
        id: "cli:agent:restart".into(),
        method: Method::AgentRestart(AgentRestartParams { target, prompt }),
    })?)
}

pub(super) fn exit(args: &[String]) -> std::io::Result<i32> {
    if !args.is_empty() {
        eprintln!("usage: herdr agent exit");
        return Ok(2);
    }
    let Some(target) = current_agent_target() else {
        return Ok(1);
    };

    super::super::print_response(&super::super::send_request(&Request {
        id: "cli:agent:exit".into(),
        method: Method::AgentExit(AgentTarget { target }),
    })?)
}

fn current_agent_target() -> Option<String> {
    std::env::var("HERDR_PANE_ID")
        .ok()
        .filter(|target| !target.trim().is_empty())
        .or_else(|| {
            eprintln!("agent lifecycle commands require the caller's HERDR_PANE_ID");
            None
        })
}
