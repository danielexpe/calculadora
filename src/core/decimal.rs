use rust_decimal::{Decimal, RoundingStrategy};

pub const MAX_INPUT_DIGITS: usize = 15;
pub const INTERNAL_SCALE: u32 = 8;
pub const MAX_DISPLAY_CHARS: usize = 16;
pub const FIXED_SIGNIFICANT_DIGITS: i32 = 12;
pub const SCIENTIFIC_DECIMALS: u32 = 6;

const MINUS: char = '\u{2212}';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberLocale {
    PtBr,
    En,
}

impl NumberLocale {
    pub const fn decimal_separator(self) -> char {
        match self {
            Self::PtBr => ',',
            Self::En => '.',
        }
    }

    pub const fn grouping_separator(self) -> char {
        match self {
            Self::PtBr => '.',
            Self::En => ',',
        }
    }
}

pub fn round_internal(value: Decimal) -> Decimal {
    let rounded =
        value.round_dp_with_strategy(INTERNAL_SCALE, RoundingStrategy::MidpointAwayFromZero);
    if rounded.is_zero() {
        Decimal::ZERO
    } else {
        rounded
    }
}

pub fn format_value(value: Decimal, locale: NumberLocale) -> String {
    if value.is_zero() {
        return "0".to_string();
    }

    let negative = value.is_sign_negative();
    let abs = value.abs();
    let exponent = adjusted_exponent(abs);

    let scientific = exponent >= 12 || exponent <= -10;
    let mut body = if scientific {
        format_scientific(abs, exponent, locale)
    } else {
        format_fixed(abs, locale)
    };

    if !scientific && with_sign(&body, negative).chars().count() > MAX_DISPLAY_CHARS {
        body = format_scientific(abs, exponent, locale);
    }

    with_sign(&body, negative)
}

pub fn format_entry(raw: &str, negative: bool, locale: NumberLocale) -> String {
    if raw.is_empty() {
        return "0".to_string();
    }

    let mut parts = raw.splitn(2, '.');
    let integer = parts.next().unwrap_or("");
    let fraction = parts.next();
    let integer = if integer.is_empty() { "0" } else { integer };
    let has_nonzero_digit = raw
        .chars()
        .any(|character| character.is_ascii_digit() && character != '0');
    let sign = negative && has_nonzero_digit;

    let grouped = render_entry_body(integer, fraction, locale, locale.grouping_separator(), sign);
    if grouped.chars().count() <= MAX_DISPLAY_CHARS {
        return grouped;
    }

    render_entry_body(integer, fraction, locale, '\0', sign)
}

fn render_entry_body(
    integer: &str,
    fraction: Option<&str>,
    locale: NumberLocale,
    grouping: char,
    negative: bool,
) -> String {
    let mut body = if grouping == '\0' {
        integer.to_string()
    } else {
        group_digits(integer, grouping)
    };

    if let Some(fraction) = fraction {
        body.push(locale.decimal_separator());
        body.push_str(fraction);
    }

    if negative {
        format!("{MINUS}{body}")
    } else {
        body
    }
}

fn with_sign(body: &str, negative: bool) -> String {
    if negative {
        format!("{MINUS}{body}")
    } else {
        body.to_string()
    }
}

fn adjusted_exponent(value: Decimal) -> i32 {
    debug_assert!(!value.is_zero());
    let mantissa_digits = value.mantissa().abs().to_string().len() as i32;
    mantissa_digits - value.scale() as i32 - 1
}

fn format_fixed(value: Decimal, locale: NumberLocale) -> String {
    let exponent = adjusted_exponent(value);
    let decimal_places = (FIXED_SIGNIFICANT_DIGITS - 1 - exponent).max(0) as u32;
    let rounded =
        value.round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointAwayFromZero);

    match plain_parts(rounded) {
        Some((integer, fraction)) => {
            let integer = group_digits(&integer, locale.grouping_separator());
            if fraction.is_empty() {
                integer
            } else {
                format!("{integer}{}{fraction}", locale.decimal_separator())
            }
        }
        None => "0".to_string(),
    }
}

fn format_scientific(value: Decimal, mut exponent: i32, locale: NumberLocale) -> String {
    let mut mantissa = if exponent >= 0 {
        value.checked_div(pow10(exponent as u32)).unwrap_or(value)
    } else {
        value
            .checked_mul(pow10((-exponent) as u32))
            .unwrap_or(value)
    };

    mantissa = mantissa
        .round_dp_with_strategy(SCIENTIFIC_DECIMALS, RoundingStrategy::MidpointAwayFromZero);

    if mantissa >= Decimal::from(10) {
        mantissa = mantissa.checked_div(Decimal::from(10)).unwrap_or(mantissa);
        exponent += 1;
    }

    let mantissa = match plain_parts(mantissa) {
        Some((integer, fraction)) => {
            if fraction.is_empty() {
                integer
            } else {
                format!("{integer}{}{fraction}", locale.decimal_separator())
            }
        }
        None => "0".to_string(),
    };

    if exponent < 0 {
        format!("{mantissa}e{MINUS}{}", -exponent)
    } else {
        format!("{mantissa}e{exponent}")
    }
}

fn pow10(exponent: u32) -> Decimal {
    Decimal::from_i128_with_scale(10i128.pow(exponent), 0)
}

fn plain_parts(value: Decimal) -> Option<(String, String)> {
    if value.is_zero() {
        return Some(("0".to_string(), String::new()));
    }

    let mantissa = value.mantissa().abs().to_string();
    let scale = value.scale() as usize;

    let (integer, fraction) = if scale == 0 {
        (mantissa, String::new())
    } else if mantissa.len() > scale {
        let split = mantissa.len() - scale;
        (mantissa[..split].to_string(), mantissa[split..].to_string())
    } else {
        (
            "0".to_string(),
            format!("{}{}", "0".repeat(scale - mantissa.len()), mantissa),
        )
    };

    Some((integer, fraction.trim_end_matches('0').to_string()))
}

fn group_digits(integer: &str, separator: char) -> String {
    let mut grouped = String::with_capacity(integer.len() + integer.len() / 3);
    let first = integer.len() % 3;
    let first = if first == 0 { 3 } else { first };

    for (index, character) in integer.chars().enumerate() {
        if index > 0 && (index + 3 - first) % 3 == 0 {
            grouped.push(separator);
        }
        grouped.push(character);
    }

    grouped
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    raw: String,
}

impl Entry {
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn digit_count(&self) -> usize {
        self.raw
            .chars()
            .filter(|character| character.is_ascii_digit())
            .count()
    }

    pub fn has_separator(&self) -> bool {
        self.raw.contains('.')
    }

    pub fn clear(&mut self) {
        self.raw.clear();
    }

    pub fn push_digit(&mut self, digit: char) -> bool {
        debug_assert!(digit.is_ascii_digit());
        if self.digit_count() >= MAX_INPUT_DIGITS {
            return false;
        }

        if self.raw == "0" {
            if digit == '0' {
                return false;
            }
            self.raw.clear();
        }

        self.raw.push(digit);
        true
    }

    pub fn push_separator(&mut self) -> bool {
        if self.has_separator() {
            return false;
        }
        self.raw.push('.');
        true
    }

    pub fn backspace(&mut self) -> bool {
        self.raw.pop().is_some()
    }

    pub fn value(&self) -> Option<Decimal> {
        if self.raw.is_empty() {
            return None;
        }

        let normalized;
        let text = if self.raw.starts_with('.') {
            normalized = format!("0{}", self.raw);
            normalized.as_str()
        } else {
            self.raw.as_str()
        };
        let text = text.trim_end_matches('.');
        if text.is_empty() {
            return Some(Decimal::ZERO);
        }

        text.parse::<Decimal>().ok()
    }

    pub fn display(&self, negative: bool, locale: NumberLocale) -> String {
        format_entry(&self.raw, negative, locale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    #[test]
    fn formatting_is_locale_aware() {
        assert_eq!(
            format_value(decimal("1234.5"), NumberLocale::PtBr),
            "1.234,5"
        );
        assert_eq!(format_value(decimal("1234.5"), NumberLocale::En), "1,234.5");
        assert_eq!(format_value(decimal("-42"), NumberLocale::PtBr), "−42");
        assert_eq!(format_value(decimal("0"), NumberLocale::PtBr), "0");
        assert_eq!(format_value(decimal("-0"), NumberLocale::PtBr), "0");
    }

    #[test]
    fn formatting_uses_scientific_for_large_and_small_values() {
        assert_eq!(
            format_value(decimal("1234567890123"), NumberLocale::PtBr),
            "1,234568e12"
        );
        assert_eq!(
            format_value(decimal("0.000000001"), NumberLocale::PtBr),
            "0,000000001"
        );
        assert_eq!(
            format_value(decimal("0.0000000001"), NumberLocale::En),
            "1e−10"
        );
    }

    #[test]
    fn formatting_respects_maximum_width() {
        for value in [
            decimal("999999999999"),
            decimal("123456789012"),
            decimal("-999999999999"),
            decimal("0.000000001234567"),
            decimal("123456.789012"),
        ] {
            let formatted = format_value(value, NumberLocale::PtBr);
            assert!(
                formatted.chars().count() <= MAX_DISPLAY_CHARS,
                "{formatted} exceeds {MAX_DISPLAY_CHARS} characters"
            );
        }
    }

    #[test]
    fn formatting_groups_thousands() {
        assert_eq!(
            format_value(decimal("1234567"), NumberLocale::PtBr),
            "1.234.567"
        );
        assert_eq!(
            format_value(decimal("1234567"), NumberLocale::En),
            "1,234,567"
        );
    }

    #[test]
    fn rounding_uses_internal_scale_and_normalizes_negative_zero() {
        let rounded = round_internal(decimal("1.234567895"));
        assert_eq!(rounded, decimal("1.23456790"));
        assert_eq!(round_internal(decimal("-0.000000001")), Decimal::ZERO);
        assert_eq!(round_internal(decimal("0")), Decimal::ZERO);
    }

    #[test]
    fn entry_accepts_digits_and_one_separator() {
        let mut entry = Entry::default();
        assert!(entry.push_digit('0'));
        assert!(!entry.push_digit('0'));
        assert!(entry.push_digit('5'));
        assert_eq!(entry.raw(), "5");
        assert!(entry.push_separator());
        assert!(!entry.push_separator());
        assert!(entry.push_digit('2'));
        assert_eq!(entry.value(), Some(decimal("5.2")));
        assert_eq!(entry.display(false, NumberLocale::PtBr), "5,2");
    }

    #[test]
    fn entry_normalizes_leading_integer_zeros() {
        let mut entry = Entry::default();
        for digit in "0005".chars() {
            entry.push_digit(digit);
        }
        assert_eq!(entry.raw(), "5");
    }

    #[test]
    fn entry_limits_to_fifteen_digits() {
        let mut entry = Entry::default();
        for digit in "1234567890123456".chars() {
            entry.push_digit(digit);
        }
        assert_eq!(entry.digit_count(), MAX_INPUT_DIGITS);
        assert_eq!(entry.raw(), "123456789012345");
    }

    #[test]
    fn entry_backspace_and_clear() {
        let mut entry = Entry::default();
        entry.push_digit('1');
        entry.push_digit('2');
        assert!(entry.backspace());
        assert_eq!(entry.raw(), "1");
        entry.clear();
        assert!(entry.is_empty());
        assert_eq!(entry.value(), None);
    }

    #[test]
    fn entry_accepts_bare_and_trailing_separator() {
        let mut entry = Entry::default();
        entry.push_separator();
        assert_eq!(entry.value(), Some(Decimal::ZERO));
        assert_eq!(entry.display(false, NumberLocale::En), "0.");
        entry.push_digit('5');
        assert_eq!(entry.display(false, NumberLocale::En), "0.5");
        assert_eq!(entry.value(), Some(decimal("0.5")));
    }

    #[test]
    fn entry_display_uses_minus_and_negative_flag() {
        let mut entry = Entry::default();
        entry.push_digit('4');
        entry.push_digit('2');
        assert_eq!(entry.display(true, NumberLocale::PtBr), "−42");
        assert_eq!(entry.display(false, NumberLocale::PtBr), "42");
    }

    #[test]
    fn entry_display_groups_thousands_when_it_fits() {
        let mut entry = Entry::default();
        for digit in "12345".chars() {
            entry.push_digit(digit);
        }
        assert_eq!(entry.display(false, NumberLocale::PtBr), "12.345");
        assert_eq!(entry.display(false, NumberLocale::En), "12,345");
    }

    #[test]
    fn entry_display_drops_grouping_when_too_wide() {
        let mut entry = Entry::default();
        for digit in "123456789012345".chars() {
            entry.push_digit(digit);
        }
        let displayed = entry.display(false, NumberLocale::PtBr);
        assert!(!displayed.contains('.'));
        assert_eq!(displayed, "123456789012345");
    }

    #[test]
    fn entry_display_keeps_trailing_separator() {
        let mut entry = Entry::default();
        entry.push_digit('5');
        entry.push_separator();
        assert_eq!(entry.display(false, NumberLocale::PtBr), "5,");
        assert_eq!(entry.display(false, NumberLocale::En), "5.");
    }

    #[test]
    fn entry_display_combines_sign_and_grouping() {
        let mut entry = Entry::default();
        for digit in "12345".chars() {
            entry.push_digit(digit);
        }
        entry.push_separator();
        entry.push_digit('5');
        assert_eq!(entry.display(true, NumberLocale::PtBr), "−12.345,5");
    }

    #[test]
    fn formatting_rounds_to_twelve_significant_digits() {
        assert_eq!(
            format_value(decimal("1.23456789012345"), NumberLocale::PtBr),
            "1,23456789012"
        );
    }

    #[test]
    fn formatting_carries_into_scientific_notation() {
        assert_eq!(
            format_value(decimal("999999999999.9"), NumberLocale::PtBr),
            "1e12"
        );
    }

    #[test]
    fn formatting_handles_decimal_max_within_width() {
        let formatted = format_value(Decimal::MAX, NumberLocale::PtBr);
        assert!(formatted.chars().count() <= MAX_DISPLAY_CHARS);
        assert!(formatted.contains('e'));
    }

    #[test]
    fn grouping_handles_short_integer_parts() {
        assert_eq!(format_value(decimal("123"), NumberLocale::PtBr), "123");
        assert_eq!(format_value(decimal("1234"), NumberLocale::PtBr), "1.234");
        assert_eq!(
            format_value(decimal("123456"), NumberLocale::PtBr),
            "123.456"
        );
    }
}
