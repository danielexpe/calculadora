use std::io::IsTerminal;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TtyError {
    #[error("stdin is not a TTY")]
    NotATty,
    #[error("cannot enable raw mode: {0}")]
    RawMode(#[from] std::io::Error),
}

pub fn stdin_is_tty() -> bool {
    std::io::stdin().is_terminal()
}

pub fn ensure_stdin_tty() -> Result<(), TtyError> {
    ensure_tty(stdin_is_tty())
}

pub fn ensure_tty(is_tty: bool) -> Result<(), TtyError> {
    if is_tty {
        Ok(())
    } else {
        Err(TtyError::NotATty)
    }
}

pub struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    pub fn enter() -> Result<Self, TtyError> {
        ensure_stdin_tty()?;
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self { active: true })
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = crossterm::terminal::disable_raw_mode();
            self.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_tty_accepts_true() {
        assert!(ensure_tty(true).is_ok());
    }

    #[test]
    fn ensure_tty_rejects_false() {
        assert!(matches!(ensure_tty(false), Err(TtyError::NotATty)));
    }

    #[test]
    fn error_message_is_actionable() {
        assert_eq!(TtyError::NotATty.to_string(), "stdin is not a TTY");
    }
}
