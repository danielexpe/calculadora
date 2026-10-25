use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use thiserror::Error;

use crate::cli::Cli;
use crate::core::Calculator;
use crate::i18n::SoundStatus;
use crate::input::{map_key, Action};
use crate::platform::fb::{
    install_panic_hook, install_signal_handlers, quit_requested, ConsoleGuard, Framebuffer,
};
use crate::render::fb::compose_device;
use crate::render::layout::{self, KeyId};
use crate::render::scene::{calculator_scene, CalculatorView};
use crate::render::text::TextEngine;
use crate::state::SessionState;
use crate::theme::Theme;
use crate::util::tty::{stdin_is_tty, RawModeGuard};

const IDLE_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug, Error)]
pub enum RunError {
    #[error("{0}")]
    Backend(String),
    #[error("{0}")]
    Runtime(String),
}

impl RunError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Backend(_) => 3,
            Self::Runtime(_) => 1,
        }
    }
}

pub fn run_fb(cli: &Cli) -> Result<(), RunError> {
    let mut framebuffer = Framebuffer::open(&cli.fb_device).map_err(backend)?;

    if !stdin_is_tty() {
        return Err(RunError::Backend(
            "stdin is not a TTY; run calculadora from an interactive local console".to_string(),
        ));
    }

    install_panic_hook();
    install_signal_handlers().map_err(backend)?;

    let console = ConsoleGuard::enter().map_err(backend)?;
    let raw = RawModeGuard::enter().map_err(backend)?;

    let bpp = framebuffer.info().format.bpp;
    let theme = Theme::new(cli.effective_color_from_env(), bpp);
    let engine = TextEngine::new().map_err(|error| RunError::Runtime(error.to_string()))?;
    let mut session = SessionState::new(cli.lang);
    if cli.no_audio {
        session.toggle_mute();
    }
    let calculator = Calculator::new();

    draw(&mut framebuffer, &engine, &theme, &calculator, &session)?;

    while !quit_requested() {
        match event::poll(IDLE_TIMEOUT) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(runtime(error)),
        }

        let key = match event::read() {
            Ok(Event::Key(key)) => key,
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(runtime(error)),
        };

        let mut rebuild = false;
        match map_key(key) {
            Action::Quit => break,
            Action::ToggleLanguage => {
                session.toggle_language();
                rebuild = true;
            }
            Action::ToggleMute => {
                session.toggle_mute();
                rebuild = true;
            }
            _ => {}
        }

        if rebuild {
            draw(&mut framebuffer, &engine, &theme, &calculator, &session)?;
        }
    }

    drop(raw);
    drop(console);
    Ok(())
}

fn draw(
    framebuffer: &mut Framebuffer,
    engine: &TextEngine,
    theme: &Theme,
    calculator: &Calculator,
    session: &SessionState,
) -> Result<(), RunError> {
    let info = *framebuffer.info();
    let locale = session.lang().number_locale();
    let selection = session.selected(calculator.history().len());
    let sound = if session.muted() {
        SoundStatus::Off
    } else {
        SoundStatus::On
    };

    let view = CalculatorView {
        lang: session.lang(),
        theme,
        display: calculator.display(locale),
        expression: calculator.expression(locale),
        history: calculator.history(),
        selection,
        sound,
        pressed: None::<KeyId>,
    };

    let scale = layout::scale_for(info.width, info.height);
    let pixmap = compose_device(
        &calculator_scene(&view),
        engine,
        info.width,
        info.height,
        scale,
    );
    framebuffer.present(pixmap.data()).map_err(runtime)
}

fn backend(error: impl std::fmt::Display) -> RunError {
    RunError::Backend(error.to_string())
}

fn runtime(error: impl std::fmt::Display) -> RunError {
    RunError::Runtime(error.to_string())
}
