use crate::cli::ColorMode;
use crate::render::scene::Rgba8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub canvas: Rgba8,
    pub canvas_glow: Rgba8,
    pub panel: Rgba8,
    pub card: Rgba8,
    pub display: Rgba8,
    pub key: Rgba8,
    pub key_utility: Rgba8,
    pub key_pressed: Rgba8,
    pub surface_selected: Rgba8,
    pub border: Rgba8,
    pub border_strong: Rgba8,
    pub text_primary: Rgba8,
    pub text_secondary: Rgba8,
    pub text_muted: Rgba8,
    pub accent: Rgba8,
    pub accent_strong: Rgba8,
    pub on_accent: Rgba8,
    pub operator: Rgba8,
    pub operator_bg: Rgba8,
    pub operator_border: Rgba8,
    pub warning: Rgba8,
    pub danger: Rgba8,
    pub success: Rgba8,
    pub shadow: Rgba8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    Truecolor,
    Color256,
    Color16,
    Mono,
}

impl Theme {
    pub fn new(mode: ColorMode, bpp: u32) -> Self {
        match resolve(mode, bpp) {
            Palette::Truecolor => Self::truecolor(),
            Palette::Color256 => Self::color256(),
            Palette::Color16 => Self::color16(),
            Palette::Mono => Self::mono(),
        }
    }

    pub const fn truecolor() -> Self {
        Self {
            canvas: Rgba8::hex(0x0B0D11),
            canvas_glow: Rgba8::hex(0x151A22),
            panel: Rgba8::hex(0x12161E),
            card: Rgba8::hex(0x171C25),
            display: Rgba8::hex(0x0F131A),
            key: Rgba8::hex(0x1F2430),
            key_utility: Rgba8::hex(0x252B38),
            key_pressed: Rgba8::hex(0x2B3240),
            surface_selected: Rgba8::hex(0x1F2632),
            border: Rgba8::hex(0x2B323F),
            border_strong: Rgba8::hex(0x3A4454),
            text_primary: Rgba8::hex(0xEDEFF3),
            text_secondary: Rgba8::hex(0xA2ABBA),
            text_muted: Rgba8::hex(0x7D8696),
            accent: Rgba8::hex(0x5EEAD4),
            accent_strong: Rgba8::hex(0x2DD4BF),
            on_accent: Rgba8::hex(0x07110F),
            operator: Rgba8::hex(0x5EEAD4),
            operator_bg: Rgba8::hex(0x1B2A2C),
            operator_border: Rgba8::hex(0x24403F),
            warning: Rgba8::hex(0xFBBF24),
            danger: Rgba8::hex(0xF87171),
            success: Rgba8::hex(0x34D399),
            shadow: Rgba8::new(0, 0, 0, 255),
        }
    }

    pub fn color256() -> Self {
        Self {
            canvas: xterm256(233),
            canvas_glow: xterm256(234),
            panel: xterm256(234),
            card: xterm256(235),
            display: xterm256(233),
            key: xterm256(236),
            key_utility: xterm256(237),
            key_pressed: xterm256(238),
            surface_selected: xterm256(237),
            border: xterm256(238),
            border_strong: xterm256(240),
            text_primary: xterm256(255),
            text_secondary: xterm256(250),
            text_muted: xterm256(246),
            accent: xterm256(86),
            accent_strong: xterm256(87),
            on_accent: xterm256(232),
            operator: xterm256(86),
            operator_bg: xterm256(236),
            operator_border: xterm256(239),
            warning: xterm256(214),
            danger: xterm256(203),
            success: xterm256(78),
            shadow: Rgba8::new(0, 0, 0, 255),
        }
    }

    pub const fn color16() -> Self {
        Self {
            canvas: Rgba8::new(0, 0, 0, 255),
            canvas_glow: Rgba8::new(8, 8, 8, 255),
            panel: Rgba8::new(0, 0, 0, 255),
            card: Rgba8::new(8, 8, 8, 255),
            display: Rgba8::new(0, 0, 0, 255),
            key: Rgba8::new(32, 32, 32, 255),
            key_utility: Rgba8::new(48, 48, 48, 255),
            key_pressed: Rgba8::new(72, 72, 72, 255),
            surface_selected: Rgba8::new(40, 40, 40, 255),
            border: Rgba8::new(72, 72, 72, 255),
            border_strong: Rgba8::new(150, 150, 150, 255),
            text_primary: Rgba8::new(255, 255, 255, 255),
            text_secondary: Rgba8::new(192, 192, 192, 255),
            text_muted: Rgba8::new(150, 150, 150, 255),
            accent: Rgba8::new(0, 230, 230, 255),
            accent_strong: Rgba8::new(0, 255, 255, 255),
            on_accent: Rgba8::new(0, 0, 0, 255),
            operator: Rgba8::new(0, 230, 230, 255),
            operator_bg: Rgba8::new(0, 32, 32, 255),
            operator_border: Rgba8::new(0, 120, 120, 255),
            warning: Rgba8::new(255, 200, 0, 255),
            danger: Rgba8::new(255, 80, 80, 255),
            success: Rgba8::new(0, 200, 80, 255),
            shadow: Rgba8::new(0, 0, 0, 255),
        }
    }

    pub const fn mono() -> Self {
        Self {
            canvas: Rgba8::new(0, 0, 0, 255),
            canvas_glow: Rgba8::new(18, 18, 18, 255),
            panel: Rgba8::new(10, 10, 10, 255),
            card: Rgba8::new(17, 17, 17, 255),
            display: Rgba8::new(5, 5, 5, 255),
            key: Rgba8::new(28, 28, 28, 255),
            key_utility: Rgba8::new(35, 35, 35, 255),
            key_pressed: Rgba8::new(46, 46, 46, 255),
            surface_selected: Rgba8::new(38, 38, 38, 255),
            border: Rgba8::new(58, 58, 58, 255),
            border_strong: Rgba8::new(120, 120, 120, 255),
            text_primary: Rgba8::new(255, 255, 255, 255),
            text_secondary: Rgba8::new(176, 176, 176, 255),
            text_muted: Rgba8::new(138, 138, 138, 255),
            accent: Rgba8::new(255, 255, 255, 255),
            accent_strong: Rgba8::new(255, 255, 255, 255),
            on_accent: Rgba8::new(0, 0, 0, 255),
            operator: Rgba8::new(255, 255, 255, 255),
            operator_bg: Rgba8::new(28, 28, 28, 255),
            operator_border: Rgba8::new(85, 85, 85, 255),
            warning: Rgba8::new(255, 255, 255, 255),
            danger: Rgba8::new(255, 255, 255, 255),
            success: Rgba8::new(255, 255, 255, 255),
            shadow: Rgba8::new(0, 0, 0, 255),
        }
    }
}

pub const fn resolve(mode: ColorMode, bpp: u32) -> Palette {
    match mode {
        ColorMode::Never => Palette::Mono,
        ColorMode::Truecolor => Palette::Truecolor,
        ColorMode::Color256 => Palette::Color256,
        ColorMode::Color16 => Palette::Color16,
        ColorMode::Auto => {
            if bpp >= 24 {
                Palette::Truecolor
            } else {
                Palette::Color256
            }
        }
    }
}

fn xterm256(index: u8) -> Rgba8 {
    match index {
        0..=15 => ansi16(index),
        16..=231 => {
            let index = index - 16;
            let steps = [0u8, 95, 135, 175, 215, 255];
            let red = steps[(index / 36) as usize];
            let green = steps[((index % 36) / 6) as usize];
            let blue = steps[(index % 6) as usize];
            Rgba8::new(red, green, blue, 255)
        }
        232..=255 => {
            let level = 8 + (index - 232) * 10;
            Rgba8::new(level, level, level, 255)
        }
    }
}

fn ansi16(index: u8) -> Rgba8 {
    const TABLE: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    let (red, green, blue) = TABLE[index as usize % 16];
    Rgba8::new(red, green, blue, 255)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_uses_truecolor_for_24_and_32_bpp() {
        assert_eq!(resolve(ColorMode::Auto, 32), Palette::Truecolor);
        assert_eq!(resolve(ColorMode::Auto, 24), Palette::Truecolor);
    }

    #[test]
    fn auto_uses_256_for_16_bpp() {
        assert_eq!(resolve(ColorMode::Auto, 16), Palette::Color256);
    }

    #[test]
    fn never_wins_over_any_mode() {
        assert_eq!(resolve(ColorMode::Never, 32), Palette::Mono);
        assert_eq!(resolve(ColorMode::Never, 16), Palette::Mono);
    }

    #[test]
    fn explicit_modes_are_respected() {
        assert_eq!(resolve(ColorMode::Truecolor, 16), Palette::Truecolor);
        assert_eq!(resolve(ColorMode::Color256, 32), Palette::Color256);
        assert_eq!(resolve(ColorMode::Color16, 32), Palette::Color16);
    }

    #[test]
    fn palettes_expose_documented_colors() {
        assert_eq!(Theme::truecolor().canvas, Rgba8::hex(0x0B0D11));
        assert_eq!(Theme::truecolor().accent, Rgba8::hex(0x5EEAD4));
        assert_eq!(Theme::mono().accent, Rgba8::new(255, 255, 255, 255));
    }

    fn relative_luminance(color: Rgba8) -> f64 {
        let channel = |value: u8| {
            let value = value as f64 / 255.0;
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
    }

    fn contrast(a: Rgba8, b: Rgba8) -> f64 {
        let first = relative_luminance(a);
        let second = relative_luminance(b);
        let (high, low) = if first > second {
            (first, second)
        } else {
            (second, first)
        };
        (high + 0.05) / (low + 0.05)
    }

    #[test]
    fn palettes_meet_documented_contrast_targets() {
        for theme in [
            Theme::truecolor(),
            Theme::color256(),
            Theme::color16(),
            Theme::mono(),
        ] {
            assert!(
                contrast(theme.text_primary, theme.card) >= 7.0,
                "primary contrast"
            );
            assert!(
                contrast(theme.text_secondary, theme.card) >= 4.5,
                "secondary contrast"
            );
            assert!(
                contrast(theme.text_muted, theme.card) >= 4.5,
                "muted contrast"
            );
            assert!(
                contrast(theme.accent, theme.card) >= 10.0,
                "accent contrast"
            );
            assert!(
                contrast(theme.on_accent, theme.accent) >= 4.5,
                "on-accent contrast"
            );
        }
    }

    #[test]
    fn xterm_256_maps_documented_indices() {
        assert_eq!(xterm256(233), Rgba8::new(18, 18, 18, 255));
        assert_eq!(xterm256(255), Rgba8::new(238, 238, 238, 255));
        assert_eq!(xterm256(79), Rgba8::new(95, 215, 175, 255));
        assert_eq!(xterm256(80), Rgba8::new(95, 215, 215, 255));
        assert_eq!(ansi16(14), Rgba8::new(0, 255, 255, 255));
    }
}
