use std::path::PathBuf;
use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_calculadora"))
}

fn capture_path(tag: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("calculadora-{tag}-{}.png", std::process::id()));
    path
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
    let output = binary()
        .args(["--fb-device", "/nonexistent/calculadora-fb0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("error:"));
    assert!(stderr.contains("/nonexistent/calculadora-fb0"));
}

#[test]
fn capture_writes_deterministic_png_without_tty_or_fb() {
    let path = capture_path("frame");
    let first = binary()
        .args(["--capture-frame", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(first.status.success());
    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(stdout.contains("1344x756"));
    assert!(path.is_file());

    let first_bytes = std::fs::read(&path).unwrap();
    let decoded = image::load_from_memory(&first_bytes).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (1344, 756));

    let second = binary()
        .args(["--capture-frame", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(second.status.success());
    let second_bytes = std::fs::read(&path).unwrap();
    assert_eq!(first_bytes, second_bytes);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn capture_accepts_language_and_color_flags() {
    let path = capture_path("mono-en");
    let output = binary()
        .args([
            "--lang",
            "en",
            "--color",
            "never",
            "--capture-frame",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(path.is_file());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn capture_ignores_framebuffer_device() {
    let path = capture_path("no-fb");
    let output = binary()
        .args([
            "--fb-device",
            "/nonexistent/calculadora-fb0",
            "--capture-frame",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(path.is_file());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn capture_reports_unwritable_path_with_runtime_code() {
    let output = binary()
        .args(["--capture-frame", "/nonexistent/directory/frame.png"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("/nonexistent/directory/frame.png"));
}
