use crate::core::Operator;
use crate::i18n::Lang;

pub const CANVAS_W: u32 = 1344;
pub const CANVAS_H: u32 = 756;
pub const TARGET_SCALE: f32 = 0.70;

pub const HISTORY_X: f32 = 64.0;
pub const HISTORY_Y: f32 = 48.0;
pub const HISTORY_W: f32 = 400.0;
pub const HISTORY_H: f32 = 660.0;
pub const HISTORY_TEXT_X: f32 = 92.0;
pub const HISTORY_TEXT_W: f32 = 344.0;
pub const HISTORY_DIVIDER_Y: f32 = 124.0;
pub const HISTORY_ENTRY_START_Y: f32 = 140.0;
pub const HISTORY_ROW_H: f32 = 48.0;
pub const HISTORY_VISIBLE: usize = 10;

pub const CARD_X: f32 = 496.0;
pub const CARD_Y: f32 = 48.0;
pub const CARD_W: f32 = 784.0;
pub const CARD_H: f32 = 660.0;

pub const DISPLAY_X: f32 = 536.0;
pub const DISPLAY_Y: f32 = 88.0;
pub const DISPLAY_W: f32 = 704.0;
pub const DISPLAY_H: f32 = 176.0;
pub const DISPLAY_TEXT_X: f32 = 552.0;
pub const DISPLAY_TEXT_W: f32 = 672.0;
pub const EXPRESSION_Y: f32 = 104.0;
pub const VALUE_Y: f32 = 128.0;

pub const KEY_W: f32 = 167.0;
pub const KEY_H: f32 = 58.0;
pub const KEY_GAP: f32 = 12.0;
pub const KEYPAD_X: f32 = 536.0;
pub const KEYPAD_Y: f32 = 296.0;

pub const FOOTER_X: f32 = 536.0;
pub const FOOTER_Y: f32 = 656.0;
pub const FOOTER_H: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyId {
    Clear,
    Backspace,
    Percent,
    Operator(Operator),
    Digit(char),
    ToggleSign,
    DecimalSeparator,
    Equals,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeySpec {
    pub id: KeyId,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn scale_for(width: u32, height: u32) -> f32 {
    if width == 0 || height == 0 {
        return 0.0;
    }
    (width as f32 / CANVAS_W as f32).min(height as f32 / CANVAS_H as f32) * TARGET_SCALE
}

pub fn canvas_size(scale: f32) -> (u32, u32) {
    let width = (CANVAS_W as f32 * scale).round().max(1.0) as u32;
    let height = (CANVAS_H as f32 * scale).round().max(1.0) as u32;
    (width, height)
}

pub fn centered_offset(width: u32, height: u32, scale: f32) -> (f32, f32) {
    let (canvas_w, canvas_h) = canvas_size(scale);
    (
        (width as f32 - canvas_w as f32) / 2.0,
        (height as f32 - canvas_h as f32) / 2.0,
    )
}

pub fn keys(_lang: Lang) -> [KeySpec; 20] {
    let rows: [[KeyId; 4]; 5] = [
        [
            KeyId::Clear,
            KeyId::Backspace,
            KeyId::Percent,
            KeyId::Operator(Operator::Div),
        ],
        [
            KeyId::Digit('7'),
            KeyId::Digit('8'),
            KeyId::Digit('9'),
            KeyId::Operator(Operator::Mul),
        ],
        [
            KeyId::Digit('4'),
            KeyId::Digit('5'),
            KeyId::Digit('6'),
            KeyId::Operator(Operator::Sub),
        ],
        [
            KeyId::Digit('1'),
            KeyId::Digit('2'),
            KeyId::Digit('3'),
            KeyId::Operator(Operator::Add),
        ],
        [
            KeyId::ToggleSign,
            KeyId::Digit('0'),
            KeyId::DecimalSeparator,
            KeyId::Equals,
        ],
    ];

    let mut specs = [KeySpec {
        id: KeyId::Clear,
        x: 0.0,
        y: 0.0,
        w: KEY_W,
        h: KEY_H,
    }; 20];

    for (row, ids) in rows.iter().enumerate() {
        for (column, id) in ids.iter().enumerate() {
            specs[row * 4 + column] = KeySpec {
                id: *id,
                x: KEYPAD_X + column as f32 * (KEY_W + KEY_GAP),
                y: KEYPAD_Y + row as f32 * (KEY_H + KEY_GAP),
                w: KEY_W,
                h: KEY_H,
            };
        }
    }

    specs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn scale_matches_documented_resolutions() {
        assert_close(scale_for(1920, 1080), 1.0);
        assert_close(scale_for(2560, 1440), 1.333_333);
        assert_close(scale_for(1366, 768), 0.711_111);
        assert_close(scale_for(1024, 768), 0.533_333);
    }

    #[test]
    fn canvas_is_centered() {
        assert_eq!(canvas_size(1.0), (1344, 756));
        let (x, y) = centered_offset(1920, 1080, 1.0);
        assert_close(x, 288.0);
        assert_close(y, 162.0);
        let (x, y) = centered_offset(1024, 768, scale_for(1024, 768));
        assert_close(x, 153.5);
        assert_close(y, 182.5);
    }

    #[test]
    fn keypad_matches_documented_geometry() {
        let keys = keys(Lang::Pt);
        assert_eq!(keys[0].id, KeyId::Clear);
        assert_eq!(keys[0].x, 536.0);
        assert_eq!(keys[0].y, 296.0);
        assert_eq!(keys[3].id, KeyId::Operator(Operator::Div));
        assert_eq!(keys[3].x, 1073.0);
        assert_eq!(keys[19].id, KeyId::Equals);
        assert_eq!(keys[19].x, 1073.0);
        assert_eq!(keys[19].y, 576.0);
        assert_eq!(keys[19].w, 167.0);
        assert_eq!(keys[19].h, 58.0);
    }

    #[test]
    fn every_digit_has_a_key() {
        let ids: Vec<KeyId> = keys(Lang::En).iter().map(|key| key.id).collect();
        for digit in '0'..='9' {
            assert!(ids.contains(&KeyId::Digit(digit)));
        }
        for operator in [Operator::Add, Operator::Sub, Operator::Mul, Operator::Div] {
            assert!(ids.contains(&KeyId::Operator(operator)));
        }
    }

    #[test]
    fn zero_size_has_zero_scale() {
        assert_eq!(scale_for(0, 1080), 0.0);
        assert_eq!(scale_for(1920, 0), 0.0);
    }
}
