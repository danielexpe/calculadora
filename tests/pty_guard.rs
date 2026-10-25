use std::io::IsTerminal;
use std::process::Command;

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn stty() -> String {
    let output = Command::new("stty")
        .args(["-a", "-F", "/dev/tty"])
        .output()
        .expect("run stty");
    assert!(output.status.success(), "stty failed");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn raw_mode_guard_restores_termios_in_pty() {
    if Command::new("script").arg("--version").output().is_err() {
        eprintln!("skipping PTY test: script is not available");
        return;
    }

    let executable = std::env::current_exe().expect("test executable path");
    let command = format!(
        "{} --exact raw_mode_guard_child --ignored --nocapture",
        shell_quote(&executable.display().to_string())
    );
    let output = Command::new("script")
        .args(["-q", "-e", "-c", &command, "/dev/null"])
        .output()
        .expect("run test in a pty");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "pty child failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("PTY_RAW_OK"), "missing marker:\n{stdout}");
}

#[test]
#[ignore]
fn raw_mode_guard_child() {
    if !std::io::stdin().is_terminal() {
        eprintln!("child requires a TTY");
        std::process::exit(2);
    }

    let before = stty();
    let guard = calculadora::util::tty::RawModeGuard::enter().expect("enter raw mode");
    let raw = stty();
    assert_ne!(before, raw, "raw mode did not change termios");
    assert!(raw.contains("-icanon"), "raw mode missing -icanon: {raw}");
    assert!(raw.contains("-echo"), "raw mode missing -echo: {raw}");

    drop(guard);
    let after = stty();
    assert_eq!(before, after, "termios was not restored");
    println!("PTY_RAW_OK");
}
