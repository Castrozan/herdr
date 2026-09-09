use super::*;

#[test]
fn local_keybindings_profile_preserves_process_passthroughs() {
    let config: Config = toml::from_str(
        r#"
[[keys.passthrough]]
key = "ctrl+pageup"
processes = ["nvim"]
"#,
    )
    .unwrap();

    let profile = config.local_keybindings_profile_toml().unwrap();
    assert!(profile.contains("[[keys.passthrough]]"));
    assert!(profile.contains("processes = [\"nvim\"]"));
}
