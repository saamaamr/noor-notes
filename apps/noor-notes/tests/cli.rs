use std::process::Command;

fn run_without_desktop(argument: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_noor-notes"))
        .arg(argument)
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .expect("Noor Notes binary should run")
}

#[test]
fn help_does_not_require_a_graphical_session_or_secret_service() {
    let output = run_without_desktop("--help");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Noor Notes"));
    assert!(stdout.contains("--version"));
}

#[test]
#[cfg(not(feature = "development"))]
fn version_does_not_require_a_graphical_session_or_secret_service() {
    let output = run_without_desktop("--version");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Noor Notes 1.1.2"
    );
}

#[cfg(feature = "development")]
#[test]
fn development_build_is_clearly_branded_in_cli_output() {
    let version = run_without_desktop("--version");
    let help = run_without_desktop("--help");

    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        "Noor Notes Dev 1.1.2"
    );
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.starts_with("Noor Notes Dev 1.1.2"));
    assert!(help.contains("Usage: noor-notes-dev [OPTION]"));
}
