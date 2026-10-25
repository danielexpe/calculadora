use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::core::Operator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Digit(char),
    DecimalSeparator,
    Operator(Operator),
    Equals,
    Backspace,
    Clear,
    ToggleSign,
    Percent,
    HistoryPrev,
    HistoryNext,
    ToggleLanguage,
    ToggleMute,
    Quit,
    Ignored,
}

pub fn map_key(key: KeyEvent) -> Action {
    if key.kind == KeyEventKind::Release {
        return Action::Ignored;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => Action::Quit,
            _ => Action::Ignored,
        };
    }

    if key.modifiers.contains(KeyModifiers::ALT) {
        return Action::Ignored;
    }

    let action = match key.code {
        KeyCode::Esc => Action::Quit,
        KeyCode::Enter => Action::Equals,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Delete => Action::Clear,
        KeyCode::Up => Action::HistoryPrev,
        KeyCode::Down => Action::HistoryNext,
        KeyCode::F(2) => Action::ToggleLanguage,
        KeyCode::Char(character) => map_character(character),
        _ => Action::Ignored,
    };

    if key.kind == KeyEventKind::Repeat
        && !matches!(
            action,
            Action::Digit(_) | Action::Backspace | Action::HistoryPrev | Action::HistoryNext
        )
    {
        return Action::Ignored;
    }

    action
}

fn map_character(character: char) -> Action {
    match character {
        digit if digit.is_ascii_digit() => Action::Digit(digit),
        ',' | '.' => Action::DecimalSeparator,
        '+' => Action::Operator(Operator::Add),
        '-' => Action::Operator(Operator::Sub),
        '*' => Action::Operator(Operator::Mul),
        '/' => Action::Operator(Operator::Div),
        '=' => Action::Equals,
        '%' => Action::Percent,
        's' | 'S' => Action::ToggleSign,
        'm' | 'M' => Action::ToggleMute,
        'q' | 'Q' => Action::Quit,
        'c' | 'C' => Action::Clear,
        _ => Action::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn shifted(character: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(character), KeyModifiers::SHIFT)
    }

    #[test]
    fn maps_digits_from_top_row_and_numpad() {
        for digit in '0'..='9' {
            assert_eq!(map_key(key(KeyCode::Char(digit))), Action::Digit(digit));
        }
    }

    #[test]
    fn maps_both_decimal_separators() {
        assert_eq!(map_key(key(KeyCode::Char(','))), Action::DecimalSeparator);
        assert_eq!(map_key(key(KeyCode::Char('.'))), Action::DecimalSeparator);
    }

    #[test]
    fn maps_operators() {
        assert_eq!(
            map_key(key(KeyCode::Char('+'))),
            Action::Operator(Operator::Add)
        );
        assert_eq!(
            map_key(key(KeyCode::Char('-'))),
            Action::Operator(Operator::Sub)
        );
        assert_eq!(
            map_key(key(KeyCode::Char('*'))),
            Action::Operator(Operator::Mul)
        );
        assert_eq!(
            map_key(key(KeyCode::Char('/'))),
            Action::Operator(Operator::Div)
        );
    }

    #[test]
    fn maps_control_keys() {
        assert_eq!(map_key(key(KeyCode::Enter)), Action::Equals);
        assert_eq!(map_key(key(KeyCode::Char('='))), Action::Equals);
        assert_eq!(map_key(key(KeyCode::Backspace)), Action::Backspace);
        assert_eq!(map_key(key(KeyCode::Delete)), Action::Clear);
        assert_eq!(map_key(key(KeyCode::Char('c'))), Action::Clear);
        assert_eq!(map_key(key(KeyCode::Char('C'))), Action::Clear);
        assert_eq!(map_key(key(KeyCode::Char('s'))), Action::ToggleSign);
        assert_eq!(map_key(key(KeyCode::Char('S'))), Action::ToggleSign);
        assert_eq!(map_key(shifted('%')), Action::Percent);
        assert_eq!(map_key(key(KeyCode::F(2))), Action::ToggleLanguage);
        assert_eq!(map_key(key(KeyCode::Char('m'))), Action::ToggleMute);
        assert_eq!(map_key(key(KeyCode::Char('M'))), Action::ToggleMute);
    }

    #[test]
    fn maps_history_navigation() {
        assert_eq!(map_key(key(KeyCode::Up)), Action::HistoryPrev);
        assert_eq!(map_key(key(KeyCode::Down)), Action::HistoryNext);
    }

    #[test]
    fn maps_quit_keys() {
        assert_eq!(map_key(key(KeyCode::Esc)), Action::Quit);
        assert_eq!(map_key(key(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(map_key(key(KeyCode::Char('Q'))), Action::Quit);
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Action::Quit
        );
    }

    #[test]
    fn repeat_is_limited_to_editing_and_history_keys() {
        let repeat = |code| KeyEvent::new_with_kind(code, KeyModifiers::NONE, KeyEventKind::Repeat);
        assert_eq!(
            repeat(KeyCode::Char('7')),
            KeyEvent::new_with_kind(KeyCode::Char('7'), KeyModifiers::NONE, KeyEventKind::Repeat)
        );
        assert_eq!(
            map_key(KeyEvent::new_with_kind(
                KeyCode::Char('7'),
                KeyModifiers::NONE,
                KeyEventKind::Repeat
            )),
            Action::Digit('7')
        );
        assert_eq!(
            map_key(KeyEvent::new_with_kind(
                KeyCode::Backspace,
                KeyModifiers::NONE,
                KeyEventKind::Repeat
            )),
            Action::Backspace
        );
        assert_eq!(
            map_key(KeyEvent::new_with_kind(
                KeyCode::Up,
                KeyModifiers::NONE,
                KeyEventKind::Repeat
            )),
            Action::HistoryPrev
        );
        assert_eq!(
            map_key(KeyEvent::new_with_kind(
                KeyCode::Enter,
                KeyModifiers::NONE,
                KeyEventKind::Repeat
            )),
            Action::Ignored
        );
        assert_eq!(
            map_key(KeyEvent::new_with_kind(
                KeyCode::Char('+'),
                KeyModifiers::NONE,
                KeyEventKind::Repeat
            )),
            Action::Ignored
        );
        assert_eq!(
            map_key(KeyEvent::new_with_kind(
                KeyCode::Char('c'),
                KeyModifiers::NONE,
                KeyEventKind::Repeat
            )),
            Action::Ignored
        );
    }

    #[test]
    fn ignores_release_events() {
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('7'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(map_key(release), Action::Ignored);
    }

    #[test]
    fn ignores_unknown_and_modified_keys() {
        assert_eq!(map_key(key(KeyCode::F(1))), Action::Ignored);
        assert_eq!(map_key(key(KeyCode::Left)), Action::Ignored);
        assert_eq!(map_key(key(KeyCode::Char('z'))), Action::Ignored);
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('7'), KeyModifiers::CONTROL)),
            Action::Ignored
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('7'), KeyModifiers::ALT)),
            Action::Ignored
        );
    }
}
