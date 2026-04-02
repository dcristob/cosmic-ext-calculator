pub mod engineering;
pub mod parser;
pub mod standard;

/// Angle mode for trigonometric functions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AngleMode {
    #[default]
    Deg,
    Rad,
    Grad,
}

#[derive(Debug, Clone)]
pub struct CalcResult {
    pub value: f64,
    pub display: String,
    pub alt_bases: Option<AltBases>,
}

#[derive(Debug, Clone)]
pub struct AltBases {
    pub hex: String,
    pub oct: String,
    pub bin: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CalcError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    #[error("Domain error: {0}")]
    DomainError(String),
    #[error("Overflow")]
    Overflow,
    #[error("Convergence error")]
    ConvergenceError,
}

pub trait Evaluate {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError>;
}
