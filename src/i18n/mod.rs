mod strings;

use std::fmt;
use std::str::FromStr;

use crate::core::{CalcError, NumberLocale};

pub use strings::{Strings, EN, PT};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    Pt,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Pt, Lang::En];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::En => "en",
        }
    }

    pub const fn number_locale(self) -> NumberLocale {
        match self {
            Self::Pt => NumberLocale::PtBr,
            Self::En => NumberLocale::En,
        }
    }

    pub const fn other(self) -> Self {
        match self {
            Self::Pt => Self::En,
            Self::En => Self::Pt,
        }
    }
}

impl FromStr for Lang {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pt" => Ok(Self::Pt),
            "en" => Ok(Self::En),
            other => Err(format!("invalid language: {other}; expected pt or en")),
        }
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundStatus {
    On,
    Off,
    Unavailable,
}

pub fn strings(lang: Lang) -> &'static Strings {
    match lang {
        Lang::Pt => &PT,
        Lang::En => &EN,
    }
}

pub fn error_message(lang: Lang, error: CalcError) -> &'static str {
    let strings = strings(lang);
    match error {
        CalcError::DivisionByZero => strings.error_div_zero,
        CalcError::Overflow => strings.error_overflow,
        CalcError::Other => strings.error_other,
    }
}

pub fn sound_status(lang: Lang, status: SoundStatus) -> &'static str {
    let strings = strings(lang);
    match status {
        SoundStatus::On => strings.sound_on,
        SoundStatus::Off => strings.sound_off,
        SoundStatus::Unavailable => strings.sound_unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes_parse_and_render() {
        assert_eq!("pt".parse::<Lang>(), Ok(Lang::Pt));
        assert_eq!("en".parse::<Lang>(), Ok(Lang::En));
        assert!("fr".parse::<Lang>().is_err());
        assert_eq!(Lang::Pt.code(), "pt");
        assert_eq!(Lang::En.code(), "en");
        assert_eq!(Lang::Pt.to_string(), "pt");
        assert_eq!(Lang::En.to_string(), "en");
    }

    #[test]
    fn number_locale_matches_language() {
        assert_eq!(Lang::Pt.number_locale(), NumberLocale::PtBr);
        assert_eq!(Lang::En.number_locale(), NumberLocale::En);
    }

    #[test]
    fn toggle_switches_language() {
        assert_eq!(Lang::Pt.other(), Lang::En);
        assert_eq!(Lang::En.other(), Lang::Pt);
    }

    #[test]
    fn catalog_is_complete_and_non_empty() {
        for lang in Lang::ALL {
            for text in strings(lang).values() {
                assert!(!text.is_empty());
            }
        }
    }

    #[test]
    fn translated_strings_differ() {
        let pt = strings(Lang::Pt);
        let en = strings(Lang::En);
        assert_ne!(pt.history_title, en.history_title);
        assert_ne!(pt.history_empty, en.history_empty);
        assert_ne!(pt.footer_hints, en.footer_hints);
        assert_ne!(pt.sound_on, en.sound_on);
        assert_ne!(pt.error_div_zero, en.error_div_zero);
        assert_ne!(pt.error_overflow, en.error_overflow);
        assert_ne!(pt.error_other, en.error_other);
    }

    #[test]
    fn errors_are_mapped_to_catalog() {
        assert_eq!(
            error_message(Lang::Pt, CalcError::DivisionByZero),
            "Divisão por zero"
        );
        assert_eq!(
            error_message(Lang::En, CalcError::DivisionByZero),
            "Division by zero"
        );
        assert_eq!(
            error_message(Lang::Pt, CalcError::Overflow),
            "Número muito grande"
        );
        assert_eq!(
            error_message(Lang::En, CalcError::Overflow),
            "Number too large"
        );
    }

    #[test]
    fn sound_status_is_localized() {
        assert_eq!(sound_status(Lang::Pt, SoundStatus::On), "Som: ligado");
        assert_eq!(sound_status(Lang::En, SoundStatus::On), "Sound: on");
        assert_eq!(
            sound_status(Lang::Pt, SoundStatus::Unavailable),
            "Som: indisponível"
        );
    }
}
