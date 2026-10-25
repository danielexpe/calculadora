use std::collections::VecDeque;

use rust_decimal::Decimal;

use super::{
    decimal::{format_value, NumberLocale},
    operator::Operator,
};

pub const HISTORY_CAPACITY: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub lhs: Decimal,
    pub op: Operator,
    pub rhs_input: Decimal,
    pub rhs_effective: Decimal,
    pub percent: bool,
    pub result: Decimal,
}

impl HistoryEntry {
    pub fn expression(&self, locale: NumberLocale) -> String {
        let percent = if self.percent { "%" } else { "" };
        format!(
            "{} {} {}{} =",
            format_value(self.lhs, locale),
            self.op.symbol(),
            format_value(self.rhs_input, locale),
            percent
        )
    }

    pub fn formatted(&self, locale: NumberLocale) -> String {
        format!(
            "{} {}",
            self.expression(locale),
            format_value(self.result, locale)
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct History {
    entries: VecDeque<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: HistoryEntry) {
        if self.entries.len() == HISTORY_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &HistoryEntry> {
        self.entries.iter()
    }

    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    pub fn last(&self) -> Option<&HistoryEntry> {
        self.entries.back()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn entry(lhs: &str, rhs: &str) -> HistoryEntry {
        HistoryEntry {
            lhs: decimal(lhs),
            op: Operator::Add,
            rhs_input: decimal(rhs),
            rhs_effective: decimal(rhs),
            percent: false,
            result: decimal(lhs) + decimal(rhs),
        }
    }

    #[test]
    fn formats_plain_entry() {
        let entry = entry("200", "10");
        assert_eq!(entry.expression(NumberLocale::PtBr), "200 + 10 =");
        assert_eq!(entry.formatted(NumberLocale::PtBr), "200 + 10 = 210");
    }

    #[test]
    fn formats_percent_entry_with_typed_operand() {
        let entry = HistoryEntry {
            lhs: decimal("200"),
            op: Operator::Add,
            rhs_input: decimal("10"),
            rhs_effective: decimal("20"),
            percent: true,
            result: decimal("220"),
        };
        assert_eq!(entry.formatted(NumberLocale::PtBr), "200 + 10% = 220");
        assert_eq!(entry.formatted(NumberLocale::En), "200 + 10% = 220");
    }

    #[test]
    fn respects_capacity() {
        let mut history = History::new();
        for index in 0..(HISTORY_CAPACITY + 5) {
            history.push(entry(&index.to_string(), "1"));
        }
        assert_eq!(history.len(), HISTORY_CAPACITY);
        assert_eq!(history.get(0).unwrap().lhs, decimal("5"));
    }
}
