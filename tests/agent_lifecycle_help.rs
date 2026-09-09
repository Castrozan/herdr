use std::process::Command;

#[test]
fn agent_help_lists_native_lifecycle_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_herdr"))
        .args(["agent", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("restart"), "agent help: {stdout}");
    assert!(stdout.contains("exit"), "agent help: {stdout}");
}
