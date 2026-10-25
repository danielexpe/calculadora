use rust_decimal::Decimal;

use super::{
    decimal::{format_value, round_internal, Entry, NumberLocale},
    error::CalcError,
    history::{History, HistoryEntry},
    operator::Operator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcAction {
    Digit(char),
    DecimalSeparator,
    Operator(Operator),
    Equals,
    Backspace,
    Clear,
    ToggleSign,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcEvent {
    DigitAccepted,
    SeparatorAccepted,
    OperatorApplied,
    EqualsApplied,
    PercentApplied,
    Cleared,
    BackspaceApplied,
    SignToggled,
    Recalled,
    Error(CalcError),
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayState {
    Value(String),
    Error(CalcError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PercentOperand {
    original: Decimal,
    effective: Decimal,
}

#[derive(Debug, Clone, Default)]
pub struct Calculator {
    entry: Entry,
    negative: bool,
    accumulator: Option<Decimal>,
    pending: Option<Operator>,
    percent: Option<PercentOperand>,
    result_mode: bool,
    last_result: Option<Decimal>,
    last_expression: Option<HistoryEntry>,
    error: Option<CalcError>,
    history: History,
}

impl Calculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle(&mut self, action: CalcAction) -> CalcEvent {
        if self.error.is_some() {
            return match action {
                CalcAction::Digit(digit) => {
                    self.reset_all_keep_history();
                    self.on_digit(digit)
                }
                CalcAction::DecimalSeparator => {
                    self.reset_all_keep_history();
                    self.on_separator()
                }
                CalcAction::Clear => {
                    self.reset_all_keep_history();
                    CalcEvent::Cleared
                }
                _ => CalcEvent::Ignored,
            };
        }

        match action {
            CalcAction::Digit(digit) => self.on_digit(digit),
            CalcAction::DecimalSeparator => self.on_separator(),
            CalcAction::Operator(operator) => self.on_operator(operator),
            CalcAction::Equals => self.on_equals(),
            CalcAction::Backspace => self.on_backspace(),
            CalcAction::Clear => {
                self.reset_all_keep_history();
                CalcEvent::Cleared
            }
            CalcAction::ToggleSign => self.on_toggle_sign(),
            CalcAction::Percent => self.on_percent(),
        }
    }

    pub fn recall(&mut self, value: Decimal) -> CalcEvent {
        self.reset_all_keep_history();
        self.last_result = Some(if value.is_zero() {
            Decimal::ZERO
        } else {
            value
        });
        self.result_mode = true;
        CalcEvent::Recalled
    }

    pub fn display(&self, locale: NumberLocale) -> DisplayState {
        if let Some(error) = self.error {
            return DisplayState::Error(error);
        }

        if let Some(percent) = &self.percent {
            return DisplayState::Value(format_value(percent.effective, locale));
        }

        if !self.entry.is_empty() {
            return DisplayState::Value(self.entry.display(self.negative, locale));
        }

        if self.result_mode {
            return DisplayState::Value(format_value(
                self.last_result.unwrap_or(Decimal::ZERO),
                locale,
            ));
        }

        if let Some(accumulator) = self.accumulator {
            return DisplayState::Value(format_value(accumulator, locale));
        }

        DisplayState::Value("0".to_string())
    }

    pub fn expression(&self, locale: NumberLocale) -> Option<String> {
        if self.error.is_some() {
            return None;
        }

        if self.result_mode {
            if let Some(expression) = &self.last_expression {
                return Some(expression.expression(locale));
            }
        }

        if let Some(operator) = self.pending {
            if let Some(accumulator) = self.accumulator {
                return Some(format!(
                    "{} {}",
                    format_value(accumulator, locale),
                    operator.symbol()
                ));
            }
        }

        None
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn error(&self) -> Option<CalcError> {
        self.error
    }

    pub fn pending(&self) -> Option<Operator> {
        self.pending
    }

    pub fn is_result_mode(&self) -> bool {
        self.result_mode
    }

    fn on_digit(&mut self, digit: char) -> CalcEvent {
        if !digit.is_ascii_digit() {
            return CalcEvent::Ignored;
        }

        if self.result_mode {
            self.reset_all_keep_history();
        } else if self.percent.is_some() {
            self.start_new_operand_keep_pending();
        }

        if self.entry.push_digit(digit) {
            self.last_expression = None;
            CalcEvent::DigitAccepted
        } else {
            CalcEvent::Ignored
        }
    }

    fn on_separator(&mut self) -> CalcEvent {
        if self.result_mode {
            self.reset_all_keep_history();
        } else if self.percent.is_some() {
            self.start_new_operand_keep_pending();
        }

        if self.entry.push_separator() {
            self.last_expression = None;
            CalcEvent::SeparatorAccepted
        } else {
            CalcEvent::Ignored
        }
    }

    fn on_backspace(&mut self) -> CalcEvent {
        if self.result_mode || self.percent.is_some() {
            return CalcEvent::Ignored;
        }

        if self.entry.backspace() {
            if self.entry.is_empty() {
                self.negative = false;
            }
            self.last_expression = None;
            CalcEvent::BackspaceApplied
        } else {
            CalcEvent::Ignored
        }
    }

    fn on_toggle_sign(&mut self) -> CalcEvent {
        if let Some(percent) = &mut self.percent {
            percent.original = -percent.original;
            percent.effective = -percent.effective;
            return CalcEvent::SignToggled;
        }

        if self.result_mode {
            let Some(result) = self.last_result else {
                return CalcEvent::Ignored;
            };
            if result.is_zero() {
                return CalcEvent::Ignored;
            }
            self.last_result = Some(-result);
            self.last_expression = None;
            return CalcEvent::SignToggled;
        }

        let Some(value) = self.entry.value() else {
            return CalcEvent::Ignored;
        };
        if value.is_zero() {
            return CalcEvent::Ignored;
        }

        self.negative = !self.negative;
        self.last_expression = None;
        CalcEvent::SignToggled
    }

    fn on_percent(&mut self) -> CalcEvent {
        if self.percent.is_some() {
            return CalcEvent::Ignored;
        }

        if self.result_mode {
            let Some(result) = self.last_result else {
                return CalcEvent::Ignored;
            };
            let Some(effective) = result.checked_div(Decimal::from(100)) else {
                self.set_error(CalcError::Overflow);
                return CalcEvent::Error(CalcError::Overflow);
            };
            self.percent = Some(PercentOperand {
                original: result,
                effective: round_internal(effective),
            });
            self.result_mode = false;
            self.last_expression = None;
            return CalcEvent::PercentApplied;
        }

        let Some(operand) = self.entry_value() else {
            return CalcEvent::Ignored;
        };

        let effective = match self.pending {
            Some(operator) if operator.is_additive() => {
                let Some(accumulator) = self.accumulator else {
                    return CalcEvent::Ignored;
                };
                accumulator
                    .checked_mul(operand)
                    .and_then(|value| value.checked_div(Decimal::from(100)))
            }
            _ => operand.checked_div(Decimal::from(100)),
        };

        let Some(effective) = effective else {
            self.set_error(CalcError::Overflow);
            return CalcEvent::Error(CalcError::Overflow);
        };

        self.percent = Some(PercentOperand {
            original: operand,
            effective: round_internal(effective),
        });
        self.last_expression = None;
        CalcEvent::PercentApplied
    }

    fn on_operator(&mut self, operator: Operator) -> CalcEvent {
        if let Some(pending) = self.pending {
            if let Some((rhs_input, rhs_effective, percent)) = self.operand() {
                let Some(lhs) = self.accumulator else {
                    return CalcEvent::Ignored;
                };

                return match pending.apply(lhs, rhs_effective) {
                    Ok(result) => {
                        self.history.push(HistoryEntry {
                            lhs,
                            op: pending,
                            rhs_input,
                            rhs_effective,
                            percent,
                            result,
                        });
                        self.accumulator = Some(result);
                        self.pending = Some(operator);
                        self.entry.clear();
                        self.negative = false;
                        self.percent = None;
                        self.result_mode = false;
                        self.last_result = None;
                        self.last_expression = None;
                        CalcEvent::OperatorApplied
                    }
                    Err(error) => {
                        self.set_error(error);
                        CalcEvent::Error(error)
                    }
                };
            }

            self.pending = Some(operator);
            self.last_expression = None;
            return CalcEvent::OperatorApplied;
        }

        let Some(value) = self.current_value() else {
            return CalcEvent::Ignored;
        };

        self.accumulator = Some(value);
        self.pending = Some(operator);
        self.entry.clear();
        self.negative = false;
        self.percent = None;
        self.result_mode = false;
        self.last_result = None;
        self.last_expression = None;
        CalcEvent::OperatorApplied
    }

    fn on_equals(&mut self) -> CalcEvent {
        let Some(pending) = self.pending else {
            return CalcEvent::Ignored;
        };
        let Some(lhs) = self.accumulator else {
            return CalcEvent::Ignored;
        };

        let (rhs_input, rhs_effective, percent) = self.operand().unwrap_or((lhs, lhs, false));

        match pending.apply(lhs, rhs_effective) {
            Ok(result) => {
                let entry = HistoryEntry {
                    lhs,
                    op: pending,
                    rhs_input,
                    rhs_effective,
                    percent,
                    result,
                };
                self.history.push(entry.clone());
                self.accumulator = Some(result);
                self.pending = None;
                self.entry.clear();
                self.negative = false;
                self.percent = None;
                self.result_mode = true;
                self.last_result = Some(result);
                self.last_expression = Some(entry);
                CalcEvent::EqualsApplied
            }
            Err(error) => {
                self.set_error(error);
                CalcEvent::Error(error)
            }
        }
    }

    fn current_value(&self) -> Option<Decimal> {
        if let Some(percent) = &self.percent {
            Some(percent.effective)
        } else if let Some(value) = self.entry_value() {
            Some(value)
        } else if self.result_mode {
            self.last_result
        } else {
            None
        }
    }

    fn operand(&self) -> Option<(Decimal, Decimal, bool)> {
        if let Some(percent) = &self.percent {
            Some((percent.original, percent.effective, true))
        } else {
            self.entry_value().map(|value| (value, value, false))
        }
    }

    fn entry_value(&self) -> Option<Decimal> {
        self.entry
            .value()
            .map(|value| if self.negative { -value } else { value })
    }

    fn start_new_operand_keep_pending(&mut self) {
        self.entry.clear();
        self.negative = false;
        self.percent = None;
        self.result_mode = false;
        self.last_result = None;
        self.last_expression = None;
        if self.pending.is_none() {
            self.accumulator = None;
        }
    }

    fn set_error(&mut self, error: CalcError) {
        self.entry.clear();
        self.negative = false;
        self.accumulator = None;
        self.pending = None;
        self.percent = None;
        self.result_mode = false;
        self.last_result = None;
        self.last_expression = None;
        self.error = Some(error);
    }

    fn reset_all_keep_history(&mut self) {
        self.entry.clear();
        self.negative = false;
        self.accumulator = None;
        self.pending = None;
        self.percent = None;
        self.result_mode = false;
        self.last_result = None;
        self.last_expression = None;
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn calc() -> Calculator {
        Calculator::new()
    }

    fn digits(calculator: &mut Calculator, digits: &str) {
        for digit in digits.chars() {
            calculator.handle(CalcAction::Digit(digit));
        }
    }

    fn value(calculator: &Calculator) -> String {
        match calculator.display(NumberLocale::PtBr) {
            DisplayState::Value(value) => value,
            DisplayState::Error(error) => panic!("unexpected error: {error}"),
        }
    }

    fn expression(calculator: &Calculator) -> Option<String> {
        calculator.expression(NumberLocale::PtBr)
    }

    #[test]
    fn starts_empty() {
        let calculator = calc();
        assert_eq!(value(&calculator), "0");
        assert_eq!(expression(&calculator), None);
        assert!(calculator.history().is_empty());
    }

    fn calculate(lhs: &str, operator: Operator, rhs: &str) -> String {
        let mut calculator = calc();
        digits(&mut calculator, lhs);
        calculator.handle(CalcAction::Operator(operator));
        digits(&mut calculator, rhs);
        calculator.handle(CalcAction::Equals);
        value(&calculator)
    }

    #[test]
    fn subtracts_numbers() {
        assert_eq!(calculate("7", Operator::Sub, "10"), "−3");
    }

    #[test]
    fn multiplies_numbers() {
        assert_eq!(calculate("6", Operator::Mul, "7"), "42");
    }

    #[test]
    fn divides_numbers() {
        assert_eq!(calculate("7", Operator::Div, "2"), "3,5");
        assert_eq!(calculate("1", Operator::Div, "3"), "0,33333333");
    }

    #[test]
    fn separator_after_result_starts_new_calculation() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::DecimalSeparator);
        digits(&mut calculator, "5");
        assert_eq!(value(&calculator), "0,5");
    }

    #[test]
    fn operator_after_result_uses_result() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "4");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "9");
        assert_eq!(calculator.history().len(), 2);
    }

    #[test]
    fn equals_does_not_repeat_last_operation() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        assert_eq!(calculator.handle(CalcAction::Equals), CalcEvent::Ignored);
        assert_eq!(value(&calculator), "5");
        assert_eq!(calculator.history().len(), 1);
    }

    #[test]
    fn zero_sign_toggle_is_noop() {
        let mut calculator = calc();
        digits(&mut calculator, "0");
        assert_eq!(
            calculator.handle(CalcAction::ToggleSign),
            CalcEvent::Ignored
        );
        assert_eq!(value(&calculator), "0");
    }

    #[test]
    fn backspace_to_empty_resets_sign() {
        let mut calculator = calc();
        digits(&mut calculator, "42");
        calculator.handle(CalcAction::ToggleSign);
        calculator.handle(CalcAction::Backspace);
        calculator.handle(CalcAction::Backspace);
        assert_eq!(value(&calculator), "0");
        digits(&mut calculator, "7");
        assert_eq!(value(&calculator), "7");
    }

    #[test]
    fn percent_with_pending_and_no_rhs_is_ignored() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Add));
        assert_eq!(calculator.handle(CalcAction::Percent), CalcEvent::Ignored);
        assert_eq!(value(&calculator), "200");
    }

    #[test]
    fn percent_without_pending_can_feed_an_operator() {
        let mut calculator = calc();
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "5");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "5,1");
        assert_eq!(calculator.history().len(), 1);
    }

    #[test]
    fn adds_numbers() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        assert_eq!(expression(&calculator).as_deref(), Some("2 +"));
        assert_eq!(
            calculator.handle(CalcAction::Equals),
            CalcEvent::EqualsApplied
        );
        assert_eq!(value(&calculator), "5");
        assert_eq!(expression(&calculator).as_deref(), Some("2 + 3 ="));
        assert_eq!(calculator.history().len(), 1);
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "2 + 3 = 5"
        );
    }

    #[test]
    fn decimal_arithmetic_is_exact() {
        let mut calculator = calc();
        digits(&mut calculator, "0");
        calculator.handle(CalcAction::DecimalSeparator);
        digits(&mut calculator, "1");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "0");
        calculator.handle(CalcAction::DecimalSeparator);
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "0,3");
    }

    #[test]
    fn chains_operations_sequentially() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Operator(Operator::Mul));
        digits(&mut calculator, "4");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "20");
        assert_eq!(calculator.history().len(), 2);
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "5 × 4 = 20"
        );
    }

    #[test]
    fn replaces_pending_operator_without_history() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "5");
        assert_eq!(calculator.history().len(), 1);
    }

    #[test]
    fn equals_without_pending_is_noop() {
        let mut calculator = calc();
        digits(&mut calculator, "5");
        assert_eq!(calculator.handle(CalcAction::Equals), CalcEvent::Ignored);
        assert_eq!(value(&calculator), "5");
        assert!(calculator.history().is_empty());
    }

    #[test]
    fn equals_without_rhs_uses_accumulator() {
        let mut calculator = calc();
        digits(&mut calculator, "5");
        calculator.handle(CalcAction::Operator(Operator::Add));
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "10");
    }

    #[test]
    fn digit_after_result_starts_new_calculation() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        digits(&mut calculator, "7");
        assert_eq!(value(&calculator), "7");
        assert_eq!(expression(&calculator), None);
        assert_eq!(calculator.history().len(), 1);
    }

    #[test]
    fn backspace_after_result_is_ignored() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        assert_eq!(calculator.handle(CalcAction::Backspace), CalcEvent::Ignored);
        assert_eq!(value(&calculator), "5");
    }

    #[test]
    fn sign_toggle_works_on_entry_and_result() {
        let mut calculator = calc();
        digits(&mut calculator, "42");
        calculator.handle(CalcAction::ToggleSign);
        assert_eq!(value(&calculator), "−42");
        calculator.handle(CalcAction::ToggleSign);
        assert_eq!(value(&calculator), "42");

        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::ToggleSign);
        assert_eq!(value(&calculator), "−44");
        assert_eq!(expression(&calculator), None);
    }

    #[test]
    fn backspace_removes_last_digit() {
        let mut calculator = calc();
        digits(&mut calculator, "123");
        assert_eq!(
            calculator.handle(CalcAction::Backspace),
            CalcEvent::BackspaceApplied
        );
        assert_eq!(value(&calculator), "12");
    }

    #[test]
    fn clear_resets_calculation_but_keeps_history() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::Clear);
        assert_eq!(value(&calculator), "0");
        assert_eq!(calculator.history().len(), 1);
    }

    #[test]
    fn additive_percent_uses_accumulator() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        assert_eq!(value(&calculator), "20");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "220");
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "200 + 10% = 220"
        );
    }

    #[test]
    fn subtractive_percent_uses_accumulator() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Sub));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "180");
    }

    #[test]
    fn multiplicative_percent_is_fraction() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Mul));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "20");
    }

    #[test]
    fn division_percent_is_fraction() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Div));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "2.000");
    }

    #[test]
    fn percent_without_pending_divides_by_100() {
        let mut calculator = calc();
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        assert_eq!(value(&calculator), "0,1");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "0,1");
    }

    #[test]
    fn repeated_percent_is_ignored() {
        let mut calculator = calc();
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        assert_eq!(calculator.handle(CalcAction::Percent), CalcEvent::Ignored);
        assert_eq!(value(&calculator), "0,1");
    }

    #[test]
    fn digit_after_percent_starts_new_operand() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        digits(&mut calculator, "5");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "205");
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "200 + 5 = 205"
        );
    }

    #[test]
    fn backspace_after_percent_is_ignored() {
        let mut calculator = calc();
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        assert_eq!(calculator.handle(CalcAction::Backspace), CalcEvent::Ignored);
        assert_eq!(value(&calculator), "0,1");
    }

    #[test]
    fn sign_after_percent_negates_typed_and_effective_values() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        calculator.handle(CalcAction::ToggleSign);
        assert_eq!(value(&calculator), "−20");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "180");
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "200 + −10% = 180"
        );
    }

    #[test]
    fn percent_on_result_divides_result() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::Percent);
        assert_eq!(value(&calculator), "0,05");
    }

    #[test]
    fn division_by_zero_enters_error_and_recovers() {
        let mut calculator = calc();
        digits(&mut calculator, "1");
        calculator.handle(CalcAction::Operator(Operator::Div));
        digits(&mut calculator, "0");
        assert_eq!(
            calculator.handle(CalcAction::Equals),
            CalcEvent::Error(CalcError::DivisionByZero)
        );
        assert_eq!(
            calculator.display(NumberLocale::PtBr),
            DisplayState::Error(CalcError::DivisionByZero)
        );
        assert!(calculator.history().is_empty());
        assert_eq!(
            calculator.handle(CalcAction::Operator(Operator::Add)),
            CalcEvent::Ignored
        );
        assert_eq!(calculator.handle(CalcAction::Backspace), CalcEvent::Ignored);
        digits(&mut calculator, "7");
        assert_eq!(value(&calculator), "7");
    }

    #[test]
    fn clear_recovers_from_error() {
        let mut calculator = calc();
        digits(&mut calculator, "1");
        calculator.handle(CalcAction::Operator(Operator::Div));
        digits(&mut calculator, "0");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::Clear);
        assert_eq!(value(&calculator), "0");
        assert_eq!(calculator.error(), None);
    }

    #[test]
    fn percent_then_operator_commits_percent_history() {
        let mut calculator = calc();
        digits(&mut calculator, "200");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "10");
        calculator.handle(CalcAction::Percent);
        calculator.handle(CalcAction::Operator(Operator::Add));
        assert_eq!(value(&calculator), "220");
        assert_eq!(calculator.history().len(), 1);
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "200 + 10% = 220"
        );
        digits(&mut calculator, "5");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "225");
        assert_eq!(calculator.history().len(), 2);
    }

    #[test]
    fn percent_and_operators_are_ignored_in_error_state() {
        let mut calculator = calc();
        digits(&mut calculator, "1");
        calculator.handle(CalcAction::Operator(Operator::Div));
        digits(&mut calculator, "0");
        calculator.handle(CalcAction::Equals);
        assert_eq!(calculator.handle(CalcAction::Percent), CalcEvent::Ignored);
        assert_eq!(
            calculator.handle(CalcAction::Operator(Operator::Add)),
            CalcEvent::Ignored
        );
        assert_eq!(expression(&calculator), None);
        assert_eq!(
            calculator.display(NumberLocale::PtBr),
            DisplayState::Error(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn detects_overflow_without_panic() {
        let mut calculator = calc();
        digits(&mut calculator, "999999999999999");
        calculator.handle(CalcAction::Operator(Operator::Mul));
        digits(&mut calculator, "999999999999999");
        assert_eq!(
            calculator.handle(CalcAction::Equals),
            CalcEvent::Error(CalcError::Overflow)
        );
        assert_eq!(
            calculator.display(NumberLocale::PtBr),
            DisplayState::Error(CalcError::Overflow)
        );
    }

    #[test]
    fn result_can_be_used_as_next_accumulator() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "4");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "9");
        assert_eq!(calculator.history().len(), 2);
        assert_eq!(
            calculator
                .history()
                .last()
                .unwrap()
                .formatted(NumberLocale::PtBr),
            "5 + 4 = 9"
        );
    }

    #[test]
    fn recall_sets_result_and_cancels_pending() {
        let mut calculator = calc();
        digits(&mut calculator, "2");
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "3");
        calculator.handle(CalcAction::Equals);
        calculator.handle(CalcAction::Operator(Operator::Add));
        digits(&mut calculator, "9");
        assert_eq!(calculator.recall(decimal("42")), CalcEvent::Recalled);
        assert_eq!(value(&calculator), "42");
        assert_eq!(calculator.pending(), None);
        assert_eq!(expression(&calculator), None);
        assert_eq!(calculator.history().len(), 1);
        digits(&mut calculator, "1");
        assert_eq!(value(&calculator), "1");
    }

    #[test]
    fn locale_changes_display_and_expression() {
        let mut calculator = calc();
        digits(&mut calculator, "1234");
        calculator.handle(CalcAction::DecimalSeparator);
        digits(&mut calculator, "5");
        assert_eq!(value(&calculator), "1.234,5");
        assert_eq!(
            calculator.display(NumberLocale::En),
            DisplayState::Value("1,234.5".to_string())
        );
    }

    #[test]
    fn negative_zero_is_not_displayed() {
        let mut calculator = calc();
        digits(&mut calculator, "0");
        calculator.handle(CalcAction::Operator(Operator::Sub));
        digits(&mut calculator, "0");
        calculator.handle(CalcAction::Equals);
        assert_eq!(value(&calculator), "0");
    }

    #[test]
    fn deterministic_action_sequences_never_panic() {
        use super::super::decimal::MAX_INPUT_DIGITS;
        use super::super::history::HISTORY_CAPACITY;

        let mut seed = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            seed
        };

        for _ in 0..10_000 {
            let mut calculator = calc();
            let length = 1 + (next() % 200) as usize;
            for _ in 0..length {
                let random = next();
                let action = match random % 12 {
                    0..=4 => CalcAction::Digit(char::from(b'0' + (random % 10) as u8)),
                    5 => CalcAction::DecimalSeparator,
                    6 => CalcAction::Operator(match (random / 10) % 4 {
                        0 => Operator::Add,
                        1 => Operator::Sub,
                        2 => Operator::Mul,
                        _ => Operator::Div,
                    }),
                    7 => CalcAction::Equals,
                    8 => CalcAction::Backspace,
                    9 => CalcAction::Clear,
                    10 => CalcAction::ToggleSign,
                    _ => CalcAction::Percent,
                };

                calculator.handle(action);

                if calculator.pending().is_some() {
                    assert!(calculator.accumulator.is_some());
                }
                if calculator.error().is_some() {
                    assert!(calculator.pending().is_none());
                }
                assert!(calculator.entry.digit_count() <= MAX_INPUT_DIGITS);
                assert!(calculator.entry.raw().matches('.').count() <= 1);
                assert!(calculator.history().len() <= HISTORY_CAPACITY);
                assert!(!calculator.result_mode || calculator.pending.is_none());
                if let DisplayState::Value(text) = calculator.display(NumberLocale::PtBr) {
                    assert!(text.chars().count() <= 17, "display too wide: {text:?}");
                }
            }
        }
    }

    #[test]
    fn history_capacity_is_enforced_by_calculator() {
        let mut calculator = calc();
        for index in 0..(super::super::history::HISTORY_CAPACITY + 3) {
            calculator.history.push(HistoryEntry {
                lhs: Decimal::from(index as i64),
                op: Operator::Add,
                rhs_input: Decimal::ONE,
                rhs_effective: Decimal::ONE,
                percent: false,
                result: Decimal::from(index as i64) + Decimal::ONE,
            });
        }
        assert_eq!(
            calculator.history().len(),
            super::super::history::HISTORY_CAPACITY
        );
    }
}
