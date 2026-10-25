use crate::i18n::Lang;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    lang: Lang,
    muted: bool,
    history_selection: Option<usize>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new(Lang::Pt)
    }
}

impl SessionState {
    pub fn new(lang: Lang) -> Self {
        Self {
            lang,
            muted: false,
            history_selection: None,
        }
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn history_selection(&self) -> Option<usize> {
        self.history_selection
    }

    pub fn selected(&self, history_len: usize) -> Option<usize> {
        self.history_selection.filter(|index| *index < history_len)
    }

    pub fn toggle_language(&mut self) -> Lang {
        self.lang = self.lang.other();
        self.lang
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }

    pub fn clear_selection(&mut self) {
        self.history_selection = None;
    }

    pub fn select_previous(&mut self, history_len: usize) -> Option<usize> {
        if history_len == 0 {
            self.history_selection = None;
            return None;
        }

        let next = match self.history_selection {
            None => history_len - 1,
            Some(0) => 0,
            Some(index) => index - 1,
        };
        self.history_selection = Some(next);
        Some(next)
    }

    pub fn select_next(&mut self, history_len: usize) -> Option<usize> {
        let index = self.history_selection?;

        if history_len == 0 {
            self.history_selection = None;
            return None;
        }

        if index + 1 < history_len {
            self.history_selection = Some(index + 1);
            Some(index + 1)
        } else {
            self.history_selection = None;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_portuguese_without_selection() {
        let state = SessionState::default();
        assert_eq!(state.lang(), Lang::Pt);
        assert!(!state.muted());
        assert_eq!(state.history_selection(), None);
    }

    #[test]
    fn toggles_language_and_mute() {
        let mut state = SessionState::default();
        assert_eq!(state.toggle_language(), Lang::En);
        assert_eq!(state.toggle_language(), Lang::Pt);
        assert!(state.toggle_mute());
        assert!(!state.toggle_mute());
    }

    #[test]
    fn selects_latest_then_walks_back() {
        let mut state = SessionState::default();
        assert_eq!(state.select_previous(5), Some(4));
        assert_eq!(state.select_previous(5), Some(3));
        assert_eq!(state.select_previous(5), Some(2));
    }

    #[test]
    fn moves_forward_and_clears_past_latest() {
        let mut state = SessionState::default();
        state.select_previous(3);
        state.select_previous(3);
        assert_eq!(state.history_selection(), Some(1));
        assert_eq!(state.select_next(3), Some(2));
        assert_eq!(state.select_next(3), None);
        assert_eq!(state.history_selection(), None);
    }

    #[test]
    fn selection_is_clamped_when_history_shrinks() {
        let mut state = SessionState::default();
        state.select_previous(5);
        assert_eq!(state.selected(5), Some(4));
        assert_eq!(state.selected(2), None);
    }

    #[test]
    fn empty_history_clears_selection() {
        let mut state = SessionState::default();
        state.select_previous(0);
        assert_eq!(state.history_selection(), None);
        assert_eq!(state.select_next(0), None);
    }
}
