use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn sigterm_sets_quit_flag_in_child() {
    let executable = std::env::current_exe().expect("test executable path");
    let output = Command::new(executable)
        .args(["--exact", "signal_child", "--ignored", "--nocapture"])
        .output()
        .expect("run signal child");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "signal child failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("SIGNAL_OK"), "missing marker:\n{stdout}");
}

#[test]
#[ignore]
fn signal_child() {
    calculadora::platform::fb::install_signal_handlers().expect("install handlers");
    assert!(!calculadora::platform::fb::quit_requested());

    let status = Command::new("kill")
        .args(["-TERM", &std::process::id().to_string()])
        .status()
        .expect("send SIGTERM");
    assert!(status.success());

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if calculadora::platform::fb::quit_requested() {
            println!("SIGNAL_OK");
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    panic!("SIGTERM did not set the quit flag");
}
