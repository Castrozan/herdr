use super::*;

#[test]
fn agent_lifecycle_requests_deserialize() {
    let restart = serde_json::json!({
        "id": "restart",
        "method": "agent.restart",
        "params": {"target": "w1:p1", "prompt": "continue"}
    });
    let exit = serde_json::json!({
        "id": "exit",
        "method": "agent.exit",
        "params": {"target": "w1:p1"}
    });

    assert!(serde_json::from_value::<Request>(restart).is_ok());
    assert!(serde_json::from_value::<Request>(exit).is_ok());
}
