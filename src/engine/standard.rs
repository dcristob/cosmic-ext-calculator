use super::{CalcError, CalcResult, Evaluate};

pub struct StandardEngine;

impl Evaluate for StandardEngine {
    fn evaluate(&self, _expr: &str) -> Result<CalcResult, CalcError> {
        Err(CalcError::InvalidExpression("not yet implemented".into()))
    }
}
