use super::*;

#[test]
fn agent_lifecycle_requests_round_trip() {
    let restart = Request {
        id: "restart".into(),
        method: Method::AgentRestart(AgentRestartParams {
            target: "w1:p1".into(),
            prompt: Some("continue".into()),
        }),
    };
    let exit = Request {
        id: "exit".into(),
        method: Method::AgentExit(AgentTarget {
            target: "w1:p1".into(),
        }),
    };

    let restart_json = serde_json::to_string(&restart).unwrap();
    let exit_json = serde_json::to_string(&exit).unwrap();

    assert_eq!(
        serde_json::from_str::<Request>(&restart_json).unwrap(),
        restart
    );
    assert_eq!(serde_json::from_str::<Request>(&exit_json).unwrap(), exit);
    assert!(restart_json.contains("\"method\":\"agent.restart\""));
    assert!(exit_json.contains("\"method\":\"agent.exit\""));
}
