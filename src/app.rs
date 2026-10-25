use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use thiserror::Error;

use crate::audio::{Audio, SoundEvent};
use crate::cli::Cli;
use crate::core::{CalcAction, CalcEvent, Calculator};
use crate::i18n::{Lang, SoundStatus};
use crate::input::{map_key, Action};
use crate::platform::fb::{
    install_panic_hook, install_signal_handlers, quit_requested, ConsoleGuard, Framebuffer,
};
use crate::render::fb::Compositor;
use crate::render::layout::{self, KeyId};
use crate::render::scene::{calculator_scene, CalculatorView, Scene};
use crate::render::text::TextEngine;
use crate::state::SessionState;
use crate::theme::Theme;
use crate::util::tty::{stdin_is_tty, RawModeGuard};

const IDLE_TIMEOUT: Duration = Duration::from_millis(100);
const ANIMATION_TIMEOUT: Duration = Duration::from_millis(16);
const CONFIRM_DEBOUNCE: Duration = Duration::from_millis(120);
const FLASH_DURATION: Duration = Duration::from_millis(90);
const FLASH_REDUCED: Duration = Duration::from_millis(60);
const HISTORY_FADE: Duration = Duration::from_millis(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Unchanged,
    Changed,
    Quit,
}

#[derive(Debug, Clone, Copy)]
struct Flash {
    key: KeyId,
    started: Instant,
}

#[derive(Debug, Clone, Copy)]
struct FadeAnimation {
    started: Instant,
    progress: f32,
}

pub struct App {
    calculator: Calculator,
    session: SessionState,
    animations: bool,
    sound: SoundStatus,
    flash: Option<Flash>,
    history_fade: Option<FadeAnimation>,
    last_confirm: Option<Instant>,
    last_event: CalcEvent,
    sound_request: Option<SoundEvent>,
}

impl App {
    pub fn new(lang: Lang, muted: bool, animations: bool) -> Self {
        let mut session = SessionState::new(lang);
        if muted {
            session.toggle_mute();
        }
        Self {
            calculator: Calculator::new(),
            session,
            animations,
            sound: if muted {
                SoundStatus::Off
            } else {
                SoundStatus::On
            },
            flash: None,
            history_fade: None,
            last_confirm: None,
            last_event: CalcEvent::Ignored,
            sound_request: None,
        }
    }

    pub fn calculator(&self) -> &Calculator {
        &self.calculator
    }

    pub fn session(&self) -> &SessionState {
        &self.session
    }

    pub fn sound_status(&self) -> SoundStatus {
        self.sound
    }

    pub fn set_sound_status(&mut self, status: SoundStatus) {
        self.sound = status;
    }

    pub fn last_event(&self) -> CalcEvent {
        self.last_event
    }

    pub fn pressed_key(&self) -> Option<KeyId> {
        self.flash.map(|flash| flash.key)
    }

    pub fn is_animating(&self) -> bool {
        self.flash.is_some() || self.history_fade.is_some()
    }

    pub fn history_fade(&self) -> Option<f32> {
        self.history_fade.map(|fade| fade.progress)
    }

    pub fn take_sound_request(&mut self) -> Option<SoundEvent> {
        self.sound_request.take()
    }

    pub fn update(&mut self, now: Instant) -> bool {
        let mut changed = false;

        if let Some(flash) = self.flash {
            if now.saturating_duration_since(flash.started) >= self.flash_duration() {
                self.flash = None;
                changed = true;
            }
        }

        if let Some(fade) = self.history_fade {
            let progress = now.saturating_duration_since(fade.started).as_secs_f32()
                / HISTORY_FADE.as_secs_f32();
            if progress >= 1.0 {
                self.history_fade = None;
            } else {
                self.history_fade = Some(FadeAnimation {
                    started: fade.started,
                    progress,
                });
            }
            changed = true;
        }

        changed
    }

    pub fn handle(&mut self, action: Action) -> Outcome {
        self.handle_at(action, Instant::now())
    }

    pub fn handle_at(&mut self, action: Action, now: Instant) -> Outcome {
        match action {
            Action::Quit => return Outcome::Quit,
            Action::Ignored => return Outcome::Unchanged,
            Action::ToggleLanguage => {
                self.session.toggle_language();
                return Outcome::Changed;
            }
            Action::ToggleMute => {
                self.toggle_mute();
                return Outcome::Changed;
            }
            Action::HistoryPrev => {
                let before = self.session.history_selection();
                self.session
                    .select_previous(self.calculator.history().len());
                return selection_outcome(before, self.session.history_selection());
            }
            Action::HistoryNext => {
                let before = self.session.history_selection();
                self.session.select_next(self.calculator.history().len());
                return selection_outcome(before, self.session.history_selection());
            }
            Action::Equals => {
                if !self.confirm_allowed(now) {
                    return Outcome::Unchanged;
                }
                if let Some(index) = self.session.selected(self.calculator.history().len()) {
                    let result = self
                        .calculator
                        .history()
                        .get(index)
                        .expect("selected history entry exists")
                        .result;
                    self.calculator.recall(result);
                    self.session.clear_selection();
                    self.last_event = CalcEvent::Recalled;
                    self.flash = Some(Flash {
                        key: KeyId::Equals,
                        started: now,
                    });
                    self.sound_request = Some(SoundEvent::Click);
                    return Outcome::Changed;
                }
                return self.dispatch(CalcAction::Equals, KeyId::Equals, now);
            }
            _ => {}
        }

        let Some((calculation, key)) = calculation_action(action) else {
            return Outcome::Unchanged;
        };

        let had_selection = self.session.history_selection().is_some();
        self.session.clear_selection();
        let outcome = self.dispatch(calculation, key, now);
        if outcome == Outcome::Unchanged && had_selection {
            Outcome::Changed
        } else {
            outcome
        }
    }

    pub fn scene(&self, theme: &Theme) -> Scene {
        let locale = self.session.lang().number_locale();
        let selection = self.session.selected(self.calculator.history().len());
        let view = CalculatorView {
            lang: self.session.lang(),
            theme,
            display: self.calculator.display(locale),
            expression: self.calculator.expression(locale),
            history: self.calculator.history(),
            selection,
            sound: self.sound,
            pressed: self.pressed_key(),
            history_fade: self.history_fade(),
        };
        calculator_scene(&view)
    }

    pub fn timeout(&self) -> Duration {
        if self.is_animating() {
            ANIMATION_TIMEOUT
        } else {
            IDLE_TIMEOUT
        }
    }

    fn dispatch(&mut self, action: CalcAction, key: KeyId, now: Instant) -> Outcome {
        let before_len = self.calculator.history().len();
        let before_last = self.calculator.history().last().cloned();
        let event = self.calculator.handle(action);
        self.last_event = event;

        if event == CalcEvent::Ignored {
            return Outcome::Unchanged;
        }

        let after = self.calculator.history();
        let pushed = after.len() > before_len || after.last().cloned() != before_last;
        if pushed && self.animations {
            self.history_fade = Some(FadeAnimation {
                started: now,
                progress: 0.0,
            });
        }

        self.sound_request = Some(match event {
            CalcEvent::Error(_) => SoundEvent::Error,
            _ => SoundEvent::Click,
        });
        self.flash = Some(Flash { key, started: now });
        Outcome::Changed
    }

    fn toggle_mute(&mut self) {
        let muted = self.session.toggle_mute();
        if self.sound != SoundStatus::Unavailable {
            self.sound = if muted {
                SoundStatus::Off
            } else {
                SoundStatus::On
            };
        }
    }

    fn confirm_allowed(&mut self, now: Instant) -> bool {
        if let Some(last) = self.last_confirm {
            if now.saturating_duration_since(last) < CONFIRM_DEBOUNCE {
                return false;
            }
        }
        self.last_confirm = Some(now);
        true
    }

    fn flash_duration(&self) -> Duration {
        if self.animations {
            FLASH_DURATION
        } else {
            FLASH_REDUCED
        }
    }
}

fn selection_outcome(before: Option<usize>, after: Option<usize>) -> Outcome {
    if before == after {
        Outcome::Unchanged
    } else {
        Outcome::Changed
    }
}

fn calculation_action(action: Action) -> Option<(CalcAction, KeyId)> {
    let pair = match action {
        Action::Digit(digit) => (CalcAction::Digit(digit), KeyId::Digit(digit)),
        Action::DecimalSeparator => (CalcAction::DecimalSeparator, KeyId::DecimalSeparator),
        Action::Operator(operator) => (CalcAction::Operator(operator), KeyId::Operator(operator)),
        Action::Equals => (CalcAction::Equals, KeyId::Equals),
        Action::Backspace => (CalcAction::Backspace, KeyId::Backspace),
        Action::Clear => (CalcAction::Clear, KeyId::Clear),
        Action::ToggleSign => (CalcAction::ToggleSign, KeyId::ToggleSign),
        Action::Percent => (CalcAction::Percent, KeyId::Percent),
        _ => return None,
    };
    Some(pair)
}

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

    let bpp = framebuffer.info().format.bpp;
    let theme = Theme::new(cli.effective_color_from_env(), bpp);
    let engine = TextEngine::new().map_err(|error| RunError::Runtime(error.to_string()))?;
    let mut app = App::new(cli.lang, cli.no_audio, !cli.no_animations);
    let audio = Audio::start();
    if !audio.is_available() {
        app.set_sound_status(SoundStatus::Unavailable);
    }

    let info = *framebuffer.info();
    let compositor = Compositor::new(
        &app.scene(&theme).background,
        info.width,
        info.height,
        layout::scale_for(info.width, info.height),
    );

    let console = ConsoleGuard::enter().map_err(backend)?;
    let raw = RawModeGuard::enter().map_err(backend)?;

    draw(&mut framebuffer, &engine, &theme, &app, &compositor)?;

    while !quit_requested() {
        let now = Instant::now();
        if app.update(now) {
            draw(&mut framebuffer, &engine, &theme, &app, &compositor)?;
        }

        let timeout = app.timeout();
        match event::poll(timeout) {
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

        let outcome = app.handle_at(map_key(key), Instant::now());
        if let Some(event) = app.take_sound_request() {
            if !app.session().muted() {
                let _ = audio.play(event);
            }
        }
        match outcome {
            Outcome::Quit => break,
            Outcome::Changed => draw(&mut framebuffer, &engine, &theme, &app, &compositor)?,
            Outcome::Unchanged => {}
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
    app: &App,
    compositor: &Compositor,
) -> Result<(), RunError> {
    let pixmap = compositor.compose(&app.scene(theme), engine);
    framebuffer.present(pixmap.data()).map_err(runtime)
}

fn backend(error: impl std::fmt::Display) -> RunError {
    RunError::Backend(error.to_string())
}

fn runtime(error: impl std::fmt::Display) -> RunError {
    RunError::Runtime(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CalcError, DisplayState, Operator};

    fn app() -> App {
        App::new(Lang::Pt, false, true)
    }

    fn digits(app: &mut App, text: &str, now: Instant) {
        for digit in text.chars() {
            app.handle_at(Action::Digit(digit), now);
        }
    }

    fn at(offset_ms: u64) -> Instant {
        Instant::now() + Duration::from_millis(offset_ms)
    }

    fn value(app: &App) -> String {
        match app.calculator().display(Lang::Pt.number_locale()) {
            DisplayState::Value(value) => value,
            DisplayState::Error(error) => format!("error: {error}"),
        }
    }

    #[test]
    fn global_actions_preserve_calculation() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "42", now);
        assert_eq!(app.handle_at(Action::ToggleLanguage, now), Outcome::Changed);
        assert_eq!(app.session().lang(), Lang::En);
        assert_eq!(app.handle_at(Action::ToggleMute, now), Outcome::Changed);
        assert!(app.session().muted());
        assert_eq!(app.sound_status(), SoundStatus::Off);
        assert_eq!(value(&app), "42");
    }

    #[test]
    fn full_flow_updates_display_and_history() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "200", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "20", now);
        assert_eq!(app.handle_at(Action::Equals, now), Outcome::Changed);
        assert_eq!(value(&app), "220");
        assert_eq!(app.calculator().history().len(), 1);
        assert_eq!(app.pressed_key(), Some(KeyId::Equals));
    }

    #[test]
    fn history_selection_recalls_without_creating_entry() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "9", now);
        app.update(at(200));

        assert_eq!(app.handle_at(Action::HistoryPrev, now), Outcome::Changed);
        assert_eq!(app.session().history_selection(), Some(0));
        assert_eq!(value(&app), "9");
        assert_eq!(app.handle_at(Action::Equals, at(200)), Outcome::Changed);
        assert_eq!(value(&app), "5");
        assert_eq!(app.calculator().pending(), None);
        assert_eq!(app.session().history_selection(), None);
        assert_eq!(app.calculator().history().len(), 1);
        assert_eq!(app.pressed_key(), Some(KeyId::Equals));
    }

    #[test]
    fn calculation_actions_cancel_history_selection() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.handle_at(Action::HistoryPrev, now);
        assert_eq!(app.session().history_selection(), Some(0));
        app.handle_at(Action::Digit('7'), now);
        assert_eq!(app.session().history_selection(), None);
        assert_eq!(value(&app), "7");
    }

    #[test]
    fn ignored_action_still_redraws_when_it_cancels_selection() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.update(at(100));
        app.handle_at(Action::HistoryPrev, now);
        assert_eq!(app.session().history_selection(), Some(0));
        assert_eq!(app.handle_at(Action::Backspace, now), Outcome::Changed);
        assert_eq!(app.session().history_selection(), None);
        assert_eq!(app.pressed_key(), None);
    }

    #[test]
    fn confirmations_are_debounced() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.update(at(200));
        app.handle_at(Action::HistoryPrev, at(300));

        assert_eq!(app.handle_at(Action::Equals, at(300)), Outcome::Changed);
        app.handle_at(Action::HistoryPrev, at(320));
        assert_eq!(app.handle_at(Action::Equals, at(400)), Outcome::Unchanged);
        assert_eq!(app.session().history_selection(), Some(0));
        assert_eq!(app.handle_at(Action::Equals, at(500)), Outcome::Changed);
        assert_eq!(app.session().history_selection(), None);
    }

    #[test]
    fn history_navigation_clamps_at_ends() {
        let now = at(0);
        let mut app = app();
        assert_eq!(app.handle_at(Action::HistoryPrev, now), Outcome::Unchanged);
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.handle_at(Action::HistoryPrev, at(200));
        assert_eq!(app.session().history_selection(), Some(0));
        assert_eq!(
            app.handle_at(Action::HistoryPrev, at(200)),
            Outcome::Unchanged
        );
        assert_eq!(
            app.handle_at(Action::HistoryNext, at(200)),
            Outcome::Changed
        );
        assert_eq!(app.session().history_selection(), None);
    }

    #[test]
    fn flash_expires_after_ninety_ms() {
        let now = at(0);
        let mut app = app();
        app.handle_at(Action::Digit('7'), now);
        assert_eq!(app.pressed_key(), Some(KeyId::Digit('7')));
        assert!(!app.update(at(89)));
        assert!(app.update(at(90)));
        assert_eq!(app.pressed_key(), None);
    }

    #[test]
    fn reduced_flash_expires_after_sixty_ms() {
        let now = at(0);
        let mut app = App::new(Lang::Pt, false, false);
        app.handle_at(Action::Digit('7'), now);
        assert!(!app.update(at(59)));
        assert!(app.update(at(60)));
    }

    #[test]
    fn ignored_actions_do_not_flash() {
        let now = at(0);
        let mut app = app();
        assert_eq!(app.handle_at(Action::Backspace, now), Outcome::Unchanged);
        assert_eq!(app.handle_at(Action::Equals, now), Outcome::Unchanged);
        assert_eq!(app.pressed_key(), None);
        assert_eq!(app.last_event(), CalcEvent::Ignored);
    }

    #[test]
    fn error_flashes_and_digit_recovers() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "1", now);
        app.handle_at(Action::Operator(Operator::Div), now);
        digits(&mut app, "0", now);
        app.handle_at(Action::Equals, now);
        assert_eq!(
            app.calculator().display(Lang::Pt.number_locale()),
            DisplayState::Error(CalcError::DivisionByZero)
        );
        assert_eq!(app.pressed_key(), Some(KeyId::Equals));
        app.handle_at(Action::Digit('7'), now);
        assert_eq!(value(&app), "7");
        assert_eq!(app.pressed_key(), Some(KeyId::Digit('7')));
    }

    #[test]
    fn sound_requests_follow_accepted_and_error_events() {
        let now = at(0);
        let mut app = app();
        assert_eq!(app.handle_at(Action::Backspace, now), Outcome::Unchanged);
        assert_eq!(app.take_sound_request(), None);

        app.handle_at(Action::Digit('1'), now);
        assert_eq!(app.take_sound_request(), Some(SoundEvent::Click));

        app.handle_at(Action::Operator(Operator::Div), now);
        app.handle_at(Action::Digit('0'), now);
        app.handle_at(Action::Equals, now);
        assert_eq!(app.take_sound_request(), Some(SoundEvent::Error));

        app.handle_at(Action::Digit('7'), now);
        assert_eq!(app.take_sound_request(), Some(SoundEvent::Click));
        assert_eq!(app.take_sound_request(), None);
    }

    #[test]
    fn recall_requests_click_without_history_growth() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.update(at(200));
        app.take_sound_request();
        let history_len = app.calculator().history().len();

        app.handle_at(Action::HistoryPrev, at(200));
        app.handle_at(Action::Equals, at(400));
        assert_eq!(app.take_sound_request(), Some(SoundEvent::Click));
        assert_eq!(app.calculator().history().len(), history_len);
    }

    #[test]
    fn history_fade_animates_and_expires() {
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        assert_eq!(app.history_fade(), Some(0.0));
        assert!(app.update(at(60)));
        let progress = app.history_fade().expect("fade still active");
        assert!((0.4..=0.7).contains(&progress), "progress = {progress}");
        assert!(app.update(at(200)));
        assert_eq!(app.history_fade(), None);
    }

    #[test]
    fn no_animations_disables_history_fade() {
        let now = at(0);
        let mut app = App::new(Lang::Pt, false, false);
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        assert_eq!(app.history_fade(), None);
    }

    #[test]
    fn unavailable_sound_stays_unavailable_when_muting() {
        let mut app = app();
        app.set_sound_status(SoundStatus::Unavailable);
        app.handle_at(Action::ToggleMute, at(0));
        assert_eq!(app.sound_status(), SoundStatus::Unavailable);
        app.handle_at(Action::ToggleMute, at(200));
        assert_eq!(app.sound_status(), SoundStatus::Unavailable);
    }

    #[test]
    fn scene_reflects_history_selection() {
        let theme = Theme::truecolor();
        let now = at(0);
        let mut app = app();
        digits(&mut app, "2", now);
        app.handle_at(Action::Operator(Operator::Add), now);
        digits(&mut app, "3", now);
        app.handle_at(Action::Equals, now);
        app.update(at(200));
        app.handle_at(Action::HistoryPrev, at(200));
        let scene = app.scene(&theme);
        assert!(scene.items.iter().any(|item| matches!(
            item,
            crate::render::scene::Item::Rect(rect) if rect.fill == theme.accent
        )));
    }
}
