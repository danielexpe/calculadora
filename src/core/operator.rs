use rust_decimal::Decimal;

use super::{decimal::round_internal, error::CalcError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operator {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "\u{2212}",
            Self::Mul => "\u{00d7}",
            Self::Div => "\u{00f7}",
        }
    }

    pub const fn is_additive(self) -> bool {
        matches!(self, Self::Add | Self::Sub)
    }

    pub fn apply(self, lhs: Decimal, rhs: Decimal) -> Result<Decimal, CalcError> {
        if self == Self::Div && rhs.is_zero() {
            return Err(CalcError::DivisionByZero);
        }

        let value = match self {
            Self::Add => lhs.checked_add(rhs),
            Self::Sub => lhs.checked_sub(rhs),
            Self::Mul => lhs.checked_mul(rhs),
            Self::Div => lhs.checked_div(rhs),
        };

        value.map(round_internal).ok_or(CalcError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    #[test]
    fn symbols_are_stable() {
        assert_eq!(Operator::Add.symbol(), "+");
        assert_eq!(Operator::Sub.symbol(), "−");
        assert_eq!(Operator::Mul.symbol(), "×");
        assert_eq!(Operator::Div.symbol(), "÷");
    }

    #[test]
    fn applies_each_operation() {
        assert_eq!(
            Operator::Add.apply(decimal("2"), decimal("3")),
            Ok(decimal("5"))
        );
        assert_eq!(
            Operator::Sub.apply(decimal("2"), decimal("5")),
            Ok(decimal("-3"))
        );
        assert_eq!(
            Operator::Mul.apply(decimal("2"), decimal("3")),
            Ok(decimal("6"))
        );
        assert_eq!(
            Operator::Div.apply(decimal("7"), decimal("2")),
            Ok(decimal("3.5"))
        );
    }

    #[test]
    fn rejects_division_by_zero() {
        assert_eq!(
            Operator::Div.apply(decimal("1"), Decimal::ZERO),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn rounds_to_internal_scale() {
        assert_eq!(
            Operator::Div.apply(decimal("1"), decimal("3")),
            Ok(decimal("0.33333333"))
        );
    }

    #[test]
    fn detects_decimal_overflow() {
        assert_eq!(
            Operator::Mul.apply(Decimal::MAX, Decimal::MAX),
            Err(CalcError::Overflow)
        );
    }
}
