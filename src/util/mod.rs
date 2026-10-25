pub mod tty;

pub use tty::{ensure_stdin_tty, ensure_tty, stdin_is_tty, RawModeGuard, TtyError};
