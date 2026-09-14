//! Exercise startup without contacting the user's compositor or config.
use std::process::Command;

#[test]
fn empty_pins_reach_wayland_initialization() {
    let temp = std::env::temp_dir().join(format!("lnp-dock-startup-{}", std::process::id()));
    std::fs::create_dir_all(temp.join("lnp-dock")).unwrap();
    let pins = temp.join("lnp-dock/pins");
    std::fs::write(&pins, "# All apps have been unpinned.\n").unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_lnp-dock"))
        .env("XDG_CONFIG_HOME", &temp)
        .env("XDG_RUNTIME_DIR", &temp)
        .env("WAYLAND_DISPLAY", "missing-test-compositor")
        .env_remove("WAYLAND_SOCKET")
        .output()
        .unwrap();
    let contents = std::fs::read_to_string(&pins).unwrap();
    std::fs::remove_dir_all(&temp).unwrap();

    // An unavailable compositor is the expected boundary here; an empty pin
    // list must not cause the earlier "no launchers" error or reset the pins.
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("connecting to the Wayland compositor"), "{error}");
    assert_eq!(contents, "# All apps have been unpinned.\n");
}
