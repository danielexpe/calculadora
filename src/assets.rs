use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

pub const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
pub const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");
pub const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
pub const JETBRAINS_MONO_MEDIUM: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf");

#[derive(Debug)]
pub struct Fonts {
    pub inter_regular: Font,
    pub inter_semibold: Font,
    pub jetbrains_mono_regular: Font,
    pub jetbrains_mono_medium: Font,
}

static FONTS: OnceLock<Result<Fonts, &'static str>> = OnceLock::new();

pub fn fonts() -> Result<&'static Fonts, &'static str> {
    match FONTS.get_or_init(|| {
        Ok(Fonts {
            inter_regular: Font::from_bytes(INTER_REGULAR, FontSettings::default())?,
            inter_semibold: Font::from_bytes(INTER_SEMIBOLD, FontSettings::default())?,
            jetbrains_mono_regular: Font::from_bytes(
                JETBRAINS_MONO_REGULAR,
                FontSettings::default(),
            )?,
            jetbrains_mono_medium: Font::from_bytes(
                JETBRAINS_MONO_MEDIUM,
                FontSettings::default(),
            )?,
        })
    }) {
        Ok(fonts) => Ok(fonts),
        Err(error) => Err(error),
    }
}

pub const REQUIRED_GLYPHS: &str = concat!(
    "0123456789",
    ",.+",
    "\u{2212}",
    "\u{00d7}",
    "\u{00f7}",
    "\u{00b1}",
    "%=",
    "e",
    "\u{00b7}",
    ":",
    "\u{2191}",
    "\u{2193}",
    "\u{2026}",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "\u{00e1}\u{00e3}\u{00e7}\u{00e9}\u{00ed}\u{00f3}\u{00f5}\u{00fa}",
    "\u{00c1}\u{00c3}\u{00c7}\u{00c9}\u{00cd}\u{00d3}\u{00d5}\u{00da}",
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Lang, EN, PT};

    fn missing(font: &Font, text: &str) -> Vec<char> {
        let mut missing = Vec::new();
        for character in text.chars() {
            if !missing.contains(&character) && !font.has_glyph(character) {
                missing.push(character);
            }
        }
        missing
    }

    fn catalog_text() -> String {
        let mut text = String::from(REQUIRED_GLYPHS);
        for strings in [&PT, &EN] {
            for value in strings.values() {
                text.push_str(value);
            }
        }
        text
    }

    #[test]
    fn all_embedded_fonts_decode() {
        let fonts = fonts().expect("embedded fonts must decode");
        assert!(fonts.inter_regular.has_glyph('0'));
        assert!(fonts.inter_semibold.has_glyph('H'));
        assert!(fonts.jetbrains_mono_regular.has_glyph('0'));
        assert!(fonts.jetbrains_mono_medium.has_glyph('='));
    }

    #[test]
    fn jetbrains_mono_covers_numeric_and_operator_glyphs() {
        let fonts = fonts().expect("embedded fonts must decode");
        let missing = missing(&fonts.jetbrains_mono_medium, REQUIRED_GLYPHS);
        assert!(missing.is_empty(), "JetBrains Mono missing: {missing:?}");
    }

    #[test]
    fn inter_covers_catalog_text() {
        let fonts = fonts().expect("embedded fonts must decode");
        let text = catalog_text();
        let regular_missing = missing(&fonts.inter_regular, &text);
        assert!(
            regular_missing.is_empty(),
            "Inter Regular missing: {regular_missing:?}"
        );
        let semibold_missing = missing(&fonts.inter_semibold, &text);
        assert!(
            semibold_missing.is_empty(),
            "Inter SemiBold missing: {semibold_missing:?}"
        );
    }

    #[test]
    fn catalog_has_both_languages() {
        assert_eq!(Lang::ALL, [Lang::Pt, Lang::En]);
    }

    #[test]
    fn active_entry_fits_display_width_at_64px() {
        let fonts = fonts().expect("embedded fonts must decode");
        let advance = fonts.jetbrains_mono_medium.metrics('0', 64.0).advance_width;
        assert!(
            advance * 17.0 <= 672.0,
            "17 characters at 64px take {} px",
            advance * 17.0
        );
    }
}
