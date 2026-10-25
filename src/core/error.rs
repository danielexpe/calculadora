use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CalcError {
    #[error("division by zero")]
    DivisionByZero,
    #[error("number too large")]
    Overflow,
    #[error("calculation error")]
    Other,
}
