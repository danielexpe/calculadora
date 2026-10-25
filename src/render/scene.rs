use crate::core::{DisplayState, History};
use crate::i18n::{error_message, sound_status, strings, Lang, SoundStatus};
use crate::theme::Theme;

use super::layout::{self, KeyId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn hex(rgb: u32) -> Self {
        Self::new(
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
            255,
        )
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub const fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background {
    Solid(Rgba8),
    VerticalGradient(Rgba8, Rgba8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontKind {
    InterRegular,
    InterSemiBold,
    MonoRegular,
    MonoMedium,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub color: Rgba8,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub fill: Rgba8,
    pub stroke: Option<Stroke>,
    pub shadow: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text: String,
    pub font: FontKind,
    pub size: f32,
    pub color: Rgba8,
    pub align: Align,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub width: f32,
    pub color: Rgba8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Rect(Rect),
    Text(Text),
    Line(Line),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub background: Background,
    pub items: Vec<Item>,
}

impl Scene {
    pub fn new(background: Background) -> Self {
        Self {
            background,
            items: Vec::new(),
        }
    }

    pub fn rect(&mut self, rect: Rect) {
        self.items.push(Item::Rect(rect));
    }

    pub fn text(&mut self, text: Text) {
        self.items.push(Item::Text(text));
    }

    pub fn line(&mut self, line: Line) {
        self.items.push(Item::Line(line));
    }
}

pub struct CalculatorView<'a> {
    pub lang: Lang,
    pub theme: &'a Theme,
    pub display: DisplayState,
    pub expression: Option<String>,
    pub history: &'a History,
    pub selection: Option<usize>,
    pub sound: SoundStatus,
    pub pressed: Option<KeyId>,
}

pub fn calculator_scene(view: &CalculatorView<'_>) -> Scene {
    let theme = view.theme;
    let labels = strings(view.lang);
    let mut scene = Scene::new(Background::VerticalGradient(
        theme.canvas,
        theme.canvas_glow,
    ));

    scene.rect(panel(
        (
            layout::HISTORY_X,
            layout::HISTORY_Y,
            layout::HISTORY_W,
            layout::HISTORY_H,
        ),
        24.0,
        theme.panel,
        theme.border,
        false,
    ));
    scene.rect(panel(
        (
            layout::CARD_X,
            layout::CARD_Y,
            layout::CARD_W,
            layout::CARD_H,
        ),
        28.0,
        theme.card,
        theme.border,
        true,
    ));

    scene.text(Text {
        x: layout::HISTORY_TEXT_X,
        y: 80.0,
        w: layout::HISTORY_TEXT_W,
        h: 32.0,
        text: labels.history_title.to_string(),
        font: FontKind::InterSemiBold,
        size: 22.0,
        color: theme.text_primary,
        align: Align::Left,
    });
    scene.line(Line {
        x1: layout::HISTORY_TEXT_X,
        y1: layout::HISTORY_DIVIDER_Y,
        x2: layout::HISTORY_TEXT_X + layout::HISTORY_TEXT_W,
        y2: layout::HISTORY_DIVIDER_Y,
        width: 1.0,
        color: theme.border,
    });

    history(&mut scene, view);

    let error = match &view.display {
        DisplayState::Error(error) => Some(*error),
        DisplayState::Value(_) => None,
    };
    let display_border = if error.is_some() {
        theme.danger
    } else {
        theme.border
    };
    scene.rect(panel(
        (
            layout::DISPLAY_X,
            layout::DISPLAY_Y,
            layout::DISPLAY_W,
            layout::DISPLAY_H,
        ),
        20.0,
        theme.display,
        display_border,
        false,
    ));

    let (expression, value, value_color, value_size) = match error {
        Some(error) => (
            error_message(view.lang, error).to_string(),
            labels.error_title.to_string(),
            theme.danger,
            56.0,
        ),
        None => (
            view.expression.clone().unwrap_or_default(),
            match &view.display {
                DisplayState::Value(value) => value.clone(),
                DisplayState::Error(_) => String::new(),
            },
            theme.text_primary,
            64.0,
        ),
    };

    scene.text(Text {
        x: layout::DISPLAY_TEXT_X,
        y: layout::EXPRESSION_Y,
        w: layout::DISPLAY_TEXT_W,
        h: 24.0,
        text: expression,
        font: FontKind::MonoRegular,
        size: 18.0,
        color: theme.text_secondary,
        align: Align::Right,
    });
    scene.text(Text {
        x: layout::DISPLAY_TEXT_X,
        y: layout::VALUE_Y,
        w: layout::DISPLAY_TEXT_W,
        h: 88.0,
        text: value,
        font: FontKind::MonoMedium,
        size: value_size,
        color: value_color,
        align: Align::Right,
    });

    keypad(&mut scene, view, theme);

    scene.text(Text {
        x: layout::FOOTER_X,
        y: layout::FOOTER_Y,
        w: 500.0,
        h: layout::FOOTER_H,
        text: labels.footer_hints.to_string(),
        font: FontKind::InterRegular,
        size: 14.0,
        color: theme.text_muted,
        align: Align::Left,
    });
    scene.text(Text {
        x: 1064.0,
        y: layout::FOOTER_Y,
        w: 24.0,
        h: layout::FOOTER_H,
        text: match view.lang {
            Lang::Pt => labels.lang_pt.to_string(),
            Lang::En => labels.lang_en.to_string(),
        },
        font: FontKind::InterRegular,
        size: 14.0,
        color: theme.text_secondary,
        align: Align::Left,
    });
    scene.text(Text {
        x: 1092.0,
        y: layout::FOOTER_Y,
        w: 148.0,
        h: layout::FOOTER_H,
        text: sound_status(view.lang, view.sound).to_string(),
        font: FontKind::InterRegular,
        size: 14.0,
        color: theme.text_secondary,
        align: Align::Right,
    });

    scene
}

fn history(scene: &mut Scene, view: &CalculatorView<'_>) {
    let theme = view.theme;
    let labels = strings(view.lang);
    let area_y = layout::HISTORY_ENTRY_START_Y;
    let area_h = layout::HISTORY_ROW_H * layout::HISTORY_VISIBLE as f32;

    if view.history.is_empty() {
        scene.text(Text {
            x: layout::HISTORY_TEXT_X,
            y: area_y,
            w: layout::HISTORY_TEXT_W,
            h: area_h,
            text: labels.history_empty.to_string(),
            font: FontKind::InterRegular,
            size: 17.0,
            color: theme.text_muted,
            align: Align::Center,
        });
        return;
    }

    let len = view.history.len();
    let visible = len.min(layout::HISTORY_VISIBLE);
    let first = len - visible;

    for (slot, index) in (first..len).enumerate() {
        let row = slot + layout::HISTORY_VISIBLE - visible;
        let y = area_y + row as f32 * layout::HISTORY_ROW_H;
        let entry = view.history.get(index).expect("history index in range");
        let selected = view.selection == Some(index);

        if selected {
            scene.rect(Rect {
                x: 88.0,
                y,
                w: 352.0,
                h: layout::HISTORY_ROW_H,
                radius: 8.0,
                fill: theme.surface_selected,
                stroke: None,
                shadow: false,
            });
            scene.rect(Rect {
                x: 88.0,
                y,
                w: 3.0,
                h: layout::HISTORY_ROW_H,
                radius: 1.5,
                fill: theme.accent,
                stroke: None,
                shadow: false,
            });
        }

        scene.text(Text {
            x: layout::HISTORY_TEXT_X,
            y: y + 2.0,
            w: layout::HISTORY_TEXT_W,
            h: 20.0,
            text: entry.expression(view.lang.number_locale()),
            font: FontKind::MonoRegular,
            size: 18.0,
            color: theme.text_secondary,
            align: Align::Right,
        });
        scene.text(Text {
            x: layout::HISTORY_TEXT_X,
            y: y + 20.0,
            w: layout::HISTORY_TEXT_W,
            h: 26.0,
            text: crate::core::format_value(entry.result, view.lang.number_locale()),
            font: FontKind::MonoRegular,
            size: 22.0,
            color: theme.text_primary,
            align: Align::Right,
        });
    }
}

fn keypad(scene: &mut Scene, view: &CalculatorView<'_>, theme: &Theme) {
    for spec in layout::keys(view.lang) {
        let pressed = view.pressed == Some(spec.id);
        let offset = if pressed { 1.0 } else { 0.0 };
        let (fill, stroke, color, font, size) = key_style(spec.id, pressed, theme);

        scene.rect(Rect {
            x: spec.x,
            y: spec.y + offset,
            w: spec.w,
            h: spec.h,
            radius: 14.0,
            fill,
            stroke: Some(Stroke {
                color: stroke,
                width: 1.0,
            }),
            shadow: false,
        });
        scene.text(Text {
            x: spec.x,
            y: spec.y + offset,
            w: spec.w,
            h: spec.h,
            text: key_label(spec.id, view.lang),
            font,
            size,
            color,
            align: Align::Center,
        });
    }
}

fn key_style(id: KeyId, pressed: bool, theme: &Theme) -> (Rgba8, Rgba8, Rgba8, FontKind, f32) {
    if pressed {
        return (
            theme.key_pressed,
            theme.accent_strong,
            theme.text_primary,
            FontKind::MonoMedium,
            30.0,
        );
    }

    match id {
        KeyId::Equals => (
            theme.accent,
            theme.accent,
            theme.on_accent,
            FontKind::MonoMedium,
            34.0,
        ),
        KeyId::Operator(_) => (
            theme.operator_bg,
            theme.operator_border,
            theme.operator,
            FontKind::MonoMedium,
            32.0,
        ),
        KeyId::Digit(_) => (
            theme.key,
            theme.border,
            theme.text_primary,
            FontKind::MonoMedium,
            30.0,
        ),
        KeyId::DecimalSeparator => (
            theme.key,
            theme.border,
            theme.text_primary,
            FontKind::MonoMedium,
            30.0,
        ),
        KeyId::ToggleSign => (
            theme.key,
            theme.border,
            theme.text_primary,
            FontKind::MonoMedium,
            30.0,
        ),
        KeyId::Clear => (
            theme.key_utility,
            theme.border,
            theme.danger,
            FontKind::InterSemiBold,
            20.0,
        ),
        KeyId::Backspace => (
            theme.key_utility,
            theme.border,
            theme.text_secondary,
            FontKind::InterSemiBold,
            20.0,
        ),
        KeyId::Percent => (
            theme.key_utility,
            theme.border,
            theme.warning,
            FontKind::InterSemiBold,
            22.0,
        ),
    }
}

fn key_label(id: KeyId, lang: Lang) -> String {
    match id {
        KeyId::Digit(digit) => digit.to_string(),
        KeyId::DecimalSeparator => lang.number_locale().decimal_separator().to_string(),
        KeyId::Operator(operator) => operator.symbol().to_string(),
        KeyId::Clear => "AC".to_string(),
        KeyId::Backspace => "DEL".to_string(),
        KeyId::Percent => "%".to_string(),
        KeyId::ToggleSign => "\u{00b1}".to_string(),
        KeyId::Equals => "=".to_string(),
    }
}

fn panel(
    size: (f32, f32, f32, f32),
    radius: f32,
    fill: Rgba8,
    border: Rgba8,
    shadow: bool,
) -> Rect {
    let (x, y, w, h) = size;
    Rect {
        x,
        y,
        w,
        h,
        radius,
        fill,
        stroke: Some(Stroke {
            color: border,
            width: 1.0,
        }),
        shadow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Calculator;
    use crate::theme::Theme;

    fn view<'a>(theme: &'a Theme, history: &'a History) -> CalculatorView<'a> {
        CalculatorView {
            lang: Lang::Pt,
            theme,
            display: DisplayState::Value("0".to_string()),
            expression: None,
            history,
            selection: None,
            sound: SoundStatus::On,
            pressed: None,
        }
    }

    #[test]
    fn initial_scene_contains_panels_keys_and_empty_history() {
        let theme = Theme::truecolor();
        let history = History::new();
        let scene = calculator_scene(&view(&theme, &history));
        let rects = scene
            .items
            .iter()
            .filter(|item| matches!(item, Item::Rect(_)))
            .count();
        let texts = scene
            .items
            .iter()
            .filter(|item| matches!(item, Item::Text(_)))
            .count();
        assert_eq!(rects, 2 + 1 + 20);
        assert!(texts > 23);
        assert!(scene.items.iter().any(|item| matches!(
            item,
            Item::Text(Text { text, .. }) if text == "Sem operações ainda"
        )));
    }

    #[test]
    fn scene_uses_language_for_separator_and_footer() {
        let theme = Theme::truecolor();
        let history = History::new();
        let mut view = view(&theme, &history);
        view.lang = Lang::En;
        let scene = calculator_scene(&view);
        assert!(scene.items.iter().any(|item| matches!(
            item,
            Item::Text(Text { text, .. }) if text == "."
        )));
        assert!(scene.items.iter().any(|item| matches!(
            item,
            Item::Text(Text { text, .. }) if text == "No operations yet"
        )));
    }

    #[test]
    fn error_scene_shows_localized_cause() {
        let theme = Theme::truecolor();
        let history = History::new();
        let mut view = view(&theme, &history);
        view.display = DisplayState::Error(crate::core::CalcError::DivisionByZero);
        let scene = calculator_scene(&view);
        assert!(scene.items.iter().any(|item| matches!(
            item,
            Item::Text(Text { text, .. }) if text == "Erro"
        )));
        assert!(scene.items.iter().any(|item| matches!(
            item,
            Item::Text(Text { text, .. }) if text == "Divisão por zero"
        )));
    }

    #[test]
    fn history_entries_are_rendered() {
        let theme = Theme::truecolor();
        let mut calculator = Calculator::new();
        for digit in "2".chars() {
            calculator.handle(crate::core::CalcAction::Digit(digit));
        }
        calculator.handle(crate::core::CalcAction::Operator(
            crate::core::Operator::Add,
        ));
        for digit in "3".chars() {
            calculator.handle(crate::core::CalcAction::Digit(digit));
        }
        calculator.handle(crate::core::CalcAction::Equals);
        let scene = calculator_scene(&view(&theme, calculator.history()));
        assert!(scene
            .items
            .iter()
            .any(|item| matches!(item, Item::Text(Text { text, .. }) if text == "2 + 3 =")));
        assert!(scene
            .items
            .iter()
            .any(|item| matches!(item, Item::Text(Text { text, .. }) if text == "5")));
    }

    #[test]
    fn selected_entry_adds_highlight_bar() {
        let theme = Theme::truecolor();
        let mut calculator = Calculator::new();
        calculator.handle(crate::core::CalcAction::Digit('2'));
        calculator.handle(crate::core::CalcAction::Operator(
            crate::core::Operator::Add,
        ));
        calculator.handle(crate::core::CalcAction::Digit('3'));
        calculator.handle(crate::core::CalcAction::Equals);
        let mut view = view(&theme, calculator.history());
        view.selection = Some(0);
        let scene = calculator_scene(&view);
        assert!(scene.items.iter().any(|item| matches!(
            item,
            Item::Rect(Rect { fill, .. }) if *fill == theme.accent
        )));
    }
}
