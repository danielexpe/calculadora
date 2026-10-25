use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_calculadora"))
}

#[test]
fn help_exits_successfully() {
    let output = binary().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--lang"));
    assert!(stdout.contains("--backend"));
    assert!(stdout.contains("--capture-frame"));
}

#[test]
fn version_exits_successfully() {
    let output = binary().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("calculadora"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn invalid_backend_uses_cli_exit_code() {
    let output = binary().args(["--backend", "tui"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn missing_runtime_uses_backend_exit_code() {
    let output = binary().output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("framebuffer runtime is not implemented"));
    assert!(stderr.contains("phase 2"));
}

#[test]
fn capture_placeholder_uses_runtime_exit_code() {
    let output = binary()
        .args(["--capture-frame", "/tmp/calculadora.png"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("frame capture is not implemented"));
}
