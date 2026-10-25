use tiny_skia::{
    Color, FillRule, FilterQuality, LineCap, LinearGradient, Paint, PathBuilder, Pixmap,
    PixmapPaint, Point, Rect as SkRect, SpreadMode, Stroke as SkStroke, Transform,
};

use super::layout::{self, CANVAS_H, CANVAS_W};
use super::scene::{Background, Item, Line, Rect, Rgba8, Scene, Text};
use super::text::TextEngine;

pub fn render_scene(scene: &Scene, engine: &TextEngine) -> Pixmap {
    let mut pixmap = Pixmap::new(CANVAS_W, CANVAS_H).expect("canvas size is valid");
    pixmap.fill(Color::TRANSPARENT);
    draw_items(&mut pixmap, &scene.items, engine);
    pixmap
}

pub fn capture_frame(scene: &Scene, engine: &TextEngine) -> Pixmap {
    let mut canvas = Pixmap::new(CANVAS_W, CANVAS_H).expect("canvas size is valid");
    paint_background(&mut canvas, &scene.background);
    let items = render_scene(scene, engine);
    canvas.draw_pixmap(
        0,
        0,
        items.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    canvas
}

pub struct Compositor {
    background: Pixmap,
    scale: f32,
    dx: f32,
    dy: f32,
    quality: FilterQuality,
}

impl Compositor {
    pub fn new(background: &Background, width: u32, height: u32, scale: f32) -> Self {
        let mut pixmap = Pixmap::new(width, height).expect("device size is valid");
        paint_background(&mut pixmap, background);
        let (dx, dy) = layout::centered_offset(width, height, scale);
        let quality = if (scale - 1.0).abs() < 0.001 {
            FilterQuality::Nearest
        } else {
            FilterQuality::Bilinear
        };
        Self {
            background: pixmap,
            scale,
            dx,
            dy,
            quality,
        }
    }

    pub fn compose(&self, scene: &Scene, engine: &TextEngine) -> Pixmap {
        let mut device = self.background.clone();
        let items = render_scene(scene, engine);
        let paint = PixmapPaint {
            opacity: 1.0,
            blend_mode: tiny_skia::BlendMode::SourceOver,
            quality: self.quality,
        };
        let transform =
            Transform::from_scale(self.scale, self.scale).post_translate(self.dx, self.dy);
        device.draw_pixmap(0, 0, items.as_ref(), &paint, transform, None);
        device
    }
}

pub fn compose_device(
    scene: &Scene,
    engine: &TextEngine,
    width: u32,
    height: u32,
    scale: f32,
) -> Pixmap {
    Compositor::new(&scene.background, width, height, scale).compose(scene, engine)
}

fn paint_background(pixmap: &mut Pixmap, background: &Background) {
    match background {
        Background::Solid(color) => {
            pixmap.fill(to_color(*color));
        }
        Background::VerticalGradient(top, bottom) => {
            let Some(shader) = LinearGradient::new(
                Point::from_xy(0.0, 0.0),
                Point::from_xy(0.0, pixmap.height() as f32),
                vec![
                    tiny_skia::GradientStop::new(0.0, to_color(*top)),
                    tiny_skia::GradientStop::new(1.0, to_color(*bottom)),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) else {
                return;
            };
            let paint = Paint {
                shader,
                ..Paint::default()
            };
            let rect = SkRect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
                .expect("pixmap rect is valid");
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }
}

fn draw_items(pixmap: &mut Pixmap, items: &[Item], engine: &TextEngine) {
    for item in items {
        match item {
            Item::Rect(rect) => draw_rect(pixmap, rect),
            Item::Text(text) => draw_text(pixmap, text, engine),
            Item::Line(line) => draw_line(pixmap, line),
        }
    }
}

fn draw_rect(pixmap: &mut Pixmap, rect: &Rect) {
    let Some(sk_rect) = SkRect::from_xywh(rect.x, rect.y, rect.w, rect.h) else {
        return;
    };
    let path = rounded_path(sk_rect, rect.radius);

    if rect.shadow {
        for (offset, alpha) in [(10.0, 115), (4.0, 46)] {
            let shadow_rect = match SkRect::from_xywh(rect.x, rect.y + offset, rect.w, rect.h) {
                Some(rect) => rect,
                None => continue,
            };
            let shadow_path = rounded_path(shadow_rect, rect.radius);
            let mut paint = Paint::default();
            paint.set_color_rgba8(0, 0, 0, alpha);
            pixmap.fill_path(
                &shadow_path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    let mut paint = Paint::default();
    paint.set_color_rgba8(rect.fill.r, rect.fill.g, rect.fill.b, rect.fill.a);
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    if let Some(stroke) = rect.stroke {
        let mut paint = Paint::default();
        paint.set_color_rgba8(
            stroke.color.r,
            stroke.color.g,
            stroke.color.b,
            stroke.color.a,
        );
        let stroke = SkStroke {
            width: stroke.width,
            ..SkStroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_line(pixmap: &mut Pixmap, line: &Line) {
    let mut builder = PathBuilder::new();
    builder.move_to(line.x1, line.y1);
    builder.line_to(line.x2, line.y2);
    let Some(path) = builder.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(line.color.r, line.color.g, line.color.b, line.color.a);
    let stroke = SkStroke {
        width: line.width,
        line_cap: LineCap::Square,
        ..SkStroke::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}

fn draw_text(pixmap: &mut Pixmap, text: &Text, engine: &TextEngine) {
    engine.paint(
        pixmap, &text.text, text.x, text.y, text.w, text.h, text.size, text.font, text.color,
        text.align,
    );
}

fn rounded_path(rect: SkRect, radius: f32) -> tiny_skia::Path {
    let radius = radius.max(0.0).min(rect.width().min(rect.height()) / 2.0);
    let mut builder = PathBuilder::new();
    let x = rect.left();
    let y = rect.top();
    let right = rect.right();
    let bottom = rect.bottom();

    builder.move_to(x + radius, y);
    builder.line_to(right - radius, y);
    builder.quad_to(right, y, right, y + radius);
    builder.line_to(right, bottom - radius);
    builder.quad_to(right, bottom, right - radius, bottom);
    builder.line_to(x + radius, bottom);
    builder.quad_to(x, bottom, x, bottom - radius);
    builder.line_to(x, y + radius);
    builder.quad_to(x, y, x + radius, y);
    builder.close();

    builder.finish().expect("rounded rect path is valid")
}

fn to_color(color: Rgba8) -> Color {
    Color::from_rgba8(color.r, color.g, color.b, color.a)
}

pub fn straight_rgba(pixmap: &Pixmap) -> Vec<u8> {
    let data = pixmap.data();
    let mut output = Vec::with_capacity(data.len());
    for pixel in data.as_chunks::<4>().0 {
        let alpha = pixel[3] as u16;
        if alpha == 255 || alpha == 0 {
            output.extend_from_slice(pixel);
        } else {
            let convert = |value: u8| ((value as u16 * 255 + alpha / 2) / alpha).min(255) as u8;
            output.extend_from_slice(&[
                convert(pixel[0]),
                convert(pixel[1]),
                convert(pixel[2]),
                255,
            ]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Calculator;
    use crate::i18n::{Lang, SoundStatus};
    use crate::render::scene::{calculator_scene, CalculatorView};
    use crate::theme::Theme;

    fn scene<'a>(theme: &'a Theme, history: &'a crate::core::History) -> Scene {
        calculator_scene(&CalculatorView {
            lang: Lang::Pt,
            theme,
            display: crate::core::DisplayState::Value("0".to_string()),
            expression: None,
            history,
            selection: None,
            sound: SoundStatus::On,
            pressed: None,
            history_fade: None,
        })
    }

    #[test]
    fn capture_is_canvas_sized_and_not_empty() {
        let engine = TextEngine::new().unwrap();
        let theme = Theme::truecolor();
        let history = crate::core::History::new();
        let pixmap = capture_frame(&scene(&theme, &history), &engine);
        assert_eq!(pixmap.width(), CANVAS_W);
        assert_eq!(pixmap.height(), CANVAS_H);
        let unique = pixmap
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<std::collections::HashSet<_>>();
        assert!(unique.len() > 10, "scene has too few colors");
    }

    #[test]
    fn capture_is_deterministic() {
        let engine = TextEngine::new().unwrap();
        let theme = Theme::truecolor();
        let history = crate::core::History::new();
        let first = capture_frame(&scene(&theme, &history), &engine);
        let second = capture_frame(&scene(&theme, &history), &engine);
        assert_eq!(first.data(), second.data());
    }

    #[test]
    fn cached_compositor_matches_direct_composition() {
        let engine = TextEngine::new().unwrap();
        let theme = Theme::truecolor();
        let history = crate::core::History::new();
        let scene = scene(&theme, &history);
        let compositor = Compositor::new(&scene.background, 1920, 1080, 1.0);
        let cached = compositor.compose(&scene, &engine);
        let direct = compose_device(&scene, &engine, 1920, 1080, 1.0);
        assert_eq!(cached.data(), direct.data());
    }

    #[test]
    fn device_composition_matches_resolution() {
        let engine = TextEngine::new().unwrap();
        let theme = Theme::truecolor();
        let history = crate::core::History::new();
        let scene = scene(&theme, &history);
        for (width, height) in [(1920, 1080), (2560, 1440), (1366, 768), (1024, 768)] {
            let pixmap = compose_device(
                &scene,
                &engine,
                width,
                height,
                layout::scale_for(width, height),
            );
            assert_eq!((pixmap.width(), pixmap.height()), (width, height));
        }
    }

    #[test]
    fn history_changes_capture() {
        let engine = TextEngine::new().unwrap();
        let theme = Theme::truecolor();
        let mut calculator = Calculator::new();
        calculator.handle(crate::core::CalcAction::Digit('2'));
        calculator.handle(crate::core::CalcAction::Operator(
            crate::core::Operator::Add,
        ));
        calculator.handle(crate::core::CalcAction::Digit('3'));
        calculator.handle(crate::core::CalcAction::Equals);
        let empty_history = crate::core::History::new();
        let empty = capture_frame(&scene(&theme, &empty_history), &engine);
        let filled = capture_frame(&scene(&theme, calculator.history()), &engine);
        assert_ne!(empty.data(), filled.data());
    }

    #[test]
    fn straight_rgba_keeps_opaque_pixels() {
        let mut pixmap = Pixmap::new(2, 2).unwrap();
        pixmap.fill(Color::from_rgba8(10, 20, 30, 255));
        assert_eq!(
            straight_rgba(&pixmap),
            vec![10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255]
        );
    }
}
