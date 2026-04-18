use super::parser::Parser;
use super::{CalcError, CalcResult, Evaluate};

pub struct StandardEngine;

impl StandardEngine {
    fn format_display(value: f64) -> String {
        if value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{}", value as i64)
        } else {
            let s = format!("{:.10}", value);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            s.to_string()
        }
    }
}

impl Evaluate for StandardEngine {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError> {
        let mut parser = Parser::new();
        let value = parser.parse(expr)?;

        if value.is_infinite() {
            return Err(CalcError::Overflow);
        }
        if value.is_nan() {
            return Err(CalcError::DomainError("result is not a number".into()));
        }

        Ok(CalcResult {
            value,
            display: Self::format_display(value),
            alt_bases: None,
        })
    }
}
