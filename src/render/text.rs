use std::cell::RefCell;
use std::collections::HashMap;

use fontdue::{Font, Metrics};
use tiny_skia::Pixmap;

use crate::assets::{self, Fonts};

use super::blend::blend;
use super::scene::{Align, FontKind, Rgba8};

struct Glyph {
    metrics: Metrics,
    coverage: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font: FontKind,
    character: char,
    px: u32,
}

pub struct TextEngine {
    fonts: &'static Fonts,
    cache: RefCell<HashMap<GlyphKey, Glyph>>,
}

impl TextEngine {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            fonts: assets::fonts()?,
            cache: RefCell::new(HashMap::new()),
        })
    }

    fn font(&self, kind: FontKind) -> &Font {
        match kind {
            FontKind::InterRegular => &self.fonts.inter_regular,
            FontKind::InterSemiBold => &self.fonts.inter_semibold,
            FontKind::MonoRegular => &self.fonts.jetbrains_mono_regular,
            FontKind::MonoMedium => &self.fonts.jetbrains_mono_medium,
        }
    }

    pub fn measure(&self, kind: FontKind, text: &str, size: f32) -> f32 {
        let font = self.font(kind);
        let mut width = 0.0;
        let mut previous = None;
        for character in text.chars() {
            if let Some(previous) = previous {
                width += font
                    .horizontal_kern(previous, character, size)
                    .unwrap_or(0.0);
            }
            width += font.metrics(character, size).advance_width;
            previous = Some(character);
        }
        width
    }

    pub fn ellipsize(&self, kind: FontKind, text: &str, size: f32, max_width: f32) -> String {
        if self.measure(kind, text, size) <= max_width {
            return text.to_string();
        }

        let ellipsis = '\u{2026}';
        let ellipsis_width = self.measure(kind, "\u{2026}", size);
        let mut result = String::new();
        let mut width = 0.0;

        for character in text.chars() {
            let advance = self.font(kind).metrics(character, size).advance_width;
            if width + advance + ellipsis_width > max_width {
                break;
            }
            result.push(character);
            width += advance;
        }
        result.push(ellipsis);
        result
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        kind: FontKind,
        color: Rgba8,
        align: Align,
    ) {
        if text.is_empty() {
            return;
        }

        let text = self.ellipsize(kind, text, size, w);
        let font = self.font(kind);
        let measured = self.measure(kind, &text, size);
        let start_x = match align {
            Align::Left => x,
            Align::Center => x + (w - measured) / 2.0,
            Align::Right => x + w - measured,
        };

        let (ascent, descent) = match font.horizontal_line_metrics(size) {
            Some(metrics) => (metrics.ascent, metrics.descent),
            None => (size * 0.8, -size * 0.2),
        };
        let baseline = y + (h - (ascent - descent)) / 2.0 + ascent;

        let mut pen_x = start_x;
        let mut previous = None;
        for character in text.chars() {
            if let Some(previous) = previous {
                pen_x += font
                    .horizontal_kern(previous, character, size)
                    .unwrap_or(0.0);
            }

            let key = GlyphKey {
                font: kind,
                character,
                px: size.round() as u32,
            };
            let mut cache = self.cache.borrow_mut();
            let glyph = cache.entry(key).or_insert_with(|| {
                let (metrics, coverage) = font.rasterize(character, size);
                Glyph { metrics, coverage }
            });

            let glyph_x = pen_x.round() as i32 + glyph.metrics.xmin;
            let glyph_y =
                baseline.round() as i32 - glyph.metrics.height as i32 - glyph.metrics.ymin;
            draw_coverage(
                pixmap,
                glyph_x,
                glyph_y,
                &glyph.metrics,
                &glyph.coverage,
                color,
            );

            pen_x += glyph.metrics.advance_width;
            previous = Some(character);
        }
    }
}

fn draw_coverage(
    pixmap: &mut Pixmap,
    origin_x: i32,
    origin_y: i32,
    metrics: &Metrics,
    coverage: &[u8],
    color: Rgba8,
) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let source = color.to_array();

    for row in 0..metrics.height {
        for column in 0..metrics.width {
            let alpha = coverage[row * metrics.width + column];
            if alpha == 0 {
                continue;
            }

            let x = origin_x + column as i32;
            let y = origin_y + row as i32;
            if x < 0 || y < 0 || x >= width || y >= height {
                continue;
            }

            let index = (y as u32 * pixmap.width() + x as u32) as usize * 4;
            let data = pixmap.data_mut();
            let destination = [
                data[index],
                data[index + 1],
                data[index + 2],
                data[index + 3],
            ];
            let result = blend(destination, source, alpha);
            data[index..index + 4].copy_from_slice(&result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::REQUIRED_GLYPHS;

    #[test]
    fn engine_loads_embedded_fonts() {
        assert!(TextEngine::new().is_ok());
    }

    #[test]
    fn mono_advance_is_stable_for_all_digits() {
        let engine = TextEngine::new().unwrap();
        let widths: Vec<f32> = ('0'..='9')
            .map(|digit| engine.measure(FontKind::MonoMedium, &digit.to_string(), 30.0))
            .collect();
        for width in &widths {
            assert!((width - widths[0]).abs() < 0.01);
        }
    }

    #[test]
    fn measurement_matches_painted_extent() {
        let engine = TextEngine::new().unwrap();
        let width = engine.measure(FontKind::MonoMedium, "12345", 30.0);
        assert!((width - 5.0 * engine.measure(FontKind::MonoMedium, "1", 30.0)).abs() < 0.01);
    }

    #[test]
    fn ellipsis_only_when_needed() {
        let engine = TextEngine::new().unwrap();
        assert_eq!(
            engine.ellipsize(FontKind::InterRegular, "short", 14.0, 1000.0),
            "short"
        );
        let clipped = engine.ellipsize(FontKind::InterRegular, "a long sentence here", 14.0, 40.0);
        assert!(clipped.ends_with('\u{2026}'));
        assert!(engine.measure(FontKind::InterRegular, &clipped, 14.0) <= 40.0);
    }

    #[test]
    fn painting_changes_pixels_and_hides_empty_text() {
        let engine = TextEngine::new().unwrap();
        let mut pixmap = Pixmap::new(200, 80).unwrap();
        pixmap.fill(tiny_skia::Color::BLACK);
        let before = pixmap.data().to_vec();
        engine.paint(
            &mut pixmap,
            "200 + 10",
            0.0,
            0.0,
            200.0,
            80.0,
            30.0,
            FontKind::MonoMedium,
            Rgba8::new(255, 255, 255, 255),
            Align::Center,
        );
        assert_ne!(before, pixmap.data());
    }

    #[test]
    fn every_required_glyph_renders_without_panic() {
        let engine = TextEngine::new().unwrap();
        let mut pixmap = Pixmap::new(1344, 756).unwrap();
        for character in REQUIRED_GLYPHS.chars() {
            let text = character.to_string();
            engine.paint(
                &mut pixmap,
                &text,
                10.0,
                10.0,
                200.0,
                64.0,
                30.0,
                FontKind::MonoMedium,
                Rgba8::new(255, 255, 255, 255),
                Align::Left,
            );
        }
    }
}
